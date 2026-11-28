//! Conversão verificada de programas Ferro (`.fer`) em Rust compilável.

mod codegen;
mod screen;

pub use screen::{ScreenCargoProject, export_screen_cargo_project};

use ferro_hir::Diagnostic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMapQuality {
    Exact,
    Approximate,
    GeneratedOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SourceMapSegment {
    pub generated: SourceRange,
    pub source: Option<SourceRange>,
    pub quality: SourceMapQuality,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedRust {
    pub code: String,
    pub mappings: Vec<SourceMapSegment>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Ty {
    Inteiro,
    Real,
    Logico,
    Cadeia,
    Caracter,
}

/// Gera Rust para o dialeto educacional Ferro.
pub fn export_ferro_rs(source: &str) -> Result<String, Vec<Diagnostic>> {
    export_ferro_rs_with_map(source).map(|generated| generated.code)
}

/// Projeto Cargo pronto gerado a partir de um programa `.fer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedCargoProject {
    pub package_name: String,
    pub cargo_toml: String,
    pub main_rs: String,
    pub readme_md: String,
    pub mappings: Vec<SourceMapSegment>,
}

/// Gera Rust para Ferro e preserva os spans HIR de origem.
pub fn export_ferro_rs_with_map(source: &str) -> Result<GeneratedRust, Vec<Diagnostic>> {
    export_ferro_rs_with_options(source, &ferro_frontend::LowerOptions::default())
}

/// Como [`export_ferro_rs_with_map`], com opções do frontend: por exemplo, o
/// code-behind de uma tela, com os registros externos da janela e sem `inicio`
/// (o `main` gerado fica vazio).
pub fn export_ferro_rs_with_options(
    source: &str,
    options: &ferro_frontend::LowerOptions,
) -> Result<GeneratedRust, Vec<Diagnostic>> {
    export_with(source, options, &[])
}

/// Serviços do sistema das telas: só a exportação de tela traz o código
/// deles (com as dependências no `Cargo.toml`).
pub(crate) const SCREEN_SERVICES: &[&str] = &["Dialogo", "AreaTransferencia", "Notificacao"];

fn export_with(
    source: &str,
    options: &ferro_frontend::LowerOptions,
    allowed: &[&str],
) -> Result<GeneratedRust, Vec<Diagnostic>> {
    let hir = ferro_frontend::lower_with(source, options)?;
    if let Some((module, name, span)) = find_first_unsupported_host_call(&hir, allowed) {
        return Err(vec![Diagnostic::error(
            "ferro-rust.host-call-unsupported",
            format!(
                "A exportação Rust para chamadas nativas `{module}.{name}` ainda não está disponível."
            ),
            span,
        )]);
    }
    Ok(codegen::render_ferro_with_map(&hir))
}

/// Gera um projeto Cargo completo (Edition 2024, sem dependências externas) a partir
/// de um programa `.fer`.
pub fn export_ferro_rs_cargo_project(
    source: &str,
    requested_name: &str,
) -> Result<ExportedCargoProject, Vec<Diagnostic>> {
    let generated = export_ferro_rs_with_map(source)?;
    let package_name = sanitize_cargo_package_name(requested_name);
    let cargo_toml =
        format!("[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n");
    let readme_md = format!(
        "# {package_name}\n\nProjeto Rust gerado a partir de Ferro (`.fer`).\n\n## Como executar\n\n```bash\ncargo run\n```\n"
    );
    Ok(ExportedCargoProject {
        package_name,
        cargo_toml,
        main_rs: generated.code,
        readme_md,
        mappings: generated.mappings,
    })
}

/// Grava um [`ExportedCargoProject`] em disco (`Cargo.toml`, `src/main.rs` e `README.md`).
pub fn write_cargo_project(
    project: &ExportedCargoProject,
    dir: &std::path::Path,
) -> std::io::Result<()> {
    let src_dir = dir.join("src");
    std::fs::create_dir_all(&src_dir)?;
    std::fs::write(dir.join("Cargo.toml"), &project.cargo_toml)?;
    std::fs::write(src_dir.join("main.rs"), &project.main_rs)?;
    std::fs::write(dir.join("README.md"), &project.readme_md)?;
    Ok(())
}

/// Code-behind de uma tela, com os serviços do sistema liberados.
pub(crate) fn export_screen_code(
    source: &str,
    options: &ferro_frontend::LowerOptions,
) -> Result<GeneratedRust, Vec<Diagnostic>> {
    export_with(source, options, SCREEN_SERVICES)
}

/// Nome Rust gerado para um registro, campo ou função FERRO (`pt_nome`), para
/// quem escreve código que conversa com o gerado.
pub fn rust_name(name: &str) -> String {
    rust_ident(name)
}

/// Nome de pacote Cargo válido a partir do nome do projeto.
pub fn cargo_package_name(raw: &str) -> String {
    sanitize_cargo_package_name(raw)
}

fn sanitize_cargo_package_name(raw: &str) -> String {
    let mut sanitized = String::new();
    let mut last_dash = false;
    for ch in raw.chars() {
        let lower = ch.to_ascii_lowercase();
        if lower.is_ascii_alphanumeric() || lower == '_' {
            sanitized.push(lower);
            last_dash = false;
        } else if !sanitized.is_empty() && !last_dash {
            sanitized.push('-');
            last_dash = true;
        }
    }
    let trimmed = sanitized.trim_matches('-');
    if trimmed.is_empty() {
        return "programa-ferro".to_owned();
    }
    if trimmed.as_bytes()[0].is_ascii_digit() {
        format!("ferro-{trimmed}")
    } else {
        trimmed.to_owned()
    }
}

fn find_first_unsupported_host_call<'a>(
    program: &'a ferro_hir::Program,
    extra: &[&str],
) -> Option<(&'a str, &'a str, ferro_hir::Span)> {
    find_host_call_in_stmts(&program.body, extra).or_else(|| {
        program.functions.iter().find_map(|function| {
            find_host_call_in_stmts(&function.body, extra).or_else(|| {
                function
                    .tail_expression
                    .as_ref()
                    .and_then(|item| find_host_call_in_expr(item, extra))
            })
        })
    })
}

fn find_host_call_in_stmts<'a>(
    stmts: &'a [ferro_hir::Stmt],
    extra: &[&str],
) -> Option<(&'a str, &'a str, ferro_hir::Span)> {
    use ferro_hir::StmtKind as S;
    for stmt in stmts {
        let found = match &stmt.kind {
            S::Declare { initializer, .. } => initializer
                .as_ref()
                .and_then(|item| find_host_call_in_expr(item, extra)),
            S::Assign { value, .. } | S::FieldAssign { value, .. } => {
                find_host_call_in_expr(value, extra)
            }
            S::Choose { value, arms } => find_host_call_in_expr(value, extra).or_else(|| {
                arms.iter()
                    .find_map(|arm| find_host_call_in_stmts(&arm.body, extra))
            }),
            S::Read { .. } | S::LoopControl { .. } => None,
            S::Call { arguments, .. } | S::Write(arguments) => arguments
                .iter()
                .find_map(|item| find_host_call_in_expr(item, extra)),
            S::HostCall {
                module,
                name,
                arguments,
            } => {
                if !matches!(
                    module.as_str(),
                    "Matematica"
                        | "Texto"
                        | "Tipos"
                        | "Util"
                        | "Calendario"
                        | "Arquivos"
                        | "Json"
                        | "Internet"
                ) && !extra.contains(&module.as_str())
                {
                    Some((module.as_str(), name.as_str(), stmt.span))
                } else {
                    arguments
                        .iter()
                        .find_map(|item| find_host_call_in_expr(item, extra))
                }
            }
            S::If {
                condition,
                then_branch,
                else_branch,
            } => find_host_call_in_expr(condition, extra)
                .or_else(|| find_host_call_in_stmts(then_branch, extra))
                .or_else(|| {
                    else_branch
                        .as_deref()
                        .and_then(|item| find_host_call_in_stmts(item, extra))
                }),
            S::While { condition, body } | S::DoWhile { body, condition } => {
                find_host_call_in_expr(condition, extra)
                    .or_else(|| find_host_call_in_stmts(body, extra))
            }
            S::For {
                initializer,
                condition,
                step,
                body,
            } => find_host_call_in_stmts(std::slice::from_ref(initializer.as_ref()), extra)
                .or_else(|| find_host_call_in_expr(condition, extra))
                .or_else(|| find_host_call_in_stmts(std::slice::from_ref(step.as_ref()), extra))
                .or_else(|| find_host_call_in_stmts(body, extra)),
            S::Return { value } => value
                .as_ref()
                .and_then(|item| find_host_call_in_expr(item, extra)),
            S::IndexAssign { index, value, .. } => find_host_call_in_expr(index, extra)
                .or_else(|| find_host_call_in_expr(value, extra)),
            S::VectorPush { value, .. } => find_host_call_in_expr(value, extra),
            S::VectorPop { .. } => None,
        };
        if found.is_some() {
            return found;
        }
    }
    None
}

