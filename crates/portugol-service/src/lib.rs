//! Fachada do núcleo para a IDE.
//!
//! * [`check`]: sintaxe + semântica, com posições prontas para o editor
//!   (offsets em unidades UTF-16, como o CodeMirror usa);
//! * [`start`]: compila e executa o programa numa thread própria, entregando
//!   saída, pedidos de `leia` e o resultado a um [`RunObserver`].
//!
//! Não depende do Tauri: a camada da janela só traduz os eventos.

mod exercises;
mod outline;
mod position;
mod rename;
mod run;

use serde::Serialize;

use portugol_syntax::{Diagnostic, Severity, parse};

pub use exercises::{Exercise, ExerciseResult, check_exercise, exercises};
pub use outline::{SymbolInfo, SymbolKind};
pub use portugol_vm::{FileFilter, Host, Sound, graphics};
pub use position::Position;
pub use rename::{RenameEdit, RenameTarget, rename_symbol, rename_target};
pub use run::{
    DebugFrameInfo, DebugInfo, DebugVarInfo, RunHandle, RunObserver, RunOptions, RunOutcome, start,
    start_with_host,
};

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

#[derive(Serialize)]
pub struct FormatEdit {
    from: usize,
    to: usize,
    insert: String,
}

pub fn format_source(source: &str) -> Option<Vec<FormatEdit>> {
    portugol_syntax::format::format(source).map(|edits| {
        edits
            .into_iter()
            .map(|edit| FormatEdit {
                from: edit.from,
                to: edit.to,
                insert: edit.insert,
            })
            .collect()
    })
}

#[derive(Serialize)]
pub struct Inspection {
    pub diagnostics: Vec<DiagnosticInfo>,
    pub symbols: Vec<SymbolInfo>,
}

/// Analisa o código como a IDE faz a cada edição.
pub fn check(source: &str) -> Vec<DiagnosticInfo> {
    let parsed = analyze_source(source);
    let index = position::LineIndex::new(source);
    present_diagnostics(&index, &parsed.diagnostics)
}

/// Retorna também a estrutura da AST recuperada pelo parser durante a edição.
pub fn inspect(source: &str) -> Inspection {
    let parsed = analyze_source(source);
    let index = position::LineIndex::new(source);
    Inspection {
        diagnostics: present_diagnostics(&index, &parsed.diagnostics),
        symbols: outline::outline(&parsed.program, &index),
    }
}

fn present_diagnostics(
    index: &position::LineIndex,
    diagnostics: &[Diagnostic],
) -> Vec<DiagnosticInfo> {
    diagnostics
        .iter()
        .map(|diagnostic| describe(index, diagnostic))
        .collect()
}

fn analyze_source(source: &str) -> portugol_syntax::ParseResult {
    let mut parsed = parse(source);
    if !parsed.has_errors() {
        parsed
            .diagnostics
            .extend(portugol_semantics::analyze(&parsed.program));
    }
    parsed
}

/// Sintaxe + semântica; devolve também a AST quando não há erros.
pub(crate) fn analyze(source: &str) -> (Vec<Diagnostic>, Option<portugol_syntax::Program>) {
    let parsed = analyze_source(source);
    let program = (!parsed.has_errors()).then_some(parsed.program);
    (parsed.diagnostics, program)
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
