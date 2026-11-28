use ferro_hir::{AssignOp, Expr, ExprKind, LoopControl, Program, Stmt, StmtKind, Type};

use crate::{SourceMapSegment, default_value, rust_ident};

use super::expr::{push_expr, record_statement_mapping};
use super::types::{binding_info, map_type, rust_enum_variant_path, rust_type};

pub(super) fn render_statements(
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
                    rust_type(declaration.ty, program)
                ));
                let initializer = initializer.as_ref().map(|value| {
                    output.push_str(" = ");
                    push_expr(output, mappings, value, program, true, pors);
                });
                if initializer.is_none() {
                    if declaration.ty.is_vector() {
                        output.push_str(" = Vec::new();");
                    } else {
                        output
                            .push_str(&format!(" = {};", default_value(map_type(declaration.ty))));
                    }
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
                        output.push_str(&format!("{indent}{name} = __ferro_concat(&{name}, "));
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
            StmtKind::FieldAssign {
                binding,
                fields,
                op,
                value,
            } => {
                let declaration = binding_info(program, *binding);
                let mut place = rust_ident(&declaration.source_name);
                let mut current = declaration.ty;
                let mut field_ty = Type::Unit;
                for field_index in fields {
                    let field = match current {
                        Type::Record(type_id) => program
                            .record_types
                            .iter()
                            .find(|definition| definition.id == type_id)
                            .and_then(|definition| definition.fields.get(*field_index)),
                        _ => None,
                    };
                    place.push('.');
                    place.push_str(
                        &field
                            .map(|field| rust_ident(&field.name))
                            .unwrap_or_else(|| "__invalid_field".to_owned()),
                    );
                    field_ty = field.map(|field| field.ty).unwrap_or(Type::Unit);
                    current = field_ty;
                }
                if field_ty == Type::Integer {
                    let method = match op {
                        AssignOp::Add => Some("wrapping_add"),
                        AssignOp::Subtract => Some("wrapping_sub"),
                        AssignOp::Multiply => Some("wrapping_mul"),
                        AssignOp::Divide => Some("wrapping_div"),
                        AssignOp::Remainder => Some("wrapping_rem"),
                        _ => None,
                    };
                    if let Some(method) = method {
                        output.push_str(&format!("{indent}{place} = ({place}).{method}("));
                        push_expr(output, mappings, value, program, false, pors);
                        output.push_str(");\n");
                        record_statement_mapping(mappings, start, output.len(), statement);
                        continue;
                    }
                }
                if field_ty == Type::String && *op == AssignOp::Add && pors {
                    output.push_str(&format!("{indent}{place} = __ferro_concat(&{place}, "));
                    push_expr(output, mappings, value, program, false, pors);
                    output.push_str(".as_str());\n");
                } else {
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
                    output.push_str(&format!("{indent}{place} {assignment_op} "));
                    push_expr(output, mappings, value, program, *op == AssignOp::Set, pors);
                    output.push_str(";\n");
                }
            }
            StmtKind::IndexAssign {
                binding,
                index,
                value,
            } => {
                let name = rust_ident(&binding_info(program, *binding).source_name);
                let checked_index = format!("__ferro_index_{}", statement.span.start);
                output.push_str(&format!("{indent}let {checked_index} = __ferro_index("));
                push_expr(output, mappings, index, program, false, pors);
                output.push_str(&format!(
                    ", {name}.len());\n{indent}{name}[{checked_index}] = "
                ));
                push_expr(output, mappings, value, program, true, pors);
                output.push_str(";\n");
            }
            StmtKind::VectorPush { binding, value } => {
                let name = rust_ident(&binding_info(program, *binding).source_name);
                output.push_str(&format!("{indent}{name}.push("));
                push_expr(output, mappings, value, program, true, pors);
                output.push_str(");\n");
            }
            StmtKind::VectorPop { binding } => {
                let name = rust_ident(&binding_info(program, *binding).source_name);
                output.push_str(&format!("{indent}let _ = {name}.pop();\n"));
            }
            StmtKind::Write(values) => {
                let temps: Vec<String> = values
                    .iter()
                    .enumerate()
                    .map(|(index, _)| format!("__ferro_write_{index}"))
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
                        Type::Real => format!("__ferro_java_double({temp})"),
                        _ => temp.clone(),
                    };
                    output.push_str(&format!("{indent}print!(\"{{}}\", {expression});\n"));
                }
                output.push_str(&format!("{indent}let _ = io::stdout().flush();\n"));
            }
            StmtKind::Read { binding, .. } => {
                let declaration = binding_info(program, *binding);
                let name = rust_ident(&declaration.source_name);
                let input = format!("__ferro_input_{}", statement.span.start);
                let label = match declaration.ty {
                    Type::Integer => "inteiro",
                    Type::Real => "real",
                    Type::Boolean => "lógico",
                    Type::String => "cadeia",
                    Type::Character => "caracter",
                    Type::VectorInteger
                    | Type::VectorReal
                    | Type::VectorBoolean
                    | Type::VectorString
                    | Type::VectorCharacter => "vetor",
                    Type::VectorRecord(_) | Type::VectorEnum(_) => "vetor",
                    Type::Unit => unreachable!("variável não pode ter tipo unit"),
                    Type::Record(_) => unreachable!("leitura de registro ainda não suportada"),
                    Type::Enum(_) => unreachable!("leitura de enum ainda não suportada"),
                };
                output.push_str(&format!(
                    "{indent}let {input} = __ferro_read_line(\"{label}\");\n"
                ));
                match declaration.ty {
                    Type::String if pors => output.push_str(&format!("{indent}{name} = __ferro_string(&{input});\n")),
                    Type::Boolean => output.push_str(&format!("{indent}{name} = match {input}.trim() {{ \"verdadeiro\" | \"true\" => true, \"falso\" | \"false\" => false, _ => {{ eprintln!(\"Entrada inválida: esperado lógico.\"); std::process::exit(1); }} }};\n")),
                    _ => output.push_str(&format!("{indent}{name} = __ferro_parse::<{}>(&{input}, \"{label}\");\n", rust_type(declaration.ty, program))),
                }
            }
            StmtKind::Call {
                function,
                arguments,
            } => {
                output.push_str(&indent);
                push_expr(
                    output,
                    mappings,
                    &Expr {
                        kind: ExprKind::Call {
                            function: *function,
                            arguments: arguments.clone(),
                        },
                        ty: Type::Boolean,
                        span: statement.span,
                    },
                    program,
                    false,
                    pors,
                );
                output.push_str(";\n");
            }
            StmtKind::HostCall {
                module,
                name,
                arguments,
            } => {
                output.push_str(&indent);
                push_expr(
                    output,
                    mappings,
                    &Expr {
                        kind: ExprKind::HostCall {
                            module: module.clone(),
                            name: name.clone(),
                            arguments: arguments.clone(),
                        },
                        ty: Type::Unit,
                        span: statement.span,
                    },
                    program,
                    false,
                    pors,
                );
                output.push_str(";\n");
            }
            StmtKind::Return { value } => {
                output.push_str(&format!("{indent}return"));
                if let Some(value) = value {
                    output.push(' ');
                    push_expr(
                        output,
                        mappings,
                        value,
                        program,
                        matches!(value.ty, Type::Record(_) | Type::Enum(_)),
                        pors,
                    );
                }
                output.push_str(";\n");
            }
            StmtKind::Choose { value, arms } => {
                let Type::Enum(type_id) = value.ty else {
                    unreachable!("escolha é validada pelo frontend")
                };
                let definition = program
                    .enum_types
                    .iter()
                    .find(|definition| definition.id == type_id)
                    .expect("enum existe");
                output.push_str(&format!("{indent}match "));
                push_expr(output, mappings, value, program, true, pors);
                output.push_str(" {\n");
                for arm in arms {
                    if let Some(variant_index) = arm.variant_index {
                        let variant_path = rust_enum_variant_path(definition, variant_index);
                        output.push_str(&format!("{indent}    {variant_path}"));
                        if !arm.payload_bindings.is_empty() {
                            let names = arm
                                .payload_bindings
                                .iter()
                                .map(|id| rust_ident(&binding_info(program, *id).source_name))
                                .collect::<Vec<_>>()
                                .join(", ");
                            output.push_str(&format!("({names})"));
                        }
                    } else {
                        output.push_str(&format!("{indent}    _"));
                    }
                    output.push_str(" => {\n");
                    render_statements(&arm.body, program, depth + 2, output, mappings, pors, None);
                    output.push_str(&format!("{indent}    }},\n"));
                }
                output.push_str(&format!("{indent}}}\n"));
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
            StmtKind::DoWhile { body, condition } => {
                let flag = format!("__ferro_do_{}", statement.span.start);
                output.push_str(&format!("{indent}let mut {flag} = true;\n"));
                output.push_str(&format!("{indent}while {flag} || ("));
                push_expr(output, mappings, condition, program, false, pors);
                output.push_str(") {\n");
                output.push_str(&format!("{indent}    {flag} = false;\n"));
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
                output.push_str(&format!("{loop_indent}'ferro_for_{label}: while "));
                push_expr(output, mappings, condition, program, false, pors);
                output.push_str(" {\n");
                let body_scope_indent = "    ".repeat(depth + 2);
                output.push_str(&format!("{body_scope_indent}'ferro_body_{label}: {{\n"));
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
                        output.push_str(&format!("{indent}break 'ferro_body_{label};\n"));
                    } else {
                        output.push_str(&format!("{indent}continue;\n"));
                    }
                } else {
                    if let Some(label) = for_label {
                        output.push_str(&format!("{indent}break 'ferro_for_{label};\n"));
                    } else {
                        output.push_str(&format!("{indent}break;\n"));
                    }
                }
            }
        }
        record_statement_mapping(mappings, start, output.len(), statement);
    }
}
