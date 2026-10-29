//! Lexer baseado em `logos`.
//!
//! O lexer é tolerante: caracteres inválidos viram diagnóstico e a análise
//! continua, pois a IDE precisa mostrar o máximo de erros possível de uma vez.

use crate::diagnostic::Diagnostic;
use crate::span::Span;
use crate::token::{Token, TokenKind};
use logos::Logos;

pub struct Lexed {
    pub tokens: Vec<Token>,
    pub diagnostics: Vec<Diagnostic>,
}

pub fn lex(source: &str) -> Lexed {
    let mut lexer = TokenKind::lexer(source);
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    while let Some(result) = lexer.next() {
        let range = lexer.span();
        let span = Span::new(range.start as u32, range.end as u32);

        match result {
            Ok(kind) => tokens.push(Token { kind, span }),
            Err(()) => {
                let invalid = &source[range.clone()];
                diagnostics.push(Diagnostic::error(
                    "lex.invalid-char",
                    format!("caractere inválido: {:?}", invalid),
                    span,
                ));
            }
        }
    }

    let end = source.len() as u32;
    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(end, end),
    });

    Lexed {
        tokens,
        diagnostics,
    }
}