fn find_host_call_in_expr<'a>(
    expr: &'a ferro_hir::Expr,
    extra: &[&str],
) -> Option<(&'a str, &'a str, ferro_hir::Span)> {
    use ferro_hir::ExprKind as E;
    match &expr.kind {
        E::HostCall {
            module,
            name,
            arguments,
        } => {
            if !matches!(
                module.as_str(),
                "Matematica"
                    | "Texto"
                    | "Tipos"
                    | "Util"
                    | "Calendario"
                    | "Arquivos"
                    | "Json"
                    | "Internet"
            ) && !extra.contains(&module.as_str())
            {
                Some((module.as_str(), name.as_str(), expr.span))
            } else {
                arguments
                    .iter()
                    .find_map(|item| find_host_call_in_expr(item, extra))
            }
        }
        E::Call { arguments, .. }
        | E::VectorLiteral(arguments)
        | E::RecordLiteral {
            fields: arguments, ..
        }
        | E::EnumConstructor {
            payload: arguments, ..
        } => arguments
            .iter()
            .find_map(|item| find_host_call_in_expr(item, extra)),
        E::Field { value, .. }
        | E::Index { index: value, .. }
        | E::VectorGet { index: value, .. }
        | E::Parenthesized(value)
        | E::Try { operand: value }
        | E::Unary { operand: value, .. } => find_host_call_in_expr(value, extra),
        E::Binary { left, right, .. } => {
            find_host_call_in_expr(left, extra).or_else(|| find_host_call_in_expr(right, extra))
        }
        E::InterpolatedString { parts } => parts.iter().find_map(|part| match part {
            ferro_hir::StringPart::Literal(_, _) => None,
            ferro_hir::StringPart::Expression(value, _) => find_host_call_in_expr(value, extra),
        }),
        E::Read { .. } => None,
        E::Binding(_)
        | E::Length { .. }
        | E::VectorPop { .. }
        | E::Integer(_)
        | E::Real(_)
        | E::Boolean(_)
        | E::String(_)
        | E::Character(_) => None,
    }
}

