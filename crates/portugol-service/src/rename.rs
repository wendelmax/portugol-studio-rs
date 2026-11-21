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

#[derive(Clone, Copy)]
struct PortugolRsOccurrence {
    binding: portugol_hir::BindingId,
    span: portugol_hir::Span,
}

struct PortugolRsBindings {
    declarations: Vec<portugol_hir::Binding>,
    occurrences: Vec<PortugolRsOccurrence>,
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

fn portugol_rs_bindings(source: &str) -> Result<PortugolRsBindings, String> {
    let program = portugol_rs_frontend::lower(source)
        .map_err(|_| "Corrija os erros de sintaxe e semântica antes de renomear.".to_string())?;
    let mut occurrences: Vec<_> = program
        .bindings
        .iter()
        .map(|binding| PortugolRsOccurrence {
            binding: binding.id,
            span: binding.span,
        })
        .collect();
    collect_portugol_rs_statements(&program.body, &program.bindings, &mut occurrences);
    occurrences.sort_by_key(|occurrence| occurrence.span.start);
    Ok(PortugolRsBindings {
        declarations: program.bindings,
        occurrences,
    })
}

fn collect_portugol_rs_statements(
    statements: &[portugol_hir::Stmt],
    declarations: &[portugol_hir::Binding],
    occurrences: &mut Vec<PortugolRsOccurrence>,
) {
    use portugol_hir::StmtKind;

    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { initializer, .. } => {
                if let Some(initializer) = initializer {
                    collect_portugol_rs_expression(initializer, occurrences);
                }
            }
            StmtKind::Assign { binding, value, .. } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(PortugolRsOccurrence {
                        binding: *binding,
                        span: portugol_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_portugol_rs_expression(value, occurrences);
            }
            StmtKind::Write(values) => {
                for value in values {
                    collect_portugol_rs_expression(value, occurrences);
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_portugol_rs_expression(condition, occurrences);
                collect_portugol_rs_statements(then_branch, declarations, occurrences);
                if let Some(else_branch) = else_branch {
                    collect_portugol_rs_statements(else_branch, declarations, occurrences);
                }
            }
            StmtKind::While { condition, body } => {
                collect_portugol_rs_expression(condition, occurrences);
                collect_portugol_rs_statements(body, declarations, occurrences);
            }
        }
    }
}

fn collect_portugol_rs_expression(
    expression: &portugol_hir::Expr,
    occurrences: &mut Vec<PortugolRsOccurrence>,
) {
    use portugol_hir::ExprKind;

    match &expression.kind {
        ExprKind::Binding(binding) => occurrences.push(PortugolRsOccurrence {
            binding: *binding,
            span: expression.span,
        }),
        ExprKind::Parenthesized(inner) | ExprKind::Unary { operand: inner, .. } => {
            collect_portugol_rs_expression(inner, occurrences);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_portugol_rs_expression(left, occurrences);
            collect_portugol_rs_expression(right, occurrences);
        }
        ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Character(_) => {}
    }
}

fn locate_portugol_rs(
    offset: u32,
    index: &LineIndex<'_>,
    bindings: &PortugolRsBindings,
) -> Result<(portugol_hir::BindingId, String, Vec<portugol_hir::Span>), String> {
    let selected = bindings
        .occurrences
        .iter()
        .find(|occurrence| {
            let start = index.position(occurrence.span.start).offset;
            let end = index.position(occurrence.span.end).offset;
            start <= offset && offset <= end
        })
        .ok_or("Posicione o cursor em um símbolo do programa.")?;
    let declaration = bindings
        .declarations
        .get(selected.binding.0 as usize)
        .ok_or("Não foi possível resolver o símbolo selecionado.")?;
    let spans = bindings
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.binding == selected.binding)
        .map(|occurrence| occurrence.span)
        .collect();
    Ok((selected.binding, declaration.source_name.clone(), spans))
}

pub fn rename_target_portugol_rs(source: &str, offset: u32) -> Result<RenameTarget, String> {
    let bindings = portugol_rs_bindings(source)?;
    let index = LineIndex::new(source);
    let (_, name, spans) = locate_portugol_rs(offset, &index, &bindings)?;
    Ok(RenameTarget {
        name,
        occurrences: spans.len(),
    })
}

