//! Tabela única dos formatos de arquivo do Ferro Studio (ADR 0023, Emenda 2).
//!
//! CLI, serviço, Tauri e IDE perguntam aqui o que é cada arquivo, em vez de
//! comparar extensões por conta própria. `.fer` e `.licao` são as extensões
//! canônicas; `.fer` e `.fer` não são reconhecidas.
//!
//! A IDE recebe a mesma tabela no arquivo gerado
//! `apps/desktop/src/file-formats.generated.ts`. O teste `ts_table_is_current`
//! falha se ele ficar desatualizado; para regenerar, rode
//! `UPDATE_FILE_FORMATS=1 cargo test -p ferro-formatos`.
//!
//! Sem dependências e compilável para `wasm32-unknown-unknown`.

use std::path::Path;

/// Tipo de arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Código-fonte da linguagem do estúdio.
    Source,
    /// Código Rust real.
    Rust,
    /// Exercício com correção automática.
    Lesson,
    /// Desenho de uma tela desktop (pseudo-XAML).
    Design,
    /// Manifesto de projeto.
    Manifest,
}

impl Kind {
    /// Arquivos de programa: podem ser criados pelo aluno, executados e
    /// salvos fora de um workspace.
    pub fn is_program(self) -> bool {
        matches!(self, Kind::Source | Kind::Rust)
    }

    /// Marcação de projetos (`.design` e manifesto): não executa sozinha.
    pub fn is_markup(self) -> bool {
        matches!(self, Kind::Design | Kind::Manifest)
    }

    /// Identificador estável, usado no arquivo gerado para a IDE.
    pub fn id(self) -> &'static str {
        match self {
            Kind::Source => "source",
            Kind::Rust => "rust",
            Kind::Lesson => "lesson",
            Kind::Design => "design",
            Kind::Manifest => "manifest",
        }
    }
}

/// Um formato de arquivo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Format {
    pub kind: Kind,
    /// Extensão canônica, sem o ponto inicial. Pode ter mais de uma parte
    /// (`licao`).
    pub extension: &'static str,
    /// Extensões antigas ainda aceitas durante a transição.
    pub legacy: &'static [&'static str],
    /// Nome mostrado ao aluno em diálogos e filtros.
    pub label: &'static str,
    pub mime: &'static str,
}

/// Todos os formatos, na ordem em que aparecem nas mensagens.
pub const FORMATS: &[Format] = &[
    Format {
        kind: Kind::Source,
        extension: "fer",
        legacy: &[],
        label: "Ferro",
        mime: "text/x-ferro",
    },
    Format {
        kind: Kind::Rust,
        extension: "rs",
        legacy: &[],
        label: "Rust",
        mime: "text/x-rust",
    },
    Format {
        kind: Kind::Lesson,
        extension: "licao",
        legacy: &[],
        label: "Lição Ferro",
        mime: "application/json",
    },
    Format {
        kind: Kind::Design,
        extension: "design",
        legacy: &[],
        label: "Tela (pseudo-XAML)",
        mime: "application/xml",
    },
    Format {
        kind: Kind::Manifest,
        extension: "ferproj",
        legacy: &[],
        label: "Projeto Ferro",
        mime: "application/xml",
    },
];

/// Extensões legadas que não são abertas nem convertidas (ADR 0016).
pub const RETIRED: &[&str] = &["por", "pex"];

/// Formato de um tipo.
pub fn format(kind: Kind) -> &'static Format {
    FORMATS
        .iter()
        .find(|format| format.kind == kind)
        .expect("todo tipo tem um formato na tabela")
}

/// Extensão canônica de um tipo, sem o ponto. É `const` para que outros
/// crates derivem constantes dela.
pub const fn extension(kind: Kind) -> &'static str {
    let mut index = 0;
    while index < FORMATS.len() {
        if FORMATS[index].kind as u8 == kind as u8 {
            return FORMATS[index].extension;
        }
        index += 1;
    }
    panic!("todo tipo tem um formato na tabela")
}

/// Resultado de [`classify`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Match {
    pub format: &'static Format,
    /// A extensão encontrada, sem o ponto.
    pub extension: &'static str,
    /// O arquivo usa uma extensão antiga, aceita só durante a transição.
    pub legacy: bool,
}

impl Match {
    pub fn kind(&self) -> Kind {
        self.format.kind
    }
}

/// `name` termina com `.extension`, ignorando maiúsculas.
fn has_suffix(name: &str, extension: &str) -> bool {
    let Some(start) = name.len().checked_sub(extension.len() + 1) else {
        return false;
    };
    name.is_char_boundary(start)
        && name.as_bytes()[start] == b'.'
        && name[start + 1..].eq_ignore_ascii_case(extension)
}