fn rust_ident(name: &str) -> String {
    format!("pt_{name}")
}

fn default_value(ty: Ty) -> &'static str {
    match ty {
        Ty::Inteiro => "0_i32",
        Ty::Real => "0.0_f64",
        Ty::Logico => "false",
        Ty::Cadeia => "String::new()",
        Ty::Caracter => "'\\0'",
    }
}

fn format_real(value: f64) -> String {
    if value.is_nan() {
        return "f64::NAN".to_owned();
    }
    if value == f64::INFINITY {
        return "f64::INFINITY".to_owned();
    }
    if value == f64::NEG_INFINITY {
        return "f64::NEG_INFINITY".to_owned();
    }
    let value = value.to_string();
    if value.contains(['.', 'e', 'E']) {
        value
    } else {
        format!("{value}.0")
    }
}

const JAVA_DOUBLE_HELPER: &str = r#"fn __ferro_java_double(value: f64) -> String {
    if value.is_nan() { return "NaN".to_owned(); }
    if value.is_infinite() { return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_owned(); }
    if value == 0.0 { return if value.is_sign_negative() { "-0.0" } else { "0.0" }.to_owned(); }
    let sci = format!("{:e}", value.abs());
    let (mantissa, exponent) = sci.split_once('e').expect("{:e} format");
    let exponent: i32 = exponent.parse().expect("exponent");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();
    let mut out = String::new();
    if value < 0.0 { out.push('-'); }
    let magnitude = value.abs();
    if (1e-3..1e7).contains(&magnitude) {
        if exponent >= 0 {
            let int_len = exponent as usize + 1;
            if digits.len() <= int_len {
                out.push_str(&digits);
                out.extend(std::iter::repeat_n('0', int_len - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..int_len]);
                out.push('.');
                out.push_str(&digits[int_len..]);
            }
        } else {
            out.push_str("0.");
            out.extend(std::iter::repeat_n('0', (-exponent - 1) as usize));
            out.push_str(&digits);
        }
    } else {
        out.push_str(&digits[..1]); out.push('.');
        if digits.len() > 1 { out.push_str(&digits[1..]); } else { out.push('0'); }
        out.push_str(&format!("E{exponent}"));
    }
    out
}"#;

#[cfg(test)]
mod tests {
    use super::{SourceMapQuality, SourceRange, export_ferro_rs, export_ferro_rs_with_map};
    use std::process::Command;

    #[test]
    fn exports_ferro_rs_to_rust_and_executes_with_expected_output() {
        let source =
            "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }";
        assert_rust_output(export_ferro_rs(source).unwrap(), "17");
    }

