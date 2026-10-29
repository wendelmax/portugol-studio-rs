//! Núcleo de análise sintática do Portugol 2.0.
//!
//! Ponto de entrada principal: [`parse`].
//!
//! ```no_run
//! let fonte = "programa { funcao inicio() { inteiro x = 1 } }";
//! let resultado = portugol_syntax::parse(fonte);
//! assert!(resultado.diagnostics.is_empty());
//! ```

pub mod ast;
pub mod diagnostic;
pub mod lexer;
pub mod parser;
pub mod span;
pub mod token;

pub use ast::Program;
pub use diagnostic::{Diagnostic, Severity};
pub use span::Span;

/// Resultado de uma análise sintática completa (lexer + parser).
#[derive(Debug)]
pub struct ParseResult {
    pub program: Program,
    pub diagnostics: Vec<Diagnostic>,
}

impl ParseResult {
    pub fn has_errors(&self) -> bool {
        self.diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    }
}

/// Analisa um código-fonte Portugol e devolve a AST junto com os diagnósticos.
///
/// A análise nunca entra em pânico por causa do código do usuário: erros viram
/// diagnósticos e o parser tenta se recuperar para continuar analisando.
pub fn parse(source: &str) -> ParseResult {
    let lexed = lexer::lex(source);
    let (program, diagnostics) =
        parser::Parser::new(source, &lexed.tokens, lexed.diagnostics).parse_file();

    ParseResult {
        program,
        diagnostics,
    }
}
