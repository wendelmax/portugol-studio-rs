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
    let uses_ui = pors && program_uses_host_module(program, "UI");
    let uses_matematica = pors && program_uses_host_module(program, "Matematica");
    let uses_texto = pors && program_uses_host_module(program, "Texto");
    let uses_tipos = pors && program_uses_host_module(program, "Tipos");
    let uses_util = pors && program_uses_host_module(program, "Util");
    let uses_calendario = pors && program_uses_host_module(program, "Calendario");
    let uses_arquivos = pors && program_uses_host_module(program, "Arquivos");
    let uses_json = pors && program_uses_host_module(program, "Json");
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
    if uses_matematica {
        output.push_str(PORS_MATEMATICA_RUNTIME);
    }
    if uses_texto {
        output.push_str(PORS_TEXTO_RUNTIME);
    }
    if uses_tipos {
        output.push_str(PORS_TIPOS_RUNTIME);
    }
    if uses_util || uses_calendario {
        output.push_str(PORS_CLOCK_RUNTIME);
    }
    if uses_util {
        output.push_str(PORS_UTIL_RUNTIME);
    }
    if uses_calendario {
        output.push_str(PORS_CALENDARIO_RUNTIME);
    }
    if uses_arquivos {
        output.push_str(PORS_ARQUIVOS_RUNTIME);
    }
    if uses_json {
        output.push_str(PORS_JSON_RUNTIME);
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

fn render_stdlib_host_call(
    rust_mod: &str,
    name: &str,
    arguments: &[Expr],
    program: &Program,
    pors: bool,
) -> Vec<ExpressionPart> {
    let mut parts = vec![ExpressionPart::Text(format!("{rust_mod}::{name}("))];
    for (index, argument) in arguments.iter().enumerate() {
        if index > 0 {
            parts.push(ExpressionPart::Text(", ".to_owned()));
        }
        let rendered = render_expr(argument, program, pors);
        if argument.ty == Type::String {
            parts.push(ExpressionPart::Text("&(".to_owned()));
            parts.push(ExpressionPart::Nested(rendered));
            parts.push(ExpressionPart::Text(")".to_owned()));
        } else {
            parts.push(ExpressionPart::Nested(rendered));
        }
    }
    parts.push(ExpressionPart::Text(")".to_owned()));
    parts
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

fn program_uses_host_module(program: &Program, target: &str) -> bool {
    stmts_use_host_module(&program.body, target)
        || program
            .functions
            .iter()
            .any(|function| stmts_use_host_module(&function.body, target))
}

fn stmts_use_host_module(stmts: &[Stmt], target: &str) -> bool {
    stmts.iter().any(|stmt| match &stmt.kind {
        StmtKind::Declare { initializer, .. } => initializer
            .as_ref()
            .is_some_and(|expr| expr_uses_host_module(expr, target)),
        StmtKind::Assign { value, .. } | StmtKind::FieldAssign { value, .. } => {
            expr_uses_host_module(value, target)
        }
        StmtKind::Choose { value, arms } => {
            expr_uses_host_module(value, target)
                || arms
                    .iter()
                    .any(|arm| stmts_use_host_module(&arm.body, target))
        }
        StmtKind::Read { .. } | StmtKind::LoopControl { .. } => false,
        StmtKind::Call { arguments, .. } | StmtKind::Write(arguments) => arguments
            .iter()
            .any(|expr| expr_uses_host_module(expr, target)),
        StmtKind::HostCall {
            module, arguments, ..
        } => {
            module == target
                || arguments
                    .iter()
                    .any(|expr| expr_uses_host_module(expr, target))
        }
        StmtKind::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_host_module(condition, target)
                || stmts_use_host_module(then_branch, target)
                || else_branch
                    .as_deref()
                    .is_some_and(|branch| stmts_use_host_module(branch, target))
        }
        StmtKind::While { condition, body } => {
            expr_uses_host_module(condition, target) || stmts_use_host_module(body, target)
        }
        StmtKind::For {
            initializer,
            condition,
            step,
            body,
        } => {
            stmts_use_host_module(std::slice::from_ref(initializer.as_ref()), target)
                || expr_uses_host_module(condition, target)
                || stmts_use_host_module(std::slice::from_ref(step.as_ref()), target)
                || stmts_use_host_module(body, target)
        }
        StmtKind::Return { value } => value
            .as_ref()
            .is_some_and(|expr| expr_uses_host_module(expr, target)),
        StmtKind::IndexAssign { index, value, .. } => {
            expr_uses_host_module(index, target) || expr_uses_host_module(value, target)
        }
        StmtKind::VectorPush { value, .. } => expr_uses_host_module(value, target),
        StmtKind::VectorPop { .. } => false,
    })
}

fn expr_uses_host_module(expr: &Expr, target: &str) -> bool {
    match &expr.kind {
        ExprKind::HostCall {
            module, arguments, ..
        } => {
            module == target
                || arguments
                    .iter()
                    .any(|arg| expr_uses_host_module(arg, target))
        }
        ExprKind::Call { arguments, .. }
        | ExprKind::VectorLiteral(arguments)
        | ExprKind::RecordLiteral {
            fields: arguments, ..
        }
        | ExprKind::EnumConstructor {
            payload: arguments, ..
        } => arguments
            .iter()
            .any(|arg| expr_uses_host_module(arg, target)),
        ExprKind::Field { value, .. }
        | ExprKind::Index { index: value, .. }
        | ExprKind::VectorGet { index: value, .. }
        | ExprKind::Parenthesized(value)
        | ExprKind::Try { operand: value }
        | ExprKind::Unary { operand: value, .. } => expr_uses_host_module(value, target),
        ExprKind::Binary { left, right, .. } => {
            expr_uses_host_module(left, target) || expr_uses_host_module(right, target)
        }
        ExprKind::Binding(_)
        | ExprKind::Length { .. }
        | ExprKind::VectorPop { .. }
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

const PORS_MATEMATICA_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_matematica {
    pub fn potencia(base: f64, expoente: f64) -> f64 { base.powf(expoente) }
    pub fn raiz(radicando: f64, indice: f64) -> f64 { radicando.powf(1.0 / indice) }
    pub fn raiz_quadrada(numero: f64) -> f64 { numero.sqrt() }
    pub fn arredondar(numero: f64, casas: i32) -> f64 {
        let factor = 10_f64.powi(casas.clamp(0, 15));
        (numero * factor).round() / factor
    }
    pub fn piso(numero: f64) -> f64 { numero.floor() }
    pub fn teto(numero: f64) -> f64 { numero.ceil() }
    pub fn logaritmo(numero: f64, base: f64) -> f64 { numero.ln() / base.ln() }
    pub fn seno(angulo: f64) -> f64 { angulo.sin() }
    pub fn cosseno(angulo: f64) -> f64 { angulo.cos() }
    pub fn tangente(angulo: f64) -> f64 { angulo.tan() }
    pub fn valor_absoluto(numero: f64) -> f64 { numero.abs() }
    pub fn valor_absoluto_inteiro(numero: i32) -> i32 { numero.wrapping_abs() }
    pub fn maior_numero(a: f64, b: f64) -> f64 { if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) } }
    pub fn menor_numero(a: f64, b: f64) -> f64 { if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) } }
    pub fn maior_inteiro(a: i32, b: i32) -> i32 { a.max(b) }
    pub fn menor_inteiro(a: i32, b: i32) -> i32 { a.min(b) }
}
"#;