    #[test]
    fn exports_and_executes_ferro_rs_while_with_source_maps() {
        let source = "funcao inicio() { seja mutavel i: inteiro = 0; enquanto (i < 3) { escreva(i); i += 1; } }";
        let generated = export_ferro_rs_with_map(source).unwrap();
        assert!(generated.code.contains("while (pt_i) < (3_i32) {"));
        for snippet in ["enquanto (i < 3)", "i < 3", "escreva(i)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing mapping for {snippet}"
            );
        }
        let source_start = source.find("enquanto (").unwrap() as u32;
        let mapping = generated
            .mappings
            .iter()
            .find(|mapping| {
                mapping
                    .source
                    .is_some_and(|range| range.start == source_start)
                    && mapping.quality == SourceMapQuality::Exact
            })
            .expect("enquanto deve ter um source map exato");
        assert!(
            generated.code[mapping.generated.start as usize..mapping.generated.end as usize]
                .trim_start()
                .starts_with("while ")
        );
        assert_rust_output(generated.code, "012");
    }

    #[test]
    fn exports_and_executes_ferro_rs_for_with_source_maps() {
        let source =
            "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { escreva(i); } }";
        let generated = export_ferro_rs_with_map(source).unwrap();
        assert!(
            generated
                .code
                .contains("pt_i = (pt_i).wrapping_add(1_i32);")
        );
        for snippet in ["seja mutavel i", "i < 3", "i += 1", "escreva(i)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        let source_start = source.find("para (").unwrap() as u32;
        let mapping = generated
            .mappings
            .iter()
            .find(|mapping| {
                mapping
                    .source
                    .is_some_and(|range| range.start == source_start)
                    && mapping.quality == SourceMapQuality::Exact
            })
            .expect("para deve ter um source map exato");
        let generated_loop = generated.code
            [mapping.generated.start as usize..mapping.generated.end as usize]
            .trim_start();
        assert!(generated_loop.starts_with("{\n"));
        assert!(generated_loop.contains("'ferro_for_"));
        assert!(generated_loop.contains(": while "));
        assert_rust_output(generated.code, "012");
    }

    #[test]
    fn exports_and_executes_ferro_rs_loop_control_with_source_maps() {
        let source = "funcao inicio() { seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 5; i += 1) { visitas += 1; se (visitas > 8) { pare; } se (i == 1) { continue; } se (i == 4) { pare; } escreva(i); } }";
        let generated = export_ferro_rs_with_map(source).unwrap();
        let loop_label = source.find("para").unwrap();
        for (keyword, generated_control) in [
            ("continue;", format!("break 'ferro_body_{loop_label};")),
            ("pare;", format!("break 'ferro_for_{loop_label};")),
        ] {
            let start = source.find(keyword).unwrap() as u32;
            let end = start + keyword.len() as u32;
            let control_mapping = generated
                .mappings
                .iter()
                .find(|mapping| mapping.source == Some(SourceRange { start, end }))
                .unwrap_or_else(|| panic!("missing full source mapping for {keyword}"));
            assert_eq!(control_mapping.quality, SourceMapQuality::Exact);
            let mapped_rust = &generated.code
                [control_mapping.generated.start as usize..control_mapping.generated.end as usize];
            assert_eq!(mapped_rust.trim(), generated_control);
        }
        let step_start = source.find("i += 1)").unwrap() as u32;
        let step_end = step_start + "i += 1".len() as u32;
        let step_mappings: Vec<_> = generated
            .mappings
            .iter()
            .filter(|mapping| {
                mapping.source
                    == Some(SourceRange {
                        start: step_start,
                        end: step_end,
                    })
            })
            .collect();
        assert_eq!(
            step_mappings.len(),
            1,
            "the for step has one source mapping"
        );
        let step_mapping = step_mappings[0];
        assert_eq!(step_mapping.quality, SourceMapQuality::Exact);
        let mapped_step = &generated.code
            [step_mapping.generated.start as usize..step_mapping.generated.end as usize];
        assert_eq!(mapped_step.trim(), "pt_i = (pt_i).wrapping_add(1_i32);");
        assert_rust_output(generated.code, "023");

        let while_source = "funcao inicio() { seja mutavel i: inteiro = 0; enquanto (i < 5) { i += 1; se (i == 2) { continue; } se (i == 4) { pare; } escreva(i); } }";
        assert_rust_output(export_ferro_rs(while_source).unwrap(), "13");

        let nested_source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { seja mutavel j: inteiro = 0; enquanto (j < 3) { j += 1; se (j == 1) { continue; } se (j == 2) { pare; } escreva(j); } escreva(i); } }";
        assert_rust_output(export_ferro_rs(nested_source).unwrap(), "012");
    }

    #[test]
    fn counted_loop_continue_uses_header_bindings_when_body_shadows_names() {
        let source = "funcao inicio() { seja mutavel passo: inteiro = 1; seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 4; i += passo) { visitas += 1; se (visitas > 4) { pare; } escreva(i); seja passo: inteiro = 2; continue; } }";
        assert_rust_output(export_ferro_rs(source).unwrap(), "0123");

        let source = "funcao inicio() { seja mutavel visitas: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 4; i += 1) { visitas += 1; se (visitas > 4) { pare; } escreva(i); seja i: inteiro = 9; continue; } }";
        assert_rust_output(export_ferro_rs(source).unwrap(), "0123");
    }

    #[test]
    fn counted_loop_keeps_utf8_source_maps_in_bounds_and_executes_nested_shadowing() {
        let source = "// comentário antes do loop: café 😃\nfuncao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { escreva(\"é\", i); } escreva(i); } }";
        let generated = export_ferro_rs_with_map(source).unwrap();
        assert!(generated.mappings.iter().all(|mapping| {
            mapping.generated.start < mapping.generated.end
                && mapping.generated.end as usize <= generated.code.len()
                && generated
                    .code
                    .is_char_boundary(mapping.generated.start as usize)
                && generated
                    .code
                    .is_char_boundary(mapping.generated.end as usize)
                && mapping.source.is_none_or(|range| {
                    range.start < range.end
                        && range.end as usize <= source.len()
                        && source.is_char_boundary(range.start as usize)
                        && source.is_char_boundary(range.end as usize)
                })
        }));
        for snippet in [
            "para (",
            "seja mutavel i",
            "i < 2",
            "i += 1",
            "escreva(\"é\", i)",
        ] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        assert!(
            generated
                .mappings
                .iter()
                .filter(|mapping| mapping.quality == SourceMapQuality::GeneratedOnly)
                .all(|mapping| mapping.source.is_none())
        );
        assert_rust_output(generated.code, "é0é10é0é11");
    }

    #[test]
    fn counted_loop_supports_real_control_values() {
        let source = "funcao inicio() { para (seja mutavel i: real = 0.0; i < 1.5; i += 0.5) { escreva(i); } }";
        let generated = export_ferro_rs(source).unwrap();
        assert_rust_output(generated, "0.00.51.0");
    }

    #[test]
    fn adds_string_budget_to_ferro_rs_codegen() {
        let ferro = export_ferro_rs("funcao inicio() { escreva(\"olá\"); }").unwrap();
        assert!(ferro.contains("__ferro_account"));
    }

    #[test]
    fn maps_generated_rust_declarations_expressions_and_nested_statements_to_source() {
        let source = "funcao inicio() { seja mutavel idade: inteiro = 16; se (verdadeiro) { escreva(idade); } }";
        let generated = export_ferro_rs_with_map(source).unwrap();

        assert!(generated.code.contains("let mut pt_idade: i32 = 16_i32;"));
        for snippet in ["seja mutavel", "verdadeiro", "escreva(idade)"] {
            let start = source.find(snippet).unwrap() as u32;
            assert!(
                generated.mappings.iter().any(|mapping| {
                    mapping.source.is_some_and(|range| range.start == start)
                        && mapping.quality == SourceMapQuality::Exact
                }),
                "missing exact source mapping for {snippet}"
            );
        }
        assert!(generated.mappings.iter().all(|mapping| {
            mapping.generated.start < mapping.generated.end
                && mapping.generated.end as usize <= generated.code.len()
                && mapping.source.is_none_or(|range| {
                    range.start < range.end && range.end as usize <= source.len()
                })
        }));
        assert!(generated.mappings.windows(2).all(|pair| {
            (pair[0].generated.start, pair[0].generated.end)
                <= (pair[1].generated.start, pair[1].generated.end)
        }));
    }

    #[test]
    fn marks_rust_prelude_as_generated_only() {
        let source = "funcao inicio() { escreva(1); }";
        let generated = export_ferro_rs_with_map(source).unwrap();
        let prelude = generated.code.find("use std::io").unwrap() as u32;

        assert!(generated.mappings.iter().any(|mapping| {
            mapping.generated.start <= prelude
                && prelude < mapping.generated.end
                && mapping.source.is_none()
                && mapping.quality == SourceMapQuality::GeneratedOnly
        }));
    }

    #[test]
    fn rejects_ferro_rs_assignment_to_immutable_binding() {
        let source = "funcao inicio() { seja idade: inteiro = 16; idade = 17; }";
        let errors = export_ferro_rs(source).unwrap_err();
        assert!(
            errors
                .iter()
                .any(|error| error.code == "ferro.immutable-assignment")
        );
    }

    #[test]
    fn exports_and_executes_builtin_option_result_and_try_as_native_rust() {
        let source = r#"
registro Item { valor: inteiro }

funcao buscar(n: inteiro) -> Opcao<Item> {
    se (n > 0) {
        retorne Opcao::Algum(Item { valor: n * 10 });
    }
    retorne Opcao::Nenhum;
}

funcao duplicar_opcional(opt: Opcao<inteiro>) -> Opcao<inteiro> {
    seja a: inteiro = opt?;
    seja b: inteiro = opt?;
    retorne Opcao::Algum(a + b);
}

funcao somar_com_dez(n: inteiro) -> Opcao<inteiro> {
    retorne Opcao::Algum(10 + buscar(n)?.valor);
}

funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao rotular_divisao(a: inteiro, b: inteiro) -> Resultado<cadeia, cadeia> {
    seja total: inteiro = 1 + dividir(a, b)?;
    se (total >= 5) {
        retorne Resultado::Ok("alto");
    }
    retorne Resultado::Ok("baixo");
}

funcao inicio() {
    seja lista: vetor<Opcao<inteiro>> = [Opcao::Algum(7), Opcao::Nenhum];
    escolha duplicar_opcional(lista[0]) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha somar_com_dez(3) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha somar_com_dez(0) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha rotular_divisao(20, 4) {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("E:", e, "|"); }
    }
    escolha rotular_divisao(20, 0) {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva("E:", e); }
    }
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("Option<pt_Item>"));
        assert!(generated.contains("Option<i32>"));
        assert!(generated.contains("Result<i32, String>"));
        assert!(generated.contains("Result<String, String>"));
        assert!(generated.contains("Option::Some("));
        assert!(generated.contains("Option::None"));
        assert!(generated.contains("Result::Ok("));
        assert!(generated.contains("Result::Err("));
        assert!(generated.contains(")?"));
        assert!(!generated.contains("enum pt_Opcao"));
        assert!(!generated.contains("enum pt_Resultado"));
        assert_rust_output(generated, "14|40|N|alto|E:zero");
    }

    #[test]
    fn exports_and_executes_ferro_matematica_texto_and_tipos_with_identical_semantics() {
        let source = r#"
funcao somar_entradas(a: cadeia, b: cadeia) -> Resultado<inteiro, cadeia> {
    seja x: inteiro = Tipos.para_inteiro(Texto.aparar(a))?;
    seja y: inteiro = Tipos.para_inteiro_base(b, 16)?;
    retorne Resultado::Ok(x + y);
}

funcao inicio() {
    seja pi2: real = Matematica.arredondar(Matematica.PI, 2);
    seja pot: real = Matematica.potencia(2.0, 3.0);
    seja raiz: real = Matematica.raiz_quadrada(81.0);
    seja abs_i: inteiro = Matematica.valor_absoluto_inteiro(-7);
    seja max_i: inteiro = Matematica.maior_inteiro(3, 9);
    escreva(pi2, "|", pot, "|", raiz, "|", abs_i, "|", max_i, "|");

    seja frase: cadeia = "  olá 🦀  ";
    seja limpa: cadeia = Texto.aparar(frase);
    seja alta: cadeia = Texto.caixa_alta(limpa);
    seja trocada: cadeia = Texto.substituir(limpa, "olá", "oi");
    seja pad: cadeia = Texto.preencher_a_esquerda('0', 5, "42");
    escreva(Texto.numero_caracteres(limpa), "|", alta, "|", trocada, "|", pad, "|");

    escolha Texto.obter_caracter(limpa, 4) {
        caso Opcao::Algum(c) => { escreva(c, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha Texto.posicao_texto("🦀", limpa, 0) {
        caso Opcao::Algum(pos) => { escreva(pos, "|"); }
        caso Opcao::Nenhum => { escreva("-1|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 0, 3) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 2, 99) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }

    escolha somar_entradas(" 26 ", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha somar_entradas("abc", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.para_real("2.5") {
        caso Resultado::Ok(r) => { escreva(Tipos.real_para_cadeia(r), "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.inteiro_para_cadeia_base(255, 16) {
        caso Resultado::Ok(hex) => { escreva(hex, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Tipos.cadeia_e_inteiro("123"), "|", Tipos.cadeia_e_logico("verdadeiro"));
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("mod __ferro_matematica"));
        assert!(generated.contains("mod __ferro_texto"));
        assert!(generated.contains("mod __ferro_tipos"));
        assert_rust_output(
            generated,
            "3.14|8.0|9.0|7|9|5|OLÁ 🦀|oi 🦀|00042|🦀|4|olá|ERR|42|ERR|2.5|FF|verdadeiro|verdadeiro",
        );

        let minimal = export_ferro_rs("funcao inicio() { escreva(1); }").unwrap();
        assert!(!minimal.contains("mod __ferro_matematica"));
        assert!(!minimal.contains("mod __ferro_texto"));
        assert!(!minimal.contains("mod __ferro_tipos"));
    }

    #[test]
    fn exports_and_executes_dynamic_vectors_and_texto_dividir_as_native_rust() {
        let source = r#"
registro Ponto { x: inteiro }

funcao somar_topo_e_primeiro(mutavel itens: vetor<inteiro>) -> Opcao<inteiro> {
    seja topo: inteiro = remover(itens)?;
    seja primeiro: inteiro = obter(itens, 0)?;
    retorne Opcao::Algum(topo + primeiro);
}

funcao inicio() {
    seja mutavel nums: vetor<inteiro> = [];
    escreva(tamanho(nums), "|");
    adicionar(nums, 5);
    adicionar(nums, 15);
    adicionar(nums, 25);
    escreva(tamanho(nums), "|");
    escolha somar_topo_e_primeiro(nums) {
        caso Opcao::Algum(s) => { escreva(s, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    remover(nums);
    escreva(tamanho(nums), "|");
    escolha obter(nums, -1) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha obter(nums, 99) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja mutavel pontos: vetor<Ponto> = [];
    adicionar(pontos, Ponto { x: 7 });
    escolha obter(pontos, 0) {
        caso Opcao::Algum(p) => { escreva(p.x, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja partes: vetor<cadeia> = Texto.dividir("um,dois,três", ",");
    escreva(tamanho(partes), "|", partes[1], "|");
    seja letras: vetor<cadeia> = Texto.dividir("a🦀", "");
    escreva(tamanho(letras), "|", letras[1]);
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("Vec::<i32>::new()"));
        assert!(generated.contains(".push("));
        assert!(generated.contains(".pop()"));
        assert!(generated.contains(".get("));
        assert!(generated.contains("__ferro_texto::dividir("));
        assert_rust_output(generated, "0|3|30|2|N|N|7|3|dois|2|🦀");
    }

    #[test]
    fn exports_and_executes_ferro_util_and_calendario_deterministically() {
        let source = r#"
funcao inicio() {
    Util.definir_semente(42);
    escolha Util.sorteia(1, 100) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Util.sorteia(10, 5) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Util.tempo_decorrido(), "|");
    Util.aguarde(1500);
    escreva(Util.tempo_decorrido(), "|");

    escreva(
        Calendario.data_iso_atual(), "|",
        Calendario.ano_atual(), "-",
        Calendario.mes_atual(), "-",
        Calendario.dia_mes_atual(), "|",
        Calendario.dia_semana_atual(), "|",
        Calendario.hora_atual(falso), ":",
        Calendario.hora_atual(verdadeiro), ":",
        Calendario.minuto_atual(), ":",
        Calendario.segundo_atual(), ".",
        Calendario.milisegundo_atual(), "|",
        Calendario.eh_bissexto(2024), "|",
        Calendario.DIA_TERCA_FEIRA, "|",
        Calendario.MES_NOVEMBRO, "|"
    );
    escolha Calendario.dia_semana_completo(Calendario.dia_semana_atual(), falso, falso) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.nome_mes(Calendario.mes_atual()) {
        caso Resultado::Ok(m) => { escreva(m, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, Calendario.MES_FEVEREIRO) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, 13) {
        caso Resultado::Ok(d) => { escreva(d); }
        caso Resultado::Erro(_) => { escreva("ERR"); }
    }
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("mod __ferro_clock"));
        assert!(generated.contains("mod __ferro_util"));
        assert!(generated.contains("mod __ferro_calendario"));

        let dir = std::env::temp_dir().join(format!("ferro-rust-util-cal-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary)
            .env("FERRO_SIMULATED_EPOCH_MS", "1700000000500")
            .output()
            .unwrap();
        assert!(output.status.success());
        let stdout = String::from_utf8(output.stdout).unwrap();
        assert!(
            stdout.ends_with(
                "|ERR|0|1500|2023-11-14|2023-11-14|3|22:10:13:22.0|verdadeiro|3|11|Terça-Feira|Novembro|29|ERR"
            ),
            "saída inesperada: {stdout}"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_workspace_confined_ferro_arquivos_in_native_rust() {
        let source = r#"
funcao ciclo() -> Resultado<cadeia, cadeia> {
    seja _w: logico = Arquivos.escrever_texto("sub/notas.txt", "alpha\n")?;
    seja _a: logico = Arquivos.acrescentar_texto("sub/notas.txt", "beta\n")?;
    seja tem: logico = Arquivos.existe("sub/notas.txt")?;
    seja linhas: vetor<cadeia> = Arquivos.ler_linhas("sub/notas.txt")?;
    seja itens: vetor<cadeia> = Arquivos.listar("sub")?;
    seja _r: logico = Arquivos.remover("sub/notas.txt")?;
    seja depois: logico = Arquivos.existe("sub/notas.txt")?;
    retorne Resultado::Ok(
        Tipos.logico_para_cadeia(tem) + "|" +
        Tipos.inteiro_para_cadeia(tamanho(linhas)) + "|" +
        linhas[1] + "|" +
        itens[0] + "|" +
        Tipos.logico_para_cadeia(depois)
    );
}

funcao inicio() {
    escolha ciclo() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escolha Arquivos.ler_texto("../fora.txt") {
        caso Resultado::Ok(_) => { escreva("LEAK|"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_PARENT|"); }
    }
    escolha Arquivos.ler_texto("/etc/passwd") {
        caso Resultado::Ok(_) => { escreva("LEAK"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_ABS"); }
    }
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("mod __ferro_arquivos"));

        let dir = std::env::temp_dir().join(format!("ferro-rust-arquivos-{}", std::process::id()));
        let ws = dir.join("workspace");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&ws).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary)
            .env("FERRO_WORKSPACE_DIR", &ws)
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap(),
            "verdadeiro|2|beta|notas.txt|falso|BLOCKED_PARENT|BLOCKED_ABS"
        );
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_zero_dep_ferro_json_in_native_rust() {
        let source = r#"
funcao executar() -> Resultado<cadeia, cadeia> {
    seja mutavel doc: cadeia = "{}";
    doc = Json.definir_cadeia(doc, "nome", "Ada \"Lovelace\"\n")?;
    doc = Json.definir_inteiro(doc, "ano", 1843)?;
    doc = Json.definir_real(doc, "nota", 9.5)?;
    doc = Json.definir_logico(doc, "ativo", verdadeiro)?;
    doc = Json.definir_json(doc, "tags", "[\"ferro\",\"rust\"]")?;
    doc = Json.definir_json(doc, "nums", "[10, 20, 30]")?;
    doc = Json.definir_json(doc, "meta", "{\"emoji\":\"\\uD83D\\uDE00\"}")?;

    seja nome: cadeia = Json.obter_cadeia(doc, "nome")?;
    seja ano: inteiro = Json.obter_inteiro(doc, "ano")?;
    seja nota: real = Json.obter_real(doc, "nota")?;
    seja ativo: logico = Json.obter_logico(doc, "ativo")?;
    seja tags: vetor<cadeia> = Json.obter_lista_cadeia(doc, "tags")?;
    seja nums: vetor<inteiro> = Json.obter_lista_inteiro(doc, "nums")?;
    seja meta: cadeia = Json.obter_json(doc, "meta")?;
    seja emoji: cadeia = Json.obter_cadeia(meta, "emoji")?;
    seja chaves: vetor<cadeia> = Json.obter_chaves(doc)?;

    seja sem_ativo: cadeia = Json.remover_chave(doc, "ativo")?;
    seja tem_ativo: logico = Json.contem_chave(sem_ativo, "ativo")?;
    seja bonito: cadeia = Json.formatar("{\"a\":1,\"b\":[true,null]}")?;
    seja compacto: cadeia = Json.minificar(bonito)?;

    retorne Resultado::Ok(
        Texto.aparar(nome) + "|" +
        Tipos.inteiro_para_cadeia(ano) + "|" +
        Tipos.real_para_cadeia(nota) + "|" +
        Tipos.logico_para_cadeia(ativo) + "|" +
        tags[1] + "|" +
        Tipos.inteiro_para_cadeia(nums[0] + nums[2]) + "|" +
        emoji + "|" +
        Tipos.inteiro_para_cadeia(tamanho(chaves)) + "|" +
        Tipos.logico_para_cadeia(tem_ativo) + "|" +
        compacto + "|" +
        Json.escapar_cadeia("a\"b")
    );
}

funcao inicio() {
    escolha executar() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escreva(Json.valido("{\"ok\":1}"), "|", Json.valido("{invalido}"), "|");
    escolha Json.obter_inteiro("{\"x\":3.5}", "x") {
        caso Resultado::Ok(_) => { escreva("BAD_INT|"); }
        caso Resultado::Erro(_) => { escreva("INT_ERR|"); }
    }
    seja mutavel fundo: cadeia = "0";
    para (seja mutavel i: inteiro = 0; i < 70; i += 1) {
        fundo = "[" + fundo + "]";
    }
    escreva(Json.valido(fundo));
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("mod __ferro_json"));
        assert_rust_output(
            generated,
            "Ada \"Lovelace\"|1843|9.5|verdadeiro|rust|40|\u{1F600}|7|falso|{\"a\":1,\"b\":[true,null]}|\"a\\\"b\"|verdadeiro|falso|INT_ERR|falso",
        );
    }

    #[test]
    fn exports_and_executes_bitwise_shifts_and_choose_wildcard_in_native_rust() {
        let source = r#"
enum Cor {
    Vermelho,
    Verde,
    Azul
}

funcao rotulo(c: Cor) -> cadeia {
    escolha c {
        caso Cor::Vermelho => { retorne "R"; }
        caso _ => { retorne "OUTRA"; }
    }
}

funcao inicio() {
    seja mutavel bits: inteiro = (1 << 4) | 5;
    bits &= 21;
    bits |= 2;
    bits ^= 1;
    seja shifted: inteiro = bits >> 1;
    seja not_zero: inteiro = ~0;
    escreva(
        bits, "|",
        shifted, "|",
        not_zero, "|",
        rotulo(Cor::Vermelho), "|",
        rotulo(Cor::Azul)
    );
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert!(generated.contains("wrapping_shl"));
        assert!(generated.contains("wrapping_shr"));
        assert!(generated.contains("_ =>"));
        assert_rust_output(generated, "22|11|-1|R|OUTRA");
    }

    #[test]
    fn exports_and_executes_consented_ferro_internet_in_native_rust() {
        use std::io::{Read, Write};
        use std::net::TcpListener;
        use std::thread;
        use std::time::Duration;

        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let addr = listener.local_addr().unwrap();
        let server = thread::spawn(move || {
            for _ in 0..3 {
                let Ok((mut stream, _)) = listener.accept() else {
                    break;
                };
                let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                let mut raw = Vec::new();
                let mut chunk = [0u8; 1024];
                let header_end = loop {
                    if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                        break pos;
                    }
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break raw.len(),
                        Ok(n) => raw.extend_from_slice(&chunk[..n]),
                    }
                };
                let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
                let content_len: usize = head
                    .lines()
                    .find_map(|line| {
                        let (k, v) = line.split_once(':')?;
                        k.eq_ignore_ascii_case("content-length")
                            .then(|| v.trim().parse().ok())
                            .flatten()
                    })
                    .unwrap_or(0);
                let body_start = (header_end + 4).min(raw.len());
                while raw.len().saturating_sub(body_start) < content_len {
                    match stream.read(&mut chunk) {
                        Ok(0) | Err(_) => break,
                        Ok(n) => raw.extend_from_slice(&chunk[..n]),
                    }
                }
                let first = head.lines().next().unwrap_or("");
                if first.starts_with("GET /ola ") {
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nOla Rust!",
                    );
                } else if first.starts_with("POST /eco ") {
                    let body = &raw[body_start..raw.len().min(body_start + content_len)];
                    let resp = format!(
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        body.len(),
                        String::from_utf8_lossy(body)
                    );
                    let _ = stream.write_all(resp.as_bytes());
                } else if first.starts_with("HEAD /ola ") {
                    let _ = stream.write_all(
                        b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\n",
                    );
                }
            }
        });

        let base = format!("http://{addr}");
        let source = format!(
            r#"
funcao executar() -> Resultado<cadeia, cadeia> {{
    Internet.definir_tempo_limite(2000);
    seja ok: logico = Internet.endereco_disponivel("{base}/ola")?;
    seja texto: cadeia = Internet.obter_texto("{base}/ola")?;
    seja eco: cadeia = Internet.enviar_json("{base}/eco", "{{\"ok\":1}}")?;
    retorne Resultado::Ok(Tipos.logico_para_cadeia(ok) + "|" + texto + "|" + eco);
}}

funcao inicio() {{
    escolha executar() {{
        caso Resultado::Ok(s) => {{ escreva(s); }}
        caso Resultado::Erro(e) => {{ escreva("ERR:", e); }}
    }}
}}
"#
        );
        let generated = export_ferro_rs(&source).unwrap();
        assert!(generated.contains("mod __ferro_internet"));

        let dir = std::env::temp_dir().join(format!("ferro-rust-internet-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let source_path = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source_path, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source_path)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );

        // Sem FERRO_ALLOW_NETWORK=1 deve recusar acesso.
        let denied = Command::new(&binary)
            .env_remove("FERRO_ALLOW_NETWORK")
            .output()
            .unwrap();
        assert!(denied.status.success());
        assert!(
            String::from_utf8(denied.stdout)
                .unwrap()
                .starts_with("ERR:Acesso à rede bloqueado")
        );

        // Com FERRO_ALLOW_NETWORK=1 executa GET, POST e HEAD localmente.
        let allowed = Command::new(&binary)
            .env("FERRO_ALLOW_NETWORK", "1")
            .output()
            .unwrap();
        assert!(allowed.status.success());
        assert_eq!(
            String::from_utf8(allowed.stdout).unwrap(),
            "verdadeiro|Ola Rust!|{\"ok\":1}"
        );
        let _ = server.join();
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn exports_and_executes_do_while_with_break_and_continue_in_native_rust() {
        let source = r#"
funcao inicio() {
    seja mutavel vezes: inteiro = 0;
    faca {
        vezes += 1;
    } enquanto (falso);

    seja mutavel i: inteiro = 0;
    seja mutavel soma: inteiro = 0;
    faca {
        i += 1;
        se (i == 2) {
            continue;
        }
        soma += i;
        se (i == 4) {
            pare;
        }
    } enquanto (i < 10);

    escreva(vezes, "|", i, "|", soma);
}
"#;
        let generated = export_ferro_rs(source).unwrap();
        assert_rust_output(generated, "1|4|8");
    }

    fn assert_rust_output(generated: String, expected: &str) {
        use std::sync::atomic::{AtomicU64, Ordering};
        static NEXT_TEST_DIR: AtomicU64 = AtomicU64::new(0);
        let dir = std::env::temp_dir().join(format!(
            "ferro-rust-{}-{}",
            std::process::id(),
            NEXT_TEST_DIR.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let source = dir.join("main.rs");
        let binary = dir.join(format!("program{}", std::env::consts::EXE_SUFFIX));
        std::fs::write(&source, generated).unwrap();
        let compilation = Command::new("rustc")
            .arg("--edition=2024")
            .arg(&source)
            .arg("-o")
            .arg(&binary)
            .output()
            .unwrap();
        assert!(
            compilation.status.success(),
            "{}",
            String::from_utf8_lossy(&compilation.stderr)
        );
        let output = Command::new(&binary).output().unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap(), expected);
        std::fs::remove_dir_all(dir).unwrap();
    }
}
