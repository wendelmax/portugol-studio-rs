//! Formatação canônica de espaços para Ferro (`.pors`), preservando
//! comentários, literais e semântica.

use crate::lexer::{Kind, Token, lex};

const INDENT: &str = "    ";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WhitespaceEdit {
    pub from: usize,
    pub to: usize,
    pub insert: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SyntaxKind {
    Funcao,
    Se,
    Senao,
    Faca,
    Enquanto,
    Para,
    Retorne,
    Ident,
    Literal,
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    ColonColon,
    Dot,
    Semicolon,
    Comma,
    Arrow,
    FatArrow,
    UnarySign,
    Bang,
    Question,
    Operator,
    Eof,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Piece {
    Syntax(SyntaxKind),
    LineComment,
    BlockComment,
}

fn classify(kind: &Kind, prev_syntax: Option<SyntaxKind>) -> SyntaxKind {
    match kind {
        Kind::Ident(word) => match word.as_str() {
            "funcao" => SyntaxKind::Funcao,
            "se" => SyntaxKind::Se,
            "senao" => SyntaxKind::Senao,
            "faca" => SyntaxKind::Faca,
            "enquanto" => SyntaxKind::Enquanto,
            "para" => SyntaxKind::Para,
            "retorne" => SyntaxKind::Retorne,
            _ => SyntaxKind::Ident,
        },
        Kind::Integer(_) | Kind::Real(_) | Kind::String { .. } | Kind::Character(_) => {
            SyntaxKind::Literal
        }
        Kind::LParen => SyntaxKind::LParen,
        Kind::RParen => SyntaxKind::RParen,
        Kind::LBracket => SyntaxKind::LBracket,
        Kind::RBracket => SyntaxKind::RBracket,
        Kind::LBrace => SyntaxKind::LBrace,
        Kind::RBrace => SyntaxKind::RBrace,
        Kind::Colon => SyntaxKind::Colon,
        Kind::ColonColon => SyntaxKind::ColonColon,
        Kind::Dot => SyntaxKind::Dot,
        Kind::Semicolon => SyntaxKind::Semicolon,
        Kind::Comma => SyntaxKind::Comma,
        Kind::Arrow => SyntaxKind::Arrow,
        Kind::FatArrow => SyntaxKind::FatArrow,
        Kind::Bang | Kind::Tilde => SyntaxKind::Bang,
        Kind::Question => SyntaxKind::Question,
        Kind::Plus | Kind::Minus => {
            let unary = matches!(
                prev_syntax,
                None | Some(
                    SyntaxKind::LParen
                        | SyntaxKind::Comma
                        | SyntaxKind::Semicolon
                        | SyntaxKind::Arrow
                        | SyntaxKind::UnarySign
                        | SyntaxKind::Bang
                        | SyntaxKind::Operator
                        | SyntaxKind::LBrace
                        | SyntaxKind::Retorne
                )
            );
            if unary {
                SyntaxKind::UnarySign
            } else {
                SyntaxKind::Operator
            }
        }
        Kind::Star
        | Kind::Slash
        | Kind::Percent
        | Kind::PlusEq
        | Kind::MinusEq
        | Kind::StarEq
        | Kind::SlashEq
        | Kind::PercentEq
        | Kind::AmpEq
        | Kind::PipeEq
        | Kind::CaretEq
        | Kind::Assign
        | Kind::Equal
        | Kind::NotEqual
        | Kind::Greater
        | Kind::Less
        | Kind::GreaterEqual
        | Kind::LessEqual
        | Kind::Shl
        | Kind::Shr
        | Kind::Amp
        | Kind::Pipe
        | Kind::Caret
        | Kind::And
        | Kind::Or => SyntaxKind::Operator,
        Kind::Eof => SyntaxKind::Eof,
    }
}

fn spacing(left: Piece, right: Piece) -> &'static str {
    use Piece::*;
    use SyntaxKind::*;
    match (left, right) {
        (Syntax(Se | Enquanto | Para), Syntax(LParen)) => " ",
        (Syntax(Ident), Syntax(LParen)) => "",
        (Syntax(LParen | LBracket | Bang | UnarySign), _) => "",
        (Syntax(Ident | RBracket), Syntax(LBracket)) => "",
        (_, Syntax(RParen | RBracket | Comma | Semicolon | Colon | Question | Eof)) => "",
        (_, Syntax(Dot)) | (Syntax(Dot), _) => "",
        (_, Syntax(ColonColon)) | (Syntax(ColonColon), _) => "",
        (Syntax(LBrace), Syntax(RBrace)) => "",
        _ => " ",
    }
}

/// Retorna edições somente de espaço em coordenadas UTF-16, ou `None` quando o
/// fonte `.pors` possui erros de sintaxe/semântica.
pub fn format(source: &str) -> Option<Vec<WhitespaceEdit>> {
    crate::lower(source).ok()?;
    let (tokens, diagnostics) = lex(source);
    if !diagnostics.is_empty() {
        return None;
    }

    let mut pieces = Vec::new();
    let mut cursor = 0usize;
    let mut prev_syntax = None;

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
        let syntax = classify(&token.kind, prev_syntax);
        if syntax != SyntaxKind::Eof {
            prev_syntax = Some(syntax);
        }
        cursor = token.span.end as usize;
        pieces.push((start, cursor, Piece::Syntax(syntax)));
    }

    let mut edits = Vec::new();
    let mut rendered = String::new();
    let mut depth = 0isize;
    let mut paren_depth = 0usize;
    let mut previous: Option<Piece> = None;
    let mut end = 0usize;
    let mut offset = 0usize;

    for (start, next_end, piece) in pieces {
        if piece == Piece::Syntax(SyntaxKind::RBrace) {
            depth -= 1;
        }
        if piece == Piece::Syntax(SyntaxKind::RParen) {
            paren_depth = paren_depth.saturating_sub(1);
        }

        let original = &source[end..start];
        let replacement = match previous {
            None => String::new(),
            Some(_) if piece == Piece::Syntax(SyntaxKind::Eof) => "\n".into(),
            Some(left) => {
                let trailing_comment = !original.contains('\n')
                    && left != Piece::LineComment
                    && (piece == Piece::LineComment
                        || (piece == Piece::BlockComment
                            && left != Piece::Syntax(SyntaxKind::LBrace)));
                let empty_block = left == Piece::Syntax(SyntaxKind::LBrace)
                    && piece == Piece::Syntax(SyntaxKind::RBrace);
                let else_after_brace = left == Piece::Syntax(SyntaxKind::RBrace)
                    && matches!(
                        piece,
                        Piece::Syntax(SyntaxKind::Senao | SyntaxKind::Enquanto)
                    );
                let top_level_function_gap = depth == 0
                    && left == Piece::Syntax(SyntaxKind::RBrace)
                    && piece == Piece::Syntax(SyntaxKind::Funcao);
                let top_level_comment_gap = depth == 0
                    && left == Piece::Syntax(SyntaxKind::RBrace)
                    && matches!(piece, Piece::LineComment | Piece::BlockComment)
                    && original.contains('\n');
                let statement_break = left == Piece::Syntax(SyntaxKind::Semicolon)
                    && paren_depth == 0
                    && !trailing_comment;
                let block_break = matches!(
                    left,
                    Piece::Syntax(SyntaxKind::LBrace | SyntaxKind::RBrace) | Piece::LineComment
                ) || piece == Piece::Syntax(SyntaxKind::RBrace);

                if empty_block {
                    String::new()
                } else if else_after_brace {
                    " ".into()
                } else if top_level_function_gap || top_level_comment_gap {
                    "\n\n".into()
                } else if !trailing_comment
                    && (statement_break || block_break || original.contains('\n'))
                {
                    let newlines = if original.matches('\n').count() > 1 {
                        "\n\n"
                    } else {
                        "\n"
                    };
                    format!("{newlines}{}", INDENT.repeat(depth.max(0) as usize))
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

        if piece == Piece::Syntax(SyntaxKind::LBrace) {
            depth += 1;
        }
        if piece == Piece::Syntax(SyntaxKind::LParen) {
            paren_depth += 1;
        }
    }

    let (formatted_tokens, formatted_diagnostics) = lex(&rendered);
    let token_text = |token: &Token| &source[token.span.start as usize..token.span.end as usize];
    let same_tokens = formatted_diagnostics.is_empty()
        && formatted_tokens.len() == tokens.len()
        && tokens.iter().zip(&formatted_tokens).all(|(before, after)| {
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
    fn keeps_default_pors_template_canonically_formatted() {
        let template = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";
        assert_eq!(format(template), Some(Vec::new()));
    }

    #[test]
    fn formats_functions_loops_conditionals_unary_operators_and_comments() {
        let compact = "// Cabeçalho 🦀\nfuncao dobro(mutavel valor:inteiro)->inteiro{se(!falso){valor+=-1;retorne valor*2;}senao{retorne 0;}}funcao inicio(){para(seja mutavel i:inteiro=0;i<3;i+=1){/* 😀 */escreva(dobro(i),\"a  b\");}}";
        let expected = "// Cabeçalho 🦀\nfuncao dobro(mutavel valor: inteiro) -> inteiro {\n    se (!falso) {\n        valor += -1;\n        retorne valor * 2;\n    } senao {\n        retorne 0;\n    }\n}\n\nfuncao inicio() {\n    para (seja mutavel i: inteiro = 0; i < 3; i += 1) {\n        /* 😀 */ escreva(dobro(i), \"a  b\");\n    }\n}\n";
        let formatted = apply(compact, format(compact).expect("fonte válida"));
        assert_eq!(formatted, expected);
        assert_eq!(format(&formatted), Some(Vec::new()));
    }

    #[test]
    fn formats_chained_senao_se_without_adding_an_extra_block() {
        let source =
            "funcao inicio(){se(1==1){escreva(1);}senao se(1==2){escreva(2);}senao{escreva(3);}}";
        let expected = "funcao inicio() {\n    se (1 == 1) {\n        escreva(1);\n    } senao se (1 == 2) {\n        escreva(2);\n    } senao {\n        escreva(3);\n    }\n}\n";
        let formatted = apply(source, format(source).expect("fonte válida"));
        assert_eq!(formatted, expected);
        assert_eq!(format(&formatted), Some(Vec::new()));
    }

    #[test]
    fn preserves_empty_blocks_trailing_comments_and_void_return() {
        let compact = "funcao vazio(){retorne;}funcao inicio(){seja mutavel x:inteiro=1; // ok\nenquanto(falso){}vazio();}";
        let expected = "funcao vazio() {\n    retorne;\n}\n\nfuncao inicio() {\n    seja mutavel x: inteiro = 1; // ok\n    enquanto (falso) {}\n    vazio();\n}\n";
        let formatted = apply(compact, format(compact).expect("fonte válida"));
        assert_eq!(formatted, expected);
        assert_eq!(format(&formatted), Some(Vec::new()));
    }

    #[test]
    fn formats_vector_literals_and_indexing_without_internal_spaces() {
        let source = "funcao inicio(){seja mutavel valores:vetor<inteiro> =[1,2];valores[0]=valores[1];escreva(tamanho(valores));}";
        let formatted = apply(source, format(source).expect("fonte com vetor válida"));
        assert_eq!(
            formatted,
            "funcao inicio() {\n    seja mutavel valores: vetor < inteiro > = [1, 2];\n    valores[0] = valores[1];\n    escreva(tamanho(valores));\n}\n"
        );
        assert_eq!(format(&formatted), Some(Vec::new()));
    }

    #[test]
    fn preserves_utf16_edit_offsets_after_unicode_strings_and_rejects_invalid_source() {
        let source = "funcao inicio(){escreva(\"🦀 😀 ação\");seja x:inteiro=1+2;}";
        let formatted = apply(source, format(source).expect("fonte Unicode válida"));
        assert_eq!(
            formatted,
            "funcao inicio() {\n    escreva(\"🦀 😀 ação\");\n    seja x: inteiro = 1 + 2;\n}\n"
        );
        assert!(format("funcao inicio() { seja x: inteiro = ; }").is_none());
        assert!(format("funcao inicio() { escreva(desconhecido); }").is_none());
    }

    #[test]
    fn formats_try_operator_without_preceding_space() {
        let source = "funcao f(v:Opcao<inteiro>)->Opcao<inteiro>{seja x:inteiro=v ? +1;retorne Opcao::Algum(x);}funcao inicio(){}";
        let formatted = apply(source, format(source).expect("fonte com Opcao e ? válida"));
        assert!(formatted.contains("seja x: inteiro = v? + 1;"));
        assert_eq!(format(&formatted), Some(Vec::new()));
    }

    #[test]
    fn preserves_interpolation_content_verbatim() {
        let source = "funcao inicio(){seja nome=\"Ada\";escreva(\"Olá, { nome }!\");}";
        let formatted = apply(
            source,
            format(source).expect("fonte com interpolação válida"),
        );
        assert!(formatted.contains("\"Olá, { nome }!\""));
        assert_eq!(format(&formatted), Some(Vec::new()));
    }
}