const PORS_TEXTO_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_texto {
    pub fn numero_caracteres(texto: &str) -> i32 { texto.chars().count() as i32 }
    pub fn caixa_alta(texto: &str) -> String {
        let out = texto.to_uppercase();
        super::__pors_account(out.len());
        out
    }
    pub fn caixa_baixa(texto: &str) -> String {
        let out = texto.to_lowercase();
        super::__pors_account(out.len());
        out
    }
    pub fn aparar(texto: &str) -> String { super::__pors_string(texto.trim()) }
    pub fn contem(texto: &str, trecho: &str) -> bool { texto.contains(trecho) }
    pub fn comeca_com(texto: &str, prefixo: &str) -> bool { texto.starts_with(prefixo) }
    pub fn termina_com(texto: &str, sufixo: &str) -> bool { texto.ends_with(sufixo) }
    pub fn substituir(texto: &str, antigo: &str, novo: &str) -> String {
        let expected = if antigo.is_empty() {
            let slots = texto.chars().count().saturating_add(1);
            texto.len().saturating_add(slots.saturating_mul(novo.len()))
        } else {
            let count = texto.matches(antigo).count();
            let removed = count.saturating_mul(antigo.len());
            let added = count.saturating_mul(novo.len());
            texto.len().saturating_sub(removed).saturating_add(added)
        };
        super::__pors_account(expected);
        texto.replace(antigo, novo)
    }
    pub fn preencher_a_esquerda(car: char, tamanho: i32, texto: &str) -> String {
        let len = texto.chars().count();
        if tamanho <= 0 || len >= tamanho as usize {
            return super::__pors_string(texto);
        }
        let pad = tamanho as usize - len;
        let expected = pad.saturating_mul(car.len_utf8()).saturating_add(texto.len());
        super::__pors_account(expected);
        let mut out = String::with_capacity(expected);
        for _ in 0..pad {
            out.push(car);
        }
        out.push_str(texto);
        out
    }
    pub fn obter_caracter(texto: &str, indice: i32) -> Option<char> {
        if indice < 0 {
            return None;
        }
        texto.chars().nth(indice as usize)
    }
    pub fn posicao_texto(trecho: &str, texto: &str, inicio: i32) -> Option<i32> {
        let chars: Vec<char> = texto.chars().collect();
        let needle: Vec<char> = trecho.chars().collect();
        let start = inicio.max(0) as usize;
        if start > chars.len() {
            return None;
        }
        if needle.is_empty() {
            return Some(start as i32);
        }
        if start == chars.len() {
            return None;
        }
        chars[start..]
            .windows(needle.len())
            .position(|window| window == needle.as_slice())
            .map(|at| (start + at) as i32)
    }
    pub fn extrair_subtexto(texto: &str, inicio: i32, fim: i32) -> Result<String, String> {
        let chars: Vec<char> = texto.chars().collect();
        if inicio < 0 || fim < inicio || fim as usize > chars.len() {
            return Err(super::__pors_string(
                "Posição inicial ou final inválida para extrair subtexto",
            ));
        }
        let slice: String = chars[inicio as usize..fim as usize].iter().collect();
        super::__pors_account(slice.len());
        Ok(slice)
    }
    pub fn dividir(texto: &str, separador: &str) -> Vec<String> {
        if separador.is_empty() {
            super::__pors_account(texto.len());
            return texto.chars().map(|ch| ch.to_string()).collect();
        }
        let parts: Vec<&str> = texto.split(separador).collect();
        let total: usize = parts.iter().map(|part| part.len()).sum();
        super::__pors_account(total);
        parts.into_iter().map(str::to_owned).collect()
    }
}
"#;

