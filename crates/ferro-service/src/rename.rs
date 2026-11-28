use serde::Serialize;

use crate::{ferro_function_name_start, position::LineIndex};

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
struct FerroRustOccurrence {
    binding: FerroRustBindingTarget,
    span: ferro_hir::Span,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FerroRustBindingOwner {
    Entry,
    Function(ferro_hir::FunctionId),
}

#[derive(Clone, Copy, PartialEq, Eq)]
struct FerroRustBindingTarget {
    owner: FerroRustBindingOwner,
    id: ferro_hir::BindingId,
}

struct FerroRustDeclaration {
    target: FerroRustBindingTarget,
    name: String,
    span: ferro_hir::Span,
}

struct FerroRustBindings {
    declarations: Vec<FerroRustDeclaration>,
    occurrences: Vec<FerroRustOccurrence>,
}

struct FerroRustFunctionReferences {
    id: ferro_hir::FunctionId,
    name: String,
    declaration: ferro_hir::Span,
    occurrences: Vec<ferro_hir::Span>,
}

fn ferro_rs_bindings(source: &str) -> Result<FerroRustBindings, String> {
    let program = ferro_frontend::lower(source)
        .map_err(|_| "Corrija os erros de sintaxe e semântica antes de renomear.".to_string())?;
    let mut declarations: Vec<_> = program
        .bindings
        .iter()
        .map(|binding| FerroRustDeclaration {
            target: FerroRustBindingTarget {
                owner: FerroRustBindingOwner::Entry,
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
                .map(|binding| FerroRustDeclaration {
                    target: FerroRustBindingTarget {
                        owner: FerroRustBindingOwner::Function(function.id),
                        id: binding.id,
                    },
                    name: binding.source_name.clone(),
                    span: binding.span,
                }),
        );
    }
    let mut occurrences: Vec<_> = declarations
        .iter()
        .map(|declaration| FerroRustOccurrence {
            binding: declaration.target,
            span: declaration.span,
        })
        .collect();
    collect_ferro_rs_statements(
        source,
        &program.body,
        &program.bindings,
        FerroRustBindingOwner::Entry,
        &mut occurrences,
    );
    for function in &program.functions {
        collect_ferro_rs_statements(
            source,
            &function.body,
            &function.bindings,
            FerroRustBindingOwner::Function(function.id),
            &mut occurrences,
        );
    }
    occurrences.sort_by_key(|occurrence| occurrence.span.start);
    Ok(FerroRustBindings {
        declarations,
        occurrences,
    })
}

fn ferro_rs_function_references(source: &str) -> Result<Vec<FerroRustFunctionReferences>, String> {
    let program = ferro_frontend::lower(source)
        .map_err(|_| "Corrija os erros de sintaxe e semântica antes de renomear.".to_string())?;
    let mut functions: Vec<_> = program
        .functions
        .iter()
        .map(|function| {
            let start = ferro_function_name_start(source, function.span.start);
            let declaration =
                ferro_hir::Span::new(start, start + function.source_name.len() as u32);
            FerroRustFunctionReferences {
                id: function.id,
                name: function.source_name.clone(),
                declaration,
                occurrences: vec![declaration],
            }
        })
        .collect();
    collect_ferro_function_calls(&program.body, &mut functions);
    for function in &program.functions {
        collect_ferro_function_calls(&function.body, &mut functions);
    }
    for function in &mut functions {
        function.occurrences.sort_by_key(|span| span.start);
    }
    Ok(functions)
}

fn collect_ferro_function_calls(
    statements: &[ferro_hir::Stmt],
    functions: &mut [FerroRustFunctionReferences],
) {
    use ferro_hir::StmtKind;

    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { initializer, .. } => {
                if let Some(expression) = initializer {
                    collect_ferro_function_calls_in_expr(expression, functions);
                }
            }
            StmtKind::Assign { value, .. } => {
                collect_ferro_function_calls_in_expr(value, functions)
            }
            StmtKind::FieldAssign { value, .. } => {
                collect_ferro_function_calls_in_expr(value, functions)
            }
            StmtKind::IndexAssign { index, value, .. } => {
                collect_ferro_function_calls_in_expr(index, functions);
                collect_ferro_function_calls_in_expr(value, functions);
            }
            StmtKind::VectorPush { value, .. } => {
                collect_ferro_function_calls_in_expr(value, functions);
            }
            StmtKind::VectorPop { .. } | StmtKind::Read { .. } | StmtKind::LoopControl { .. } => {}
            StmtKind::Call {
                function,
                arguments,
            } => {
                if let Some(target) = functions.iter_mut().find(|target| target.id == *function) {
                    target.occurrences.push(ferro_hir::Span::new(
                        statement.span.start,
                        statement.span.start + target.name.len() as u32,
                    ));
                }
                for argument in arguments {
                    collect_ferro_function_calls_in_expr(argument, functions);
                }
            }
            StmtKind::HostCall { arguments, .. } => {
                for argument in arguments {
                    collect_ferro_function_calls_in_expr(argument, functions);
                }
            }
            StmtKind::Write(values) => {
                for value in values {
                    collect_ferro_function_calls_in_expr(value, functions);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_ferro_function_calls_in_expr(value, functions);
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                collect_ferro_function_calls_in_expr(condition, functions);
                collect_ferro_function_calls(then_branch, functions);
                if let Some(else_branch) = else_branch {
                    collect_ferro_function_calls(else_branch, functions);
                }
            }
            StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
                collect_ferro_function_calls_in_expr(condition, functions);
                collect_ferro_function_calls(body, functions);
            }
            StmtKind::For {
                initializer,
                condition,
                step,
                body,
            } => {
                collect_ferro_function_calls(std::slice::from_ref(initializer.as_ref()), functions);
                collect_ferro_function_calls_in_expr(condition, functions);
                collect_ferro_function_calls(std::slice::from_ref(step.as_ref()), functions);
                collect_ferro_function_calls(body, functions);
            }
            StmtKind::Choose { value, arms } => {
                collect_ferro_function_calls_in_expr(value, functions);
                for arm in arms {
                    collect_ferro_function_calls(&arm.body, functions);
                }
            }
        }
    }
}

