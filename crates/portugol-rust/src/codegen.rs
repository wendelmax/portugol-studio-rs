use portugol_hir::{
    AssignOp, BinaryOp, BindingId, Expr, ExprKind, LoopControl, Program, Span, Stmt, StmtKind,
    Type, UnaryOp,
};

use crate::{
    GeneratedRust, JAVA_DOUBLE_HELPER, SourceMapQuality, SourceMapSegment, SourceRange,
    default_value, format_real, rust_ident,
};

pub(crate) fn render_with_map(program: &Program) -> GeneratedRust {
    render(program, false)
}

pub(crate) fn render_pors_with_map(program: &Program) -> GeneratedRust {
    render(program, true)
}

fn render(program: &Program, pors: bool) -> GeneratedRust {
    let has_read = statements_have_read(&program.body);
    let mut output = String::from("use std::io::{self, Write};\n\n");
    output.push_str(JAVA_DOUBLE_HELPER);
    if has_read {
        output.push_str("\nuse std::str::FromStr;\nfn __portugol_read_line(expected: &str) -> String {\n    if std::env::var_os(\"PORTUGOL_STUDIO_INPUT_PROTOCOL\").is_some() { print!(\"\\u{1e}PORTUGOL_INPUT:{expected}\\u{1e}\\n\"); let _ = io::stdout().flush(); }\n    let mut line = String::new();\n    if io::stdin().read_line(&mut line).is_err() { eprintln!(\"Não foi possível ler a entrada.\"); std::process::exit(1); }\n    line.trim_end_matches(['\\r', '\\n']).to_owned()\n}\nfn __portugol_parse<T: FromStr>(input: &str, expected: &str) -> T {\n    input.trim().parse().unwrap_or_else(|_| { eprintln!(\"Entrada inválida: esperado {expected}.\"); std::process::exit(1); })\n}\n");
    }
    if pors {
        output.push_str("\nthread_local! { static __PORS_STRING_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }\nfn __pors_account(len: usize) { __PORS_STRING_BYTES.with(|used| { let next = used.get().saturating_add(len); if next > 16 * 1024 * 1024 { std::process::exit(78); } used.set(next); }); }\nfn __pors_string(value: &str) -> String { __pors_account(value.len()); value.to_owned() }\nfn __pors_char(value: char) -> String { __pors_account(value.len_utf8()); value.to_string() }\nfn __pors_concat(left: &str, right: &str) -> String { let len = left.len().saturating_add(right.len()); __pors_account(len); let mut value = String::with_capacity(len); value.push_str(left); value.push_str(right); value }\n");
    }
    output.push_str("\nfn main() {\n");
    let body_start = output.len();
    let mut mappings = Vec::new();
    render_statements(
        &program.body,
        program,
        1,
        &mut output,
        &mut mappings,
        pors,
        None,
    );
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
    pors: bool,
    for_label: Option<u32>,
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
                    push_expr(output, mappings, value, program, true, pors);
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
                        push_expr(output, mappings, value, program, false, pors);
                        output.push_str(");\n");
                        record_statement_mapping(mappings, start, output.len(), statement);
                        continue;
                    }
                }
                if ty == Type::String && *op == AssignOp::Add {
                    if pors {
                        output.push_str(&format!("{indent}{name} = __pors_concat(&{name}, "));
                        push_expr(output, mappings, value, program, false, pors);
                        output.push_str(".as_str());\n");
                    } else {
                        output
                            .push_str(&format!("{indent}{name} = format!(\"{{}}{{}}\", {name}, "));
                        push_expr(output, mappings, value, program, true, pors);
                        output.push_str(");\n");
                    }
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
                push_expr(output, mappings, value, program, *op == AssignOp::Set, pors);
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
                    push_expr(output, mappings, value, program, true, pors);
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
            StmtKind::Read { binding, .. } => {
                let declaration = binding_info(program, *binding);
                let name = rust_ident(&declaration.source_name);
                let input = format!("__portugol_input_{}", statement.span.start);
                let label = match declaration.ty {
                    Type::Integer => "inteiro",
                    Type::Real => "real",
                    Type::Boolean => "lógico",
                    Type::String => "cadeia",
                    Type::Character => "caracter",
                };
                output.push_str(&format!(
                    "{indent}let {input} = __portugol_read_line(\"{label}\");\n"
                ));
                match declaration.ty {
                    Type::String if pors => output.push_str(&format!("{indent}{name} = __pors_string(&{input});\n")),
                    Type::Boolean => output.push_str(&format!("{indent}{name} = match {input}.trim() {{ \"verdadeiro\" | \"true\" => true, \"falso\" | \"false\" => false, _ => {{ eprintln!(\"Entrada inválida: esperado lógico.\"); std::process::exit(1); }} }};\n")),
                    _ => output.push_str(&format!("{indent}{name} = __portugol_parse::<{}>(&{input}, \"{label}\");\n", rust_type(declaration.ty))),
                }
            }
            StmtKind::If {
                condition,
                then_branch,
                else_branch,
            } => {
                output.push_str(&format!("{indent}if "));
                push_expr(output, mappings, condition, program, false, pors);
                output.push_str(" {\n");
                render_statements(
                    then_branch,
                    program,
                    depth + 1,
                    output,
                    mappings,
                    pors,
                    for_label,
                );
                output.push_str(&format!("{indent}}}"));
                if let Some(else_branch) = else_branch {
                    output.push_str(" else {\n");
                    render_statements(
                        else_branch,
                        program,
                        depth + 1,
                        output,
                        mappings,
                        pors,
                        for_label,
                    );
                    output.push_str(&format!("{indent}}}"));
                }
                output.push('\n');
            }
            StmtKind::While { condition, body } => {
                output.push_str(&format!("{indent}while "));
                push_expr(output, mappings, condition, program, false, pors);
                output.push_str(" {\n");
                render_statements(body, program, depth + 1, output, mappings, pors, None);
                output.push_str(&format!("{indent}}}\n"));
            }
            StmtKind::For {
                initializer,
                condition,
                step,
                body,
            } => {
                let label = statement.span.start;
                output.push_str(&format!("{indent}{{\n"));
                render_statements(
                    std::slice::from_ref(initializer.as_ref()),
                    program,
                    depth + 1,
                    output,
                    mappings,
                    pors,
                    None,
                );
                let loop_indent = "    ".repeat(depth + 1);
                output.push_str(&format!("{loop_indent}'pors_for_{label}: while "));
                push_expr(output, mappings, condition, program, false, pors);
                output.push_str(" {\n");
                let body_scope_indent = "    ".repeat(depth + 2);
                output.push_str(&format!("{body_scope_indent}'pors_body_{label}: {{\n"));
                render_statements(
                    body,
                    program,
                    depth + 3,
                    output,
                    mappings,
                    pors,
                    Some(label),
                );
                output.push_str(&format!("{body_scope_indent}}}\n"));
                render_statements(
                    std::slice::from_ref(step.as_ref()),
                    program,
                    depth + 2,
                    output,
                    mappings,
                    pors,
                    None,
                );
                output.push_str(&format!("{loop_indent}}}\n{indent}}}\n"));
            }
            StmtKind::LoopControl { control } => {
                if *control == LoopControl::Continue {
                    if let Some(label) = for_label {
                        output.push_str(&format!("{indent}break 'pors_body_{label};\n"));
                    } else {
                        output.push_str(&format!("{indent}continue;\n"));
                    }
                } else {
                    if let Some(label) = for_label {
                        output.push_str(&format!("{indent}break 'pors_for_{label};\n"));
                    } else {
                        output.push_str(&format!("{indent}break;\n"));
                    }
                }
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

fn render_expr(expr: &Expr, program: &Program, pors: bool) -> MappedExpression {
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
        ExprKind::String(value) => vec![ExpressionPart::Text(if pors {
            format!("__pors_string({value:?})")
        } else {
            format!("String::from({value:?})")
        })],
        ExprKind::Character(value) => vec![ExpressionPart::Text(format!("{value:?}"))],
        ExprKind::Parenthesized(inner) => vec![
            ExpressionPart::Text("(".to_owned()),
            ExpressionPart::Nested(render_expr(inner, program, pors)),
            ExpressionPart::Text(")".to_owned()),
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

fn owned_value(expr: &Expr, program: &Program, pors: bool) -> MappedExpression {
    let value = render_expr(expr, program, pors);
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

fn push_expr(
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

fn statements_have_read(statements: &[Stmt]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        StmtKind::Read { .. } => true,
        StmtKind::If {
            then_branch,
            else_branch,
            ..
        } => {
            statements_have_read(then_branch)
                || else_branch.as_deref().is_some_and(statements_have_read)
        }
        StmtKind::While { body, .. } | StmtKind::For { body, .. } => statements_have_read(body),
        _ => false,
    })
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