const PORS_TIPOS_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_tipos {
    pub fn para_inteiro(texto: &str) -> Result<i32, String> {
        texto.trim().parse::<i32>().map_err(|_| {
            super::__pors_string(&format!("O valor '{texto}' não é um inteiro válido"))
        })
    }
    pub fn para_inteiro_base(texto: &str, base: i32) -> Result<i32, String> {
        if !matches!(base, 2 | 10 | 16) {
            return Err(super::__pors_string(&format!(
                "Base inválida ({base}); use 2, 10 ou 16"
            )));
        }
        let trimmed = texto.trim();
        let (neg, rest) = match trimmed.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, trimmed),
        };
        let digits = match base {
            2 => rest
                .strip_prefix("0b")
                .or_else(|| rest.strip_prefix("0B"))
                .unwrap_or(rest),
            16 => rest
                .strip_prefix("0x")
                .or_else(|| rest.strip_prefix("0X"))
                .unwrap_or(rest),
            _ => rest,
        };
        if digits.is_empty() || digits.starts_with('+') || digits.starts_with('-') {
            return Err(super::__pors_string(&format!(
                "O valor '{texto}' não é um inteiro válido"
            )));
        }
        let candidate = if neg {
            format!("-{digits}")
        } else {
            digits.to_owned()
        };
        i32::from_str_radix(&candidate, base as u32).map_err(|_| {
            super::__pors_string(&format!("O valor '{texto}' não é um inteiro válido"))
        })
    }
    pub fn para_real(texto: &str) -> Result<f64, String> {
        match texto.trim().parse::<f64>() {
            Ok(valor) if valor.is_finite() => Ok(valor),
            _ => Err(super::__pors_string(&format!(
                "O valor '{texto}' não é um número real válido"
            ))),
        }
    }
    pub fn para_logico(texto: &str) -> Result<bool, String> {
        match texto.trim() {
            "verdadeiro" => Ok(true),
            "falso" => Ok(false),
            _ => Err(super::__pors_string(&format!(
                "O valor '{texto}' não é um valor lógico válido"
            ))),
        }
    }
    pub fn para_caracter(texto: &str) -> Result<char, String> {
        let mut chars = texto.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Ok(ch),
            _ => Err(super::__pors_string(&format!(
                "O valor '{texto}' não é um caracter válido"
            ))),
        }
    }
    pub fn inteiro_para_real(valor: i32) -> f64 { valor as f64 }
    pub fn real_para_inteiro(valor: f64) -> i32 { valor as i32 }
    pub fn inteiro_para_cadeia(valor: i32) -> String { super::__pors_string(&valor.to_string()) }
    pub fn inteiro_para_cadeia_base(valor: i32, base: i32) -> Result<String, String> {
        let out = match base {
            2 => format!("{:b}", valor as u32),
            10 => valor.to_string(),
            16 => format!("{:X}", valor as u32),
            _ => {
                return Err(super::__pors_string(&format!(
                    "Base inválida ({base}); use 2, 10 ou 16"
                )));
            }
        };
        super::__pors_account(out.len());
        Ok(out)
    }
    pub fn real_para_cadeia(valor: f64) -> String {
        super::__pors_string(&super::__portugol_java_double(valor))
    }
    pub fn logico_para_cadeia(valor: bool) -> String {
        super::__pors_string(if valor { "verdadeiro" } else { "falso" })
    }
    pub fn caracter_para_cadeia(valor: char) -> String { super::__pors_string(&valor.to_string()) }
    pub fn logico_para_inteiro(valor: bool) -> i32 { i32::from(valor) }
    pub fn caracter_para_inteiro(valor: char) -> Result<i32, String> {
        match valor.to_digit(10) {
            Some(digit) => Ok(digit as i32),
            None => Err(super::__pors_string(&format!(
                "O caracter '{valor}' não é um dígito válido"
            ))),
        }
    }
    pub fn inteiro_para_caracter(valor: i32) -> Result<char, String> {
        if (0..=9).contains(&valor) {
            Ok(char::from(b'0' + valor as u8))
        } else {
            Err(super::__pors_string(&format!(
                "O inteiro '{valor}' não corresponde a um dígito (0 a 9)"
            )))
        }
    }
    pub fn cadeia_e_inteiro(texto: &str) -> bool { texto.trim().parse::<i32>().is_ok() }
    pub fn cadeia_e_real(texto: &str) -> bool {
        texto.trim().parse::<f64>().is_ok_and(f64::is_finite)
    }
    pub fn cadeia_e_logico(texto: &str) -> bool {
        matches!(texto.trim(), "verdadeiro" | "falso")
    }
    pub fn cadeia_e_caracter(texto: &str) -> bool { texto.chars().count() == 1 }
}
"#;

