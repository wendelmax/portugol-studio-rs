use std::collections::HashMap;

use portugol_semantics::analyzer::symbol_bindings;
use portugol_syntax::Severity;
use portugol_syntax::lexer::lex;
use portugol_syntax::token::{Token, TokenKind};
use serde::Serialize;

use crate::{pors_function_name_start, position::LineIndex};

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
    binding: PortugolRsBindingTarget,
    span: portugol_hir::Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PortugolRsBindingOwner {
    Entry,
    Function(portugol_hir::FunctionId),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct PortugolRsBindingTarget {
    owner: PortugolRsBindingOwner,
    id: portugol_hir::BindingId,
}

struct PortugolRsDeclaration {
    target: PortugolRsBindingTarget,
    name: String,
    span: portugol_hir::Span,
}

struct PortugolRsBindings {
    declarations: Vec<PortugolRsDeclaration>,
    occurrences: Vec<PortugolRsOccurrence>,
}

struct PortugolRsFunctionReferences {
    id: portugol_hir::FunctionId,
    name: String,
    declaration: portugol_hir::Span,
    occurrences: Vec<portugol_hir::Span>,
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
    let mut declarations: Vec<_> = program
        .bindings
        .iter()
        .map(|binding| PortugolRsDeclaration {
            target: PortugolRsBindingTarget {
                owner: PortugolRsBindingOwner::Entry,
                id: binding.id,
            },
            name: binding.source_name.clone(),
            span: binding.span,
        })
        .collect();
    for function in &program.functions {
        declarations.extend(
            function
                .bindings
                .iter()
                .map(|binding| PortugolRsDeclaration {
                    target: PortugolRsBindingTarget {
                        owner: PortugolRsBindingOwner::Function(function.id),
                        id: binding.id,
                    },
                    name: binding.source_name.clone(),
                    span: binding.span,
                }),
        );
    }
    let mut occurrences: Vec<_> = declarations
        .iter()
        .map(|declaration| PortugolRsOccurrence {
            binding: declaration.target,
            span: declaration.span,
        })
        .collect();
    collect_portugol_rs_statements(
        &program.body,
        &program.bindings,
        PortugolRsBindingOwner::Entry,
        &mut occurrences,
    );
    for function in &program.functions {
        collect_portugol_rs_statements(
            &function.body,
            &function.bindings,
            PortugolRsBindingOwner::Function(function.id),
            &mut occurrences,
        );
    }
    occurrences.sort_by_key(|occurrence| occurrence.span.start);
    Ok(PortugolRsBindings {
        declarations,
        occurrences,
    })
}

fn portugol_rs_function_references(
    source: &str,
) -> Result<Vec<PortugolRsFunctionReferences>, String> {
    let program = portugol_rs_frontend::lower(source)
        .map_err(|_| "Corrija os erros de sintaxe e semântica antes de renomear.".to_string())?;
    let mut functions: Vec<_> = program
        .functions
        .iter()
        .map(|function| {
            let start = pors_function_name_start(source, function.span.start);
            let declaration =
                portugol_hir::Span::new(start, start + function.source_name.len() as u32);
            PortugolRsFunctionReferences {
                id: function.id,
                name: function.source_name.clone(),
                declaration,
                occurrences: vec![declaration],
            }
        })
        .collect();
    collect_pors_function_calls(&program.body, &mut functions);
    for function in &program.functions {
        collect_pors_function_calls(&function.body, &mut functions);
    }
    for function in &mut functions {
        function.occurrences.sort_by_key(|span| span.start);
    }
    Ok(functions)
}

fn collect_pors_function_calls(
    statements: &[portugol_hir::Stmt],
    functions: &mut [PortugolRsFunctionReferences],
) {
    use portugol_hir::StmtKind;

    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { initializer, .. } => {
                if let Some(expression) = initializer {
                    collect_pors_function_calls_in_expr(expression, functions);
                }
            }
            StmtKind::Assign { value, .. } => collect_pors_function_calls_in_expr(value, functions),
            StmtKind::FieldAssign { value, .. } => {
                collect_pors_function_calls_in_expr(value, functions)
            }
            StmtKind::IndexAssign { index, value, .. } => {
                collect_pors_function_calls_in_expr(index, functions);
                collect_pors_function_calls_in_expr(value, functions);
            }
            StmtKind::Read { .. } | StmtKind::LoopControl { .. } => {}
            StmtKind::Call {
                function,
                arguments,
            } => {
                if let Some(target) = functions.iter_mut().find(|target| target.id == *function) {
                    target.occurrences.push(portugol_hir::Span::new(
                        statement.span.start,
                        statement.span.start + target.name.len() as u32,
                    ));
                }
                for argument in arguments {
                    collect_pors_function_calls_in_expr(argument, functions);
                }
            }
            StmtKind::Write(values) => {
                for value in values {
                    collect_pors_function_calls_in_expr(value, functions);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_pors_function_calls_in_expr(value, functions);
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_pors_function_calls_in_expr(condition, functions);
                collect_pors_function_calls(then_branch, functions);
                if let Some(else_branch) = else_branch {
                    collect_pors_function_calls(else_branch, functions);
                }
            }
            StmtKind::While { condition, body } => {
                collect_pors_function_calls_in_expr(condition, functions);
                collect_pors_function_calls(body, functions);
            }
            StmtKind::For {
                initializer,
                condition,
                step,
                body,
            } => {
                collect_pors_function_calls(std::slice::from_ref(initializer.as_ref()), functions);
                collect_pors_function_calls_in_expr(condition, functions);
                collect_pors_function_calls(std::slice::from_ref(step.as_ref()), functions);
                collect_pors_function_calls(body, functions);
            }
            StmtKind::Choose { value, arms } => {
                collect_pors_function_calls_in_expr(value, functions);
                for arm in arms {
                    collect_pors_function_calls(&arm.body, functions);
                }
            }
        }
    }
}

