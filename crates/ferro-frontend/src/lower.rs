use ferro_hir::Diagnostic;
use ferro_hir::Program;

use crate::{lexer, parser::Parser};

/// Analisa sintaxe e regras iniciais de Ferro e produz HIR tipada.
pub fn lower(source: &str) -> Result<Program, Vec<Diagnostic>> {
    lower_with(source, &LowerOptions::default())
}

/// Tipo de um campo de registro sintetizado fora do código-fonte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExternalType {
    Integer,
    Boolean,
    String,
    VectorString,
    /// Registro ou enum pelo nome: outro registro externo ou um tipo
    /// declarado no próprio arquivo (resolvido depois de lê-lo).
    Named(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalField {
    pub name: String,
    pub ty: ExternalType,
}

/// Registro que existe para o programa sem estar no código-fonte, como a
/// janela de uma tela `.design` vista pelo code-behind (ADR 0018).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExternalRecord {
    pub name: String,
    pub fields: Vec<ExternalField>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LowerOptions {
    /// Registros registrados antes do código, na ordem dada; um registro só
    /// pode citar pelo nome os externos que vêm antes dele.
    pub records: Vec<ExternalRecord>,
    /// Exige `funcao inicio()`. O code-behind de uma tela não tem `inicio`:
    /// quem chama os handlers é a janela.
    pub require_entry: bool,
}

impl Default for LowerOptions {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            require_entry: true,
        }
    }
}

/// [`lower`] com registros externos e a exigência de `inicio` configurável.
pub fn lower_with(source: &str, options: &LowerOptions) -> Result<Program, Vec<Diagnostic>> {
    lower_with_warnings(source, options).map(|(program, _)| program)
}

/// [`lower_with`] que também devolve os avisos: o programa roda, mas algo
/// merece atenção (como o `UI.*` obsoleto do ADR 0021).
pub fn lower_with_warnings(
    source: &str,
    options: &LowerOptions,
) -> Result<(Program, Vec<Diagnostic>), Vec<Diagnostic>> {
    let (tokens, diagnostics) = lexer::lex(source);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    Parser::new(tokens, source.to_owned())
        .with_options(options)
        .program_with_warnings()
}