const PORS_CLOCK_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_clock {
    use std::cell::RefCell;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    enum ClockState {
        Real(Instant),
        Simulated { elapsed_ms: u64, epoch_ms: u64 },
    }

    thread_local! {
        static CLOCK: RefCell<ClockState> = RefCell::new(match std::env::var("PORTUGOL_PORS_SIMULATED_EPOCH_MS").ok().and_then(|v| v.parse::<u64>().ok()) {
            Some(epoch_ms) => ClockState::Simulated { elapsed_ms: 0, epoch_ms },
            None => ClockState::Real(Instant::now()),
        });
    }

    pub fn elapsed_ms() -> u64 {
        CLOCK.with(|cell| match &*cell.borrow() {
            ClockState::Real(started) => started.elapsed().as_millis() as u64,
            ClockState::Simulated { elapsed_ms, .. } => *elapsed_ms,
        })
    }

    pub fn now_epoch_ms() -> i64 {
        CLOCK.with(|cell| match &*cell.borrow() {
            ClockState::Real(_) => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            ClockState::Simulated { elapsed_ms, epoch_ms } => epoch_ms.saturating_add(*elapsed_ms) as i64,
        })
    }

    pub fn sleep_ms(ms: u64) {
        let simulated = CLOCK.with(|cell| {
            if let ClockState::Simulated { elapsed_ms, .. } = &mut *cell.borrow_mut() {
                *elapsed_ms = elapsed_ms.saturating_add(ms);
                true
            } else {
                false
            }
        });
        if !simulated {
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}
"#;

const PORS_UTIL_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_util {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    const MAX_SLEEP_MS: i32 = 60_000;

    thread_local! {
        static RNG: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E37_79B9_7F4A_7C15)
                | 1,
        );
    }

    fn next_u64() -> u64 {
        RNG.with(|cell| {
            let mut x = cell.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            cell.set(x);
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        })
    }

    pub fn definir_semente(semente: i32) {
        RNG.with(|cell| cell.set((semente as u32 as u64) | 1));
    }

    pub fn sorteia(min: i32, max: i32) -> Result<i32, String> {
        if min > max {
            return Err(super::__pors_string(&format!(
                "O valor mínimo ({min}) não pode ser maior que o máximo ({max})"
            )));
        }
        let span = (max as i64 - min as i64 + 1) as u64;
        let offset = (next_u64() % span) as i64;
        Ok((min as i64 + offset) as i32)
    }

    pub fn tempo_decorrido() -> i32 {
        super::__pors_clock::elapsed_ms().min(i32::MAX as u64) as i32
    }

    pub fn aguarde(ms: i32) {
        if ms < 0 {
            eprintln!("Erro de execução: o intervalo de espera em `Util.aguarde` não pode ser negativo.");
            std::process::exit(1);
        }
        if ms > MAX_SLEEP_MS {
            eprintln!("Erro de execução: o intervalo de espera ({ms} ms) excede o limite de {MAX_SLEEP_MS} ms por chamada.");
            std::process::exit(1);
        }
        super::__pors_clock::sleep_ms(ms as u64);
    }
}
"#;

const PORS_CALENDARIO_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_calendario {
    struct UtcDateTime {
        year: i32,
        month: u32,
        day: u32,
        weekday_1_to_7: i32,
        hour: u32,
        minute: u32,
        second: u32,
        millis: u32,
    }

    fn civil_from_days(z: i64) -> (i32, u32, u32) {
        let z = z + 719_468;
        let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
        let doe = (z - era * 146_097) as u64;
        let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
        let y = (yoe as i64) + era * 400;
        let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
        let mp = (5 * doy + 2) / 153;
        let d = doy - (153 * mp + 2) / 5 + 1;
        let m = if mp < 10 { mp + 3 } else { mp - 9 };
        let year = (y + i64::from(m <= 2)) as i32;
        (year, m as u32, d as u32)
    }

    fn now_utc() -> UtcDateTime {
        let epoch_ms = super::__pors_clock::now_epoch_ms();
        let millis = epoch_ms.rem_euclid(1_000) as u32;
        let total_secs = epoch_ms.div_euclid(1_000);
        let secs_of_day = total_secs.rem_euclid(86_400) as u32;
        let days = total_secs.div_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let weekday_1_to_7 = ((days + 4).rem_euclid(7) + 1) as i32;
        UtcDateTime {
            year,
            month,
            day,
            weekday_1_to_7,
            hour: secs_of_day / 3_600,
            minute: (secs_of_day % 3_600) / 60,
            second: secs_of_day % 60,
            millis,
        }
    }

    pub fn ano_atual() -> i32 { now_utc().year }
    pub fn mes_atual() -> i32 { now_utc().month as i32 }
    pub fn dia_mes_atual() -> i32 { now_utc().day as i32 }
    pub fn dia_semana_atual() -> i32 { now_utc().weekday_1_to_7 }
    pub fn hora_atual(formato_12h: bool) -> i32 {
        let h = now_utc().hour as i32;
        if formato_12h { h % 12 } else { h }
    }
    pub fn minuto_atual() -> i32 { now_utc().minute as i32 }
    pub fn segundo_atual() -> i32 { now_utc().second as i32 }
    pub fn milisegundo_atual() -> i32 { now_utc().millis as i32 }
    pub fn eh_bissexto(ano: i32) -> bool {
        (ano % 4 == 0 && ano % 100 != 0) || ano % 400 == 0
    }
    pub fn dias_no_mes(ano: i32, mes: i32) -> Result<i32, String> {
        match mes {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 if eh_bissexto(ano) => Ok(29),
            2 => Ok(28),
            _ => Err(super::__pors_string(&format!(
                "'{mes}' não corresponde a um mês válido (1 a 12)"
            ))),
        }
    }
    pub fn nome_mes(mes: i32) -> Result<String, String> {
        const MESES: [&str; 12] = [
            "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
            "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
        ];
        if !(1..=12).contains(&mes) {
            return Err(super::__pors_string(&format!(
                "'{mes}' não corresponde a um mês válido (1 a 12)"
            )));
        }
        Ok(super::__pors_string(MESES[(mes - 1) as usize]))
    }
    fn format_day(names: [&str; 7], dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        if !(1..=7).contains(&dia) {
            return Err(super::__pors_string(&format!(
                "'{dia}' não corresponde a um dia da semana válido (1 a 7)"
            )));
        }
        let raw = names[(dia - 1) as usize];
        let out = if alta {
            raw.to_uppercase()
        } else if baixa {
            raw.to_lowercase()
        } else {
            raw.to_owned()
        };
        super::__pors_account(out.len());
        Ok(out)
    }
    pub fn dia_semana_completo(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(
            ["Domingo", "Segunda-Feira", "Terça-Feira", "Quarta-Feira", "Quinta-Feira", "Sexta-Feira", "Sábado"],
            dia,
            alta,
            baixa,
        )
    }
    pub fn dia_semana_curto(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(
            ["Domingo", "Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"],
            dia,
            alta,
            baixa,
        )
    }
    pub fn dia_semana_abreviado(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"], dia, alta, baixa)
    }
    pub fn data_iso_atual() -> String {
        let now = now_utc();
        let iso = format!("{:04}-{:02}-{:02}", now.year, now.month, now.day);
        super::__pors_account(iso.len());
        iso
    }
}
"#;