fn collect_ferro_function_calls_in_expr(
    expression: &ferro_hir::Expr,
    functions: &mut [FerroRustFunctionReferences],
) {
    use ferro_hir::ExprKind;

    match &expression.kind {
        ExprKind::Call {
            function,
            arguments,
        } => {
            if let Some(target) = functions.iter_mut().find(|target| target.id == *function) {
                target.occurrences.push(ferro_hir::Span::new(
                    expression.span.start,
                    expression.span.start + target.name.len() as u32,
                ));
            }
            for argument in arguments {
                collect_ferro_function_calls_in_expr(argument, functions);
            }
        }
        ExprKind::HostCall { arguments, .. } => {
            for argument in arguments {
                collect_ferro_function_calls_in_expr(argument, functions);
            }
        }
        ExprKind::VectorLiteral(items) => {
            for item in items {
                collect_ferro_function_calls_in_expr(item, functions);
            }
        }
        ExprKind::RecordLiteral { fields, .. } => {
            for field in fields {
                collect_ferro_function_calls_in_expr(field, functions);
            }
        }
        ExprKind::EnumConstructor { payload, .. } => {
            for item in payload {
                collect_ferro_function_calls_in_expr(item, functions);
            }
        }
        ExprKind::Field { value, .. } => collect_ferro_function_calls_in_expr(value, functions),
        ExprKind::Index { index, .. } | ExprKind::VectorGet { index, .. } => {
            collect_ferro_function_calls_in_expr(index, functions)
        }
        ExprKind::Length { .. } | ExprKind::VectorPop { .. } => {}
        ExprKind::Parenthesized(inner)
        | ExprKind::Try { operand: inner }
        | ExprKind::Unary { operand: inner, .. } => {
            collect_ferro_function_calls_in_expr(inner, functions);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_ferro_function_calls_in_expr(left, functions);
            collect_ferro_function_calls_in_expr(right, functions);
        }
        ExprKind::InterpolatedString { parts } => {
            for part in parts {
                if let ferro_hir::StringPart::Expression(value, _) = part {
                    collect_ferro_function_calls_in_expr(value, functions);
                }
            }
        }
        ExprKind::Binding(_)
        | ExprKind::Read { .. }
        | ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Character(_) => {}
    }
}

