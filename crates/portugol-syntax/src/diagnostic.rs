//! Diagnósticos produzidos pelo lexer, parser e (futuramente) pelo analisador
//! semântico.

use crate::span::{Span, line_col};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Error,
    Warning,
}

impl fmt::Display for Severity {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Severity::Error => f.write_str("erro"),
            Severity::Warning => f.write_str("aviso"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub severity: Severity,
    /// Código estável, útil para testes e para a UI (ex.: `syntax.expected-token`).
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    /// Dica opcional exibida abaixo da mensagem.
    pub help: Option<String>,
}

impl Diagnostic {
    pub fn error(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Error,
            code,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn warning(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            severity: Severity::Warning,
            code,
            message: message.into(),
            span,
            help: None,
        }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Renderiza no formato `arquivo:linha:coluna: erro: mensagem`.
    pub fn render(&self, source: &str, file: &str) -> String {
        let (line, column) = line_col(source, self.span.start);
        let mut out = format!(
            "{}:{}:{}: {}[{}]: {}",
            file, line, column, self.severity, self.code, self.message
        );

        if let Some(help) = &self.help {
            out.push_str("\n  ajuda: ");
            out.push_str(help);
        }

        out
    }
}