const PORS_ARQUIVOS_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_arquivos {
    use std::io::Write as _;
    use std::path::{Component, Path, PathBuf};

    const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
    const MAX_LIST_ENTRIES: usize = 1_024;

    fn resolve(raw: &str, allow_root: bool) -> Result<PathBuf, String> {
        let root = std::env::var_os("PORTUGOL_PORS_WORKSPACE_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| super::__pors_string("Diretório de workspace indisponível."))?;
        if raw.contains('\0') {
            return Err(super::__pors_string("Caminho inválido: contém caractere nulo."));
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() && !allow_root {
            return Err(super::__pors_string("O caminho do arquivo não pode ser vazio."));
        }
        let rel = Path::new(trimmed);
        if rel.is_absolute() {
            return Err(super::__pors_string("Apenas caminhos relativos ao workspace são permitidos."));
        }
        let mut clean = PathBuf::new();
        for component in rel.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(super::__pors_string(
                        "Caminho fora do workspace não é permitido (`..` ou raiz absoluta).",
                    ));
                }
            }
        }
        if clean.as_os_str().is_empty() && !allow_root {
            return Err(super::__pors_string("O caminho deve apontar para um arquivo dentro do workspace."));
        }
        let canonical_root = std::fs::canonicalize(&root)
            .map_err(|err| super::__pors_string(&format!("Workspace inacessível: {err}")))?;
        let target = canonical_root.join(&clean);
        let mut probe = target.as_path();
        while !probe.exists() {
            let Some(parent) = probe.parent() else { break };
            probe = parent;
        }
        if probe.exists() {
            let canonical_probe = std::fs::canonicalize(probe)
                .map_err(|err| super::__pors_string(&format!("Não foi possível validar o caminho: {err}")))?;
            if !canonical_probe.starts_with(&canonical_root) {
                return Err(super::__pors_string("Caminho escapa do workspace (link simbólico bloqueado)."));
            }
        }
        Ok(target)
    }

    pub fn ler_texto(caminho: &str) -> Result<String, String> {
        let target = resolve(caminho, false)?;
        let meta = std::fs::metadata(&target)
            .map_err(|err| super::__pors_string(&format!("Não foi possível abrir '{caminho}': {err}")))?;
        if !meta.is_file() {
            return Err(super::__pors_string(&format!("'{caminho}' não é um arquivo regular.")));
        }
        if meta.len() as usize > MAX_FILE_BYTES {
            return Err(super::__pors_string(&format!(
                "O arquivo '{caminho}' excede o limite de {MAX_FILE_BYTES} bytes."
            )));
        }
        super::__pors_account(meta.len() as usize);
        std::fs::read_to_string(&target)
            .map_err(|err| super::__pors_string(&format!("Erro ao ler '{caminho}': {err}")))
    }

    pub fn ler_linhas(caminho: &str) -> Result<Vec<String>, String> {
        let content = ler_texto(caminho)?;
        Ok(content.lines().map(str::to_owned).collect())
    }

    pub fn escrever_texto(caminho: &str, conteudo: &str) -> Result<bool, String> {
        if conteudo.len() > MAX_FILE_BYTES {
            return Err(super::__pors_string(&format!(
                "O conteúdo excede o limite de {MAX_FILE_BYTES} bytes por arquivo."
            )));
        }
        let target = resolve(caminho, false)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                super::__pors_string(&format!("Não foi possível criar diretório para '{caminho}': {err}"))
            })?;
        }
        std::fs::write(&target, conteudo)
            .map(|()| true)
            .map_err(|err| super::__pors_string(&format!("Erro ao gravar '{caminho}': {err}")))
    }

    pub fn acrescentar_texto(caminho: &str, conteudo: &str) -> Result<bool, String> {
        let target = resolve(caminho, false)?;
        let existing_len = std::fs::metadata(&target).map(|m| m.len() as usize).unwrap_or(0);
        if existing_len.saturating_add(conteudo.len()) > MAX_FILE_BYTES {
            return Err(super::__pors_string(&format!(
                "O arquivo '{caminho}' excederia o limite de {MAX_FILE_BYTES} bytes."
            )));
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                super::__pors_string(&format!("Não foi possível criar diretório para '{caminho}': {err}"))
            })?;
        }
        let mut handle = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target)
            .map_err(|err| super::__pors_string(&format!("Erro ao acrescentar em '{caminho}': {err}")))?;
        handle
            .write_all(conteudo.as_bytes())
            .map(|()| true)
            .map_err(|err| super::__pors_string(&format!("Erro ao acrescentar em '{caminho}': {err}")))
    }

    pub fn existe(caminho: &str) -> Result<bool, String> {
        Ok(resolve(caminho, true)?.exists())
    }

    pub fn remover(caminho: &str) -> Result<bool, String> {
        let target = resolve(caminho, false)?;
        std::fs::remove_file(&target)
            .map(|()| true)
            .map_err(|err| super::__pors_string(&format!("Erro ao remover '{caminho}': {err}")))
    }

    pub fn listar(diretorio: &str) -> Result<Vec<String>, String> {
        let target = resolve(diretorio, true)?;
        let entries = std::fs::read_dir(&target)
            .map_err(|err| super::__pors_string(&format!("Erro ao listar '{diretorio}': {err}")))?;
        let mut names = Vec::new();
        let mut total_bytes = 0usize;
        for entry in entries {
            let entry = entry.map_err(|err| super::__pors_string(&format!("Erro ao listar '{diretorio}': {err}")))?;
            if names.len() >= MAX_LIST_ENTRIES {
                return Err(super::__pors_string(&format!(
                    "Diretório excede o limite de {MAX_LIST_ENTRIES} entradas."
                )));
            }
            let item = entry.file_name().to_string_lossy().into_owned();
            total_bytes = total_bytes.saturating_add(item.len());
            names.push(item);
        }
        names.sort();
        super::__pors_account(total_bytes);
        Ok(names)
    }
}
"#;

