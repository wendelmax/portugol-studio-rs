use portugol_hir::{
    AssignOp, BinaryOp, BindingId, Expr, ExprKind, Program, Span, Stmt, StmtKind, Type, UnaryOp,
};

use crate::{
    GeneratedRust, JAVA_DOUBLE_HELPER, SourceMapQuality, SourceMapSegment, SourceRange,
    default_value, format_real, rust_ident,
};

pub(crate) fn render_with_map(program: &Program) -> GeneratedRust {
    let mut output = String::from("use std::io::{self, Write};\n\n");
    output.push_str(JAVA_DOUBLE_HELPER);
    output.push_str("\nfn main() {\n");
    let body_start = output.len();
    let mut mappings = Vec::new();
    render_statements(&program.body, program, 1, &mut output, &mut mappings);
    let body_end = output.len();
    output.push_str("}\n");
    if body_start > 0 {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: 0,
                end: body_start as u32,
            },
            source: None,
            quality: SourceMapQuality::GeneratedOnly,
        });
    }
    if output.len() > body_end {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: body_end as u32,
                end: output.len() as u32,
            },
            source: None,
            quality: SourceMapQuality::GeneratedOnly,
        });
    }
    mappings.sort_by_key(|mapping| (mapping.generated.start, mapping.generated.end));
    GeneratedRust {
        code: output,
        mappings,
    }
}

fn render_statements(
    statements: &[Stmt],
    program: &Program,
    depth: usize,
    output: &mut String,
    mappings: &mut Vec<SourceMapSegment>,
) {
    let indent = "    ".repeat(depth);
    for statement in statements {
        let start = output.len();
        match &statement.kind {
            StmtKind::Declare {
                binding,
                initializer,
            } => {
                let declaration = binding_info(program, *binding);
                output.push_str(&format!(
                    "{indent}{} {}: {}",
                    if declaration.is_const {
                        "let"
                    } else {
                        "let mut"
                    },
                    rust_ident(&declaration.source_name),
                    rust_type(declaration.ty)
                ));
                let initializer = initializer.as_ref().map(|value| {
                    output.push_str(" = ");
                    push_expr(output, mappings, value, program, true);
                });
                if initializer.is_none() {
                    output.push_str(&format!(" = {};", default_value(map_type(declaration.ty))));
                } else {
                    output.push(';');
                }
                output.push('\n');
            }
            StmtKind::Assign { binding, op, value } => {
                let declaration = binding_info(program, *binding);
                let name = rust_ident(&declaration.source_name);
                let ty = declaration.ty;
                if ty == Type::Integer {
                    let method = match op {
                        AssignOp::Add => Some("wrapping_add"),
                        AssignOp::Subtract => Some("wrapping_sub"),
                        AssignOp::Multiply => Some("wrapping_mul"),
                        AssignOp::Divide => Some("wrapping_div"),
                        AssignOp::Remainder => Some("wrapping_rem"),
                        _ => None,
                    };
                    if let Some(method) = method {
                        output.push_str(&format!("{indent}{name} = ({name}).{method}("));
                        push_expr(output, mappings, value, program, false);
                        output.push_str(");\n");
                        record_statement_mapping(mappings, start, output.len(), statement);
                        continue;
                    }
                }
                if ty == Type::String && *op == AssignOp::Add {
                    output.push_str(&format!("{indent}{name} = format!(\"{{}}{{}}\", {name}, "));
                    push_expr(output, mappings, value, program, true);
                    output.push_str(");\n");
                    record_statement_mapping(mappings, start, output.len(), statement);
                    continue;
                }
                let assignment_op = match op {
                    AssignOp::Set => "=",
                    AssignOp::Add => "+=",
                    AssignOp::Subtract => "-=",
                    AssignOp::Multiply => "*=",
                    AssignOp::Divide => "/=",
                    AssignOp::Remainder => "%=",
                    AssignOp::Xor => "^=",
                    AssignOp::BitAnd => "&=",
                    AssignOp::BitOr => "|=",
                };
                output.push_str(&format!("{indent}{name} {assignment_op} "));
                push_expr(output, mappings, value, program, *op == AssignOp::Set);
                output.push_str(";\n");
            }
            StmtKind::Write(values) => {
                let temps: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("__portugol_write_{index}"))
                    .collect();
                for (temp, value) in temps.iter().zip(values) {
                    output.push_str(&format!("{indent}let {temp} = "));
                    push_expr(output, mappings, value, program, true);
                    output.push_str(";\n");
                }
                for (temp, value) in temps.iter().zip(values) {
                    let expression = match value.ty {
                        Type::Boolean => {
                            format!("if {temp} {{ \"verdadeiro\" }} else {{ \"falso\" }}")
                        }
                        Type::Real => format!("__portugol_java_double({temp})"),
                        _ => temp.clone(),
                    };
                    output.push_str(&format!("{indent}print!(\"{{}}\", {expression});\n"));
                }
                output.push_str(&format!("{indent}let _ = io::stdout().flush();\n"));
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                output.push_str(&format!("{indent}if "));
                push_expr(output, mappings, condition, program, false);
                output.push_str(" {\n");
                render_statements(then_branch, program, depth + 1, output, mappings);
                output.push_str(&format!("{indent}}}"));
                if let Some(else_branch) = else_branch {
                    output.push_str(" else {\n");
                    render_statements(else_branch, program, depth + 1, output, mappings);
                    output.push_str(&format!("{indent}}}"));
                }
                output.push('\n');
            }
        }
        record_statement_mapping(mappings, start, output.len(), statement);
    }
}