fn selected_ferro_function(
    offset: u32,
    index: &LineIndex<'_>,
    functions: &[FerroRustFunctionReferences],
) -> Option<(
    ferro_hir::FunctionId,
    String,
    ferro_hir::Span,
    Vec<ferro_hir::Span>,
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

fn ferro_function_signature(
    functions: &[FerroRustFunctionReferences],
) -> Vec<ferro_hir::FunctionId> {
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

fn rename_ferro_function(
    source: &str,
    index: &LineIndex<'_>,
    old_functions: &[FerroRustFunctionReferences],
    selected: (
        ferro_hir::FunctionId,
        String,
        ferro_hir::Span,
        Vec<ferro_hir::Span>,
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
    let checked = ferro_rs_function_references(&updated).map_err(|_| {
        "Esse nome entra em conflito com outro símbolo ou altera as referências.".to_string()
    })?;
    if ferro_function_signature(old_functions) != ferro_function_signature(&checked)
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

fn find_call_first_arg_span(
    source: &str,
    container: ferro_hir::Span,
    name: &str,
) -> Option<ferro_hir::Span> {
    let slice = source.get(container.start as usize..container.end as usize)?;
    let open = slice.find('(')?;
    let bytes = slice.as_bytes();
    let mut index = open + 1;
    while index < bytes.len() {
        if slice[index..].starts_with("//") {
            index += slice[index..].find('\n')? + 1;
        } else if slice[index..].starts_with("/*") {
            let offset = slice[index + 2..].find("*/")?;
            index += offset + 4;
        } else if let Some(ch) = slice[index..].chars().next() {
            if ch.is_whitespace() {
                index += ch.len_utf8();
            } else {
                break;
            }
        } else {
            break;
        }
    }
    let rest = slice.get(index..)?;
    if !rest.starts_with(name) {
        return None;
    }
    if let Some(next) = rest[name.len()..].chars().next()
        && (next.is_alphanumeric() || next == '_')
    {
        return None;
    }
    let start = container.start + index as u32;
    Some(ferro_hir::Span::new(start, start + name.len() as u32))
}

fn push_call_binding_occurrence(
    source: &str,
    container: ferro_hir::Span,
    binding: ferro_hir::BindingId,
    declarations: &[ferro_hir::Binding],
    owner: FerroRustBindingOwner,
    occurrences: &mut Vec<FerroRustOccurrence>,
) {
    let Some(declaration) = declarations.get(binding.0 as usize) else {
        return;
    };
    if let Some(span) = find_call_first_arg_span(source, container, &declaration.source_name) {
        occurrences.push(FerroRustOccurrence {
            binding: FerroRustBindingTarget { owner, id: binding },
            span,
        });
    }
}

fn collect_ferro_rs_statements(
    source: &str,
    statements: &[ferro_hir::Stmt],
    declarations: &[ferro_hir::Binding],
    owner: FerroRustBindingOwner,
    occurrences: &mut Vec<FerroRustOccurrence>,
) {
    use ferro_hir::StmtKind;

    for statement in statements {
        match &statement.kind {
            StmtKind::Declare { initializer, .. } => {
                if let Some(initializer) = initializer {
                    collect_ferro_rs_expression(
                        source,
                        initializer,
                        declarations,
                        owner,
                        occurrences,
                    );
                }
            }
            StmtKind::Assign { binding, value, .. } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(FerroRustOccurrence {
                        binding: FerroRustBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: ferro_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
            }
            StmtKind::IndexAssign {
                binding,
                index,
                value,
            } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(FerroRustOccurrence {
                        binding: FerroRustBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: ferro_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_ferro_rs_expression(source, index, declarations, owner, occurrences);
                collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
            }
            StmtKind::FieldAssign { binding, value, .. } => {
                if let Some(declaration) = declarations.get(binding.0 as usize) {
                    occurrences.push(FerroRustOccurrence {
                        binding: FerroRustBindingTarget {
                            owner,
                            id: *binding,
                        },
                        span: ferro_hir::Span::new(
                            statement.span.start,
                            statement.span.start + declaration.source_name.len() as u32,
                        ),
                    });
                }
                collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
            }
            StmtKind::Write(values) => {
                for value in values {
                    collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
                }
            }
            StmtKind::Call { arguments, .. } | StmtKind::HostCall { arguments, .. } => {
                for value in arguments {
                    collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
                }
            }
            StmtKind::Return { value } => {
                if let Some(value) = value {
                    collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
                }
            }
            StmtKind::Read { binding, name_span } => occurrences.push(FerroRustOccurrence {
                binding: FerroRustBindingTarget {
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
                collect_ferro_rs_expression(source, condition, declarations, owner, occurrences);
                collect_ferro_rs_statements(source, then_branch, declarations, owner, occurrences);
                if let Some(else_branch) = else_branch {
                    collect_ferro_rs_statements(
                        source,
                        else_branch,
                        declarations,
                        owner,
                        occurrences,
                    );
                }
            }
            StmtKind::While { condition, body } | StmtKind::DoWhile { body, condition } => {
                collect_ferro_rs_expression(source, condition, declarations, owner, occurrences);
                collect_ferro_rs_statements(source, body, declarations, owner, occurrences);
            }
            StmtKind::For {
                initializer,
                condition,
                step,
                body,
            } => {
                collect_ferro_rs_statements(
                    source,
                    std::slice::from_ref(initializer.as_ref()),
                    declarations,
                    owner,
                    occurrences,
                );
                collect_ferro_rs_expression(source, condition, declarations, owner, occurrences);
                collect_ferro_rs_statements(
                    source,
                    std::slice::from_ref(step.as_ref()),
                    declarations,
                    owner,
                    occurrences,
                );
                collect_ferro_rs_statements(source, body, declarations, owner, occurrences);
            }
            StmtKind::Choose { value, arms } => {
                collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
                for arm in arms {
                    collect_ferro_rs_statements(
                        source,
                        &arm.body,
                        declarations,
                        owner,
                        occurrences,
                    );
                }
            }
            StmtKind::VectorPush { binding, value } => {
                push_call_binding_occurrence(
                    source,
                    statement.span,
                    *binding,
                    declarations,
                    owner,
                    occurrences,
                );
                collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
            }
            StmtKind::VectorPop { binding } => {
                push_call_binding_occurrence(
                    source,
                    statement.span,
                    *binding,
                    declarations,
                    owner,
                    occurrences,
                );
            }
            StmtKind::LoopControl { .. } => {}
        }
    }
}

fn collect_ferro_rs_expression(
    source: &str,
    expression: &ferro_hir::Expr,
    declarations: &[ferro_hir::Binding],
    owner: FerroRustBindingOwner,
    occurrences: &mut Vec<FerroRustOccurrence>,
) {
    use ferro_hir::ExprKind;

    match &expression.kind {
        ExprKind::Binding(binding) => occurrences.push(FerroRustOccurrence {
            binding: FerroRustBindingTarget {
                owner,
                id: *binding,
            },
            span: expression.span,
        }),
        ExprKind::Parenthesized(inner)
        | ExprKind::Try { operand: inner }
        | ExprKind::Unary { operand: inner, .. } => {
            collect_ferro_rs_expression(source, inner, declarations, owner, occurrences);
        }
        ExprKind::Binary { left, right, .. } => {
            collect_ferro_rs_expression(source, left, declarations, owner, occurrences);
            collect_ferro_rs_expression(source, right, declarations, owner, occurrences);
        }
        ExprKind::Call { arguments, .. } | ExprKind::HostCall { arguments, .. } => {
            for argument in arguments {
                collect_ferro_rs_expression(source, argument, declarations, owner, occurrences);
            }
        }
        ExprKind::VectorLiteral(items) => {
            for item in items {
                collect_ferro_rs_expression(source, item, declarations, owner, occurrences);
            }
        }
        ExprKind::RecordLiteral { fields, .. } => {
            for field in fields {
                collect_ferro_rs_expression(source, field, declarations, owner, occurrences);
            }
        }
        ExprKind::EnumConstructor { payload, .. } => {
            for item in payload {
                collect_ferro_rs_expression(source, item, declarations, owner, occurrences);
            }
        }
        ExprKind::Field { value, .. } => {
            collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
        }
        ExprKind::Index { binding, index } => {
            if let Some(declaration) = declarations.get(binding.0 as usize) {
                occurrences.push(FerroRustOccurrence {
                    binding: FerroRustBindingTarget {
                        owner,
                        id: *binding,
                    },
                    span: ferro_hir::Span::new(
                        expression.span.start,
                        expression.span.start + declaration.source_name.len() as u32,
                    ),
                });
            }
            collect_ferro_rs_expression(source, index, declarations, owner, occurrences);
        }
        ExprKind::Length { binding } | ExprKind::VectorPop { binding } => {
            push_call_binding_occurrence(
                source,
                expression.span,
                *binding,
                declarations,
                owner,
                occurrences,
            );
        }
        ExprKind::VectorGet { binding, index } => {
            push_call_binding_occurrence(
                source,
                expression.span,
                *binding,
                declarations,
                owner,
                occurrences,
            );
            collect_ferro_rs_expression(source, index, declarations, owner, occurrences);
        }
        ExprKind::InterpolatedString { parts } => {
            for part in parts {
                if let ferro_hir::StringPart::Expression(value, _) = part {
                    collect_ferro_rs_expression(source, value, declarations, owner, occurrences);
                }
            }
        }
        ExprKind::Read { .. }
        | ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Character(_) => {}
    }
}

fn locate_ferro_rs(
    offset: u32,
    index: &LineIndex<'_>,
    bindings: &FerroRustBindings,
) -> Result<(FerroRustBindingTarget, String, Vec<ferro_hir::Span>), String> {
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

pub fn rename_target_ferro_rs(source: &str, offset: u32) -> Result<RenameTarget, String> {
    let index = LineIndex::new(source);
    let functions = ferro_rs_function_references(source)?;
    if let Some((_, name, _, spans)) = selected_ferro_function(offset, &index, &functions) {
        return Ok(RenameTarget {
            name,
            occurrences: spans.len(),
        });
    }
    let bindings = ferro_rs_bindings(source)?;
    let (_, name, spans) = locate_ferro_rs(offset, &index, &bindings)?;
    Ok(RenameTarget {
        name,
        occurrences: spans.len(),
    })
}

pub fn resolve_symbol_ferro_rs(source: &str, offset: u32) -> Result<SymbolLocation, String> {
    let index = LineIndex::new(source);
    let functions = ferro_rs_function_references(source)?;
    if let Some((_, _, declaration, _)) = selected_ferro_function(offset, &index, &functions) {
        return Ok(SymbolLocation {
            start: index.position(declaration.start),
            end: index.position(declaration.end),
        });
    }
    let bindings = ferro_rs_bindings(source)?;
    let (binding, _, _) = locate_ferro_rs(offset, &index, &bindings)?;
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

pub fn rename_symbol_ferro_rs(
    source: &str,
    offset: u32,
    name: &str,
) -> Result<Vec<RenameEdit>, String> {
    let probe = format!("funcao inicio() {{ seja {name}: inteiro = 1; }}");
    if ferro_frontend::lower(&probe).is_err() {
        return Err("Digite um identificador válido do Ferro.".into());
    }

    let index = LineIndex::new(source);
    let functions = ferro_rs_function_references(source)?;
    if let Some(selected) = selected_ferro_function(offset, &index, &functions) {
        return rename_ferro_function(source, &index, &functions, selected, name);
    }
    let old = ferro_rs_bindings(source)?;
    let (binding, old_name, spans) = locate_ferro_rs(offset, &index, &old)?;
    if name == old_name {
        return Ok(Vec::new());
    }

    let mut updated = source.to_owned();
    for span in spans.iter().rev() {
        updated.replace_range(span.start as usize..span.end as usize, name);
    }
    let checked = ferro_rs_bindings(&updated).map_err(|_| {
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
    fn ferro_rs_resolves_semantic_bindings_with_shadowing_and_utf16_offsets() {
        let source = "funcao inicio() {\n    seja mutavel valor: inteiro = 1;\n    escreva(\"😀 valor\");\n    valor += 1;\n    se (verdadeiro) {\n        seja valor: inteiro = 2;\n        escreva(valor);\n    }\n    escreva(valor);\n}\n";
        let outer_use = source.rfind("escreva(valor)").unwrap() + "escreva(".len();
        let inner_use = source.find("escreva(valor)").unwrap() + "escreva(".len();
        let as_utf16 = |byte_offset: usize| source[..byte_offset].encode_utf16().count() as u32;

        let outer = resolve_symbol_ferro_rs(source, as_utf16(outer_use)).unwrap();
        let inner = resolve_symbol_ferro_rs(source, as_utf16(inner_use)).unwrap();

        assert_eq!(outer.start.offset, as_utf16(source.find("valor:").unwrap()));
        assert_eq!(
            inner.start.offset,
            as_utf16(source.find("seja valor").unwrap() + "seja ".len())
        );
    }

    #[test]
    fn ferro_rs_rename_updates_only_the_selected_binding() {
        let source = "funcao inicio() { seja mutavel valor: inteiro = 1; escreva(\"valor\"); valor += 1; se (verdadeiro) { seja valor: inteiro = 2; escreva(valor); } escreva(valor); }";
        let cursor_byte = source.rfind("escreva(valor)").unwrap() + "escreva(".len();
        let cursor = source[..cursor_byte].encode_utf16().count() as u32;

        let target = rename_target_ferro_rs(source, cursor).unwrap();
        let edits = rename_symbol_ferro_rs(source, cursor, "total").unwrap();

        assert_eq!(target.name, "valor");
        assert_eq!(target.occurrences, 3);
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "total"));
    }

    #[test]
    fn ferro_rs_rename_includes_the_leia_target() {
        let source =
            "funcao inicio() { seja mutavel idade: inteiro = 0; leia(idade); escreva(idade); }";
        let cursor = source.find("idade:").unwrap() as u32;
        let edits = rename_symbol_ferro_rs(source, cursor, "anos").unwrap();
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "anos"));
    }

    #[test]
    fn ferro_rs_rename_in_counted_loop_updates_header_and_body_bindings() {
        let source = "funcao inicio() { para (seja mutavel i: inteiro = 1; i < 4; i += i) { para (seja mutavel i: inteiro = 0; i < 1; i += 1) { escreva(i); } escreva(i); } }";
        let cursor = source.rfind("escreva(i)").unwrap() as u32 + "escreva(".len() as u32;

        let target = rename_target_ferro_rs(source, cursor).unwrap();
        let edits = rename_symbol_ferro_rs(source, cursor, "contador").unwrap();

        assert_eq!(target.name, "i");
        assert_eq!(target.occurrences, 5);
        assert_eq!(edits.len(), 5);
        assert!(edits.iter().all(|edit| edit.insert == "contador"));
    }

    #[test]
    fn ferro_rs_rename_rejects_capture_and_keeps_entry_function_name() {
        let source = "funcao inicio() { seja mutavel valor: inteiro = 1; seja outro: inteiro = 2; escreva(valor, outro); }";
        let value = source.find("valor:").unwrap() as u32;
        let entry = source.find("inicio").unwrap() as u32;

        assert!(rename_symbol_ferro_rs(source, value, "outro").is_err());
        assert!(rename_target_ferro_rs(source, entry).is_err());
    }

    #[test]
    fn ferro_rs_resolves_and_renames_helper_function_declarations_and_calls() {
        let source = "funcao somar(a: inteiro) -> inteiro { retorne a + 1; }\nfuncao inicio() { somar(2); escreva(somar(3)); }";
        let declaration_byte = source.find("somar").unwrap();
        let call_byte = source.rfind("somar").unwrap();
        let as_utf16 = |offset: usize| source[..offset].encode_utf16().count() as u32;

        let target = rename_target_ferro_rs(source, as_utf16(call_byte)).unwrap();
        let location = resolve_symbol_ferro_rs(source, as_utf16(call_byte)).unwrap();
        let edits = rename_symbol_ferro_rs(source, as_utf16(call_byte), "somar_um").unwrap();

        assert_eq!(target.name, "somar");
        assert_eq!(target.occurrences, 3);
        assert_eq!(location.start.offset, as_utf16(declaration_byte));
        assert_eq!(edits.len(), 3);
        assert!(edits.iter().all(|edit| edit.insert == "somar_um"));
    }

    #[test]
    fn ferro_rs_renames_parameters_and_locals_inside_helper_functions() {
        let source = "funcao calcular(entrada: inteiro) -> inteiro { seja mutavel total: inteiro = entrada; total += entrada; retorne total; }\nfuncao inicio() { escreva(calcular(2)); }";
        let total_use_byte = source.rfind("total").unwrap();
        let as_utf16 = |offset: usize| source[..offset].encode_utf16().count() as u32;

        let target = rename_target_ferro_rs(source, as_utf16(total_use_byte)).unwrap();
        let edits = rename_symbol_ferro_rs(source, as_utf16(total_use_byte), "soma").unwrap();
        let parameter_byte = source.find("entrada: ").unwrap();
        let parameter_edits =
            rename_symbol_ferro_rs(source, as_utf16(parameter_byte), "valor_inicial").unwrap();

        assert_eq!(target.name, "total");
        assert_eq!(edits.len(), 3);
        assert_eq!(parameter_edits.len(), 3);
    }

    #[test]
    fn ferro_rs_resolves_and_renames_vector_bindings_in_all_operations() {
        let source = concat!(
            "funcao inicio() {\n",
            "    seja mutavel notas: vetor<inteiro> = [1, 2];\n",
            "    adicionar(notas, 3);\n",
            "    notas[0] = notas[1];\n",
            "    seja qtd: inteiro = tamanho(notas);\n",
            "    escolha obter(notas, 0) {\n",
            "        caso Opcao::Algum(n) => { escreva(n); }\n",
            "        caso Opcao::Nenhum => { escreva(qtd); }\n",
            "    }\n",
            "    seja ultimo: Opcao<inteiro> = remover(notas);\n",
            "    remover(notas);\n",
            "    escolha ultimo {\n",
            "        caso Opcao::Algum(v) => { escreva(v); }\n",
            "        caso Opcao::Nenhum => {}\n",
            "    }\n",
            "}\n",
        );
        let decl = source.find("notas:").unwrap() as u32;
        let in_push = source.find("adicionar(notas").unwrap() as u32 + "adicionar(".len() as u32;
        let in_len = source.find("tamanho(notas)").unwrap() as u32 + "tamanho(".len() as u32;
        let in_get = source.find("obter(notas").unwrap() as u32 + "obter(".len() as u32;
        let in_pop_stmt = source.rfind("remover(notas)").unwrap() as u32 + "remover(".len() as u32;

        for cursor in [in_push, in_len, in_get, in_pop_stmt] {
            let resolved = resolve_symbol_ferro_rs(source, cursor).unwrap();
            assert_eq!(resolved.start.offset, decl);
        }

        let target = rename_target_ferro_rs(source, in_len).unwrap();
        assert_eq!(target.name, "notas");
        assert_eq!(target.occurrences, 8);

        let edits = rename_symbol_ferro_rs(source, in_len, "valores").unwrap();
        assert_eq!(edits.len(), 8);
        let mut rewritten = source.to_owned();
        for edit in edits.iter().rev() {
            rewritten.replace_range(edit.from as usize..edit.to as usize, &edit.insert);
        }
        assert!(!rewritten.contains("notas"));
        assert!(rewritten.contains("valores[0] = valores[1];"));
        assert!(rewritten.contains("tamanho(valores)"));
        assert!(rewritten.contains("adicionar(valores, 3)"));
        assert!(rewritten.contains("obter(valores, 0)"));
        assert!(rewritten.contains("remover(valores);"));
        assert!(ferro_frontend::lower(&rewritten).is_ok());
    }
}
