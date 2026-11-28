mod format;
mod lexer;
mod lower;
mod parser;

pub use format::{WhitespaceEdit, format, format_with};

/// Usos `Modulo.funcao` dos módulos pedidos, na ordem do texto, pelo lexer:
/// comentários e cadeias não contam. Serve, por exemplo, para conferir as
/// permissões de uma tela sem compilar o programa.
pub fn module_uses(source: &str, modules: &[&str]) -> Vec<(String, ferro_hir::Span)> {
    let (tokens, _) = lexer::lex(source);
    tokens
        .windows(3)
        .enumerate()
        .filter_map(|(index, window)| {
            let after_dot = index > 0 && tokens[index - 1].kind == lexer::Kind::Dot;
            match (&window[0].kind, &window[1].kind, &window[2].kind) {
                (lexer::Kind::Ident(module), lexer::Kind::Dot, lexer::Kind::Ident(_))
                    if !after_dot && modules.contains(&module.as_str()) =>
                {
                    Some((module.clone(), window[0].span))
                }
                _ => None,
            }
        })
        .collect()
}
pub use lower::{
    ExternalField, ExternalRecord, ExternalType, LowerOptions, lower, lower_with,
    lower_with_warnings,
};
