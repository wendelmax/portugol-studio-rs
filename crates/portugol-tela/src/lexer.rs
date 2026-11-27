//! Análise léxica do formato `.tela`.

use portugol_hir::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum Kind {
    Ident(String),
    Integer(i64),
    String(String),
    /// Comentário de linha (`// ...`), sem o prefixo e sem espaços nas pontas.
    /// `own_line` é falso quando há outro token antes dele na mesma linha.
    Comment {
        text: String,
        own_line: bool,
    },
    LBrace,
    RBrace,
    LBracket,
    RBracket,
    Colon,
    Comma,
    Eof,
}

#[derive(Debug, Clone)]
pub(crate) struct Token {
    pub kind: Kind,
    pub span: Span,
}

pub(crate) fn lex(source: &str) -> Result<Vec<Token>, Diagnostic> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    // Há token na linha atual antes da posição corrente?
    let mut line_has_token = false;

    while index < bytes.len() {
        let start = index;
        let ch = source[index..].chars().next().unwrap_or('\0');

        if ch == '\n' {
            line_has_token = false;
            index += 1;
            continue;
        }
        if ch.is_whitespace() {
            index += ch.len_utf8();
            continue;
        }

        if source[index..].starts_with("//") {
            let end = source[index..]
                .find('\n')
                .map_or(source.len(), |offset| index + offset);
            let text = source[index + 2..end].trim().to_owned();
            tokens.push(Token {
                kind: Kind::Comment {
                    text,
                    own_line: !line_has_token,
                },
                span: span(start, end),
            });
            index = end;
            continue;
        }

        line_has_token = true;

        let simple = match ch {
            '{' => Some(Kind::LBrace),
            '}' => Some(Kind::RBrace),
            '[' => Some(Kind::LBracket),
            ']' => Some(Kind::RBracket),
            ':' => Some(Kind::Colon),
            ',' => Some(Kind::Comma),
            _ => None,
        };
        if let Some(kind) = simple {
            index += 1;
            tokens.push(Token {
                kind,
                span: span(start, index),
            });
            continue;
        }

        if ch == '"' {
            let (value, end) = lex_string(source, index)?;
            index = end;
            tokens.push(Token {
                kind: Kind::String(value),
                span: span(start, index),
            });
            continue;
        }

        let negative = ch == '-' && bytes.get(index + 1).is_some_and(u8::is_ascii_digit);
        if ch.is_ascii_digit() || negative {
            index += 1;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            let text = &source[start..index];
            let value = text.parse::<i64>().map_err(|_| {
                Diagnostic::error(
                    "tela.invalid-number",
                    format!("Número fora do intervalo suportado: {text}"),
                    span(start, index),
                )
            })?;
            tokens.push(Token {
                kind: Kind::Integer(value),
                span: span(start, index),
            });
            continue;
        }

        if ch.is_alphabetic() || ch == '_' {
            let mut end = index;
            for c in source[index..].chars() {
                if c.is_alphanumeric() || c == '_' {
                    end += c.len_utf8();
                } else {
                    break;
                }
            }
            tokens.push(Token {
                kind: Kind::Ident(source[index..end].to_owned()),
                span: span(start, end),
            });
            index = end;
            continue;
        }

        return Err(Diagnostic::error(
            "tela.unexpected-character",
            format!("Caractere inesperado: `{ch}`"),
            span(start, start + ch.len_utf8()),
        ));
    }

    tokens.push(Token {
        kind: Kind::Eof,
        span: span(source.len(), source.len()),
    });
    Ok(tokens)
}

fn lex_string(source: &str, start: usize) -> Result<(String, usize), Diagnostic> {
    let mut value = String::new();
    let mut chars = source[start + 1..].char_indices();
    while let Some((offset, c)) = chars.next() {
        let position = start + 1 + offset;
        match c {
            '"' => return Ok((value, position + 1)),
            '\n' => break,
            '\\' => match chars.next() {
                Some((_, 'n')) => value.push('\n'),
                Some((_, 't')) => value.push('\t'),
                Some((_, '"')) => value.push('"'),
                Some((_, '\\')) => value.push('\\'),
                Some((escape_offset, other)) => {
                    let escape_start = start + 1 + escape_offset - 1;
                    return Err(Diagnostic::error(
                        "tela.invalid-escape",
                        format!("Sequência de escape desconhecida: `\\{other}`"),
                        span(escape_start, escape_start + 1 + other.len_utf8()),
                    )
                    .with_help("Use \\n, \\t, \\\" ou \\\\."));
                }
                None => break,
            },
            other => value.push(other),
        }
    }
    let end = source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset);
    Err(Diagnostic::error(
        "tela.unterminated-string",
        "Cadeia sem aspas de fechamento",
        span(start, end),
    ))
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start as u32, end as u32)
}
