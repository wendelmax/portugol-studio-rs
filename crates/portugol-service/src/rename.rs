use std::collections::HashMap;

use portugol_semantics::analyzer::symbol_bindings;
use portugol_syntax::Severity;
use portugol_syntax::lexer::lex;
use portugol_syntax::token::{Token, TokenKind};
use serde::Serialize;

use crate::position::LineIndex;

#[derive(Serialize)]
pub struct RenameTarget {
    pub name: String,
    pub occurrences: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct SymbolLocation {
    pub start: crate::Position,
    pub end: crate::Position,
}

#[derive(Serialize)]
pub struct RenameEdit {
    pub from: u32,
    pub to: u32,
    pub insert: String,
}

struct Bindings {
    tokens: Vec<Token>,
    symbols: HashMap<usize, usize>,
}

fn bindings(source: &str) -> Result<Bindings, String> {
    let parsed = portugol_syntax::parse(source);
    if parsed.has_errors() {
        return Err("Corrija os erros de sintaxe antes de renomear.".into());
    }
    let (diagnostics, positions) = symbol_bindings(&parsed.program);
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return Err("A análise do programa encontrou erros.".into());
    }
    let tokens = lex(source).tokens;
    let index = |start| {
        tokens
            .binary_search_by_key(&start, |token| token.span.start)
            .ok()
    };
    let symbols = positions
        .into_iter()
        .filter_map(|(usage, declaration)| Some((index(usage)?, index(declaration)?)))
        .collect();
    Ok(Bindings { tokens, symbols })
}

fn locate(
    source: &str,
    offset: u32,
    line_index: &LineIndex<'_>,
    bindings: &Bindings,
) -> Result<(usize, String, Vec<usize>), String> {
    let tokens = &bindings.tokens;
    let before =
        tokens.partition_point(|token| line_index.position(token.span.start).offset <= offset);
    let candidate = [before.checked_sub(1), before.checked_sub(2)]
        .into_iter()
        .flatten()
        .find_map(|index| {
            let token = &tokens[index];
            let target = *bindings.symbols.get(&index)?;
            (token.kind == TokenKind::Ident && line_index.position(token.span.end).offset >= offset)
                .then_some(target)
        })
        .ok_or("Posicione o cursor em um símbolo do programa.")?;
    let span = tokens[candidate].span;
    let name = &source[span.start as usize..span.end as usize];
    let mut occurrences: Vec<_> = bindings
        .symbols
        .iter()
        .filter_map(|(&usage, &target)| (target == candidate).then_some(usage))
        .collect();
    occurrences.sort_unstable();
    Ok((candidate, name.to_owned(), occurrences))
}

fn validate_rename_target(
    source: &str,
    declaration: usize,
    name: &str,
    tokens: &[Token],
) -> Result<(), String> {
    let span = tokens[declaration].span;
    if name == "inicio" && portugol_syntax::parse(source).program.items.iter().any(|item| {
        matches!(item, portugol_syntax::ast::Item::Function(function) if function.name.span == span)
    }) {
        return Err("A função de entrada inicio precisa manter esse nome.".into());
    }
    Ok(())
}

pub fn rename_target(source: &str, offset: u32) -> Result<RenameTarget, String> {
    let bindings = bindings(source)?;
    let index = LineIndex::new(source);
    let (declaration, name, occurrences) = locate(source, offset, &index, &bindings)?;
    validate_rename_target(source, declaration, &name, &bindings.tokens)?;
    Ok(RenameTarget {
        name,
        occurrences: occurrences.len(),
    })
}

/// Resolve a symbol use to its declaration using the same semantic bindings as rename.
pub fn resolve_symbol(source: &str, offset: u32) -> Result<SymbolLocation, String> {
    let bindings = bindings(source)?;
    let index = LineIndex::new(source);
    let (declaration, _, _) = locate(source, offset, &index, &bindings)?;
    let span = bindings.tokens[declaration].span;
    Ok(SymbolLocation {
        start: index.position(span.start),
        end: index.position(span.end),
    })
}

