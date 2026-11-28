//! Projeto desktop Ferro: o manifesto `.ferproj` e o modelo *Aplicativo
//! Desktop FERRO* (ADR 0018, seção 1).
//!
//! O manifesto usa a mesma marcação do `.design`, como o `.csproj` do Visual
//! Studio:
//!
//! ```xml
//! <Projeto Nome="Meu App" Versao="0.1.0" JanelaInicial="JanelaPrincipal" Formato="1" />
//! ```

use std::collections::HashSet;

use ferro_hir::{Diagnostic, Span};

use crate::ast::{Attribute, Document};
use crate::value;

/// Extensão do manifesto, sem o ponto.
pub const MANIFEST_EXTENSION: &str = ferro_formatos::extension(ferro_formatos::Kind::Manifest);
/// Pasta das imagens e ícones do projeto.
pub const RESOURCES_DIR: &str = "recursos";
/// Nome da janela criada pelo modelo.
pub const DEFAULT_WINDOW: &str = "JanelaPrincipal";
/// Tamanho máximo do nome do projeto, em caracteres.
pub const MAX_NAME_CHARS: usize = 64;

const ATTRIBUTES: &[&str] = &["Nome", "Versao", "JanelaInicial", "Icone", "Formato"];

/// Conteúdo de um `.ferproj` válido.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Manifest {
    pub name: String,
    pub version: String,
    /// Nome da janela aberta ao executar: o arquivo `<JanelaInicial>.design`.
    pub start_window: String,
    pub start_window_span: Span,
    pub icon: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ManifestAnalysis {
    pub manifest: Option<Manifest>,
    pub diagnostics: Vec<Diagnostic>,
}

/// Lê e valida um `.ferproj`. Erros de sintaxe e de estrutura impedem o
/// manifesto; avisos não.
pub fn analyze_manifest(source: &str) -> ManifestAnalysis {
    let document = match crate::parse(source) {
        Ok(document) => document,
        Err(diagnostic) => {
            return ManifestAnalysis {
                manifest: None,
                diagnostics: vec![diagnostic],
            };
        }
    };
    let mut diagnostics = Vec::new();
    let manifest = manifest(&document, &mut diagnostics);
    let has_errors = diagnostics
        .iter()
        .any(|d| d.severity == ferro_hir::Severity::Error);
    ManifestAnalysis {
        manifest: if has_errors { None } else { manifest },
        diagnostics,
    }
}

fn manifest(document: &Document, diagnostics: &mut Vec<Diagnostic>) -> Option<Manifest> {
    let root = &document.root;
    if root.name.text != "Projeto" {
        diagnostics.push(Diagnostic::error(
            "projeto.expected-project",
            format!(
                "O elemento raiz de um .ferproj é `<Projeto>`, mas encontrei `<{}>`",
                root.name.text
            ),
            root.name.span,
        ));
        return None;
    }
    if let Some(child) = root.elements().next() {
        diagnostics.push(Diagnostic::error(
            "projeto.children-not-allowed",
            "`<Projeto>` não contém outros elementos nesta versão",
            child.name.span,
        ));
    }
    if let Some(text) = root.texts().next() {
        diagnostics.push(Diagnostic::error(
            "projeto.unexpected-text",
            "Texto solto não é permitido dentro de `<Projeto>`",
            text.span,
        ));
    }

    let mut seen = HashSet::new();
    for attribute in &root.attributes {
        let name = attribute.name.text.as_str();
        if !ATTRIBUTES.contains(&name) {
            let mut diagnostic = Diagnostic::error(
                "projeto.unknown-attribute",
                format!("`<Projeto>` não tem o atributo `{name}`"),
                attribute.name.span,
            );
            if let Some(same) = ATTRIBUTES.iter().find(|a| a.eq_ignore_ascii_case(name)) {
                diagnostic =
                    diagnostic.with_help(format!("Os atributos começam com maiúscula: `{same}`."));
            }
            diagnostics.push(diagnostic);
        } else if !seen.insert(name) {
            diagnostics.push(Diagnostic::error(
                "projeto.duplicate-attribute",
                format!("O atributo `{name}` aparece mais de uma vez"),
                attribute.name.span,
            ));
        }
    }

    if let Some(format) = root.attribute("Formato")
        && format.value.trim() != crate::CURRENT_VERSION.to_string()
    {
        diagnostics.push(Diagnostic::error(
            "projeto.unsupported-version",
            format!(
                "`Formato=\"{}\"` não é suportado; a versão atual é {}",
                format.value,
                crate::CURRENT_VERSION
            ),
            format.value_span,
        ));
    }

    let name = required(root.attribute("Nome"), "Nome", root.name.span, diagnostics)
        .and_then(|attribute| check_name(attribute, diagnostics));

    let version = match root.attribute("Versao") {
        Some(attribute) if is_version(&attribute.value) => attribute.value.clone(),
        Some(attribute) => {
            diagnostics.push(
                Diagnostic::error(
                    "projeto.invalid-version",
                    format!("Versão inválida: \"{}\"", attribute.value),
                    attribute.value_span,
                )
                .with_help("Use três números separados por ponto, como \"0.1.0\"."),
            );
            String::new()
        }
        None => "0.1.0".to_owned(),
    };

    let start = required(
        root.attribute("JanelaInicial"),
        "JanelaInicial",
        root.name.span,
        diagnostics,
    )
    .and_then(|attribute| {
        if value::is_identifier(&attribute.value) {
            Some(attribute)
        } else {
            diagnostics.push(
                Diagnostic::error(
                    "projeto.invalid-window-name",
                    format!("Nome de janela inválido: \"{}\"", attribute.value),
                    attribute.value_span,
                )
                .with_help(
                    "Use o nome do arquivo .design sem a extensão, como \"JanelaPrincipal\".",
                ),
            );
            None
        }
    });

    let icon = root.attribute("Icone").and_then(|attribute| {
        if is_resource_path(&attribute.value) {
            Some(attribute.value.clone())
        } else {
            diagnostics.push(Diagnostic::error(
                "projeto.invalid-path",
                "O caminho do ícone deve ser relativo à pasta do projeto e não pode usar `..`",
                attribute.value_span,
            ));
            None
        }
    });

    Some(Manifest {
        name: name?,
        version,
        start_window: start?.value.clone(),
        start_window_span: start?.value_span,
        icon,
    })
}

