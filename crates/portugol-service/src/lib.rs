//! Fachada do núcleo para a IDE.
//!
//! * [`check`]: sintaxe + semântica, com posições prontas para o editor
//!   (offsets em unidades UTF-16, como o CodeMirror usa);
//! * [`start`]: compila e executa o programa numa thread própria, entregando
//!   saída, pedidos de `leia` e o resultado a um [`RunObserver`].
//!
//! Não depende do Tauri: a camada da janela só traduz os eventos.

mod position;
mod run;

use serde::Serialize;

use portugol_syntax::{Diagnostic, Severity, parse};

pub use portugol_vm::FileFilter;
pub use position::Position;
pub use run::{RunHandle, RunObserver, RunOptions, RunOutcome, start};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

/// Diagnóstico serializável para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticInfo {
    pub severity: Level,
    pub code: String,
    pub message: String,
    pub help: Option<String>,
    pub start: Position,
    pub end: Position,
}

/// Analisa o código como a IDE faz a cada edição. A semântica só roda
/// quando não há erro de sintaxe, para não gerar erros em cascata.
pub fn check(source: &str) -> Vec<DiagnosticInfo> {
    let (diagnostics, _) = analyze(source);
    let index = position::LineIndex::new(source);
    diagnostics
        .iter()
        .map(|diagnostic| describe(&index, diagnostic))
        .collect()
}

/// Sintaxe + semântica; devolve também a AST quando não há erros.
pub(crate) fn analyze(source: &str) -> (Vec<Diagnostic>, Option<portugol_syntax::Program>) {
    let result = parse(source);
    let mut diagnostics = result.diagnostics;
    if !has_errors(&diagnostics) {
        diagnostics.extend(portugol_semantics::analyze(&result.program));
    }
    let program = (!has_errors(&diagnostics)).then_some(result.program);
    (diagnostics, program)
}

pub(crate) fn describe(index: &position::LineIndex, diagnostic: &Diagnostic) -> DiagnosticInfo {
    DiagnosticInfo {
        severity: match diagnostic.severity {
            Severity::Error => Level::Error,
            Severity::Warning => Level::Warning,
        },
        code: diagnostic.code.to_string(),
        message: diagnostic.message.clone(),
        help: diagnostic.help.clone(),
        start: index.position(diagnostic.span.start),
        end: index.position(diagnostic.span.end.max(diagnostic.span.start)),
    }
}

fn has_errors(diagnostics: &[Diagnostic]) -> bool {
    diagnostics.iter().any(|d| d.severity == Severity::Error)
}