/// Último componente do caminho, aceitando `/` e `\`.
fn file_name(path: &str) -> &str {
    path.rsplit(['/', '\\']).next().unwrap_or(path)
}

/// Formato de um arquivo pelo nome ou caminho. Extensões mais longas ganham
/// (`exercicio.licao` é lição, não JSON qualquer). Um nome que é só a extensão
/// (`.fer`) não conta.
pub fn classify(path: &str) -> Option<Match> {
    classify_in(FORMATS, path)
}

fn classify_in(table: &'static [Format], path: &str) -> Option<Match> {
    let name = file_name(path);
    let mut best: Option<Match> = None;
    for format in table {
        let candidates = std::iter::once((format.extension, false))
            .chain(format.legacy.iter().map(|&extension| (extension, true)));
        for (extension, legacy) in candidates {
            let longer = best.is_none_or(|b| extension.len() > b.extension.len());
            if longer && name.len() > extension.len() + 1 && has_suffix(name, extension) {
                best = Some(Match {
                    format,
                    extension,
                    legacy,
                });
            }
        }
    }
    best
}

/// [`classify`] para um [`Path`]. Só o nome do arquivo conta; bytes que não
/// são UTF-8 no nome não impedem reconhecer a extensão.
pub fn classify_path(path: &Path) -> Option<Match> {
    classify(&path.file_name()?.to_string_lossy())
}

/// Tipo do arquivo, se for de algum dos `kinds`.
pub fn kind_of(path: &Path, kinds: &[Kind]) -> Option<Kind> {
    classify_path(path)
        .map(|m| m.kind())
        .filter(|kind| kinds.contains(kind))
}

/// Se `name` usar uma extensão antiga, devolve o nome com a extensão canônica.
pub fn canonical_name(name: &str) -> Option<String> {
    let matched = classify(name).filter(|m| m.legacy)?;
    let base_len = name.len() - matched.extension.len() - 1;
    Some(format!(
        "{}.{}",
        &name[..base_len],
        matched.format.extension
    ))
}

/// Se `path` usar uma extensão antiga, devolve o caminho com a extensão canônica.
pub fn canonical_path(path: &Path) -> Option<std::path::PathBuf> {
    let name = path.file_name()?.to_str()?;
    let new_name = canonical_name(name)?;
    Some(path.with_file_name(new_name))
}

/// Extensão legada (`.por`, `.pex`).
pub fn is_retired(path: &str) -> bool {
    let name = file_name(path);
    RETIRED.iter().any(|extension| has_suffix(name, extension))
}

/// [`is_retired`] para um [`Path`].
pub fn is_retired_path(path: &Path) -> bool {
    path.file_name()
        .is_some_and(|name| is_retired(&name.to_string_lossy()))
}

/// Lista as extensões canônicas dos tipos para mensagens:
/// `".fer, .rs e .licao"`.
pub fn describe(kinds: &[Kind]) -> String {
    join(kinds, "e")
}

/// Como [`describe`], para alternativas: `".fer, .rs ou .licao"`.
pub fn describe_alternatives(kinds: &[Kind]) -> String {
    join(kinds, "ou")
}

fn join(kinds: &[Kind], conjunction: &str) -> String {
    let names: Vec<String> = FORMATS
        .iter()
        .filter(|format| kinds.contains(&format.kind))
        .map(|format| format!(".{}", format.extension))
        .collect();
    match names.as_slice() {
        [] => String::new(),
        [only] => only.clone(),
        [rest @ .., last] => format!("{} {conjunction} {last}", rest.join(", ")),
    }
}

/// Todos os tipos, na ordem da tabela.
pub fn all_kinds() -> Vec<Kind> {
    FORMATS.iter().map(|format| format.kind).collect()
}

