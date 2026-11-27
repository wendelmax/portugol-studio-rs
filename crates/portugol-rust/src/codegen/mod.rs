mod expr;
mod runtime;
mod stmt;
mod types;

use portugol_hir::{Program, Type};

use crate::{
    GeneratedRust, JAVA_DOUBLE_HELPER, SourceMapQuality, SourceMapSegment, SourceRange, rust_ident,
};

use expr::push_expr;
use runtime::{
    PORS_ARQUIVOS_RUNTIME, PORS_CALENDARIO_RUNTIME, PORS_CLOCK_RUNTIME, PORS_INTERNET_RUNTIME,
    PORS_JSON_RUNTIME, PORS_MATEMATICA_RUNTIME, PORS_TEXTO_RUNTIME, PORS_TIPOS_RUNTIME,
    PORS_UI_RUNTIME, PORS_UTIL_RUNTIME,
};
use stmt::render_statements;
use types::{
    binding_info, expression_has_read, mvu_entry_update_arity, pors_clone_value,
    program_uses_host_module, rust_enum_type_name, rust_enum_variant_path, rust_type,
    statements_have_read,
};

pub(crate) fn render_pors_with_map(program: &Program) -> GeneratedRust {
    render(program, true)
}

fn render(program: &Program, pors: bool) -> GeneratedRust {
    let has_read = statements_have_read(&program.body)
        || program.functions.iter().any(|f| {
            statements_have_read(&f.body)
                || f.tail_expression.as_ref().is_some_and(expression_has_read)
        });
    let uses_ui = pors && program_uses_host_module(program, "UI");
    let uses_matematica = pors && program_uses_host_module(program, "Matematica");
    let uses_texto = pors && program_uses_host_module(program, "Texto");
    let uses_tipos = pors && program_uses_host_module(program, "Tipos");
    let uses_util = pors && program_uses_host_module(program, "Util");
    let uses_calendario = pors && program_uses_host_module(program, "Calendario");
    let uses_arquivos = pors && program_uses_host_module(program, "Arquivos");
    let uses_json = pors && program_uses_host_module(program, "Json");
    let uses_internet = pors && program_uses_host_module(program, "Internet");
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
    if uses_internet {
        output.push_str(PORS_INTERNET_RUNTIME);
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
        if let Some(tail) = &function.tail_expression {
            output.push_str("    return ");
            push_expr(
                &mut output,
                &mut mappings,
                tail,
                &function_program,
                true,
                pors,
            );
            output.push_str(";\n");
        }
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
