use portugol_hir::{BinaryOp, Expr, ExprKind, Program, Span, Stmt, StringPart, Type, UnaryOp};

use crate::{SourceMapQuality, SourceMapSegment, SourceRange, format_real, rust_ident};

use super::types::{
    binary_operator, binding_info, integer_method, pors_clone_value, render_stdlib_host_call,
    render_ui_host_call, rust_enum_variant_path, rust_type,
};

#[derive(Debug)]
pub(super) struct MappedExpression {
    code: String,
    mappings: Vec<SourceMapSegment>,
}

pub(super) enum ExpressionPart {
    Text(String),
    Nested(MappedExpression),
}

pub(super) fn render_expr(expr: &Expr, program: &Program, pors: bool) -> MappedExpression {
    let parts = match &expr.kind {
        ExprKind::Binding(id) => vec![ExpressionPart::Text(rust_ident(
            &binding_info(program, *id).source_name,
        ))],
        ExprKind::VectorLiteral(items) => {
            if items.is_empty() {
                let elem_ty = expr
                    .ty
                    .vector_element()
                    .map(|ty| rust_type(ty, program))
                    .unwrap_or_else(|| "_".to_owned());
                vec![ExpressionPart::Text(format!("Vec::<{elem_ty}>::new()"))]
            } else {
                let mut parts = vec![ExpressionPart::Text("vec![".to_owned())];
                for (index, item) in items.iter().enumerate() {
                    if index > 0 {
                        parts.push(ExpressionPart::Text(", ".to_owned()));
                    }
                    parts.push(ExpressionPart::Nested(owned_value(item, program, pors)));
                }
                parts.push(ExpressionPart::Text("]".to_owned()));
                parts
            }
        }
        ExprKind::RecordLiteral { type_id, fields } => {
            let definition = program
                .record_types
                .iter()
                .find(|definition| definition.id == *type_id);
            let name = definition
                .map(|definition| rust_ident(&definition.name))
                .unwrap_or_else(|| "__invalid_record".to_owned());
            let mut parts = vec![ExpressionPart::Text(format!("{name} {{ "))];
            for (index, field_value) in fields.iter().enumerate() {
                if index > 0 {
                    parts.push(ExpressionPart::Text(", ".to_owned()));
                }
                let field_name = definition
                    .and_then(|definition| definition.fields.get(index))
                    .map(|field| rust_ident(&field.name))
                    .unwrap_or_else(|| "__invalid_field".to_owned());
                parts.push(ExpressionPart::Text(format!("{field_name}: ")));
                parts.push(ExpressionPart::Nested(owned_value(
                    field_value,
                    program,
                    pors,
                )));
            }
            parts.push(ExpressionPart::Text(" }".to_owned()));
            parts
        }
        ExprKind::EnumConstructor {
            type_id,
            variant_index,
            payload,
        } => {
            let definition = program
                .enum_types
                .iter()
                .find(|definition| definition.id == *type_id);
            let variant_path = definition
                .map(|definition| rust_enum_variant_path(definition, *variant_index))
                .unwrap_or_else(|| "__invalid_enum::__invalid_variant".to_owned());
            let mut parts = vec![ExpressionPart::Text(variant_path)];
            if !payload.is_empty() {
                parts.push(ExpressionPart::Text("(".to_owned()));
                for (index, item) in payload.iter().enumerate() {
                    if index > 0 {
                        parts.push(ExpressionPart::Text(", ".to_owned()));
                    }
                    parts.push(ExpressionPart::Nested(owned_value(item, program, pors)));
                }
                parts.push(ExpressionPart::Text(")".to_owned()));
            }
            parts
        }
        ExprKind::Field { value, field_index } => {
            let field_name = match value.ty {
                Type::Record(type_id) => program
                    .record_types
                    .iter()
                    .find(|definition| definition.id == type_id)
                    .and_then(|definition| definition.fields.get(*field_index))
                    .map(|field| rust_ident(&field.name)),
                _ => None,
            }
            .unwrap_or_else(|| "__invalid_field".to_owned());
            vec![
                ExpressionPart::Text("(".to_owned()),
                ExpressionPart::Nested(render_expr(value, program, pors)),
                ExpressionPart::Text(format!(").{field_name}")),
            ]
        }
        ExprKind::Index { binding, index } => vec![
            ExpressionPart::Text(format!(
                "{}[__pors_index(",
                rust_ident(&binding_info(program, *binding).source_name)
            )),
            ExpressionPart::Nested(render_expr(index, program, pors)),
            ExpressionPart::Text(format!(
                ", {}.len())]",
                rust_ident(&binding_info(program, *binding).source_name)
            )),
        ],
        ExprKind::Length { binding } => vec![ExpressionPart::Text(format!(
            "{}.len() as i32",
            rust_ident(&binding_info(program, *binding).source_name)
        ))],
        ExprKind::VectorPop { binding } => vec![ExpressionPart::Text(format!(
            "{}.pop()",
            rust_ident(&binding_info(program, *binding).source_name)
        ))],
        ExprKind::VectorGet { binding, index } => {
            let info = binding_info(program, *binding);
            let name = rust_ident(&info.source_name);
            let clone_item = info
                .ty
                .vector_element()
                .map(|elem_ty| pors_clone_value(elem_ty, "item", program))
                .unwrap_or_else(|| "(*item).clone()".to_owned());
            vec![
                ExpressionPart::Text("({ let __pors_idx = (".to_owned()),
                ExpressionPart::Nested(render_expr(index, program, pors)),
                ExpressionPart::Text(format!(
                    "); if __pors_idx < 0 {{ Option::None }} else {{ {name}.get(__pors_idx as usize).map(|item| {clone_item}) }} }})"
                )),
            ]
        }
        ExprKind::Call {
            function,
            arguments,
        } => {
            let target = program.functions.iter().find(|item| item.id == *function);
            let name = target
                .map(|item| rust_ident(&item.source_name))
                .unwrap_or_else(|| "__invalid_function".to_owned());
            let mut parts = vec![ExpressionPart::Text(format!("{name}("))];
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    parts.push(ExpressionPart::Text(", ".to_owned()));
                }
                let takes_string = target
                    .and_then(|target_function| {
                        target_function
                            .parameters
                            .get(index)
                            .map(|parameter_id| (target_function, parameter_id))
                    })
                    .is_some_and(|(target_function, parameter_id)| {
                        binding_info(
                            &Program {
                                bindings: target_function.bindings.clone(),
                                ..program.clone()
                            },
                            *parameter_id,
                        )
                        .ty == Type::String
                    });
                let takes_vector = target
                    .and_then(|target_function| {
                        target_function.parameters.get(index).map(|parameter_id| {
                            binding_info(
                                &Program {
                                    bindings: target_function.bindings.clone(),
                                    ..program.clone()
                                },
                                *parameter_id,
                            )
                            .ty
                            .is_vector()
                                || matches!(
                                    binding_info(
                                        &Program {
                                            bindings: target_function.bindings.clone(),
                                            ..program.clone()
                                        },
                                        *parameter_id
                                    )
                                    .ty,
                                    Type::Record(_) | Type::Enum(_)
                                )
                        })
                    })
                    .unwrap_or(false);
                if takes_vector {
                    parts.push(ExpressionPart::Nested(owned_value(argument, program, pors)));
                } else {
                    if takes_string {
                        parts.push(ExpressionPart::Text(if pors {
                            "__pors_string((".to_owned()
                        } else {
                            "String::clone(&(".to_owned()
                        }));
                    }
                    parts.push(ExpressionPart::Nested(render_expr(argument, program, pors)));
                    if takes_string {
                        parts.push(ExpressionPart::Text(if pors {
                            ").as_str())".to_owned()
                        } else {
                            "))".to_owned()
                        }));
                    }
                }
            }
            parts.push(ExpressionPart::Text(")".to_owned()));
            parts
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "UI" => render_ui_host_call(name, arguments, program, pors),
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Matematica" => {
            render_stdlib_host_call("__pors_matematica", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Texto" => {
            render_stdlib_host_call("__pors_texto", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Tipos" => {
            render_stdlib_host_call("__pors_tipos", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Util" => {
            render_stdlib_host_call("__pors_util", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Calendario" => {
            render_stdlib_host_call("__pors_calendario", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Arquivos" => {
            render_stdlib_host_call("__pors_arquivos", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Json" => {
            render_stdlib_host_call("__pors_json", name, arguments, program, pors)
        }
        ExprKind::HostCall {
            module,
            name,
            arguments,
        } if module == "Internet" => {
            render_stdlib_host_call("__pors_internet", name, arguments, program, pors)
        }
        ExprKind::HostCall { .. } => vec![ExpressionPart::Text("0_i32".to_owned())],
        ExprKind::Integer(value) if *value == i32::MIN => {
            vec![ExpressionPart::Text("i32::MIN".to_owned())]
        }
        ExprKind::Integer(value) => vec![ExpressionPart::Text(format!("{value}_i32"))],
        ExprKind::Real(value) => vec![ExpressionPart::Text(format_real(*value))],
        ExprKind::Boolean(value) => vec![ExpressionPart::Text(value.to_string())],
        ExprKind::String(value) => vec![ExpressionPart::Text(if pors {
            format!("__pors_string({value:?})")
        } else {
            format!("String::from({value:?})")
        })],
        ExprKind::Read { .. } => vec![ExpressionPart::Nested(compose_expression(
            vec![ExpressionPart::Text(if pors {
                "__pors_string(&__portugol_read_line(\"cadeia\"))".to_owned()
            } else {
                "__portugol_read_line(\"cadeia\")".to_owned()
            })],
            expr.span,
        ))],
        ExprKind::InterpolatedString { parts } => {
            let mut accumulator = None;
            for part in parts {
                let (piece, span) = match part {
                    StringPart::Literal(text, span) => {
                        let code = if pors {
                            format!("__pors_string({text:?})")
                        } else {
                            format!("String::from({text:?})")
                        };
                        (
                            compose_expression(vec![ExpressionPart::Text(code)], *span),
                            *span,
                        )
                    }
                    StringPart::Expression(value, span) => {
                        let ty = value.ty;
                        let rendered = render_expr(value, program, pors);
                        let owned = if pors {
                            compose_expression(
                                vec![
                                    ExpressionPart::Text("__pors_string(".to_owned()),
                                    ExpressionPart::Nested(pors_string_view(rendered, ty)),
                                    ExpressionPart::Text(")".to_owned()),
                                ],
                                Span::default(),
                            )
                        } else {
                            compose_expression(
                                vec![
                                    ExpressionPart::Text("format!(\"{}\", ".to_owned()),
                                    ExpressionPart::Nested(rendered),
                                    ExpressionPart::Text(")".to_owned()),
                                ],
                                Span::default(),
                            )
                        };
                        (
                            compose_expression(vec![ExpressionPart::Nested(owned)], *span),
                            *span,
                        )
                    }
                };
                accumulator = Some(match accumulator {
                    None => piece,
                    Some(left) => concat_string_expressions(left, piece, span, pors),
                });
            }
            vec![ExpressionPart::Nested(accumulator.unwrap_or_else(|| {
                compose_expression(
                    vec![ExpressionPart::Text(if pors {
                        "__pors_string(\"\")".to_owned()
                    } else {
                        "String::new()".to_owned()
                    })],
                    expr.span,
                )
            }))]
        }
        ExprKind::Character(value) => vec![ExpressionPart::Text(format!("{value:?}"))],
        ExprKind::Parenthesized(inner) => vec![
            ExpressionPart::Text("(".to_owned()),
            ExpressionPart::Nested(render_expr(inner, program, pors)),
            ExpressionPart::Text(")".to_owned()),
        ],
        ExprKind::Try { operand } => vec![
            ExpressionPart::Text("(".to_owned()),
            ExpressionPart::Nested(owned_value(operand, program, pors)),
            ExpressionPart::Text(")?".to_owned()),
        ],
        ExprKind::Unary { op, operand } => {
            let nested = render_expr(operand, program, pors);
            match (op, expr.ty) {
                (UnaryOp::Negate, Type::Integer) => vec![
                    ExpressionPart::Text("(".to_owned()),
                    ExpressionPart::Nested(nested),
                    ExpressionPart::Text(").wrapping_neg()".to_owned()),
                ],
                (UnaryOp::Negate, Type::Real) => vec![
                    ExpressionPart::Text("-(".to_owned()),
                    ExpressionPart::Nested(nested),
                    ExpressionPart::Text(")".to_owned()),
                ],
                (UnaryOp::Positive, _) => vec![ExpressionPart::Nested(nested)],
                (UnaryOp::Not | UnaryOp::BitNot, _) => vec![
                    ExpressionPart::Text("!(".to_owned()),
                    ExpressionPart::Nested(nested),
                    ExpressionPart::Text(")".to_owned()),
                ],
                _ => unreachable!("unary operand types are validated by the lowering"),
            }
        }
        ExprKind::Binary { op, left, right } => {
            let left_type = left.ty;
            let right_type = right.ty;
            let left = render_expr(left, program, pors);
            let right = render_expr(right, program, pors);
            if *op == BinaryOp::Add
                && ((left_type == Type::String && right_type == Type::String)
                    || (left_type == Type::Character && right_type == Type::Character))
            {
                vec![
                    ExpressionPart::Text(if pors {
                        "__pors_concat(".to_owned()
                    } else {
                        "format!(\"{}{}\", ".to_owned()
                    }),
                    ExpressionPart::Nested(if pors {
                        pors_string_view(left, left_type)
                    } else {
                        left
                    }),
                    ExpressionPart::Text(", ".to_owned()),
                    ExpressionPart::Nested(if pors {
                        pors_string_view(right, right_type)
                    } else {
                        right
                    }),
                    ExpressionPart::Text(")".to_owned()),
                ]
            } else if expr.ty == Type::Integer
                && let Some(method) = integer_method(*op)
            {
                let right = if matches!(op, BinaryOp::ShiftLeft | BinaryOp::ShiftRight) {
                    wrap_expression(right, "(", " as u32)")
                } else {
                    right
                };
                vec![
                    ExpressionPart::Text("(".to_owned()),
                    ExpressionPart::Nested(left),
                    ExpressionPart::Text(format!(").{method}(")),
                    ExpressionPart::Nested(right),
                    ExpressionPart::Text(")".to_owned()),
                ]
            } else {
                vec![
                    ExpressionPart::Text("(".to_owned()),
                    ExpressionPart::Nested(left),
                    ExpressionPart::Text(format!(") {} (", binary_operator(*op))),
                    ExpressionPart::Nested(right),
                    ExpressionPart::Text(")".to_owned()),
                ]
            }
        }
    };
    compose_expression(parts, expr.span)
}

fn pors_string_view(value: MappedExpression, ty: Type) -> MappedExpression {
    let (prefix, suffix) = if ty == Type::Character {
        ("__pors_char(", ").as_str()")
    } else {
        ("(", ").as_str()")
    };
    compose_expression(
        vec![
            ExpressionPart::Text(prefix.to_owned()),
            ExpressionPart::Nested(value),
            ExpressionPart::Text(suffix.to_owned()),
        ],
        Span::default(),
    )
}

fn concat_string_expressions(
    left: MappedExpression,
    right: MappedExpression,
    span: Span,
    pors: bool,
) -> MappedExpression {
    if pors {
        compose_expression(
            vec![
                ExpressionPart::Text("__pors_concat(".to_owned()),
                ExpressionPart::Nested(pors_string_view(left, Type::String)),
                ExpressionPart::Text(", ".to_owned()),
                ExpressionPart::Nested(pors_string_view(right, Type::String)),
                ExpressionPart::Text(")".to_owned()),
            ],
            span,
        )
    } else {
        compose_expression(
            vec![
                ExpressionPart::Text("format!(\"{}{}\", ".to_owned()),
                ExpressionPart::Nested(left),
                ExpressionPart::Text(", ".to_owned()),
                ExpressionPart::Nested(right),
                ExpressionPart::Text(")".to_owned()),
            ],
            span,
        )
    }
}

fn wrap_expression(expression: MappedExpression, prefix: &str, suffix: &str) -> MappedExpression {
    compose_expression(
        vec![
            ExpressionPart::Text(prefix.to_owned()),
            ExpressionPart::Nested(expression),
            ExpressionPart::Text(suffix.to_owned()),
        ],
        Span::default(),
    )
}

fn compose_expression(parts: Vec<ExpressionPart>, span: Span) -> MappedExpression {
    let mut code = String::new();
    let mut mappings = Vec::new();
    for part in parts {
        match part {
            ExpressionPart::Text(text) => code.push_str(&text),
            ExpressionPart::Nested(nested) => {
                let offset = code.len() as u32;
                code.push_str(&nested.code);
                mappings.extend(nested.mappings.into_iter().map(|mut mapping| {
                    mapping.generated.start += offset;
                    mapping.generated.end += offset;
                    mapping
                }));
            }
        }
    }
    if span.end > span.start && !code.is_empty() {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: 0,
                end: code.len() as u32,
            },
            source: Some(SourceRange {
                start: span.start,
                end: span.end,
            }),
            quality: SourceMapQuality::Exact,
        });
    }
    mappings.sort_by_key(|mapping| (mapping.generated.start, mapping.generated.end));
    MappedExpression { code, mappings }
}

pub(super) fn owned_value(expr: &Expr, program: &Program, pors: bool) -> MappedExpression {
    let value = render_expr(expr, program, pors);
    if matches!(
        expr.kind,
        ExprKind::VectorLiteral(_)
            | ExprKind::RecordLiteral { .. }
            | ExprKind::EnumConstructor { .. }
            | ExprKind::Call { .. }
    ) {
        return value;
    }
    if expr.ty.is_vector() {
        if let Type::VectorRecord(id) | Type::VectorEnum(id) = expr.ty {
            let value = render_expr(expr, program, pors);
            let clone = match expr.ty {
                Type::VectorRecord(_) if pors => format!("__pors_clone_{}(item)", id.0),
                Type::VectorEnum(_) if pors => format!("__pors_clone_enum_{}(item)", id.0),
                _ => "item.clone()".to_owned(),
            };
            let prefix = "(";
            let middle = ").iter().map(|item| ";
            let suffix = ").collect::<Vec<_>>()";
            let code = format!("{prefix}{}{middle}{clone}{suffix}", value.code);
            let offset = prefix.len() as u32;
            let mappings = value
                .mappings
                .into_iter()
                .map(|mut mapping| {
                    mapping.generated.start += offset;
                    mapping.generated.end += offset;
                    mapping
                })
                .collect();
            return MappedExpression { code, mappings };
        }
        let (prefix, suffix) = if pors && expr.ty == Type::VectorString {
            ("__pors_vec_string_clone(&(", "))")
        } else {
            ("(", ").clone()")
        };
        return compose_expression(
            vec![
                ExpressionPart::Text(prefix.to_owned()),
                ExpressionPart::Nested(value),
                ExpressionPart::Text(suffix.to_owned()),
            ],
            expr.span,
        );
    }
    if let Type::Record(type_id) = expr.ty
        && let Some(definition) = program
            .record_types
            .iter()
            .find(|definition| definition.id == type_id)
    {
        let id = definition.id.0;
        let value = render_expr(expr, program, pors);
        let clone = if pors {
            format!("__pors_clone_{id}(&(")
        } else {
            "(".to_owned()
        };
        let suffix = if pors { "))" } else { ").clone()" };
        return compose_expression(
            vec![
                ExpressionPart::Text(clone),
                ExpressionPart::Nested(value),
                ExpressionPart::Text(suffix.to_owned()),
            ],
            expr.span,
        );
    }
    if let Type::Enum(type_id) = expr.ty
        && program
            .enum_types
            .iter()
            .any(|definition| definition.id == type_id)
    {
        let value = render_expr(expr, program, pors);
        let prefix = if pors {
            format!("__pors_clone_enum_{}(&(", type_id.0)
        } else {
            "(".to_owned()
        };
        let suffix = if pors { "))" } else { ").clone()" };
        return compose_expression(
            vec![
                ExpressionPart::Text(prefix),
                ExpressionPart::Nested(value),
                ExpressionPart::Text(suffix.to_owned()),
            ],
            expr.span,
        );
    }
    if expr.ty == Type::String {
        if pors {
            return compose_expression(
                vec![
                    ExpressionPart::Text("__pors_string((".to_owned()),
                    ExpressionPart::Nested(value),
                    ExpressionPart::Text(").as_str())".to_owned()),
                ],
                expr.span,
            );
        }
        compose_expression(
            vec![
                ExpressionPart::Text("(".to_owned()),
                ExpressionPart::Nested(value),
                ExpressionPart::Text(").clone()".to_owned()),
            ],
            expr.span,
        )
    } else {
        value
    }
}

pub(super) fn push_expr(
    output: &mut String,
    mappings: &mut Vec<SourceMapSegment>,
    expr: &Expr,
    program: &Program,
    owned: bool,
    pors: bool,
) {
    let expression = if owned {
        owned_value(expr, program, pors)
    } else {
        render_expr(expr, program, pors)
    };
    let offset = output.len() as u32;
    output.push_str(&expression.code);
    mappings.extend(expression.mappings.into_iter().map(|mut mapping| {
        mapping.generated.start += offset;
        mapping.generated.end += offset;
        mapping
    }));
}

pub(super) fn record_statement_mapping(
    mappings: &mut Vec<SourceMapSegment>,
    start: usize,
    end: usize,
    statement: &Stmt,
) {
    if end > start && statement.span.end > statement.span.start {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: start as u32,
                end: end as u32,
            },
            source: Some(SourceRange {
                start: statement.span.start,
                end: statement.span.end,
            }),
            quality: SourceMapQuality::Exact,
        });
    }
}