/// Conteúdo de `apps/desktop/src/file-formats.generated.ts`.
pub fn typescript_table() -> String {
    fn string(value: &str) -> String {
        let mut out = String::from("\"");
        for c in value.chars() {
            match c {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                other => out.push(other),
            }
        }
        out.push('"');
        out
    }
    let mut out = String::from(
        "// Gerado por crates/ferro-formatos. Não edite à mão: rode\n\
         // `UPDATE_FILE_FORMATS=1 cargo test -p ferro-formatos`.\n\n\
         export type FileKind = \"source\" | \"rust\" | \"lesson\" | \"design\" | \"manifest\";\n\n\
         export type FileFormat = {\n  kind: FileKind;\n  extension: string;\n  legacy: readonly string[];\n  label: string;\n  mime: string;\n};\n\n\
         export const FILE_FORMATS: readonly FileFormat[] = [\n",
    );
    for format in FORMATS {
        let legacy: Vec<String> = format.legacy.iter().map(|e| string(e)).collect();
        out.push_str(&format!(
            "  {{ kind: {}, extension: {}, legacy: [{}], label: {}, mime: {} }},\n",
            string(format.kind.id()),
            string(format.extension),
            legacy.join(", "),
            string(format.label),
            string(format.mime),
        ));
    }
    out.push_str("];\n\nexport const RETIRED_EXTENSIONS: readonly string[] = [");
    let retired: Vec<String> = RETIRED.iter().map(|e| string(e)).collect();
    out.push_str(&retired.join(", "));
    out.push_str("];\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifica_pela_extensao_ignorando_maiusculas() {
        let kind = |path| classify(path).map(|m| m.kind());
        assert_eq!(kind("principal.fer"), Some(Kind::Source));
        assert_eq!(kind("principal.zzz"), None);
        assert_eq!(kind("C:\\aulas\\Main.RS"), Some(Kind::Rust));
        assert_eq!(kind("aulas/01-dobro.licao"), Some(Kind::Lesson));
        assert_eq!(kind("aulas/01-dobro.zzz"), None);
        assert_eq!(kind("App/JanelaPrincipal.design"), Some(Kind::Design));
        assert_eq!(kind("meu-app.FERPROJ"), Some(Kind::Manifest));
        assert_eq!(classify("dados.json"), None);
        assert_eq!(classify("sem-extensao"), None);
        assert_eq!(classify(".zzz"), None);
        assert_eq!(classify(".licao"), None);
        assert_eq!(classify("pasta.fer/arquivo"), None);
        assert_eq!(classify("programa.por"), None);
        let canonical = classify("ação.FER").unwrap();
        assert_eq!((canonical.extension, canonical.legacy), ("fer", false));
    }

    #[test]
    fn extensao_nao_catalogada_nao_e_aceita() {
        static TEST: &[Format] = &[Format {
            kind: Kind::Source,
            extension: "fer",
            legacy: &[],
            label: "Ferro",
            mime: "text/x-ferro",
        }];
        assert!(classify_in(TEST, "principal.zzz").is_none());
        let new = classify_in(TEST, "principal.FER").unwrap();
        assert_eq!((new.extension, new.legacy), ("fer", false));
    }

    #[test]
    fn extensoes_retiradas() {
        assert!(is_retired("antigo.POR"));
        assert!(is_retired("pacote.pex"));
        assert!(!is_retired("programa.fer"));
        assert!(!is_retired("por"));
    }

    #[test]
    fn descreve_para_mensagens() {
        assert_eq!(describe(&[Kind::Source]), ".fer");
        assert_eq!(describe(&[Kind::Rust, Kind::Source]), ".fer e .rs");
        assert_eq!(
            describe(&all_kinds()),
            ".fer, .rs, .licao, .design e .ferproj"
        );
        assert_eq!(
            describe_alternatives(&[Kind::Source, Kind::Design]),
            ".fer ou .design"
        );
    }

    #[test]
    fn caminho_canonico_retorna_none_sem_extensoes_legadas() {
        assert_eq!(canonical_name("aula.fer"), None);
        assert_eq!(canonical_name("aula.fer"), None);
        assert_eq!(canonical_path(Path::new("pasta/app.fer")), None);
    }

    #[test]
    fn tabela_sem_extensoes_repetidas() {
        let mut seen = Vec::new();
        for format in FORMATS {
            for extension in std::iter::once(&format.extension).chain(format.legacy) {
                assert!(!seen.contains(extension), "{extension} repetida");
                assert!(!RETIRED.contains(extension), "{extension} está retirada");
                seen.push(*extension);
            }
        }
        for kind in all_kinds() {
            assert_eq!(format(kind).kind, kind);
        }
    }

    #[test]
    fn ts_table_is_current() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../apps/desktop/src/file-formats.generated.ts"
        );
        let expected = typescript_table();
        if std::env::var_os("UPDATE_FILE_FORMATS").is_some() {
            std::fs::write(path, &expected).unwrap();
            return;
        }
        let current = std::fs::read_to_string(path).unwrap_or_default();
        assert!(
            current == expected,
            "{path} está desatualizado; rode `UPDATE_FILE_FORMATS=1 cargo test -p ferro-formatos`"
        );
    }
}