fn required<'a>(
    attribute: Option<&'a Attribute>,
    name: &str,
    span: Span,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<&'a Attribute> {
    if attribute.is_none() {
        diagnostics.push(Diagnostic::error(
            "projeto.missing-attribute",
            format!("Falta o atributo obrigatório `{name}` em `<Projeto>`"),
            span,
        ));
    }
    attribute
}

fn check_name(attribute: &Attribute, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    match validate_project_name(&attribute.value) {
        Ok(()) => Some(attribute.value.clone()),
        Err(message) => {
            diagnostics.push(Diagnostic::error(
                "projeto.invalid-name",
                message,
                attribute.value_span,
            ));
            None
        }
    }
}

/// Confere se `<JanelaInicial>.design` está entre os arquivos `.design` da
/// pasta do projeto (nomes de arquivo, com extensão).
pub fn check_start_window(manifest: &Manifest, design_files: &[&str]) -> Option<Diagnostic> {
    let expected = format!("{}.{}", manifest.start_window, crate::FILE_EXTENSION);
    if design_files.contains(&expected.as_str()) {
        return None;
    }
    Some(
        Diagnostic::error(
            "projeto.missing-start-window",
            format!("A janela inicial `{expected}` não existe na pasta do projeto"),
            manifest.start_window_span,
        )
        .with_help("Crie a tela ou escolha em `JanelaInicial` uma tela que já exista."),
    )
}

/// Regras do nome exibido do projeto.
pub fn validate_project_name(name: &str) -> Result<(), String> {
    let trimmed = name.trim();
    if trimmed.is_empty() {
        return Err("O nome do projeto não pode ficar vazio".to_owned());
    }
    if trimmed.chars().count() > MAX_NAME_CHARS {
        return Err(format!(
            "O nome do projeto pode ter no máximo {MAX_NAME_CHARS} caracteres"
        ));
    }
    if trimmed.chars().any(char::is_control) {
        return Err("O nome do projeto não pode ter quebras de linha nem tabulações".to_owned());
    }
    if file_stem(trimmed).is_empty() {
        return Err("O nome do projeto precisa ter ao menos uma letra ou um dígito".to_owned());
    }
    Ok(())
}

/// Nome de arquivo derivado do nome do projeto: minúsculas sem acento,
/// dígitos e `-` (`"Meu App 2"` → `"meu-app-2"`).
pub fn file_stem(name: &str) -> String {
    let mut out = String::new();
    let mut pending_dash = false;
    for c in name.chars() {
        let mapped = strip_accent(c.to_lowercase().next().unwrap_or(c));
        if mapped.is_ascii_alphanumeric() {
            if pending_dash && !out.is_empty() {
                out.push('-');
            }
            pending_dash = false;
            out.push(mapped);
        } else {
            pending_dash = true;
        }
    }
    out
}

fn strip_accent(c: char) -> char {
    match c {
        'á' | 'à' | 'â' | 'ã' | 'ä' => 'a',
        'é' | 'è' | 'ê' | 'ë' => 'e',
        'í' | 'ì' | 'î' | 'ï' => 'i',
        'ó' | 'ò' | 'ô' | 'õ' | 'ö' => 'o',
        'ú' | 'ù' | 'û' | 'ü' => 'u',
        'ç' => 'c',
        'ñ' => 'n',
        other => other,
    }
}

fn is_version(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|p| !p.is_empty() && p.len() <= 6 && p.bytes().all(|b| b.is_ascii_digit()))
}

