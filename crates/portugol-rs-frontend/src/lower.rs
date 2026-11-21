use portugol_hir::Program;
use portugol_syntax::Diagnostic;

use crate::{lexer, parser::Parser};

/// Analisa sintaxe e regras iniciais de Portugol-RS e produz HIR tipada.
pub fn lower(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let (tokens, diagnostics) = lexer::lex(source);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Parser::new(tokens).program()
}
