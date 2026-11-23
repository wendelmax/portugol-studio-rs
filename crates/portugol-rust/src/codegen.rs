use portugol_hir::{
    AssignOp, BinaryOp, BindingId, BuiltinEnum, EnumDef, Expr, ExprKind, LoopControl, Program,
    Span, Stmt, StmtKind, Type, UnaryOp,
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
    let has_read = statements_have_read(&program.body)
        || program
            .functions
            .iter()
            .any(|f| statements_have_read(&f.body));
    let uses_ui = pors && program_uses_ui_host_calls(program);
    let mvu_update_arity = if pors && program.body.is_empty() {
        mvu_entry_update_arity(program)
    } else {
        None
    };
    let mut output = String::from("use std::io::{self, Write};\n\n");
    output.push_str(JAVA_DOUBLE_HELPER);
    if has_read {
        output.push_str("\nuse std::str::FromStr;\nfn __portugol_read_line(expected: &str) -> String {\n    if std::env::var_os(\"PORTUGOL_STUDIO_INPUT_PROTOCOL\").is_some() { print!(\"\\u{1e}PORTUGOL_INPUT:{expected}\\u{1e}\\n\"); let _ = io::stdout().flush(); }\n    let mut line = String::new();\n    if io::stdin().read_line(&mut line).is_err() { eprintln!(\"Não foi possível ler a entrada.\"); std::process::exit(1); }\n    line.trim_end_matches(['\\r', '\\n']).to_owned()\n}\nfn __portugol_parse<T: FromStr>(input: &str, expected: &str) -> T {\n    input.trim().parse().unwrap_or_else(|_| { eprintln!(\"Entrada inválida: esperado {expected}.\"); std::process::exit(1); })\n}\n");
    }
    if pors {
        output.push_str("\nthread_local! { static __PORS_STRING_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }\nfn __pors_account(len: usize) { __PORS_STRING_BYTES.with(|used| { let next = used.get().saturating_add(len); if next > 16 * 1024 * 1024 { std::process::exit(78); } used.set(next); }); }\nfn __pors_string(value: &str) -> String { __pors_account(value.len()); value.to_owned() }\nfn __pors_vec_string_clone(value: &[String]) -> Vec<String> { value.iter().map(|item| __pors_string(item)).collect() }\nfn __pors_index(index: i32, len: usize) -> usize { if index < 0 || index as usize >= len { eprintln!(\"Erro de execução: índice de vetor fora do intervalo ({index}).\"); std::process::exit(1); } index as usize }\nfn __pors_char(value: char) -> String { __pors_account(value.len_utf8()); value.to_string() }\nfn __pors_concat(left: &str, right: &str) -> String { let len = left.len().saturating_add(right.len()); __pors_account(len); let mut value = String::with_capacity(len); value.push_str(left); value.push_str(right); value }\n");
    }
    if uses_ui || mvu_update_arity.is_some() {
        output.push_str(PORS_UI_RUNTIME);
    }
    for definition in &program.record_types {
        output.push_str(&format!(
            "\n#[derive(Clone)]\nstruct {} {{\n",
            rust_ident(&definition.name)
        ));
        for field in &definition.fields {
            output.push_str(&format!(
                "    {}: {},\n",
                rust_ident(&field.name),
                rust_type(field.ty, program)
            ));
        }
        output.push_str("}\n");
        if pors {
            output.push_str(&format!(
                "fn __pors_clone_{}(value: &{}) -> {} {{ {} {{\n",
                definition.id.0,
                rust_ident(&definition.name),
                rust_ident(&definition.name),
                rust_ident(&definition.name)
            ));
            for field in &definition.fields {
                let name = rust_ident(&field.name);
                let clone = match field.ty {
                    Type::String => format!("__pors_string(&value.{name})"),
                    Type::VectorString => format!("__pors_vec_string_clone(&value.{name})"),
                    Type::Record(id) => program
                        .record_types
                        .iter()
                        .find(|item| item.id == id)
                        .map(|item| format!("__pors_clone_{}(&value.{name})", item.id.0))
                        .unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::VectorRecord(id) => program
                        .record_types
                        .iter()
                        .find(|item| item.id == id)
                        .map(|item| {
                            format!(
                                "value.{name}.iter().map(|item| __pors_clone_{}(item)).collect()",
                                item.id.0
                            )
                        })
                        .unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::Enum(id) => program.enum_types.iter().find(|item| item.id == id).map(|item| format!("__pors_clone_enum_{}(&value.{name})", item.id.0)).unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::VectorEnum(id) => program.enum_types.iter().find(|item| item.id == id).map(|item| format!("value.{name}.iter().map(|item| __pors_clone_enum_{}(item)).collect()", item.id.0)).unwrap_or_else(|| format!("value.{name}.clone()")),
                    ty if ty.is_vector() => format!("value.{name}.clone()"),
                    _ => format!("value.{name}"),
                };
                output.push_str(&format!("    {name}: {clone},\n"));
            }
            output.push_str("} }\n");
        }
    }
    for definition in &program.enum_types {
        if definition.builtin.is_none() {
            output.push_str(&format!(
                "\n#[derive(Clone)]\nenum {} {{\n",
                rust_ident(&definition.name)
            ));
            for variant in &definition.variants {
                output.push_str(&format!("    {}", rust_ident(&variant.name)));
                if !variant.payload_types.is_empty() {
                    let fields = variant
                        .payload_types
                        .iter()
                        .map(|ty| rust_type(*ty, program))
                        .collect::<Vec<_>>()
                        .join(", ");
                    output.push_str(&format!("({fields})"));
                }
                output.push_str(",\n");
            }
            output.push_str("}\n");
        }
        if pors {
            let enum_ty = rust_enum_type_name(definition, program);
            output.push_str(&format!(
                "fn __pors_clone_enum_{}(value: &{enum_ty}) -> {enum_ty} {{ match value {{\n",
                definition.id.0
            ));
            for (variant_index, variant) in definition.variants.iter().enumerate() {
                let variant_path = rust_enum_variant_path(definition, variant_index);
                if variant.payload_types.is_empty() {
                    output.push_str(&format!("    {variant_path} => {variant_path},\n"));
                } else {
                    let bindings = (0..variant.payload_types.len())
                        .map(|index| format!("field_{index}"))
                        .collect::<Vec<_>>();
                    let pattern = bindings.join(", ");
                    let values = variant
                        .payload_types
                        .iter()
                        .zip(&bindings)
                        .map(|(ty, binding)| pors_clone_value(*ty, binding, program))
                        .collect::<Vec<_>>()
                        .join(", ");
                    output.push_str(&format!(
                        "    {variant_path}({pattern}) => {variant_path}({values}),\n"
                    ));
                }
            }
            output.push_str("} }\n");
        }
    }
    let mut mappings = Vec::new();
    for function in &program.functions {
        let function_program = Program {
            bindings: function.bindings.clone(),
            ..program.clone()
        };
        let params = function
            .parameters
            .iter()
            .map(|id| {
                let parameter = binding_info(&function_program, *id);
                format!(
                    "{}{}: {}",
                    if parameter.mutability == portugol_hir::Mutability::Mutable {
                        "mut "
                    } else {
                        ""
                    },
                    rust_ident(&parameter.source_name),
                    rust_type(parameter.ty, &function_program)
                )
            })
            .collect::<Vec<_>>()
            .join(", ");
        output.push_str(&format!(
            "\nfn {}({params}){} {{\n",
            rust_ident(&function.source_name),
            function
                .return_type
                .map(|ty| format!(" -> {}", rust_type(ty, &function_program)))
                .unwrap_or_default()
        ));
        render_statements(
            &function.body,
            &function_program,
            1,
            &mut output,
            &mut mappings,
            pors,
            None,
        );
        output.push_str("}\n");
    }
    output.push_str("\nfn main() {\n");
    let body_start = output.len();
    if let Some(update_arity) = mvu_update_arity {
        let init_fn = rust_ident("iniciar");
        let update_fn = rust_ident("atualizar");
        let view_fn = rust_ident("visualizar");
        let update_call = match update_arity {
            2 => format!("{update_fn}(estado, alvo)"),
            3 => format!("{update_fn}(estado, tipo, alvo)"),
            _ => format!("{update_fn}(estado, tipo, alvo, valor)"),
        };
        output.push_str(&format!(
            "    use std::io::BufRead;\n    let mut estado = {init_fn}();\n    let mut step: u64 = 0;\n    __pors_ui::begin_frame();\n    let root = {view_fn}(estado.clone());\n    println!(\"{{{{\\\"step\\\":{{step}},\\\"tree\\\":{{}}}}}}\", __pors_ui::finish_frame(root).to_json());\n    let _ = io::stdout().flush();\n    let stdin = io::stdin();\n    for line in stdin.lock().lines() {{\n        let Ok(line) = line else {{ break }};\n        let trimmed = line.trim();\n        if trimmed.is_empty() {{\n            continue;\n        }}\n        let Some((tipo, alvo, valor)) = __pors_ui::parse_event_line(trimmed) else {{\n            eprintln!(\"Evento inválido: {{trimmed}}\");\n            continue;\n        }};\n        let _ = (&tipo, &valor);\n        estado = {update_call};\n        step += 1;\n        __pors_ui::begin_frame();\n        let root = {view_fn}(estado.clone());\n        println!(\"{{{{\\\"step\\\":{{step}},\\\"tree\\\":{{}}}}}}\", __pors_ui::finish_frame(root).to_json());\n        let _ = io::stdout().flush();\n    }}\n"
        ));
    } else {
        render_statements(
            &program.body,
            program,
            1,
            &mut output,
            &mut mappings,
            pors,
            None,
        );
    }
    let body_end = output.len();
    output.push_str("}\n");
    let generated_prefix_end = mappings
        .iter()
        .map(|mapping| mapping.generated.start)
        .min()
        .unwrap_or(body_start as u32)
        .min(body_start as u32);
    if generated_prefix_end > 0 {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: 0,
                end: generated_prefix_end,
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
            StmtKind::FieldAssign {
                binding,
                field_index,
                op,
                value,
            } => {
                let declaration = binding_info(program, *binding);
                let name = rust_ident(&declaration.source_name);
                let field = match declaration.ty {
                    Type::Record(type_id) => program
                        .record_types
                        .iter()
                        .find(|definition| definition.id == type_id)
                        .and_then(|definition| definition.fields.get(*field_index)),
                    _ => None,
                };
                let field_name = field
                    .map(|field| rust_ident(&field.name))
                    .unwrap_or_else(|| "__invalid_field".to_owned());
                let field_ty = field.map(|field| field.ty).unwrap_or(Type::Unit);
                let place = format!("{name}.{field_name}");
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
                    output.push_str(&format!("{indent}{place} = __pors_concat(&{place}, "));
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
                let checked_index = format!("__pors_index_{}", statement.span.start);
                output.push_str(&format!("{indent}let {checked_index} = __pors_index("));
                push_expr(output, mappings, index, program, false, pors);
                output.push_str(&format!(
                    ", {name}.len());\n{indent}{name}[{checked_index}] = "
                ));
                push_expr(output, mappings, value, program, false, pors);
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
                    "{indent}let {input} = __portugol_read_line(\"{label}\");\n"
                ));
                match declaration.ty {
                    Type::String if pors => output.push_str(&format!("{indent}{name} = __pors_string(&{input});\n")),
                    Type::Boolean => output.push_str(&format!("{indent}{name} = match {input}.trim() {{ \"verdadeiro\" | \"true\" => true, \"falso\" | \"false\" => false, _ => {{ eprintln!(\"Entrada inválida: esperado lógico.\"); std::process::exit(1); }} }};\n")),
                    _ => output.push_str(&format!("{indent}{name} = __portugol_parse::<{}>(&{input}, \"{label}\");\n", rust_type(declaration.ty, program))),
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
                    let variant_path = rust_enum_variant_path(definition, arm.variant_index);
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
        ExprKind::VectorLiteral(items) => {
            let mut parts = vec![ExpressionPart::Text("vec![".to_owned())];
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    parts.push(ExpressionPart::Text(", ".to_owned()));
                }
                parts.push(ExpressionPart::Nested(render_expr(item, program, pors)));
            }
            parts.push(ExpressionPart::Text("]".to_owned()));
            parts
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
        StmtKind::Choose { arms, .. } => arms.iter().any(|arm| statements_have_read(&arm.body)),
        _ => false,
    })
}