fn collect_pors_function_calls_in_expr(
    expression: &portugol_hir::Expr,
    functions: &mut [PortugolRsFunctionReferences],
) {
    use portugol_hir::ExprKind;

    match &expression.kind {
        ExprKind::Call {
            function,
            arguments,
        } => {
            if let Some(target) = functions.iter_mut().find(|target| target.id == *function) {
                target.occurrences.push(portugol_hir::Span::new(
                    expression.span.start,
                    expression.span.start + target.name.len() as u32,
                ));
            }
            for argument in arguments {
                collect_pors_function_calls_in_expr(argument, functions);
            }
        }
        ExprKind::VectorLiteral(items) => {
            for item in items {
                collect_pors_function_calls_in_expr(item, functions);
            }
        }
        ExprKind::RecordLiteral { fields, .. } => {
            for field in fields {
                collect_pors_function_calls_in_expr(field, functions);
            }
        }
        ExprKind::EnumConstructor { payload, .. } => {
            for item in payload {
                collect_pors_function_calls_in_expr(item, functions);
            }
        }
        ExprKind::Field { value, .. } => collect_pors_function_calls_in_expr(value, functions),
        ExprKind::Index { index, .. } => collect_pors_function_calls_in_expr(index, functions),
        ExprKind::Length { .. } => {}
        ExprKind::Parenthesized(inner) | ExprKind::Unary { operand: inner, .. } => {
            collect_pors_function_calls_in_expr(inner, functions);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_pors_function_calls_in_expr(left, functions);
            collect_pors_function_calls_in_expr(right, functions);
        }
        ExprKind::Binding(_)
        | ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Character(_) => {}
    }
}

fn selected_pors_function(
    offset: u32,
    index: &LineIndex<'_>,
    functions: &[PortugolRsFunctionReferences],
) -> Option<(
    portugol_hir::FunctionId,
    String,
    portugol_hir::Span,
    Vec<portugol_hir::Span>,
)> {
    functions.iter().find_map(|function| {
        function
            .occurrences
            .iter()
            .any(|span| {
                let start = index.position(span.start).offset;
                let end = index.position(span.end).offset;
                start <= offset && offset <= end
            })
            .then(|| {
                (
                    function.id,
                    function.name.clone(),
                    function.declaration,
                    function.occurrences.clone(),
                )
            })
    })
}