#[derive(Debug)]
struct MappedExpression {
    code: String,
    mappings: Vec<SourceMapSegment>,
}

enum ExpressionPart {
    Text(String),
    Nested(MappedExpression),
}

fn render_expr(expr: &Expr, program: &Program) -> MappedExpression {
    let parts = match &expr.kind {
        ExprKind::Binding(id) => vec![ExpressionPart::Text(rust_ident(
            &binding_info(program, *id).source_name,
        ))],
        ExprKind::Integer(value) if *value == i32::MIN => {
            vec![ExpressionPart::Text("i32::MIN".to_owned())]
        }
        ExprKind::Integer(value) => vec![ExpressionPart::Text(format!("{value}_i32"))],
        ExprKind::Real(value) => vec![ExpressionPart::Text(format_real(*value))],
        ExprKind::Boolean(value) => vec![ExpressionPart::Text(value.to_string())],
        ExprKind::String(value) => vec![ExpressionPart::Text(format!("String::from({value:?})"))],
        ExprKind::Character(value) => vec![ExpressionPart::Text(format!("{value:?}"))],
        ExprKind::Parenthesized(inner) => vec![
            ExpressionPart::Text("(".to_owned()),
            ExpressionPart::Nested(render_expr(inner, program)),
            ExpressionPart::Text(")".to_owned()),
        ],
        ExprKind::Unary { op, operand } => {
            let nested = render_expr(operand, program);
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
            let left = render_expr(left, program);
            let right = render_expr(right, program);
            if *op == BinaryOp::Add
                && ((left_type == Type::String && right_type == Type::String)
                    || (left_type == Type::Character && right_type == Type::Character))
            {
                vec![
                    ExpressionPart::Text("format!(\"{}{}\", ".to_owned()),
                    ExpressionPart::Nested(left),
                    ExpressionPart::Text(", ".to_owned()),
                    ExpressionPart::Nested(right),
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

fn owned_value(expr: &Expr, program: &Program) -> MappedExpression {
    let value = render_expr(expr, program);
    if expr.ty == Type::String {
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

fn push_expr(
    output: &mut String,
    mappings: &mut Vec<SourceMapSegment>,
    expr: &Expr,
    program: &Program,
    owned: bool,
) {
    let expression = if owned {
        owned_value(expr, program)
    } else {
        render_expr(expr, program)
    };
    let offset = output.len() as u32;
    output.push_str(&expression.code);
    mappings.extend(expression.mappings.into_iter().map(|mut mapping| {
        mapping.generated.start += offset;
        mapping.generated.end += offset;
        mapping
    }));
}

fn record_statement_mapping(
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

fn integer_method(op: BinaryOp) -> Option<&'static str> {
    match op {
        BinaryOp::Add => Some("wrapping_add"),
        BinaryOp::Subtract => Some("wrapping_sub"),
        BinaryOp::Multiply => Some("wrapping_mul"),
        BinaryOp::Divide => Some("wrapping_div"),
        BinaryOp::Remainder => Some("wrapping_rem"),
        BinaryOp::ShiftLeft => Some("wrapping_shl"),
        BinaryOp::ShiftRight => Some("wrapping_shr"),
        _ => None,
    }
}

fn binary_operator(op: BinaryOp) -> &'static str {
    match op {
        BinaryOp::Add => "+",
        BinaryOp::Subtract => "-",
        BinaryOp::Multiply => "*",
        BinaryOp::Divide => "/",
        BinaryOp::Remainder => "%",
        BinaryOp::And => "&&",
        BinaryOp::Or => "||",
        BinaryOp::Xor => "^",
        BinaryOp::Equal => "==",
        BinaryOp::NotEqual => "!=",
        BinaryOp::Greater => ">",
        BinaryOp::Less => "<",
        BinaryOp::LessEqual => "<=",
        BinaryOp::GreaterEqual => ">=",
        BinaryOp::ShiftLeft => "<<",
        BinaryOp::ShiftRight => ">>",
        BinaryOp::BitAnd => "&",
        BinaryOp::BitOr => "|",
    }
}

fn binding_info(program: &Program, id: BindingId) -> &portugol_hir::Binding {
    &program.bindings[id.0 as usize]
}

fn rust_type(ty: Type) -> &'static str {
    match ty {
        Type::Integer => "i32",
        Type::Real => "f64",
        Type::Boolean => "bool",
        Type::String => "String",
        Type::Character => "char",
    }
}

fn map_type(ty: Type) -> crate::Ty {
    match ty {
        Type::Integer => crate::Ty::Inteiro,
        Type::Real => crate::Ty::Real,
        Type::Boolean => crate::Ty::Logico,
        Type::String => crate::Ty::Cadeia,
        Type::Character => crate::Ty::Caracter,
    }
}