fn pors_clone_value(ty: Type, value: &str, program: &Program) -> String {
    match ty {
        Type::String => format!("__pors_string({value})"),
        Type::VectorString => format!("__pors_vec_string_clone({value})"),
        Type::Record(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| format!("__pors_clone_{}({value})", item.id.0))
            .unwrap_or_else(|| format!("(*{value}).clone()")),
        Type::VectorRecord(id) => program
            .record_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| {
                format!(
                    "{value}.iter().map(|item| __pors_clone_{}(item)).collect()",
                    item.id.0
                )
            })
            .unwrap_or_else(|| format!("{value}.clone()")),
        Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| format!("__pors_clone_enum_{}({value})", item.id.0))
            .unwrap_or_else(|| format!("(*{value}).clone()")),
        Type::VectorEnum(id) => program
            .enum_types
            .iter()
            .find(|item| item.id == id)
            .map(|item| {
                format!(
                    "{value}.iter().map(|item| __pors_clone_enum_{}(item)).collect()",
                    item.id.0
                )
            })
            .unwrap_or_else(|| format!("{value}.clone()")),
        vector if vector.is_vector() => format!("{value}.clone()"),
        Type::Unit => "()".to_owned(),
        _ => format!("*{value}"),
    }
}

fn rust_type(ty: Type, program: &Program) -> String {
    match ty {
        Type::Integer => "i32".into(),
        Type::Real => "f64".into(),
        Type::Boolean => "bool".into(),
        Type::String => "String".into(),
        Type::Character => "char".into(),
        Type::Unit => "()".into(),
        Type::VectorInteger => "Vec<i32>".into(),
        Type::VectorReal => "Vec<f64>".into(),
        Type::VectorBoolean => "Vec<bool>".into(),
        Type::VectorString => "Vec<String>".into(),
        Type::VectorCharacter => "Vec<char>".into(),
        Type::Record(id) => program
            .record_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| rust_ident(&definition.name))
            .unwrap_or_else(|| "__invalid_record".into()),
        Type::VectorRecord(id) => program
            .record_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| format!("Vec<{}>", rust_ident(&definition.name)))
            .unwrap_or_else(|| "Vec<()>".into()),
        Type::Enum(id) => program
            .enum_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| rust_enum_type_name(definition, program))
            .unwrap_or_else(|| "__invalid_enum".into()),
        Type::VectorEnum(id) => program
            .enum_types
            .iter()
            .find(|definition| definition.id == id)
            .map(|definition| format!("Vec<{}>", rust_enum_type_name(definition, program)))
            .unwrap_or_else(|| "Vec<()>".into()),
    }
}

