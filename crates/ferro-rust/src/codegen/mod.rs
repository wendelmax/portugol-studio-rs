mod expr;
mod runtime;
mod stmt;
mod types;

use ferro_hir::{Program, Type};

use crate::{
    GeneratedRust, JAVA_DOUBLE_HELPER, SourceMapQuality, SourceMapSegment, SourceRange, rust_ident,
};

use expr::push_expr;
use runtime::{
    FERRO_ARQUIVOS_RUNTIME, FERRO_CALENDARIO_RUNTIME, FERRO_CLOCK_RUNTIME, FERRO_INTERNET_RUNTIME,
    FERRO_JSON_RUNTIME, FERRO_MATEMATICA_RUNTIME, FERRO_TEXTO_RUNTIME, FERRO_TIPOS_RUNTIME,
    FERRO_UI_RUNTIME, FERRO_UTIL_RUNTIME,
};
use stmt::render_statements;
use types::{
    binding_info, ferro_clone_value, mvu_entry_update_arity, program_uses_host_module,
    rust_enum_type_name, rust_enum_variant_path, rust_type,
};

pub(crate) fn render_ferro_with_map(program: &Program) -> GeneratedRust {
    render(program, true)
}

fn render(program: &Program, ferro: bool) -> GeneratedRust {
    let uses_ui = ferro && program_uses_host_module(program, "UI");
    let uses_matematica = ferro && program_uses_host_module(program, "Matematica");
    let uses_texto = ferro && program_uses_host_module(program, "Texto");
    let uses_tipos = ferro && program_uses_host_module(program, "Tipos");
    let uses_util = ferro && program_uses_host_module(program, "Util");
    let uses_calendario = ferro && program_uses_host_module(program, "Calendario");
    let uses_arquivos = ferro && program_uses_host_module(program, "Arquivos");
    let uses_json = ferro && program_uses_host_module(program, "Json");
    let uses_internet = ferro && program_uses_host_module(program, "Internet");
    let mvu_update_arity = if ferro && program.body.is_empty() {
        mvu_entry_update_arity(program)
    } else {
        None
    };
    let mut output = String::new();
    if uses_ui || mvu_update_arity.is_some() {
        output.push_str(FERRO_UI_RUNTIME);
    }
    if uses_matematica {
        output.push_str(FERRO_MATEMATICA_RUNTIME);
    }
    if uses_texto {
        output.push_str(FERRO_TEXTO_RUNTIME);
    }
    if uses_tipos {
        output.push_str(FERRO_TIPOS_RUNTIME);
    }
    if uses_util || uses_calendario {
        output.push_str(FERRO_CLOCK_RUNTIME);
    }
    if uses_util {
        output.push_str(FERRO_UTIL_RUNTIME);
    }
    if uses_calendario {
        output.push_str(FERRO_CALENDARIO_RUNTIME);
    }
    if uses_arquivos {
        output.push_str(FERRO_ARQUIVOS_RUNTIME);
    }
    if uses_json {
        output.push_str(FERRO_JSON_RUNTIME);
    }
    if uses_internet {
        output.push_str(FERRO_INTERNET_RUNTIME);
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
        if ferro {
            output.push_str(&format!(
                "fn __ferro_clone_{}(value: &{}) -> {} {{ {} {{\n",
                definition.id.0,
                rust_ident(&definition.name),
                rust_ident(&definition.name),
                rust_ident(&definition.name)
            ));
            for field in &definition.fields {
                let name = rust_ident(&field.name);
                let clone = match field.ty {
                    Type::String => format!("__ferro_string(&value.{name})"),
                    Type::VectorString => format!("__ferro_vec_string_clone(&value.{name})"),
                    Type::Record(id) => program
                        .record_types
                        .iter()
                        .find(|item| item.id == id)
                        .map(|item| format!("__ferro_clone_{}(&value.{name})", item.id.0))
                        .unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::VectorRecord(id) => program
                        .record_types
                        .iter()
                        .find(|item| item.id == id)
                        .map(|item| {
                            format!(
                                "value.{name}.iter().map(|item| __ferro_clone_{}(item)).collect()",
                                item.id.0
                            )
                        })
                        .unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::Enum(id) => program.enum_types.iter().find(|item| item.id == id).map(|item| format!("__ferro_clone_enum_{}(&value.{name})", item.id.0)).unwrap_or_else(|| format!("value.{name}.clone()")),
                    Type::VectorEnum(id) => program.enum_types.iter().find(|item| item.id == id).map(|item| format!("value.{name}.iter().map(|item| __ferro_clone_enum_{}(item)).collect()", item.id.0)).unwrap_or_else(|| format!("value.{name}.clone()")),
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
        if ferro {
            let enum_ty = rust_enum_type_name(definition, program);
            output.push_str(&format!(
                "fn __ferro_clone_enum_{}(value: &{enum_ty}) -> {enum_ty} {{ match value {{\n",
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
                        .map(|(ty, binding)| ferro_clone_value(*ty, binding, program))
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
                    if parameter.mutability == ferro_hir::Mutability::Mutable {
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
            ferro,
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
                ferro,
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
            "    use std::io::BufRead;\n    let mut estado = {init_fn}();\n    let mut step: u64 = 0;\n    __ferro_ui::begin_frame();\n    let root = {view_fn}(estado.clone());\n    println!(\"{{{{\\\"step\\\":{{step}},\\\"tree\\\":{{}}}}}}\", __ferro_ui::finish_frame(root).to_json());\n    let _ = io::stdout().flush();\n    let stdin = io::stdin();\n    for line in stdin.lock().lines() {{\n        let Ok(line) = line else {{ break }};\n        let trimmed = line.trim();\n        if trimmed.is_empty() {{\n            continue;\n        }}\n        let Some((tipo, alvo, valor)) = __ferro_ui::parse_event_line(trimmed) else {{\n            eprintln!(\"Evento inválido: {{trimmed}}\");\n            continue;\n        }};\n        let _ = (&tipo, &valor);\n        estado = {update_call};\n        step += 1;\n        __ferro_ui::begin_frame();\n        let root = {view_fn}(estado.clone());\n        println!(\"{{{{\\\"step\\\":{{step}},\\\"tree\\\":{{}}}}}}\", __ferro_ui::finish_frame(root).to_json());\n        let _ = io::stdout().flush();\n    }}\n"
        ));
    } else {
        render_statements(
            &program.body,
            program,
            1,
            &mut output,
            &mut mappings,
            ferro,
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
    let prelude = render_shared_runtime(&output, ferro);
    let prelude_len = prelude.len() as u32;
    for mapping in &mut mappings {
        mapping.generated.start += prelude_len;
        mapping.generated.end += prelude_len;
    }
    if prelude_len > 0 {
        mappings.push(SourceMapSegment {
            generated: SourceRange {
                start: 0,
                end: prelude_len,
            },
            source: None,
            quality: SourceMapQuality::GeneratedOnly,
        });
    }
    mappings.sort_by_key(|mapping| (mapping.generated.start, mapping.generated.end));
    GeneratedRust {
        code: format!("{prelude}{output}"),
        mappings,
    }
}

fn render_shared_runtime(code: &str, ferro: bool) -> String {
    let uses_read_line = code.contains("__ferro_read_line(");
    let uses_parse = code.contains("__ferro_parse::<");
    let uses_java_double = code.contains("__ferro_java_double(");
    let uses_vec_string_clone = code.contains("__ferro_vec_string_clone(");
    let uses_string = code.contains("__ferro_string(") || uses_vec_string_clone;
    let uses_index = code.contains("__ferro_index(");
    let uses_char = code.contains("__ferro_char(");
    let uses_concat = code.contains("__ferro_concat(");
    let uses_account = code.contains("__ferro_account(") || uses_string || uses_char || uses_concat;

    let mut output = String::new();
    if uses_read_line || code.contains("io::stdout") || code.contains("io::stdin") {
        output.push_str("use std::io::{self, Write};\n\n");
    }
    if uses_read_line {
        output.push_str(
            "fn __ferro_read_line(expected: &str) -> String {\n    if std::env::var_os(\"FERRO_STUDIO_INPUT_PROTOCOL\").is_some() {\n        print!(\"\\u{1e}FERRO_INPUT:{expected}\\u{1e}\\n\");\n        let _ = io::stdout().flush();\n    }\n    let mut line = String::new();\n    if io::stdin().read_line(&mut line).is_err() {\n        eprintln!(\"Não foi possível ler a entrada.\");\n        std::process::exit(1);\n    }\n    line.trim_end_matches(['\\r', '\\n']).to_owned()\n}\n\n",
        );
    }
    if uses_parse {
        output.push_str(
            "use std::str::FromStr;\nfn __ferro_parse<T: FromStr>(input: &str, expected: &str) -> T {\n    input.trim().parse().unwrap_or_else(|_| {\n        eprintln!(\"Entrada inválida: esperado {expected}.\");\n        std::process::exit(1);\n    })\n}\n\n",
        );
    }
    if uses_java_double {
        output.push_str(JAVA_DOUBLE_HELPER);
        output.push_str("\n\n");
    }
    if ferro && uses_account {
        output.push_str(
            "thread_local! {\n    static __FERRO_STRING_BYTES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };\n}\nfn __ferro_account(len: usize) {\n    __FERRO_STRING_BYTES.with(|used| {\n        let next = used.get().saturating_add(len);\n        if next > 16 * 1024 * 1024 {\n            std::process::exit(78);\n        }\n        used.set(next);\n    });\n}\n\n",
        );
    }
    if ferro && uses_string {
        output.push_str(
            "fn __ferro_string(value: &str) -> String {\n    __ferro_account(value.len());\n    value.to_owned()\n}\n\n",
        );
    }
    if ferro && uses_vec_string_clone {
        output.push_str(
            "fn __ferro_vec_string_clone(value: &[String]) -> Vec<String> {\n    value.iter().map(|item| __ferro_string(item)).collect()\n}\n\n",
        );
    }
    if ferro && uses_index {
        output.push_str(
            "fn __ferro_index(index: i32, len: usize) -> usize {\n    if index < 0 || index as usize >= len {\n        eprintln!(\"Erro de execução: índice de vetor fora do intervalo ({index}).\");\n        std::process::exit(1);\n    }\n    index as usize\n}\n\n",
        );
    }
    if ferro && uses_char {
        output.push_str(
            "fn __ferro_char(value: char) -> String {\n    __ferro_account(value.len_utf8());\n    value.to_string()\n}\n\n",
        );
    }
    if ferro && uses_concat {
        output.push_str(
            "fn __ferro_concat(left: &str, right: &str) -> String {\n    let len = left.len().saturating_add(right.len());\n    __ferro_account(len);\n    let mut value = String::with_capacity(len);\n    value.push_str(left);\n    value.push_str(right);\n    value\n}\n\n",
        );
    }
    output
}
