use portugol_hir::Diagnostic;
use portugol_hir::Program;

use crate::{lexer, parser::Parser};

/// Analisa sintaxe e regras iniciais de Portugol-RS e produz HIR tipada.
pub fn lower(source: &str) -> Result<Program, Vec<Diagnostic>> {
    let (tokens, diagnostics) = lexer::lex(source);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Parser::new(tokens, source.to_owned()).program()
}
