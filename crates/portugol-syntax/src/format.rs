//! Formatação de espaços entre tokens, preservando comentários e literais.

use std::collections::{HashMap, HashSet};

use crate::ast::{Declarations, Declarator, ForInit, Item, Stmt};
use crate::lexer::lex;
use crate::token::{Token, TokenKind as Kind};

#[derive(Debug, PartialEq, Eq)]
pub struct WhitespaceEdit {
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

#[derive(Default)]
struct Layout {
    breaks: HashSet<u32>,
    inline_braces: HashSet<u32>,
    indentation: HashMap<u32, isize>,
}

impl Layout {
    fn declarations(&mut self, declarations: &Declarations, tokens: &[Token]) {
        for declaration in &declarations.declarators {
            if matches!(declaration, Declarator::Variable { .. }) {
                continue;
            }
            let span = declaration.span();
            let first = tokens.partition_point(|token| token.span.start < span.start);
            for token in &tokens[first..] {
                if token.span.start >= span.end {
                    break;
                }
                if matches!(token.kind, Kind::LBrace | Kind::RBrace) {
                    self.inline_braces.insert(token.span.start);
                }
            }
        }
    }

    fn indent_body(&mut self, body: &[Stmt], tokens: &[Token]) {
        let Some(first) = body.first() else { return };
        let span = first.span();
        let index = tokens.partition_point(|token| token.span.start < span.start);
        if index > 0 && tokens[index - 1].kind == Kind::LBrace {
            return;
        }
        let end = body.last().unwrap().span().end;
        let next = tokens.partition_point(|token| token.span.start < end);
        *self.indentation.entry(span.start).or_default() += 1;
        *self.indentation.entry(tokens[next].span.start).or_default() -= 1;
    }