fn rust_enum_type_name(definition: &EnumDef, program: &Program) -> String {
    match definition.builtin {
        Some(BuiltinEnum::Option(inner)) => format!("Option<{}>", rust_type(inner, program)),
        Some(BuiltinEnum::Result { ok, err }) => {
            format!(
                "Result<{}, {}>",
                rust_type(ok, program),
                rust_type(err, program)
            )
        }
        None => rust_ident(&definition.name),
    }
}

fn rust_enum_variant_path(definition: &EnumDef, variant_index: usize) -> String {
    match definition.builtin {
        Some(BuiltinEnum::Option(_)) => {
            if variant_index == 0 {
                "Option::Some".to_owned()
            } else {
                "Option::None".to_owned()
            }
        }
        Some(BuiltinEnum::Result { .. }) => {
            if variant_index == 0 {
                "Result::Ok".to_owned()
            } else {
                "Result::Err".to_owned()
            }
        }
        None => {
            let variant_name = definition
                .variants
                .get(variant_index)
                .map(|variant| rust_ident(&variant.name))
                .unwrap_or_else(|| "__invalid_variant".to_owned());
            format!("{}::{variant_name}", rust_ident(&definition.name))
        }
    }
}

fn map_type(ty: Type) -> crate::Ty {
    match ty {
        Type::Integer => crate::Ty::Inteiro,
        Type::Real => crate::Ty::Real,
        Type::Boolean => crate::Ty::Logico,
        Type::String => crate::Ty::Cadeia,
        Type::Character => crate::Ty::Caracter,
        Type::Unit => unreachable!("tipo unit não possui valor Portugol"),
        Type::VectorInteger
        | Type::VectorReal
        | Type::VectorBoolean
        | Type::VectorString
        | Type::VectorCharacter
        | Type::VectorRecord(_)
        | Type::VectorEnum(_) => unreachable!("vetor não possui tipo escalar"),
        Type::Record(_) | Type::Enum(_) => unreachable!("tipo nominal não possui tipo escalar"),
    }
}

