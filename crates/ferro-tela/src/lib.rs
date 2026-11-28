//! Formato `.design`: o desenho de uma tela de um projeto desktop FERRO, em
//! pseudo-XAML (ADR 0018). Uma tela é a junção de `Nome.design` com o
//! code-behind `Nome.fer`; este crate cuida só do `.design`.
//!
//! ```xml
//! <Janela Nome="JanelaPrincipal" Titulo="Calculadora" Largura="360" Altura="240" Dados="Conta">
//!     <Pilha Margem="16" Espaco="8">
//!         <CaixaTexto Nome="txtA" TextoVazio="Primeiro número" />
//!         <Botao Nome="btnSomar" Texto="Somar" AoClicar="btnSomar_clicar" />
//!         <Rotulo Nome="lblResultado" Texto="{dados.mensagem}" />
//!     </Pilha>
//! </Janela>
//! ```
//!
//! O crate não depende de UI nem do sistema operacional e compila para
//! `wasm32-unknown-unknown`, como o restante do núcleo FERRO. Os spans seguem a
//! convenção de [`ferro_hir::Span`]: bytes UTF-8, intervalo `[start, end)`.

pub mod ast;
pub mod codebehind;
pub mod edit;
mod format;
mod parser;
pub mod project;
pub mod schema;
pub mod slint;
mod validate;
pub mod value;

pub use ast::{Attribute, Comment, Document, Element, Name, Node, Text};
pub use ferro_hir::{Diagnostic, Severity, Span};
pub use format::format;
pub use parser::parse;
pub use validate::{MAX_DEPTH, MAX_ELEMENTS, validate};

/// Versão do formato que este crate lê e escreve (`Formato="1"`).
pub const CURRENT_VERSION: u32 = 1;

/// Extensão dos arquivos de desenho, sem o ponto.
pub const FILE_EXTENSION: &str = ferro_formatos::extension(ferro_formatos::Kind::Design);

/// Resultado de [`analyze`]: a árvore, quando o arquivo é sintaticamente
/// válido, e todos os diagnósticos encontrados.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub document: Option<Document>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Analysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// Lê e valida um arquivo `.design`.
pub fn analyze(source: &str) -> Analysis {
    match parse(source) {
        Ok(document) => {
            let mut diagnostics = validate(&document);
            diagnostics.extend(codebehind::screen_type(&document).1);
            Analysis {
                document: Some(document),
                diagnostics,
            }
        }
        Err(diagnostic) => Analysis {
            document: None,
            diagnostics: vec![diagnostic],
        },
    }
}

/// Reescreve o texto na forma canônica. Erros de validação não impedem a
/// formatação; erros de sintaxe, sim.
pub fn format_source(source: &str) -> Result<String, Diagnostic> {
    parse(source).map(|document| format(&document))
}