fn is_resource_path(path: &str) -> bool {
    let normalized = path.replace('\\', "/");
    !(normalized.is_empty()
        || normalized.starts_with('/')
        || normalized.as_bytes().get(1) == Some(&b':')
        || normalized.split('/').any(|part| part == ".."))
}

/// Um arquivo do modelo, com caminho relativo à pasta do projeto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateFile {
    pub path: String,
    pub content: String,
}

/// Arquivos e pastas que o modelo *Aplicativo Desktop FERRO* cria.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub files: Vec<TemplateFile>,
    pub directories: Vec<String>,
}

/// Projeto de exemplo que o aluno pode criar já pronto (tela e código).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExampleProject {
    pub id: &'static str,
    /// Nome sugerido para o projeto.
    pub name: &'static str,
    pub description: &'static str,
    /// Arquivos da pasta do projeto, além do manifesto.
    pub files: &'static [(&'static str, &'static str)],
}

/// Exemplos de projeto desktop. O contador é o antigo exemplo `UI.*`
/// convertido em tela (ADR 0021, Etapa B).
pub const EXAMPLE_PROJECTS: &[ExampleProject] = &[ExampleProject {
    id: "contador",
    name: "Contador",
    description: "Um botão que soma 1 e um rótulo que mostra a contagem com {dados.contagem}.",
    files: &[
        (
            "JanelaPrincipal.design",
            include_str!("../assets/exemplos/contador/JanelaPrincipal.design"),
        ),
        (
            "JanelaPrincipal.fer",
            include_str!("../assets/exemplos/contador/JanelaPrincipal.fer"),
        ),
    ],
}];

/// Modelo de um projeto de exemplo, com o nome escolhido no manifesto.
pub fn example_template(id: &str, name: &str) -> Result<Template, String> {
    validate_project_name(name)?;
    let example = EXAMPLE_PROJECTS
        .iter()
        .find(|example| example.id == id)
        .ok_or_else(|| {
            let ids: Vec<_> = EXAMPLE_PROJECTS.iter().map(|e| e.id).collect();
            format!(
                "Exemplo desconhecido: `{id}`. Exemplos: {}.",
                ids.join(", ")
            )
        })?;
    let name = name.trim();
    let manifest = format!(
        "<Projeto Nome=\"{}\" Versao=\"0.1.0\" JanelaInicial=\"{DEFAULT_WINDOW}\" />\n",
        escape(name, false)
    );
    let mut files = vec![TemplateFile {
        path: format!("{}.{MANIFEST_EXTENSION}", file_stem(name)),
        content: crate::format_source(&manifest).unwrap_or(manifest),
    }];
    files.extend(example.files.iter().map(|(path, content)| TemplateFile {
        path: (*path).to_owned(),
        content: (*content).to_owned(),
    }));
    Ok(Template {
        files,
        directories: vec![RESOURCES_DIR.to_owned()],
    })
}

/// Monta o modelo *Aplicativo Desktop FERRO* para um projeto com este nome.
///
/// O modelo cria o manifesto, a tela inicial e a pasta de recursos. O
/// code-behind `.fer` não é criado: uma tela sem `.fer` é uma tela sem
/// comportamento, e o designer cria o arquivo no primeiro duplo clique num
/// controle (ADR 0018, Emenda 2).
pub fn desktop_template(name: &str) -> Result<Template, String> {
    validate_project_name(name)?;
    let name = name.trim();
    let manifest = format!(
        "<Projeto Nome=\"{}\" Versao=\"0.1.0\" JanelaInicial=\"{DEFAULT_WINDOW}\" />\n",
        escape(name, false)
    );
    let design = format!(
        "<Janela Nome=\"{DEFAULT_WINDOW}\" Titulo=\"{}\" Largura=\"400\" Altura=\"300\">\n\
         <!-- Arraste controles da caixa de ferramentas para a janela. -->\n\
         <Rotulo Nome=\"lblBoasVindas\" X=\"16\" Y=\"16\" Texto=\"Olá! Esta é a sua primeira tela.\" />\n\
         </Janela>\n",
        escape(name, true)
    );
    let canonical = |source: String| {
        crate::format_source(&source).map_err(|d| format!("modelo inválido: {}", d.message))
    };
    Ok(Template {
        files: vec![
            TemplateFile {
                path: format!("{}.{MANIFEST_EXTENSION}", file_stem(name)),
                content: canonical(manifest)?,
            },
            TemplateFile {
                path: format!("{DEFAULT_WINDOW}.{}", crate::FILE_EXTENSION),
                content: canonical(design)?,
            },
        ],
        directories: vec![RESOURCES_DIR.to_owned()],
    })
}

/// Escapa texto para um valor de atributo. Com `braces`, também escapa as
/// chaves, que num atributo de texto do `.design` abririam um vínculo.
fn escape(text: &str, braces: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            '{' if braces => out.push_str("\\{"),
            '}' if braces => out.push_str("\\}"),
            other => out.push(other),
        }
    }
    out
}