pub fn rename_symbol(source: &str, offset: u32, name: &str) -> Result<Vec<RenameEdit>, String> {
    let proposed = lex(name);
    if proposed.tokens.len() != 2
        || proposed.tokens[0].kind != TokenKind::Ident
        || proposed.tokens[0].span.len() as usize != name.len()
        || !proposed.diagnostics.is_empty()
    {
        return Err("Digite um identificador válido do Portugol.".into());
    }
    let old = bindings(source)?;
    let index = LineIndex::new(source);
    let (declaration, old_name, occurrences) = locate(source, offset, &index, &old)?;
    validate_rename_target(source, declaration, &old_name, &old.tokens)?;
    if name == old_name {
        return Ok(Vec::new());
    }
    let mut updated = source.to_owned();
    for &usage in occurrences.iter().rev() {
        let span = old.tokens[usage].span;
        updated.replace_range(span.start as usize..span.end as usize, name);
    }
    let conflict = "Esse nome entra em conflito com outro símbolo ou altera as referências.";
    let checked = bindings(&updated).map_err(|_| conflict)?;
    if checked.symbols != old.symbols {
        return Err(conflict.into());
    }
    Ok(occurrences
        .into_iter()
        .map(|usage| {
            let span = old.tokens[usage].span;
            RenameEdit {
                from: index.position(span.start).offset,
                to: index.position(span.end).offset,
                insert: name.to_owned(),
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_symbol_returns_the_declaration_for_a_local_use() {
        let source = "programa { inteiro valor = 1 funcao inicio() { escreva(valor) } }";
        let usage = source.find("valor)").unwrap() as u32;
        let location = resolve_symbol(source, usage).unwrap();
        assert_eq!(location.start.offset, source.find("valor").unwrap() as u32);
        assert_eq!(
            location.end.offset,
            location.start.offset + "valor".len() as u32
        );
    }

    #[test]
    fn resolve_symbol_respects_shadowed_bindings_and_utf16_offsets() {
        let source = "// 😀\nprograma { inteiro item = 1 funcao inicio() { escreva(item) se (verdadeiro) { inteiro item = 2 escreva(item) } escreva(item) } }";
        let occurrences: Vec<_> = source
            .match_indices("item")
            .map(|(offset, _)| offset)
            .collect();
        let as_utf16 = |byte_offset: usize| source[..byte_offset].encode_utf16().count() as u32;

        let inner = resolve_symbol(source, as_utf16(occurrences[3])).unwrap();
        let after_scope = resolve_symbol(source, as_utf16(occurrences[4])).unwrap();

        assert_eq!(inner.start.offset, as_utf16(occurrences[2]));
        assert_eq!(after_scope.start.offset, as_utf16(occurrences[0]));
    }

    #[test]
    fn resolve_symbol_rejects_positions_without_a_resolved_identifier() {
        let source = "programa { inteiro valor = 1 funcao inicio() { escreva(valor) } }";
        assert!(resolve_symbol(source, source.find("programa").unwrap() as u32).is_err());
    }

    #[test]
    fn resolve_symbol_navigates_to_functions_including_the_entry_function() {
        let source = "programa { funcao auxiliar() { } funcao inicio() { auxiliar() } }";
        let helper_use = source.find("auxiliar() }").unwrap() as u32;
        let helper = resolve_symbol(source, helper_use).unwrap();
        let entry = resolve_symbol(source, source.find("inicio").unwrap() as u32).unwrap();

        assert_eq!(helper.start.offset, source.find("auxiliar").unwrap() as u32);
        assert_eq!(entry.start.offset, source.find("inicio").unwrap() as u32);
    }

    #[test]
    fn renames_only_the_selected_binding_and_keeps_utf16_offsets() {
        let source = "programa { inteiro valor = 1 funcao inicio() { escreva(\"😀 valor\", valor) se (verdadeiro) { inteiro valor = 2 escreva(valor) } } }";
        let cursor = source[..source.find("valor)").unwrap()]
            .encode_utf16()
            .count() as u32;
        let target = rename_target(source, cursor).unwrap();
        assert_eq!(target.name, "valor");
        assert_eq!(target.occurrences, 2);
        let edits = rename_symbol(source, cursor, "total").unwrap();
        assert_eq!(edits.len(), 2);
        assert_eq!(edits[1].from, cursor);
        assert!(edits.iter().all(|edit| edit.insert == "total"));
    }

    #[test]
    fn prevents_reference_capture() {
        let source = "programa { inteiro valor = 1 funcao inicio() { inteiro outra = 2 escreva(valor, outra) } }";
        let cursor = source.find("valor").unwrap() as u32;
        assert!(rename_symbol(source, cursor, "outra").is_err());
        assert!(rename_symbol(source, cursor, "programa").is_err());
        let entry = source.find("inicio").unwrap() as u32;
        assert!(rename_target(source, entry).is_err());
    }
}
