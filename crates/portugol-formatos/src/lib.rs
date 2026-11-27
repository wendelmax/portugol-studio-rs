//! Tabela única dos formatos de arquivo do Ferro Studio (ADR 0023, Emenda 2).
//!
//! CLI, serviço, Tauri e IDE perguntam aqui o que é cada arquivo, em vez de
//! comparar extensões por conta própria. Trocar uma extensão (por exemplo,
//! `.pors` → `.fer`) passa a ser mudar uma linha desta tabela: a extensão
//! canônica vira a nova, e a antiga entra em `legacy`, que continua sendo
//! aceita durante a transição.
//!
//! A IDE recebe a mesma tabela no arquivo gerado
//! `apps/desktop/src/file-formats.generated.ts`. O teste `ts_table_is_current`
//! falha se ele ficar desatualizado; para regenerar, rode
//! `UPDATE_FILE_FORMATS=1 cargo test -p portugol-formatos`.
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
    /// (`pors.json`).
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
        extension: "pors",
        legacy: &[],
        label: "Portugol-RS",
        mime: "text/x-portugol-rs",
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
        extension: "pors.json",
        legacy: &[],
        label: "Exercício Portugol-RS",
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
        label: "Projeto desktop PORS",
        mime: "application/xml",
    },
];

/// Extensões do Portugol Studio clássico, que não são abertas nem
/// convertidas (ADR 0016).
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
/// (`exercicio.pors.json` é lição, não JSON qualquer). Um nome que é só a
/// extensão (`.pors`) não conta.
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

/// Extensão do Portugol Studio clássico (`.por`, `.pex`).
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
/// `".pors, .rs e .pors.json"`.
pub fn describe(kinds: &[Kind]) -> String {
    join(kinds, "e")
}

/// Como [`describe`], para alternativas: `".pors, .rs ou .pors.json"`.
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
        "// Gerado por crates/portugol-formatos. Não edite à mão: rode\n\
         // `UPDATE_FILE_FORMATS=1 cargo test -p portugol-formatos`.\n\n\
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
        assert_eq!(kind("principal.pors"), Some(Kind::Source));
        assert_eq!(kind("C:\\aulas\\Main.RS"), Some(Kind::Rust));
        assert_eq!(kind("aulas/01-dobro.pors.json"), Some(Kind::Lesson));
        assert_eq!(kind("App/JanelaPrincipal.design"), Some(Kind::Design));
        assert_eq!(kind("meu-app.FERPROJ"), Some(Kind::Manifest));
        assert_eq!(classify("dados.json"), None);
        assert_eq!(classify("sem-extensao"), None);
        assert_eq!(classify(".pors"), None);
        assert_eq!(classify("pasta.pors/arquivo"), None);
        assert_eq!(classify("programa.por"), None);
        assert_eq!(classify("ação.pors").unwrap().extension, "pors");
    }

    #[test]
    fn extensao_antiga_e_aceita_e_marcada() {
        static TEST: &[Format] = &[Format {
            kind: Kind::Source,
            extension: "fer",
            legacy: &["pors"],
            label: "Ferro",
            mime: "text/x-ferro",
        }];
        let old = classify_in(TEST, "principal.pors").unwrap();
        assert_eq!(
            (old.kind(), old.extension, old.legacy),
            (Kind::Source, "pors", true)
        );
        let new = classify_in(TEST, "principal.FER").unwrap();
        assert_eq!((new.extension, new.legacy), ("fer", false));
    }

    #[test]
    fn extensoes_retiradas() {
        assert!(is_retired("antigo.POR"));
        assert!(is_retired("pacote.pex"));
        assert!(!is_retired("programa.pors"));
        assert!(!is_retired("por"));
    }

    #[test]
    fn descreve_para_mensagens() {
        assert_eq!(describe(&[Kind::Source]), ".pors");
        assert_eq!(describe(&[Kind::Rust, Kind::Source]), ".pors e .rs");
        assert_eq!(
            describe(&all_kinds()),
            ".pors, .rs, .pors.json, .design e .ferproj"
        );
        assert_eq!(
            describe_alternatives(&[Kind::Source, Kind::Design]),
            ".pors ou .design"
        );
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
            "{path} está desatualizado; rode `UPDATE_FILE_FORMATS=1 cargo test -p portugol-formatos`"
        );
    }
}