const PORS_JSON_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __pors_json {
    use std::fmt::Write as _;

    const MAX_JSON_DEPTH: usize = 64;
    const MAX_JSON_BYTES: usize = 2 * 1024 * 1024;

    enum Value {
        Null,
        Bool(bool),
        Int(i64),
        Float(f64),
        Str(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    struct Parser<'a> {
        src: &'a str,
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn parse(src: &'a str) -> Result<Value, String> {
            if src.len() > MAX_JSON_BYTES {
                return Err(super::__pors_string(&format!(
                    "Documento JSON excede o limite de {MAX_JSON_BYTES} bytes."
                )));
            }
            let mut p = Self { src, bytes: src.as_bytes(), pos: 0 };
            p.skip_ws();
            let v = p.parse_value(0)?;
            p.skip_ws();
            if p.pos != p.bytes.len() {
                return Err(super::__pors_string("Conteúdo inesperado após o término do JSON."));
            }
            Ok(v)
        }

        fn skip_ws(&mut self) {
            while let Some(&b) = self.bytes.get(self.pos) {
                if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                    self.pos += 1;
                } else {
                    break;
                }
            }
        }

        fn peek(&self) -> Option<u8> {
            self.bytes.get(self.pos).copied()
        }

        fn parse_value(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__pors_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.skip_ws();
            match self.peek() {
                Some(b'{') => self.parse_object(depth + 1),
                Some(b'[') => self.parse_array(depth + 1),
                Some(b'"') => self.parse_string().map(Value::Str),
                Some(b't') => { self.expect(b"true")?; Ok(Value::Bool(true)) }
                Some(b'f') => { self.expect(b"false")?; Ok(Value::Bool(false)) }
                Some(b'n') => { self.expect(b"null")?; Ok(Value::Null) }
                Some(b'-' | b'0'..=b'9') => self.parse_number(),
                Some(_) => Err(super::__pors_string(&format!(
                    "Caractere inesperado no JSON na posição {}.",
                    self.pos
                ))),
                None => Err(super::__pors_string("Fim inesperado do JSON.")),
            }
        }

        fn expect(&mut self, lit: &[u8]) -> Result<(), String> {
            if self.bytes.get(self.pos..self.pos + lit.len()) == Some(lit) {
                self.pos += lit.len();
                Ok(())
            } else {
                Err(super::__pors_string(&format!("Token JSON inválido na posição {}.", self.pos)))
            }
        }

        fn parse_object(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__pors_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.pos += 1;
            self.skip_ws();
            let mut entries: Vec<(String, Value)> = Vec::new();
            if self.peek() == Some(b'}') {
                self.pos += 1;
                return Ok(Value::Object(entries));
            }
            loop {
                self.skip_ws();
                if self.peek() != Some(b'"') {
                    return Err(super::__pors_string(&format!(
                        "Esperada chave de texto no objeto JSON na posição {}.",
                        self.pos
                    )));
                }
                let key = self.parse_string()?;
                self.skip_ws();
                if self.peek() != Some(b':') {
                    return Err(super::__pors_string(&format!(
                        "Esperado `:` após chave JSON na posição {}.",
                        self.pos
                    )));
                }
                self.pos += 1;
                let val = self.parse_value(depth)?;
                if let Some(existing) = entries.iter_mut().find(|(k, _)| k == &key) {
                    existing.1 = val;
                } else {
                    entries.push((key, val));
                }
                self.skip_ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b'}') => { self.pos += 1; return Ok(Value::Object(entries)); }
                    _ => return Err(super::__pors_string(&format!(
                        "Esperado `,` ou `}}` no objeto JSON na posição {}.",
                        self.pos
                    ))),
                }
            }
        }

        fn parse_array(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__pors_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.pos += 1;
            self.skip_ws();
            let mut items = Vec::new();
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            loop {
                items.push(self.parse_value(depth)?);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b']') => { self.pos += 1; return Ok(Value::Array(items)); }
                    _ => return Err(super::__pors_string(&format!(
                        "Esperado `,` ou `]` na lista JSON na posição {}.",
                        self.pos
                    ))),
                }
            }
        }

        fn parse_hex4(&mut self) -> Result<u16, String> {
            let Some(slice) = self.src.get(self.pos..self.pos + 4) else {
                return Err(super::__pors_string("Escape Unicode `\\u` incompleto no JSON."));
            };
            let code = u16::from_str_radix(slice, 16)
                .map_err(|_| super::__pors_string("Escape Unicode `\\u` inválido no JSON."))?;
            self.pos += 4;
            Ok(code)
        }

        fn parse_string(&mut self) -> Result<String, String> {
            self.pos += 1;
            let mut out = String::new();
            let mut start = self.pos;
            while let Some(&b) = self.bytes.get(self.pos) {
                match b {
                    b'"' => {
                        out.push_str(&self.src[start..self.pos]);
                        self.pos += 1;
                        return Ok(out);
                    }
                    b'\\' => {
                        out.push_str(&self.src[start..self.pos]);
                        self.pos += 1;
                        let esc = self.peek().ok_or_else(|| super::__pors_string("Escape incompleto na cadeia JSON."))?;
                        self.pos += 1;
                        match esc {
                            b'"' => out.push('"'),
                            b'\\' => out.push('\\'),
                            b'/' => out.push('/'),
                            b'b' => out.push('\u{0008}'),
                            b'f' => out.push('\u{000C}'),
                            b'n' => out.push('\n'),
                            b'r' => out.push('\r'),
                            b't' => out.push('\t'),
                            b'u' => {
                                let first = self.parse_hex4()?;
                                if (0xD800..=0xDBFF).contains(&first) {
                                    if self.bytes.get(self.pos..self.pos + 2) != Some(b"\\u") {
                                        return Err(super::__pors_string("Par substituto UTF-16 incompleto no JSON."));
                                    }
                                    self.pos += 2;
                                    let second = self.parse_hex4()?;
                                    if !(0xDC00..=0xDFFF).contains(&second) {
                                        return Err(super::__pors_string("Par substituto UTF-16 inválido no JSON."));
                                    }
                                    let scalar = 0x10000 + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                                    let ch = char::from_u32(scalar)
                                        .ok_or_else(|| super::__pors_string("Código Unicode inválido no JSON."))?;
                                    out.push(ch);
                                } else if (0xDC00..=0xDFFF).contains(&first) {
                                    return Err(super::__pors_string("Substituto UTF-16 isolado inválido no JSON."));
                                } else {
                                    let ch = char::from_u32(first as u32)
                                        .ok_or_else(|| super::__pors_string("Código Unicode inválido no JSON."))?;
                                    out.push(ch);
                                }
                            }
                            _ => return Err(super::__pors_string("Sequência de escape inválida na cadeia JSON.")),
                        }
                        start = self.pos;
                    }
                    0x00..=0x1F => return Err(super::__pors_string("Caractere de controle não escapado na cadeia JSON.")),
                    _ => self.pos += 1,
                }
            }
            Err(super::__pors_string("Cadeia JSON não fechada."))
        }

        fn parse_number(&mut self) -> Result<Value, String> {
            let start = self.pos;
            if self.peek() == Some(b'-') { self.pos += 1; }
            match self.peek() {
                Some(b'0') => self.pos += 1,
                Some(b'1'..=b'9') => {
                    self.pos += 1;
                    while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
                }
                _ => return Err(super::__pors_string(&format!("Número JSON inválido na posição {}.", self.pos))),
            }
            let mut is_float = false;
            if self.peek() == Some(b'.') {
                is_float = true;
                self.pos += 1;
                if !matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(super::__pors_string(&format!("Fração decimal inválida no JSON na posição {}.", self.pos)));
                }
                while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
            }
            if matches!(self.peek(), Some(b'e' | b'E')) {
                is_float = true;
                self.pos += 1;
                if matches!(self.peek(), Some(b'+' | b'-')) { self.pos += 1; }
                if !matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(super::__pors_string(&format!("Expoente inválido no JSON na posição {}.", self.pos)));
                }
                while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
            }
            let token = &self.src[start..self.pos];
            if !is_float {
                if let Ok(n) = token.parse::<i64>() {
                    return Ok(Value::Int(n));
                }
            }
            let f = token.parse::<f64>()
                .map_err(|_| super::__pors_string(&format!("Número JSON inválido: `{token}`.")))?;
            if !f.is_finite() {
                return Err(super::__pors_string(&format!("Número JSON fora do intervalo finito: `{token}`.")));
            }
            Ok(Value::Float(f))
        }
    }

    fn escape_into(text: &str, out: &mut String) {
        out.push('"');
        for ch in text.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\u{0008}' => out.push_str("\\b"),
                '\u{000C}' => out.push_str("\\f"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    fn write_val(value: &Value, pretty: bool, indent: usize, out: &mut String) -> Result<(), String> {
        if out.len() > MAX_JSON_BYTES {
            return Err(super::__pors_string(&format!("Saída JSON excede o limite de {MAX_JSON_BYTES} bytes.")));
        }
        match value {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Int(n) => { let _ = write!(out, "{n}"); }
            Value::Float(f) => {
                if !f.is_finite() {
                    return Err(super::__pors_string("Valores NaN ou infinitos não são válidos em JSON."));
                }
                let rendered = format!("{f}");
                out.push_str(&rendered);
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    out.push_str(".0");
                }
            }
            Value::Str(s) => escape_into(s, out),
            Value::Array(items) => {
                if items.is_empty() { out.push_str("[]"); return Ok(()); }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    if pretty {
                        out.push('\n');
                        for _ in 0..indent + 2 { out.push(' '); }
                    }
                    write_val(item, pretty, indent + 2, out)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent { out.push(' '); }
                }
                out.push(']');
            }
            Value::Object(entries) => {
                if entries.is_empty() { out.push_str("{}"); return Ok(()); }
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    if pretty {
                        out.push('\n');
                        for _ in 0..indent + 2 { out.push(' '); }
                    }
                    escape_into(k, out);
                    out.push(':');
                    if pretty { out.push(' '); }
                    write_val(v, pretty, indent + 2, out)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent { out.push(' '); }
                }
                out.push('}');
            }
        }
        if out.len() > MAX_JSON_BYTES {
            return Err(super::__pors_string(&format!("Saída JSON excede o limite de {MAX_JSON_BYTES} bytes.")));
        }
        Ok(())
    }

    fn serialize(value: &Value, pretty: bool) -> Result<String, String> {
        let mut out = String::new();
        write_val(value, pretty, 0, &mut out)?;
        super::__pors_account(out.len());
        Ok(out)
    }

    fn parse_object(src: &str) -> Result<Vec<(String, Value)>, String> {
        match Parser::parse(src)? {
            Value::Object(entries) => Ok(entries),
            _ => Err(super::__pors_string("O documento JSON precisa ser um objeto (`{ ... }`).")),
        }
    }

    fn get_entry<'a>(entries: &'a [(String, Value)], key: &str) -> Result<&'a Value, String> {
        entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            .ok_or_else(|| super::__pors_string(&format!("Chave '{key}' não encontrada no objeto JSON.")))
    }

    fn set_key(src: &str, key: &str, val: Value) -> Result<String, String> {
        let mut entries = parse_object(src)?;
        if let Some(existing) = entries.iter_mut().find(|(k, _)| k == key) {
            existing.1 = val;
        } else {
            entries.push((key.to_owned(), val));
        }
        serialize(&Value::Object(entries), false)
    }

    pub fn valido(texto: &str) -> bool { Parser::parse(texto).is_ok() }
    pub fn formatar(texto: &str) -> Result<String, String> { serialize(&Parser::parse(texto)?, true) }
    pub fn minificar(texto: &str) -> Result<String, String> { serialize(&Parser::parse(texto)?, false) }
    pub fn escapar_cadeia(texto: &str) -> String {
        let mut out = String::with_capacity(texto.len().saturating_add(2));
        escape_into(texto, &mut out);
        super::__pors_account(out.len());
        out
    }
    pub fn contem_chave(json: &str, chave: &str) -> Result<bool, String> {
        Ok(parse_object(json)?.iter().any(|(k, _)| k == chave))
    }
    pub fn obter_chaves(json: &str) -> Result<Vec<String>, String> {
        let entries = parse_object(json)?;
        let total: usize = entries.iter().map(|(k, _)| k.len()).sum();
        super::__pors_account(total);
        Ok(entries.into_iter().map(|(k, _)| k).collect())
    }
    pub fn obter_cadeia(json: &str, chave: &str) -> Result<String, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Str(s) => Ok(super::__pors_string(s)),
            _ => Err(super::__pors_string(&format!("A chave '{chave}' não contém uma cadeia no JSON."))),
        }
    }
    pub fn obter_inteiro(json: &str, chave: &str) -> Result<i32, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Int(n) => i32::try_from(*n).map_err(|_| {
                super::__pors_string(&format!("O valor de '{chave}' excede o intervalo de `inteiro` (32 bits)."))
            }),
            _ => Err(super::__pors_string(&format!("A chave '{chave}' não contém um número inteiro no JSON."))),
        }
    }
    pub fn obter_real(json: &str, chave: &str) -> Result<f64, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Int(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(super::__pors_string(&format!("A chave '{chave}' não contém um número no JSON."))),
        }
    }
    pub fn obter_logico(json: &str, chave: &str) -> Result<bool, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Bool(b) => Ok(*b),
            _ => Err(super::__pors_string(&format!("A chave '{chave}' não contém um valor lógico no JSON."))),
        }
    }
    pub fn obter_json(json: &str, chave: &str) -> Result<String, String> {
        let entries = parse_object(json)?;
        serialize(get_entry(&entries, chave)?, false)
    }
    pub fn obter_lista_cadeia(json: &str, chave: &str) -> Result<Vec<String>, String> {
        let entries = parse_object(json)?;
        let Value::Array(items) = get_entry(&entries, chave)? else {
            return Err(super::__pors_string(&format!("A chave '{chave}' não contém uma lista no JSON.")));
        };
        let mut out = Vec::with_capacity(items.len());
        let mut total = 0usize;
        for item in items {
            let Value::Str(s) = item else {
                return Err(super::__pors_string(&format!("A lista em '{chave}' contém elementos que não são cadeias.")));
            };
            total = total.saturating_add(s.len());
            out.push(s.clone());
        }
        super::__pors_account(total);
        Ok(out)
    }
    pub fn obter_lista_inteiro(json: &str, chave: &str) -> Result<Vec<i32>, String> {
        let entries = parse_object(json)?;
        let Value::Array(items) = get_entry(&entries, chave)? else {
            return Err(super::__pors_string(&format!("A chave '{chave}' não contém uma lista no JSON.")));
        };
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let Value::Int(n) = item else {
                return Err(super::__pors_string(&format!("A lista em '{chave}' contém elementos que não são inteiros.")));
            };
            let v = i32::try_from(*n).map_err(|_| {
                super::__pors_string(&format!("A lista em '{chave}' contém inteiro fora do intervalo de 32 bits."))
            })?;
            out.push(v);
        }
        Ok(out)
    }
    pub fn definir_cadeia(json: &str, chave: &str, valor: &str) -> Result<String, String> {
        set_key(json, chave, Value::Str(valor.to_owned()))
    }
    pub fn definir_inteiro(json: &str, chave: &str, valor: i32) -> Result<String, String> {
        set_key(json, chave, Value::Int(i64::from(valor)))
    }
    pub fn definir_real(json: &str, chave: &str, valor: f64) -> Result<String, String> {
        if !valor.is_finite() {
            return Err(super::__pors_string("Valores NaN ou infinitos não podem ser gravados em JSON."));
        }
        set_key(json, chave, Value::Float(valor))
    }
    pub fn definir_logico(json: &str, chave: &str, valor: bool) -> Result<String, String> {
        set_key(json, chave, Value::Bool(valor))
    }
    pub fn definir_json(json: &str, chave: &str, valor_json: &str) -> Result<String, String> {
        let nested = Parser::parse(valor_json)?;
        set_key(json, chave, nested)
    }
    pub fn remover_chave(json: &str, chave: &str) -> Result<String, String> {
        let mut entries = parse_object(json)?;
        entries.retain(|(k, _)| k != chave);
        serialize(&Value::Object(entries), false)
    }
}
"#;
