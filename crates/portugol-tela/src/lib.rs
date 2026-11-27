//! Formato `.tela`: a descrição de uma janela de um projeto desktop PORS
//! (ADR 0018).
//!
//! ```text
//! versao 1
//!
//! janela JanelaPrincipal {
//!     titulo: "Calculadora"
//!     largura: 360
//!     altura: 240
//!
//!     caixa_texto txtA { x: 16  y: 16  largura: 120 }
//!     botao btnSomar { x: 16  y: 92  texto: "Somar"  ao_clicar: btnSomar_clicar }
//!     rotulo lblResultado { x: 16  y: 132  texto: "Resultado: -" }
//! }
//! ```
//!
//! O crate não depende de UI nem do sistema operacional e compila para
//! `wasm32-unknown-unknown`, como o restante do núcleo PORS. Os spans seguem a
//! convenção de [`portugol_hir::Span`]: bytes UTF-8, intervalo `[start, end)`.

pub mod ast;
mod format;
mod lexer;
mod parser;
pub mod schema;
mod validate;

pub use ast::{Body, Control, ListItem, Name, Property, Screen, Value, Window};
pub use format::format;
pub use parser::parse;
pub use portugol_hir::{Diagnostic, Severity, Span};
pub use validate::{MAX_CONTROLS, MAX_DEPTH, validate};

/// Versão do esquema que este crate lê e escreve.
pub const CURRENT_VERSION: u32 = 1;

/// Resultado de [`analyze`]: a árvore, quando o arquivo é sintaticamente
/// válido, e todos os diagnósticos encontrados.
#[derive(Debug, Clone)]
pub struct Analysis {
    pub screen: Option<Screen>,
    pub diagnostics: Vec<Diagnostic>,
}

impl Analysis {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// Lê e valida um arquivo `.tela`.
pub fn analyze(source: &str) -> Analysis {
    match parse(source) {
        Ok(screen) => {
            let diagnostics = validate(&screen);
            Analysis {
                screen: Some(screen),
                diagnostics,
            }
        }
        Err(diagnostic) => Analysis {
            screen: None,
            diagnostics: vec![diagnostic],
        },
    }
}

/// Reescreve o texto na forma canônica. Erros de validação não impedem a
/// formatação; erros de sintaxe, sim.
pub fn format_source(source: &str) -> Result<String, Diagnostic> {
    parse(source).map(|screen| format(&screen))
}
