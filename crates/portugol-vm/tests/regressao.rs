//! Testes de regressão sobre o corpus `tests/corpus` (os exemplos do Portugol
//! Studio original).
//!
//! Para cada `programa.por` existe um `programa.saida` com os diagnósticos, a
//! saída do programa e como a execução terminou. Um `programa.entrada`
//! opcional fornece as linhas lidas pelo `leia`. A execução é determinística:
//! semente fixa, relógio simulado e limite de instruções.
//!
//! Quando uma mudança altera a saída de propósito, regrave com:
//!
//! ```text
//! PORTUGOL_ATUALIZAR=1 cargo test -p portugol-vm --test regressao
//! ```
//!
//! e revise o diff dos `.saida` antes de commitar.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};

use portugol_syntax::span::line_col;
use portugol_syntax::{Severity, parse};
use portugol_vm::{BufferIo, Status, Vm};

/// Limite por programa; jogos com laço infinito param aqui.
const BUDGET: usize = 2_000_000;
/// 2024-01-01 12:00:00 UTC.
const EPOCH_MS: u64 = 1_704_110_400_000;
const SEED: u64 = 42;

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus")
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("ler tests/corpus").flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|e| e == "por") {
            files.push(path);
        }
    }
}

/// Relatório textual de uma execução, no formato dos `.saida`.
fn report(source: &str, input: &str) -> String {
    let mut out = String::new();
    let result = parse(source);
    let mut diagnostics = result.diagnostics;
    if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
        diagnostics.extend(portugol_semantics::analyze(&result.program));
    }

    out.push_str("# diagnósticos\n");
    for d in &diagnostics {
        let (line, column) = line_col(source, d.span.start);
        let _ = writeln!(
            out,
            "{} {} {line}:{column} {}",
            d.severity, d.code, d.message
        );
    }

    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        out.push_str("# resultado\nnão executado\n");
        return out;
    }

    let module = match portugol_bytecode::compile(&result.program) {
        Ok(module) => module,
        Err(error) => {
            let _ = writeln!(out, "# resultado\nerro de compilação: {}", error.message);
            return out;
        }
    };

    let mut io = BufferIo::new(input);
    let outcome = Vm::new(&module, &mut io)
        .with_seed(SEED)
        .with_simulated_time(EPOCH_MS)
        .step(BUDGET);
    out.push_str("# saída\n");
    out.push_str(&io.output);
    if !io.output.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("# resultado\n");
    match outcome {
        Ok(Status::Finished) => out.push_str("fim\n"),
        Ok(Status::Running) => out.push_str("limite de instruções\n"),
        Err(error) => {
            let (line, column) = line_col(source, error.span.start);
            let _ = writeln!(out, "erro {line}:{column}: {}", error.message);
        }
    }
    out
}

#[test]
fn corpus_do_portugol_studio() {
    let mut files = Vec::new();
    collect(&corpus_dir(), &mut files);
    files.sort();
    assert!(
        files.len() >= 196,
        "corpus incompleto: {} arquivos",
        files.len()
    );

    let update = std::env::var_os("PORTUGOL_ATUALIZAR").is_some();
    let mut failures = Vec::new();

    for file in &files {
        let source = std::fs::read_to_string(file).expect("ler .por");
        let input = std::fs::read_to_string(file.with_extension("entrada")).unwrap_or_default();
        let actual = report(&source, &input);
        let expected_path = file.with_extension("saida");

        if update {
            std::fs::write(&expected_path, &actual).expect("gravar .saida");
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
        if expected != actual {
            failures.push(format!(
                "{}\n{}",
                file.strip_prefix(corpus_dir()).unwrap_or(file).display(),
                first_difference(&expected, &actual)
            ));
        }
    }

    assert!(
        failures.is_empty(),
        "{} de {} programas mudaram de comportamento \
         (se for intencional, rode com PORTUGOL_ATUALIZAR=1):\n\n{}",
        failures.len(),
        files.len(),
        failures.join("\n\n")
    );
}

fn first_difference(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    let mut line = 1;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(e), Some(a)) if e == a => line += 1,
            (None, None) => return "  (diferença só em quebras de linha)".to_string(),
            (e, a) => {
                return format!(
                    "  linha {line}\n  esperado: {}\n  obtido:   {}",
                    e.unwrap_or("<fim>"),
                    a.unwrap_or("<fim>")
                );
            }
        }
    }
}
