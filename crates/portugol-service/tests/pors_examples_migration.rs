use portugol_rs_frontend::lower;
use serde_json::Value;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

const HELP_ASSETS: &str = "apps/desktop/src/help-assets/recursos/exemplos-pors";
const HELP_PREFIX: &str = "apps/desktop/src/help-assets/recursos/exemplos-pors/";
const SERVICE_ASSETS: &str = "crates/portugol-service/assets/exemplos-pors";
const FIXTURES: &str = "tests/fixtures";
const ALLOWLIST: &str = "tests/fixtures/pors-migration-allowlist.json";

// `escreva(...)` sem quebra explícita continua válido: escreval é apenas
// açúcar para acrescentar "\n" (RFC 003), então converter calls sem "\n"
// mudaria stdout. Só o argumento de quebra explícito é marcador legado.
const EXPLICIT_NEWLINE_WRITE: &str = "escreva-com-quebra-explicita";
const LEGACY_MARKERS: [&str; 5] = [
    EXPLICIT_NEWLINE_WRITE,
    // `leia()` é a forma preferida; o marcador versa sobre `leia(variavel)`.
    "leia(",
    ": inteiro = 0;",
    "+ \"\\n\"",
    "Tipos.para_inteiro(leia())",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn collect_pors(dir: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("não foi possível listar {}: {error}", dir.display()));
    for entry in entries {
        let entry =
            entry.unwrap_or_else(|error| panic!("entrada inválida em {}: {error}", dir.display()));
        let path = entry.path();
        if path.is_dir() {
            collect_pors(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "fer") {
            out.push(path);
        }
    }
}

fn all_examples() -> Vec<(String, PathBuf)> {
    let root = repo_root();
    let mut examples = Vec::new();
    for (dir, expected) in [(HELP_ASSETS, 77), (SERVICE_ASSETS, 133), (FIXTURES, 1)] {
        let mut files = Vec::new();
        collect_pors(&root.join(dir), &mut files);
        files.sort();
        assert_eq!(
            files.len(),
            expected,
            "{dir}: esperava {expected} arquivos .pors, encontrou {}",
            files.len()
        );
        for file in files {
            let relative = file
                .strip_prefix(&root)
                .expect("arquivo dentro da raiz")
                .to_string_lossy()
                .replace('\\', "/");
            examples.push((relative, file));
        }
    }
    assert_eq!(examples.len(), 211, "esperava 211 exemplos .pors no total");
    examples
}

fn has_read_with_argument(source: &str) -> bool {
    let mut rest = source;
    while let Some(at) = rest.find("leia(") {
        rest = rest[at + "leia(".len()..].trim_start();
        if !rest.is_empty() && !rest.starts_with(')') {
            return true;
        }
    }
    false
}

fn has_newline_concat(source: &str) -> bool {
    let mut rest = source;
    while let Some(at) = rest.find('+') {
        rest = rest[at + 1..].trim_start();
        if ["\"\\n\"", "\"\\\\n\""]
            .iter()
            .any(|literal| rest.starts_with(literal))
        {
            return true;
        }
    }
    false
}

fn literal_has_newline(inner: &str) -> bool {
    let bytes = inner.as_bytes();
    let mut at = 0;
    let mut quote = None;
    while at < bytes.len() {
        match quote {
            None => {
                if matches!(bytes[at], b'"' | b'\'') {
                    quote = Some(bytes[at]);
                }
                at += 1;
            }
            Some(delimiter) => {
                if bytes[at] == b'\\' {
                    if at + 1 < bytes.len() && bytes[at + 1] == b'n' {
                        return true;
                    }
                    at += 2;
                } else {
                    if bytes[at] == delimiter {
                        quote = None;
                    }
                    at += 1;
                }
            }
        }
    }
    false
}

fn has_explicit_newline_write(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut from = 0;
    while let Some(relative) = source[from..].find("escreva(") {
        let open = from + relative + "escreva".len();
        let mut depth = 0usize;
        let mut quote = None;
        let mut close = open;
        while close < bytes.len() {
            match quote {
                Some(delimiter) => {
                    if bytes[close] == b'\\' {
                        close += 2;
                        continue;
                    }
                    if bytes[close] == delimiter {
                        quote = None;
                    }
                }
                None => match bytes[close] {
                    b'"' | b'\'' => quote = Some(bytes[close]),
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    _ => {}
                },
            }
            close += 1;
        }
        if literal_has_newline(&source[open + 1..close]) {
            return true;
        }
        from = close + 1;
    }
    false
}

fn has_marker(source: &str, marker: &str) -> bool {
    match marker {
        EXPLICIT_NEWLINE_WRITE => has_explicit_newline_write(source),
        "leia(" => has_read_with_argument(source),
        "+ \"\\n\"" => has_newline_concat(source),
        other => source.contains(other),
    }
}

fn load_allowlist() -> BTreeSet<(String, String)> {
    let path = repo_root().join(ALLOWLIST);
    let raw = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("não foi possível ler {}: {error}", path.display()));
    let value: Value = serde_json::from_str(&raw).expect("allowlist deve ser JSON válido");
    let entries = value.as_array().expect("allowlist deve ser um array");

    let mut allowed = BTreeSet::new();
    for entry in entries {
        let object = entry.as_object().expect("cada entrada deve ser um objeto");
        let path = object
            .get("path")
            .and_then(Value::as_str)
            .expect("entrada deve ter path");
        let markers = object
            .get("markers")
            .and_then(Value::as_array)
            .expect("entrada deve ter markers");
        let reason = object
            .get("reason")
            .and_then(Value::as_str)
            .expect("entrada deve ter reason");
        assert!(!path.is_empty(), "path não pode ser vazio");
        assert!(
            !reason.trim().is_empty(),
            "reason de {path} não pode ser vazio"
        );
        for marker in markers {
            let marker = marker.as_str().expect("marker deve ser texto");
            assert!(
                LEGACY_MARKERS.contains(&marker),
                "marcador desconhecido em {path}: {marker}"
            );
            allowed.insert((path.replace('\\', "/"), marker.to_owned()));
        }
    }
    allowed
}