fn render_ui_host_call(
    name: &str,
    arguments: &[Expr],
    program: &Program,
    pors: bool,
) -> Vec<ExpressionPart> {
    match name {
        "texto" => {
            let mut parts = vec![ExpressionPart::Text("__pors_ui::texto(&[".to_owned())];
            for (index, argument) in arguments.iter().enumerate() {
                if index > 0 {
                    parts.push(ExpressionPart::Text(", ".to_owned()));
                }
                let rendered = render_expr(argument, program, pors);
                match argument.ty {
                    Type::String => {
                        parts.push(ExpressionPart::Text("(&(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(")).to_string()".to_owned()));
                    }
                    Type::Real => {
                        parts.push(ExpressionPart::Text("__portugol_java_double(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(")".to_owned()));
                    }
                    Type::Boolean => {
                        parts.push(ExpressionPart::Text("(if ".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(
                            " { \"verdadeiro\" } else { \"falso\" }).to_owned()".to_owned(),
                        ));
                    }
                    _ => {
                        parts.push(ExpressionPart::Text("(".to_owned()));
                        parts.push(ExpressionPart::Nested(rendered));
                        parts.push(ExpressionPart::Text(").to_string()".to_owned()));
                    }
                }
            }
            parts.push(ExpressionPart::Text("])".to_owned()));
            parts
        }
        "botao" => vec![
            ExpressionPart::Text("__pors_ui::botao(&(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[1], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        "entrada" => vec![
            ExpressionPart::Text("__pors_ui::entrada(&(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[1], program, pors)),
            ExpressionPart::Text("), &(".to_owned()),
            ExpressionPart::Nested(render_expr(&arguments[2], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        "coluna" | "linha" | "lista" => vec![
            ExpressionPart::Text(format!("__pors_ui::{name}(&(")),
            ExpressionPart::Nested(render_expr(&arguments[0], program, pors)),
            ExpressionPart::Text("))".to_owned()),
        ],
        _ => vec![ExpressionPart::Text("0_i32".to_owned())],
    }
}

fn mvu_entry_update_arity(program: &Program) -> Option<usize> {
    let has_init = program
        .functions
        .iter()
        .any(|f| f.source_name == "iniciar" && f.parameters.is_empty());
    let has_view = program
        .functions
        .iter()
        .any(|f| f.source_name == "visualizar" && f.parameters.len() == 1);
    let update = program
        .functions
        .iter()
        .find(|f| f.source_name == "atualizar" && (2..=4).contains(&f.parameters.len()))?;
    (has_init && has_view).then_some(update.parameters.len())
}

fn program_uses_ui_host_calls(program: &Program) -> bool {
    stmts_use_ui(&program.body)
        || program
            .functions
            .iter()
            .any(|function| stmts_use_ui(&function.body))
}

fn stmts_use_ui(stmts: &[Stmt]) -> bool {
    stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Declare { initializer, .. } => initializer.as_ref().is_some_and(expr_uses_ui),
        StmtKind::Assign { value, .. } | StmtKind::FieldAssign { value, .. } => expr_uses_ui(value),
        StmtKind::Choose { value, arms } => {
            expr_uses_ui(value) || arms.iter().any(|arm| stmts_use_ui(&arm.body))
        }
        StmtKind::Read { .. } | StmtKind::LoopControl { .. } => false,
        StmtKind::Call { arguments, .. } | StmtKind::Write(arguments) => {
            arguments.iter().any(expr_uses_ui)
        }
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_ui(condition)
                || stmts_use_ui(then_branch)
                || else_branch.as_deref().is_some_and(stmts_use_ui)
        }
        StmtKind::While { condition, body } => expr_uses_ui(condition) || stmts_use_ui(body),
        StmtKind::For {
            initializer,
            condition,
            step,
            body,
        } => {
            stmts_use_ui(std::slice::from_ref(initializer.as_ref()))
                || expr_uses_ui(condition)
                || stmts_use_ui(std::slice::from_ref(step.as_ref()))
                || stmts_use_ui(body)
        }
        StmtKind::Return { value } => value.as_ref().is_some_and(expr_uses_ui),
        StmtKind::IndexAssign { index, value, .. } => expr_uses_ui(index) || expr_uses_ui(value),
    })
}

fn expr_uses_ui(expr: &Expr) -> bool {
    match &expr.kind {
        ExprKind::HostCall { module, .. } => module == "UI",
        ExprKind::Call { arguments, .. }
        | ExprKind::VectorLiteral(arguments)
        | ExprKind::RecordLiteral {
            fields: arguments, ..
        }
        | ExprKind::EnumConstructor {
            payload: arguments, ..
        } => arguments.iter().any(expr_uses_ui),
        ExprKind::Field { value, .. }
        | ExprKind::Index { index: value, .. }
        | ExprKind::Parenthesized(value)
        | ExprKind::Try { operand: value }
        | ExprKind::Unary { operand: value, .. } => expr_uses_ui(value),
        ExprKind::Binary { left, right, .. } => expr_uses_ui(left) || expr_uses_ui(right),
        ExprKind::Binding(_)
        | ExprKind::Length { .. }
        | ExprKind::Integer(_)
        | ExprKind::Real(_)
        | ExprKind::Boolean(_)
        | ExprKind::String(_)
        | ExprKind::Character(_) => false,
    }
}

const PORS_UI_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_ui {
    use std::cell::RefCell;
    use std::fmt::Write as _;

    const MAX_NODES: usize = 1_024;
    const MAX_DEPTH: usize = 32;

    #[derive(Clone)]
    pub enum UiNode {
        Column(Vec<UiNode>),
        Row(Vec<UiNode>),
        Text(String),
        Button { id: String, label: String },
        Input { id: String, value: String, placeholder: String },
        List(Vec<UiNode>),
    }

    impl UiNode {
        pub fn to_json(&self) -> String {
            let mut out = String::new();
            self.write_json(&mut out);
            out
        }

        fn children(&self) -> &[UiNode] {
            match self {
                Self::Column(items) | Self::Row(items) | Self::List(items) => items,
                Self::Text(_) | Self::Button { .. } | Self::Input { .. } => &[],
            }
        }

        fn validate(&self, depth: usize, count: &mut usize) {
            if depth > MAX_DEPTH {
                eprintln!("Erro de UI: profundidade máxima de {MAX_DEPTH} níveis excedida.");
                std::process::exit(1);
            }
            *count += 1;
            if *count > MAX_NODES {
                eprintln!("Erro de UI: limite máximo de {MAX_NODES} nós excedido.");
                std::process::exit(1);
            }
            for child in self.children() {
                child.validate(depth + 1, count);
            }
        }

        fn write_json(&self, out: &mut String) {
            match self {
                Self::Column(children) => {
                    out.push_str("{\"kind\":\"coluna\",\"children\":");
                    write_nodes(children, out);
                    out.push('}');
                }
                Self::Row(children) => {
                    out.push_str("{\"kind\":\"linha\",\"children\":");
                    write_nodes(children, out);
                    out.push('}');
                }
                Self::Text(content) => {
                    out.push_str("{\"kind\":\"texto\",\"content\":");
                    write_json_str(content, out);
                    out.push('}');
                }
                Self::Button { id, label } => {
                    out.push_str("{\"kind\":\"botao\",\"id\":");
                    write_json_str(id, out);
                    out.push_str(",\"label\":");
                    write_json_str(label, out);
                    out.push('}');
                }
                Self::Input { id, value, placeholder } => {
                    out.push_str("{\"kind\":\"entrada\",\"id\":");
                    write_json_str(id, out);
                    out.push_str(",\"value\":");
                    write_json_str(value, out);
                    out.push_str(",\"placeholder\":");
                    write_json_str(placeholder, out);
                    out.push('}');
                }
                Self::List(items) => {
                    out.push_str("{\"kind\":\"lista\",\"items\":");
                    write_nodes(items, out);
                    out.push('}');
                }
            }
        }
    }

    fn write_nodes(nodes: &[UiNode], out: &mut String) {
        out.push('[');
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 { out.push(','); }
            node.write_json(out);
        }
        out.push(']');
    }

    fn write_json_str(value: &str, out: &mut String) {
        out.push('"');
        for ch in value.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\x08' => out.push_str("\\b"),
                '\x0c' => out.push_str("\\f"),
                c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); }
                other => out.push(other),
            }
        }
        out.push('"');
    }

    thread_local! {
        static NODES: RefCell<Vec<UiNode>> = const { RefCell::new(Vec::new()) };
    }

    pub fn begin_frame() {
        NODES.with(|nodes| nodes.borrow_mut().clear());
    }

    fn push(node: UiNode) -> i32 {
        NODES.with(|nodes| {
            let mut list = nodes.borrow_mut();
            if list.len() >= MAX_NODES {
                eprintln!("Erro de UI: limite máximo de {MAX_NODES} nós excedido.");
                std::process::exit(1);
            }
            let handle = list.len() as i32;
            list.push(node);
            handle
        })
    }

    fn resolve(handles: &[i32]) -> Vec<UiNode> {
        NODES.with(|nodes| {
            let list = nodes.borrow();
            let mut out = Vec::with_capacity(handles.len());
            for &handle in handles {
                let Some(node) = usize::try_from(handle).ok().and_then(|i| list.get(i)) else {
                    eprintln!("Erro de UI: identificador de nó inválido ({handle}).");
                    std::process::exit(1);
                };
                out.push(node.clone());
            }
            out
        })
    }

    pub fn finish_frame(root: i32) -> UiNode {
        let node = NODES.with(|nodes| {
            let list = nodes.borrow();
            usize::try_from(root)
                .ok()
                .and_then(|i| list.get(i))
                .cloned()
                .unwrap_or_else(|| {
                    eprintln!("Erro de UI: nó raiz inválido ({root}).");
                    std::process::exit(1);
                })
        });
        let mut count = 0usize;
        node.validate(1, &mut count);
        node
    }

    pub fn texto(parts: &[String]) -> i32 {
        let mut content = String::new();
        for part in parts {
            content.push_str(part);
        }
        push(UiNode::Text(content))
    }

    pub fn botao(id: &str, label: &str) -> i32 {
        push(UiNode::Button { id: id.to_owned(), label: label.to_owned() })
    }

    pub fn entrada(id: &str, value: &str, placeholder: &str) -> i32 {
        push(UiNode::Input {
            id: id.to_owned(),
            value: value.to_owned(),
            placeholder: placeholder.to_owned(),
        })
    }

    pub fn coluna(children: &[i32]) -> i32 {
        push(UiNode::Column(resolve(children)))
    }

    pub fn linha(children: &[i32]) -> i32 {
        push(UiNode::Row(resolve(children)))
    }

    pub fn lista(items: &[i32]) -> i32 {
        push(UiNode::List(resolve(items)))
    }

    pub fn parse_event_line(line: &str) -> Option<(String, String, String)> {
        if line.starts_with('{') {
            let kind = json_string_field(line, "kind")?;
            let id = json_string_field(line, "id")?;
            let value = json_string_field(line, "value").unwrap_or_default();
            return Some((kind, id, value));
        }
        if let Some(id) = line.strip_prefix("clique:") {
            return Some(("clique".to_owned(), id.to_owned(), String::new()));
        }
        if let Some(rest) = line.strip_prefix("entrada_alterada:")
            && let Some((id, value)) = rest.split_once(':')
        {
            return Some(("entrada_alterada".to_owned(), id.to_owned(), value.to_owned()));
        }
        None
    }

    fn json_string_field(raw: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let after_key = raw.split_once(&needle)?.1;
        let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();
        let mut chars = after_colon.strip_prefix('"')?.chars();
        let mut out = String::new();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => return Some(out),
                '\\' => match chars.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'b' => out.push('\x08'),
                    'f' => out.push('\x0c'),
                    other => out.push(other),
                },
                other => out.push(other),
            }
        }
        None
    }
}
"#;