fn pors_function_signature(
    functions: &[PortugolRsFunctionReferences],
) -> Vec<portugol_hir::FunctionId> {
    let mut occurrences: Vec<_> = functions
        .iter()
        .flat_map(|function| {
            function
                .occurrences
                .iter()
                .map(|span| (span.start, function.id))
        })
        .collect();
    occurrences.sort_by_key(|(start, _)| *start);
    occurrences
        .into_iter()
        .map(|(_, function)| function)
        .collect()
}

fn rename_pors_function(
    source: &str,
    index: &LineIndex<'_>,
    old_functions: &[PortugolRsFunctionReferences],
    selected: (
        portugol_hir::FunctionId,
        String,
        portugol_hir::Span,
        Vec<portugol_hir::Span>,
    ),
    name: &str,
) -> Result<Vec<RenameEdit>, String> {
    let (function_id, old_name, declaration, spans) = selected;
    if name == "inicio" {
        return Err("A função de entrada inicio precisa manter esse nome.".into());
    }
    if name == old_name {
        return Ok(Vec::new());
    }

    let mut updated = source.to_owned();
    for span in spans.iter().rev() {
        updated.replace_range(span.start as usize..span.end as usize, name);
    }
    let checked = portugol_rs_function_references(&updated).map_err(|_| {
        "Esse nome entra em conflito com outro símbolo ou altera as referências.".to_string()
    })?;
    if pors_function_signature(old_functions) != pors_function_signature(&checked)
        || !spans.contains(&declaration)
        || !checked
            .iter()
            .any(|function| function.id == function_id && function.name == name)
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

fn collect_portugol_rs_statements(
    statements: &[portugol_hir::Stmt],
    declarations: &[portugol_hir::Binding],
    owner: PortugolRsBindingOwner,
    occurrences: &mut Vec<PortugolRsOccurrence>,
) {
    use portugol_hir::StmtKind;

    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { initializer, .. } => {
                if let Some(initializer) = initializer {
                    collect_portugol_rs_expression(initializer, owner, occurrences);
                }
            }
            StmtKind::Assign { binding, value, .. } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(PortugolRsOccurrence {
                        binding: PortugolRsBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: portugol_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_portugol_rs_expression(value, owner, occurrences);
            }
            StmtKind::IndexAssign {
                binding,
                index,
                value,
            } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(PortugolRsOccurrence {
                        binding: PortugolRsBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: portugol_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_portugol_rs_expression(index, owner, occurrences);
                collect_portugol_rs_expression(value, owner, occurrences);
            }
            StmtKind::FieldAssign { binding, value, .. } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(PortugolRsOccurrence {
                        binding: PortugolRsBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: portugol_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_portugol_rs_expression(value, owner, occurrences);
            }
            StmtKind::Write(values) => {
                for value in values {
                    collect_portugol_rs_expression(value, owner, occurrences);
                }
            }
            StmtKind::Call { arguments, .. } => {
                for value in arguments {
                    collect_portugol_rs_expression(value, owner, occurrences);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_portugol_rs_expression(value, owner, occurrences);
                }
            }
            StmtKind::Read { binding, name_span } => occurrences.push(PortugolRsOccurrence {
                binding: PortugolRsBindingTarget {
                    owner,
                    id: *binding,
                },
                span: *name_span,
            }),
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_portugol_rs_expression(condition, owner, occurrences);
                collect_portugol_rs_statements(then_branch, declarations, owner, occurrences);
                if let Some(else_branch) = else_branch {
                    collect_portugol_rs_statements(else_branch, declarations, owner, occurrences);
                }
            }
            StmtKind::While { condition, body } => {
                collect_portugol_rs_expression(condition, owner, occurrences);
                collect_portugol_rs_statements(body, declarations, owner, occurrences);
            }
            StmtKind::For {
                initializer,
                condition,
                step,
                body,
            } => {
                collect_portugol_rs_statements(
                    std::slice::from_ref(initializer.as_ref()),
                    declarations,
                    owner,
                    occurrences,
                );
                collect_portugol_rs_expression(condition, owner, occurrences);
                collect_portugol_rs_statements(
                    std::slice::from_ref(step.as_ref()),
                    declarations,
                    owner,
                    occurrences,
                );
                collect_portugol_rs_statements(body, declarations, owner, occurrences);
            }
            StmtKind::Choose { value, arms } => {
                collect_portugol_rs_expression(value, owner, occurrences);
                for arm in arms {
                    collect_portugol_rs_statements(&arm.body, declarations, owner, occurrences);
                }
            }
            StmtKind::LoopControl { .. } => {}
        }
    }
}

fn collect_portugol_rs_expression(
    expression: &portugol_hir::Expr,
    owner: PortugolRsBindingOwner,
    occurrences: &mut Vec<PortugolRsOccurrence>,
) {
    use portugol_hir::ExprKind;

    match &expression.kind {
        ExprKind::Binding(binding) => occurrences.push(PortugolRsOccurrence {
            binding: PortugolRsBindingTarget {
                owner,
                id: *binding,
            },
            span: expression.span,
        }),
        ExprKind::Parenthesized(inner) | ExprKind::Unary { operand: inner, .. } => {
            collect_portugol_rs_expression(inner, owner, occurrences);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_portugol_rs_expression(left, owner, occurrences);
            collect_portugol_rs_expression(right, owner, occurrences);
        }
        ExprKind::Call { arguments, .. } => {
            for argument in arguments {
                collect_portugol_rs_expression(argument, owner, occurrences);
            }
        }
        ExprKind::VectorLiteral(items) => {
            for item in items {
                collect_portugol_rs_expression(item, owner, occurrences);
            }
        }
        ExprKind::RecordLiteral { fields, .. } => {
            for field in fields {
                collect_portugol_rs_expression(field, owner, occurrences);
            }
        }
        ExprKind::EnumConstructor { payload, .. } => {
            for item in payload {
                collect_portugol_rs_expression(item, owner, occurrences);
            }
        }
        ExprKind::Field { value, .. } => {
            collect_portugol_rs_expression(value, owner, occurrences);
        }
        ExprKind::Index { binding, index } => {
            occurrences.push(PortugolRsOccurrence {
                binding: PortugolRsBindingTarget {
                    owner,
                    id: *binding,
                },
                span: expression.span,
            });
            collect_portugol_rs_expression(index, owner, occurrences);
        }
        ExprKind::Length { binding } => occurrences.push(PortugolRsOccurrence {
            binding: PortugolRsBindingTarget {
                owner,
                id: *binding,
            },
            span: expression.span,
        }),
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
) -> Result<(PortugolRsBindingTarget, String, Vec<portugol_hir::Span>), String> {
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
        .iter()
        .find(|declaration| declaration.target == selected.binding)
        .ok_or("Não foi possível resolver o símbolo selecionado.")?;
    let spans = bindings
        .occurrences
        .iter()
        .filter(|occurrence| occurrence.binding == selected.binding)
        .map(|occurrence| occurrence.span)
        .collect();
    Ok((selected.binding, declaration.name.clone(), spans))
}

pub fn rename_target_portugol_rs(source: &str, offset: u32) -> Result<RenameTarget, String> {
    let index = LineIndex::new(source);
    let functions = portugol_rs_function_references(source)?;
    if let Some((_, name, _, spans)) = selected_pors_function(offset, &index, &functions) {
        return Ok(RenameTarget {
            name,
            occurrences: spans.len(),
        });
    }
    let bindings = portugol_rs_bindings(source)?;
    let (_, name, spans) = locate_portugol_rs(offset, &index, &bindings)?;
    Ok(RenameTarget {
        name,
        occurrences: spans.len(),
    })
}

pub fn resolve_symbol_portugol_rs(source: &str, offset: u32) -> Result<SymbolLocation, String> {
    let index = LineIndex::new(source);
    let functions = portugol_rs_function_references(source)?;
    if let Some((_, _, declaration, _)) = selected_pors_function(offset, &index, &functions) {
        return Ok(SymbolLocation {
            start: index.position(declaration.start),
            end: index.position(declaration.end),
        });
    }
    let bindings = portugol_rs_bindings(source)?;
    let (binding, _, _) = locate_portugol_rs(offset, &index, &bindings)?;
    let declaration = bindings
        .declarations
        .iter()
        .find(|declaration| declaration.target == binding)
        .ok_or("Não foi possível resolver o símbolo selecionado.")?;
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

    let index = LineIndex::new(source);
    let functions = portugol_rs_function_references(source)?;
    if let Some(selected) = selected_pors_function(offset, &index, &functions) {
        return rename_pors_function(source, &index, &functions, selected, name);
    }
    let old = portugol_rs_bindings(source)?;
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
        || !spans.iter().any(|span| {
            old.declarations
                .iter()
                .any(|declaration| declaration.target == binding && span == &declaration.span)
        })
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
    fn portugol_rs_rename_includes_the_leia_target() {
        let source =
            "funcao inicio() { seja mutavel idade: inteiro = 0; leia(idade); escreva(idade); }";
        let cursor = source.find("idade:").unwrap() as u32;
        let edits = rename_symbol_portugol_rs(source, cursor, "anos").unwrap();
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "anos"));
    }

    #[test]
    fn portugol_rs_rename_in_counted_loop_updates_header_and_body_bindings() {
        let source = "funcao inicio() { para (seja mutavel i: inteiro = 1; i < 4; i += i) { para (seja mutavel i: inteiro = 0; i < 1; i += 1) { escreva(i); } escreva(i); } }";
        let cursor = source.rfind("escreva(i)").unwrap() as u32 + "escreva(".len() as u32;

        let target = rename_target_portugol_rs(source, cursor).unwrap();
        let edits = rename_symbol_portugol_rs(source, cursor, "contador").unwrap();

        assert_eq!(target.name, "i");
        assert_eq!(target.occurrences, 5);
        assert_eq!(edits.len(), 5);
        assert!(edits.iter().all(|edit| edit.insert == "contador"));
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
    fn portugol_rs_resolves_and_renames_helper_function_declarations_and_calls() {
        let source = "funcao somar(a: inteiro) -> inteiro { retorne a + 1; }\nfuncao inicio() { somar(2); escreva(somar(3)); }";
        let declaration_byte = source.find("somar").unwrap();
        let call_byte = source.rfind("somar").unwrap();
        let as_utf16 = |offset: usize| source[..offset].encode_utf16().count() as u32;

        let target = rename_target_portugol_rs(source, as_utf16(call_byte)).unwrap();
        let location = resolve_symbol_portugol_rs(source, as_utf16(call_byte)).unwrap();
        let edits = rename_symbol_portugol_rs(source, as_utf16(call_byte), "somar_um").unwrap();

        assert_eq!(target.name, "somar");
        assert_eq!(target.occurrences, 3);
        assert_eq!(location.start.offset, as_utf16(declaration_byte));
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "somar_um"));
    }

    #[test]
    fn portugol_rs_renames_parameters_and_locals_inside_helper_functions() {
        let source = "funcao calcular(entrada: inteiro) -> inteiro { seja mutavel total: inteiro = entrada; total += entrada; retorne total; }\nfuncao inicio() { escreva(calcular(2)); }";
        let total_use_byte = source.rfind("total").unwrap();
        let as_utf16 = |offset: usize| source[..offset].encode_utf16().count() as u32;

        let target = rename_target_portugol_rs(source, as_utf16(total_use_byte)).unwrap();
        let edits = rename_symbol_portugol_rs(source, as_utf16(total_use_byte), "soma").unwrap();
        let parameter_byte = source.find("entrada: ").unwrap();
        let parameter_edits =
            rename_symbol_portugol_rs(source, as_utf16(parameter_byte), "valor_inicial").unwrap();

        assert_eq!(target.name, "total");
        assert_eq!(edits.len(), 3);
        assert_eq!(parameter_edits.len(), 3);
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