    fn statements(&mut self, statements: &[Stmt], tokens: &[Token]) {
        for statement in statements {
            self.breaks.insert(statement.span().start);
            match statement {
                Stmt::Declarations(declarations) => self.declarations(declarations, tokens),
                Stmt::If {
                    then_branch,
                    else_branch,
                    ..
                } => {
                    self.indent_body(then_branch, tokens);
                    self.statements(then_branch, tokens);
                    if let Some(body) = else_branch {
                        if !matches!(body.as_slice(), [Stmt::If { .. }]) {
                            self.indent_body(body, tokens);
                        }
                        self.statements(body, tokens);
                    }
                }
                Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => {
                    self.indent_body(body, tokens);
                    self.statements(body, tokens);
                }
                Stmt::For { init, body, .. } => {
                    if let Some(ForInit::Declarations(declarations)) = init {
                        self.declarations(declarations, tokens);
                    }
                    self.indent_body(body, tokens);
                    self.statements(body, tokens);
                }
                Stmt::Switch { cases, .. } => {
                    for case in cases {
                        self.indent_body(&case.body, tokens);
                        self.statements(&case.body, tokens);
                    }
                }
                Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Assign(_) | Stmt::Expr { .. } => {}
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Syntax(Kind),
    BlockOpen,
    BlockClose,
    LineComment,
    BlockComment,
}

fn spacing(left: Piece, right: Piece) -> &'static str {
    use Kind::*;
    use Piece::*;
    match (left, right) {
        (Syntax(Se | Enquanto | Para | Escolha | Retorne | Nao), Syntax(LParen)) => " ",
        (Syntax(Ident), Syntax(LParen | LBracket)) => "",
        (Syntax(RBracket), Syntax(LBracket)) => "",
        (_, Syntax(RParen | RBracket | Comma | Semi | Dot | Colon | Eof))
        | (Syntax(LParen | LBracket | Dot), _) => "",
        _ => " ",
    }
}

/// Returns whitespace-only edits in UTF-16 coordinates, or `None` for invalid syntax.
pub fn format(source: &str) -> Option<Vec<WhitespaceEdit>> {
    let parsed = crate::parse(source);
    if parsed.has_errors() {
        return None;
    }
    let tokens = lex(source).tokens;
    let mut layout = Layout::default();
    for include in &parsed.program.includes {
        layout.breaks.insert(include.span.start);
    }
    for item in &parsed.program.items {
        match item {
            Item::Function(function) => {
                layout.breaks.insert(function.span.start);
                layout.statements(&function.body, &tokens);
            }
            Item::Declarations(declarations) => {
                layout.breaks.insert(declarations.span.start);
                layout.declarations(declarations, &tokens);
            }
            Item::Stmt(statement) => layout.statements(std::slice::from_ref(statement), &tokens),
        }
    }

    let mut pieces = Vec::new();
    let mut cursor = 0;
    for token in &tokens {
        let start = token.span.start as usize;
        while cursor < start {
            let trivia = &source[cursor..start];
            cursor += trivia.len() - trivia.trim_start().len();
            if cursor == start {
                break;
            }
            let comment = &source[cursor..start];
            let (kind, length) = if comment.starts_with("//") {
                (
                    Piece::LineComment,
                    comment.find('\n').unwrap_or(comment.len()),
                )
            } else {
                (Piece::BlockComment, comment.find("*/")? + 2)
            };
            pieces.push((cursor, cursor + length, kind));
            cursor += length;
        }
        let kind = match token.kind {
            Kind::LBrace if !layout.inline_braces.contains(&token.span.start) => Piece::BlockOpen,
            Kind::RBrace if !layout.inline_braces.contains(&token.span.start) => Piece::BlockClose,
            kind => Piece::Syntax(kind),
        };
        cursor = token.span.end as usize;
        pieces.push((start, cursor, kind));
    }

    let mut edits = Vec::new();
    let mut rendered = String::new();
    let mut depth = 0isize;
    let mut previous = None;
    let mut end = 0;
    let mut offset = 0;
    for (start, next_end, piece) in pieces {
        depth += layout
            .indentation
            .get(&(start as u32))
            .copied()
            .unwrap_or(0);
        if piece == Piece::BlockClose {
            depth -= 1;
        }
        let original = &source[end..start];
        let replacement = match previous {
            None => String::new(),
            Some(_) if piece == Piece::Syntax(Kind::Eof) => "\n".into(),
            Some(left) => {
                let trailing_comment = matches!(piece, Piece::LineComment | Piece::BlockComment)
                    && !original.contains('\n');
                let control_break = matches!(
                    piece,
                    Piece::BlockOpen | Piece::BlockClose | Piece::Syntax(Kind::Caso | Kind::Senao)
                ) || matches!(
                    left,
                    Piece::BlockOpen | Piece::BlockClose | Piece::LineComment
                ) || layout.breaks.contains(&(start as u32))
                    && !(left == Piece::Syntax(Kind::Senao) && piece == Piece::Syntax(Kind::Se));
                if !trailing_comment && (control_break || original.contains('\n')) {
                    let newlines = if original.matches('\n').count() > 1 {
                        "\n\n"
                    } else {
                        "\n"
                    };
                    format!("{newlines}{}", "\t".repeat(depth.max(0) as usize))
                } else {
                    spacing(left, piece).into()
                }
            }
        };
        let gap_length = original.encode_utf16().count();
        if original != replacement {
            edits.push(WhitespaceEdit {
                from: offset,
                to: offset + gap_length,
                insert: replacement.clone(),
            });
        }
        let text = &source[start..next_end];
        rendered.push_str(&replacement);
        rendered.push_str(text);
        offset += gap_length + text.encode_utf16().count();
        end = next_end;
        previous = Some(piece);
        if piece == Piece::BlockOpen {
            depth += 1;
        }
    }

    // Não separam nem se fundem operadores ou outros tokens durante a edição.
    let formatted = lex(&rendered);
    let token_text = |token: &Token| &source[token.span.start as usize..token.span.end as usize];
    let same_tokens = formatted.diagnostics.is_empty()
        && formatted.tokens.len() == tokens.len()
        && tokens.iter().zip(&formatted.tokens).all(|(before, after)| {
            before.kind == after.kind
                && token_text(before)
                    == &rendered[after.span.start as usize..after.span.end as usize]
        });
    same_tokens.then_some(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn apply(source: &str, edits: Vec<WhitespaceEdit>) -> String {
        let mut units: Vec<_> = source.encode_utf16().collect();
        for edit in edits.into_iter().rev() {
            units.splice(edit.from..edit.to, edit.insert.encode_utf16());
        }
        String::from_utf16(&units).unwrap()
    }

    #[test]
    fn formats_compact_program_without_changing_comments_or_literals() {
        let compact = "programa{funcao inicio(){inteiro v[]={1,2}/* 🪁 */escreva(\"a  b\",v[0])}}";
        let expected = "programa\n{\n\tfuncao inicio()\n\t{\n\t\tinteiro v[] = { 1, 2 } /* 🪁 */\n\t\tescreva(\"a  b\", v[0])\n\t}\n}\n";
        assert_eq!(apply(compact, format(compact).unwrap()), expected);
        assert_eq!(format(expected), Some(Vec::new()));
    }

    #[test]
    fn indents_control_bodies_and_does_not_edit_invalid_syntax() {
        let compact = "programa { funcao inicio() { se (verdadeiro) escreva(1) senao escreva(2) escolha (0) { caso 0: pare caso contrario: pare } } }";
        let formatted = apply(compact, format(compact).unwrap());
        assert!(
            formatted
                .contains("\t\tse (verdadeiro)\n\t\t\tescreva(1)\n\t\tsenao\n\t\t\tescreva(2)")
        );
        assert!(
            formatted.contains("\t\t\tcaso 0:\n\t\t\t\tpare\n\t\t\tcaso contrario:\n\t\t\t\tpare")
        );
        assert_eq!(format(&formatted), Some(Vec::new()));
        assert!(format("programa { funcao inicio(").is_none());
    }
}