pub fn resolve_symbol_portugol_rs(source: &str, offset: u32) -> Result<SymbolLocation, String> {
    let bindings = portugol_rs_bindings(source)?;
    let index = LineIndex::new(source);
    let (binding, _, _) = locate_portugol_rs(offset, &index, &bindings)?;
    let declaration = &bindings.declarations[binding.0 as usize];
    Ok(SymbolLocation {
        start: index.position(declaration.span.start),
        end: index.position(declaration.span.end),
    })
}

pub fn rename_symbol_portugol_rs(
    source: &str,
    offset: u32,
    name: &str,
) -> Result<Vec<RenameEdit>, String> {
    let probe = format!("funcao inicio() {{ seja {name}: inteiro = 1; }}");
    if portugol_rs_frontend::lower(&probe).is_err() {
        return Err("Digite um identificador válido do Portugol-RS.".into());
    }

    let old = portugol_rs_bindings(source)?;
    let index = LineIndex::new(source);
    let (binding, old_name, spans) = locate_portugol_rs(offset, &index, &old)?;
    if name == old_name {
        return Ok(Vec::new());
    }

    let mut updated = source.to_owned();
    for span in spans.iter().rev() {
        updated.replace_range(span.start as usize..span.end as usize, name);
    }
    let checked = portugol_rs_bindings(&updated).map_err(|_| {
        "Esse nome entra em conflito com outro símbolo ou altera as referências.".to_string()
    })?;
    let old_signature: Vec<_> = old
        .occurrences
        .iter()
        .map(|occurrence| occurrence.binding)
        .collect();
    let new_signature: Vec<_> = checked
        .occurrences
        .iter()
        .map(|occurrence| occurrence.binding)
        .collect();
    if old_signature != new_signature
        || !spans
            .iter()
            .any(|span| span == &old.declarations[binding.0 as usize].span)
    {
        return Err(
            "Esse nome entra em conflito com outro símbolo ou altera as referências.".into(),
        );
    }

    let mut edits: Vec<_> = spans
        .into_iter()
        .map(|span| RenameEdit {
            from: index.position(span.start).offset,
            to: index.position(span.end).offset,
            insert: name.to_owned(),
        })
        .collect();
    edits.sort_by_key(|edit| edit.from);
    Ok(edits)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portugol_rs_resolves_semantic_bindings_with_shadowing_and_utf16_offsets() {
        let source = "funcao inicio() {\n    seja mutavel valor: inteiro = 1;\n    escreva(\"😀 valor\");\n    valor += 1;\n    se (verdadeiro) {\n        seja valor: inteiro = 2;\n        escreva(valor);\n    }\n    escreva(valor);\n}\n";
        let outer_use = source.rfind("escreva(valor)").unwrap() + "escreva(".len();
        let inner_use = source.find("escreva(valor)").unwrap() + "escreva(".len();
        let as_utf16 = |byte_offset: usize| source[..byte_offset].encode_utf16().count() as u32;

        let outer = resolve_symbol_portugol_rs(source, as_utf16(outer_use)).unwrap();
        let inner = resolve_symbol_portugol_rs(source, as_utf16(inner_use)).unwrap();

        assert_eq!(outer.start.offset, as_utf16(source.find("valor:").unwrap()));
        assert_eq!(
            inner.start.offset,
            as_utf16(source.find("seja valor").unwrap() + "seja ".len())
        );
    }

    #[test]
    fn portugol_rs_rename_updates_only_the_selected_binding() {
        let source = "funcao inicio() { seja mutavel valor: inteiro = 1; escreva(\"valor\"); valor += 1; se (verdadeiro) { seja valor: inteiro = 2; escreva(valor); } escreva(valor); }";
        let cursor_byte = source.rfind("escreva(valor)").unwrap() + "escreva(".len();
        let cursor = source[..cursor_byte].encode_utf16().count() as u32;

        let target = rename_target_portugol_rs(source, cursor).unwrap();
        let edits = rename_symbol_portugol_rs(source, cursor, "total").unwrap();

        assert_eq!(target.name, "valor");
        assert_eq!(target.occurrences, 3);
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "total"));
    }

    #[test]
    fn portugol_rs_rename_rejects_capture_and_keeps_entry_function_name() {
        let source = "funcao inicio() { seja mutavel valor: inteiro = 1; seja outro: inteiro = 2; escreva(valor, outro); }";
        let value = source.find("valor:").unwrap() as u32;
        let entry = source.find("inicio").unwrap() as u32;

        assert!(rename_symbol_portugol_rs(source, value, "outro").is_err());
        assert!(rename_target_portugol_rs(source, entry).is_err());
    }

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
