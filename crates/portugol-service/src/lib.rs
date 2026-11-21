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

pub use exercises::{
    Exercise, ExerciseResult, check_exercise, check_imported_exercise, exercises, inspect_exercise,
};
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
pub struct TextEdit {
    from: usize,
    to: usize,
    insert: String,
}

pub fn format_source(source: &str) -> Option<Vec<TextEdit>> {
    portugol_syntax::format::format(source).map(|edits| {
        edits
            .into_iter()
            .map(|edit| TextEdit {
                from: edit.from,
                to: edit.to,
                insert: edit.insert,
            })
            .collect()
    })
}

/// Inserção no começo do bloco programa. Usa o lexer para ignorar texto em comentários.
pub fn include_library(source: &str, name: &str) -> Result<Option<TextEdit>, String> {
    use portugol_syntax::lexer::lex;
    use portugol_syntax::token::TokenKind;
    let proposed = lex(name);
    if proposed.tokens.len() != 2
        || proposed.tokens[0].kind != TokenKind::Ident
        || proposed.tokens[0].span.len() as usize != name.len()
    {
        return Err("Biblioteca inválida.".into());
    }
    let parsed = parse(source);
    if parsed
        .program
        .includes
        .iter()
        .any(|include| include.library.name == name)
    {
        return Ok(None);
    }
    let tokens = lex(source).tokens;
    let offset = if let Some(program) = tokens
        .iter()
        .position(|token| token.kind == TokenKind::Programa)
    {
        if tokens[program + 1].kind != TokenKind::LBrace {
            return Err("Adicione a chave de abertura do programa.".into());
        }
        tokens[program + 1].span.end as usize
    } else {
        0
    };
    let insert = if offset == 0 {
        format!("inclua biblioteca {name}\n")
    } else if source[offset..].starts_with('\n') {
        format!("\n\tinclua biblioteca {name}")
    } else {
        format!("\n\tinclua biblioteca {name}\n")
    };
    let from = position::LineIndex::new(source)
        .position(offset as u32)
        .offset as usize;
    Ok(Some(TextEdit {
        from,
        to: from,
        insert,
    }))
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

#[cfg(test)]
mod library_edits {
    use super::include_library;

    #[test]
    fn inserts_inside_the_program_and_respects_existing_aliases() {
        let source = "// 🪁 programa { }\nprograma\n{\n\tfuncao inicio() {}\n}";
        let edit = include_library(source, "Matematica").unwrap().unwrap();
        let expected = source[..source.find("{\n\t").unwrap() + 1]
            .encode_utf16()
            .count();
        assert_eq!((edit.from, edit.to), (expected, expected));
        assert_eq!(edit.insert, "\n\tinclua biblioteca Matematica");
        let source = "programa { inclua biblioteca Matematica --> m funcao inicio() {} }";
        assert!(include_library(source, "Matematica").unwrap().is_none());
    }
}