#[test]
fn all_211_pors_examples_are_well_formed_and_complete_ones_compile() {
    let mut failures = Vec::new();
    let mut programs = 0;
    let mut fragments = 0;

    for (relative, path) in all_examples() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("não foi possível ler {}: {error}", path.display()));
        assert!(!source.trim().is_empty(), "{relative}: arquivo vazio");

        for boilerplate in ["programa {", "inclua biblioteca"] {
            if source.contains(boilerplate) {
                failures.push(format!(
                    "{relative}: contém boilerplate legado {boilerplate:?}"
                ));
            }
        }

        // Somente os snippets `data-type="sintaxe"` dos help-assets são
        // fragmentos: não têm `funcao inicio()` e alguns usam nomes de exemplo
        // não declarados. Programas de serviço sem `inicio()` (por exemplo
        // 08-ui-contador.pors, executado via UiSession) e o fixture continuam
        // sendo compilados normalmente.
        if relative.starts_with(HELP_PREFIX) && !source.contains("funcao inicio(") {
            fragments += 1;
            continue;
        }
        programs += 1;

        let program = match lower(&source) {
            Ok(program) => program,
            Err(diagnostics) => {
                failures.push(format!("{relative}: lower falhou: {diagnostics:?}"));
                continue;
            }
        };
        if portugol_rs_frontend::format(&source).is_none() {
            failures.push(format!("{relative}: format não produziu edições"));
        }
        if let Err(error) = portugol_bytecode::compile_pors(&program) {
            failures.push(format!("{relative}: compile_pors falhou: {error:?}"));
        }
    }

    assert_eq!(programs, 172, "esperava 172 programas .pors completos");
    assert_eq!(
        fragments, 39,
        "esperava 39 fragmentos de sintaxe nos help-assets"
    );
    assert!(
        failures.is_empty(),
        "exemplos .pors inválidos:\n{}",
        failures.join("\n")
    );
}

#[test]
fn all_211_pors_examples_avoid_legacy_syntax() {
    let allowlist = load_allowlist();
    let mut failures = Vec::new();

    for (relative, path) in all_examples() {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("não foi possível ler {}: {error}", path.display()));
        for marker in LEGACY_MARKERS {
            if has_marker(&source, marker)
                && !allowlist.contains(&(relative.clone(), marker.to_owned()))
            {
                failures.push(format!("{relative}: marcador legado {marker:?}"));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "sintaxe legada em exemplos .pors:\n{}",
        failures.join("\n")
    );
}
