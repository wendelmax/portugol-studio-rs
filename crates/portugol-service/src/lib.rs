//! Fachada do núcleo Portugol-RS para a IDE.
//!
//! * [`inspect_portugol_rs`]: análise sintática e semântica com posições UTF-16 para o editor;
//! * [`start_portugol_rs`]: compila e executa/depura o programa na VM numa thread própria,
//!   entregando saída, pedidos de `leia` e o resultado a um [`RunObserver`];
//! * [`run_portugol_rs`] / [`run_rust_source`]: execução via `rustc` / `cargo`.
//!
//! Não depende do Tauri: a camada da janela só traduz os eventos.

mod cargo_project;
mod designer;
mod markup;
mod outline;
mod pors_examples;
mod pors_exercises;
mod pors_run;
mod position;
mod rename;
mod run;
mod ui_session;

use serde::Serialize;

use portugol_hir::{Diagnostic, Severity};
use portugol_rust::{GeneratedRust, SourceMapQuality as RustSourceMapQuality};

pub use cargo_project::{
    CargoProjectCheckResult, CargoProjectDiagnostic, CargoProjectRunResult, CargoProjectTestResult,
    CargoRunTarget, CargoRunTargets, cargo_available, cargo_run_targets_with_cancel,
    check_cargo_project_with_cancel, run_cargo_project_target_with_args_and_cancel,
    run_cargo_project_target_with_cancel, run_cargo_project_with_cancel,
    test_cargo_project_with_cancel,
};
pub use designer::{
    AttributeInfo, DesignAttribute, DesignEdit, DesignNode, DesignSchema, ElementInfo,
    design_schema, design_tree, edit_design,
};
pub use markup::{MarkupKind, format_markup, inspect_markup};
pub use outline::{SymbolInfo, SymbolKind};
pub use pors_examples::{PorsExample, find_pors_example, pors_examples};
pub use pors_exercises::{
    PORS_EXERCISE_SCHEMA_VERSION, PorsExerciseCase, PorsExerciseCaseOutcome,
    PorsExerciseDefinition, PorsExerciseEvaluation, PorsExerciseStatus, PorsExerciseSummary,
    check_imported_pors_exercise, check_pors_exercise, evaluate_pors_exercise,
    inspect_pors_exercise, parse_pors_exercise_json, pors_exercises,
};
pub use pors_run::{
    PortugolRsRunPhase, PortugolRsRunResult, PortugolRsRuntimeLimitReason,
    RustSourceCompileDiagnostic, RustSourceLimitReason, RustSourceRunPhase, RustSourceRunResult,
    check_rust_source, format_rust_source, run_portugol_rs, run_portugol_rs_with_cancel,
    run_portugol_rs_with_io, run_rust_source, run_rust_source_with_args_and_cancel,
    run_rust_source_with_cancel, rust_toolchain_available, rustfmt_available,
};
pub use portugol_vm::{FileFilter, Host, Sound, graphics};
pub use position::Position;
pub use rename::{
    RenameEdit, RenameTarget, SymbolLocation, rename_symbol_portugol_rs, rename_target_portugol_rs,
    resolve_symbol_portugol_rs,
};
pub use run::{
    DebugFrameInfo, DebugInfo, DebugMappingInfo, DebugVarInfo, RunHandle, RunLimitReason,
    RunObserver, RunOptions, RunOutcome, start_portugol_rs, start_portugol_rs_with_host,
};
pub use ui_session::{PorsUiBudgetRequest, PorsUiService};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Error,
    Warning,
}

/// Diagnóstico serializável para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DiagnosticInfo {
    pub severity: Level,
    pub code: String,
    pub message: String,
    pub help: Option<String>,
    pub start: Position,
    pub end: Position,
}

#[derive(Serialize)]
pub struct TextEdit {
    from: usize,
    to: usize,
    insert: String,
}

/// Formata código Portugol-RS (`.pors`) preservando comentários e literais.
pub fn format_portugol_rs(source: &str) -> Option<Vec<TextEdit>> {
    portugol_rs_frontend::format(source).map(|edits| {
        edits
            .into_iter()
            .map(|edit| TextEdit {
                from: edit.from,
                to: edit.to,
                insert: edit.insert,
            })
            .collect()
    })
}

#[derive(Serialize)]
pub struct Inspection {
    pub diagnostics: Vec<DiagnosticInfo>,
    pub symbols: Vec<SymbolInfo>,
}

/// Resultado da prévia Portugol → Rust. Em caso de falha, `code` fica vazio e
/// `diagnostic` aponta para a construção original que impediu a tradução.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RustExport {
    pub code: Option<String>,
    pub diagnostic: Option<DiagnosticInfo>,
    pub source_map: Vec<SourceMapInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceMapQuality {
    Exact,
    Approximate,
    GeneratedOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SourceMapInfo {
    pub generated_start: Position,
    pub generated_end: Position,
    pub source_start: Option<Position>,
    pub source_end: Option<Position>,
    pub quality: SourceMapQuality,
}

fn editor_source_map(source: &str, generated: &GeneratedRust) -> Vec<SourceMapInfo> {
    let source_index = position::LineIndex::new(source);
    let generated_index = position::LineIndex::new(&generated.code);
    generated
        .mappings
        .iter()
        .map(|mapping| SourceMapInfo {
            generated_start: generated_index.position(mapping.generated.start),
            generated_end: generated_index.position(mapping.generated.end),
            source_start: mapping
                .source
                .map(|range| source_index.position(range.start)),
            source_end: mapping.source.map(|range| source_index.position(range.end)),
            quality: match mapping.quality {
                RustSourceMapQuality::Exact => SourceMapQuality::Exact,
                RustSourceMapQuality::Approximate => SourceMapQuality::Approximate,
                RustSourceMapQuality::GeneratedOnly => SourceMapQuality::GeneratedOnly,
            },
        })
        .collect()
}

/// Analisa o dialeto Portugol-RS e expõe spans no formato UTF-16 do editor.
pub fn inspect_portugol_rs(source: &str) -> Inspection {
    inspect_portugol_rs_with(source, &portugol_rs_frontend::LowerOptions::default())
}

/// Analisa o code-behind de uma tela (`Nome.fer` ao lado de `Nome.design`):
/// o registro da janela e os tipos dos controles vêm do `.design`, e o
/// arquivo não precisa de `inicio` (ADR 0018, seção 4). Com erro de sintaxe
/// no `.design`, os tipos da tela ficam de fora.
pub fn inspect_code_behind(source: &str, design_source: &str) -> Inspection {
    inspect_portugol_rs_with(source, &designer::code_behind_options(design_source))
}

fn inspect_portugol_rs_with(
    source: &str,
    options: &portugol_rs_frontend::LowerOptions,
) -> Inspection {
    let index = position::LineIndex::new(source);
    match portugol_rs_frontend::lower_with(source, options) {
        Ok(program) => {
            let mut symbols: Vec<_> = program
                .functions
                .iter()
                .map(|function| {
                    let name_start = pors_function_name_start(source, function.span.start);
                    let mut children: Vec<_> = function
                        .bindings
                        .iter()
                        .map(|binding| SymbolInfo {
                            name: binding.source_name.clone(),
                            kind: if function.parameters.contains(&binding.id) {
                                SymbolKind::Parameter
                            } else {
                                SymbolKind::Variable
                            },
                            ty: pors_type_name(binding.ty),
                            position: index.position(binding.span.start),
                            children: Vec::new(),
                        })
                        .collect();
                    children.sort_by_key(|symbol| symbol.position.offset);
                    SymbolInfo {
                        name: function.source_name.clone(),
                        kind: SymbolKind::Function,
                        ty: function.return_type.map_or("vazio", pors_type_name),
                        position: index.position(name_start),
                        children,
                    }
                })
                .collect();
            symbols.extend(program.bindings.iter().map(|binding| SymbolInfo {
                name: binding.source_name.clone(),
                // O dialeto não possui `const`: `seja` é um binding local
                // imutável, não uma constante global da linguagem clássica.
                kind: SymbolKind::Variable,
                ty: pors_type_name(binding.ty),
                position: index.position(binding.span.start),
                children: Vec::new(),
            }));
            symbols.sort_by_key(|symbol| symbol.position.offset);
            Inspection {
                diagnostics: Vec::new(),
                symbols,
            }
        }
        Err(diagnostics) => Inspection {
            diagnostics: present_diagnostics(&index, &diagnostics),
            symbols: Vec::new(),
        },
    }
}

pub(crate) fn pors_function_name_start(source: &str, function_start: u32) -> u32 {
    let start = function_start as usize;
    let tail = &source[start..];
    let mut offset = tail
        .find("funcao")
        .map_or(0, |position| position + "funcao".len());
    loop {
        let rest = &tail[offset..];
        if let Some(character) = rest.chars().next()
            && character.is_whitespace()
        {
            offset += character.len_utf8();
            continue;
        }
        if rest.starts_with("//") {
            offset += rest.find('\n').unwrap_or(rest.len());
            continue;
        }
        if rest.starts_with("/*") {
            offset += rest.find("*/").map_or(rest.len(), |end| end + 2);
            continue;
        }
        break;
    }
    start.saturating_add(offset).min(source.len()) as u32
}

fn pors_type_name(ty: portugol_hir::Type) -> &'static str {
    match ty {
        portugol_hir::Type::Integer => "inteiro",
        portugol_hir::Type::Real => "real",
        portugol_hir::Type::Boolean => "logico",
        portugol_hir::Type::String => "cadeia",
        portugol_hir::Type::Character => "caracter",
        portugol_hir::Type::Unit => "vazio",
        portugol_hir::Type::VectorInteger => "vetor<inteiro>",
        portugol_hir::Type::VectorReal => "vetor<real>",
        portugol_hir::Type::VectorBoolean => "vetor<logico>",
        portugol_hir::Type::VectorString => "vetor<cadeia>",
        portugol_hir::Type::VectorCharacter => "vetor<caracter>",
        portugol_hir::Type::Record(_) => "registro",
        portugol_hir::Type::VectorRecord(_) => "vetor<registro>",
        portugol_hir::Type::Enum(_) => "enum",
        portugol_hir::Type::VectorEnum(_) => "vetor<enum>",
    }
}

/// Traduz o dialeto Portugol-RS para Rust usando o backend compartilhado.
pub fn export_portugol_rs(source: &str) -> RustExport {
    let index = position::LineIndex::new(source);
    match portugol_rust::export_portugol_rs_with_map(source) {
        Ok(generated) => {
            let source_map = editor_source_map(source, &generated);
            RustExport {
                code: Some(generated.code),
                diagnostic: None,
                source_map,
            }
        }
        Err(diagnostics) => RustExport {
            code: None,
            diagnostic: diagnostics
                .first()
                .map(|diagnostic| describe(&index, diagnostic)),
            source_map: Vec::new(),
        },
    }
}

fn present_diagnostics(
    index: &position::LineIndex,
    diagnostics: &[Diagnostic],
) -> Vec<DiagnosticInfo> {
    diagnostics
        .iter()
        .map(|diagnostic| describe(index, diagnostic))
        .collect()
}

pub(crate) fn describe(index: &position::LineIndex, diagnostic: &Diagnostic) -> DiagnosticInfo {
    DiagnosticInfo {
        severity: match diagnostic.severity {
            Severity::Error => Level::Error,
            Severity::Warning => Level::Warning,
        },
        code: diagnostic.code.to_string(),
        message: diagnostic.message.clone(),
        help: diagnostic.help.clone(),
        start: index.position(diagnostic.span.start),
        end: index.position(diagnostic.span.end.max(diagnostic.span.start)),
    }
}

#[cfg(test)]
mod portugol_rs_service {
    use super::{export_portugol_rs, inspect_portugol_rs};

    #[test]
    fn inspection_returns_typed_symbols_for_portugol_rs() {
        let inspection = inspect_portugol_rs("funcao inicio() { seja idade: inteiro = 16; }");
        assert!(inspection.diagnostics.is_empty());
        assert_eq!(inspection.symbols[0].name, "idade");
        assert_eq!(inspection.symbols[0].ty, "inteiro");
        assert_eq!(inspection.symbols[0].kind, super::SymbolKind::Variable);
    }

    #[test]
    fn inspection_includes_helper_functions_and_parameters() {
        let source = "funcao /* helper */ somar(a: inteiro, b: inteiro) -> inteiro { seja total: inteiro = a + b; retorne total; }\nfuncao inicio() { escreva(somar(2, 3)); }";
        let inspection = inspect_portugol_rs(source);

        let function = inspection
            .symbols
            .iter()
            .find(|symbol| symbol.name == "somar")
            .expect("a função auxiliar deve aparecer no outline");
        assert_eq!(function.kind, super::SymbolKind::Function);
        assert_eq!(
            function.position.offset as usize,
            source.find("somar").unwrap()
        );
        assert_eq!(function.children.len(), 3);
        assert_eq!(function.children[0].name, "a");
        assert_eq!(function.children[0].kind, super::SymbolKind::Parameter);
        assert_eq!(function.children[2].name, "total");
        assert_eq!(function.children[2].kind, super::SymbolKind::Variable);
    }

    #[test]
    fn diagnostics_use_utf16_offsets_after_non_bmp_text() {
        let source = "// 🧠\nfuncao inicio() { seja idade: inteiro = ; }";
        let inspection = inspect_portugol_rs(source);
        let diagnostic = &inspection.diagnostics[0];
        let byte_offset = source.find(";").unwrap();
        let utf16_offset = source[..byte_offset].encode_utf16().count();
        assert_eq!(diagnostic.start.offset as usize, utf16_offset);
        assert!(diagnostic.code.starts_with("pors."));
    }

    #[test]
    fn inspection_reports_multiple_syntax_errors_and_export_rejects_source() {
        let source = "funcao primeira() { seja a: inteiro = ; }\nfuncao segunda() { seja b: inteiro = ; }\nfuncao terceira() { seja c: inteiro = ; }\nfuncao inicio() {}";
        let inspection = inspect_portugol_rs(source);
        assert_eq!(inspection.diagnostics.len(), 3);
        assert!(inspection.symbols.is_empty());

        let export = export_portugol_rs(source);
        assert!(export.code.is_none());
        assert!(export.diagnostic.is_some());
        assert!(export.source_map.is_empty());
    }

    #[test]
    fn export_uses_the_shared_rust_backend() {
        let result =
            export_portugol_rs("funcao inicio() { seja idade: inteiro = 16; escreva(idade); }");
        assert!(result.diagnostic.is_none());
        assert!(result.code.unwrap().contains("print!(\"{}\""));
    }

    #[test]
    fn export_indents_function_calls_inside_generated_blocks() {
        let result =
            export_portugol_rs("funcao mostrar() { escreva(1); }\nfuncao inicio() { mostrar(); }");
        let code = result.code.expect("PORS de funções deve ser traduzido");
        assert!(code.contains("    pt_mostrar();"), "{code}");
    }

    #[test]
    fn export_source_maps_convert_unicode_ranges_to_utf16_editor_offsets() {
        let source = "funcao inicio() { seja texto: cadeia = \"😀\"; escreva(texto); }";
        let result = export_portugol_rs(source);
        let code = result.code.as_ref().unwrap();
        let source_start_bytes = source.find("escreva").unwrap();
        let source_start_utf16 = source[..source_start_bytes].encode_utf16().count() as u32;
        let generated_start_bytes = code.find("let __portugol_write_0").unwrap();
        let generated_line_start_bytes = code[..generated_start_bytes]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        let generated_start_utf16 = code[..generated_start_bytes].encode_utf16().count() as u32;

        let mapping = result
            .source_map
            .iter()
            .find(|mapping| {
                mapping
                    .source_start
                    .is_some_and(|start| start.offset == source_start_utf16)
            })
            .expect("a instrução escreva deve mapear para seu span da origem");
        assert_eq!(
            mapping.generated_start.offset,
            code[..generated_line_start_bytes].encode_utf16().count() as u32
        );
        assert!(generated_start_utf16 > mapping.generated_start.offset);
        assert_eq!(mapping.quality, super::SourceMapQuality::Exact);
        assert!(result.source_map.iter().any(|mapping| {
            mapping.quality == super::SourceMapQuality::GeneratedOnly
                && mapping.source_start.is_none()
        }));
    }

    #[test]
    fn helper_function_source_mappings_are_not_marked_generated_only() {
        let source = "funcao dobro(valor: inteiro) -> inteiro { retorne valor * 2; }\nfuncao inicio() { escreva(dobro(3)); }";
        let result = export_portugol_rs(source);
        assert!(result.diagnostic.is_none());

        let helper_start = source.find("retorne").unwrap();
        let helper_start_utf16 = source[..helper_start].encode_utf16().count() as u32;
        let mapping = result
            .source_map
            .iter()
            .find(|mapping| {
                mapping
                    .source_start
                    .is_some_and(|position| position.offset == helper_start_utf16)
            })
            .expect("o corpo da função auxiliar precisa ter source map");
        assert_eq!(mapping.quality, super::SourceMapQuality::Exact);
        assert!(!result.source_map.iter().any(|generated_only| {
            generated_only.quality == super::SourceMapQuality::GeneratedOnly
                && generated_only.generated_start.offset <= mapping.generated_start.offset
                && generated_only.generated_end.offset >= mapping.generated_end.offset
        }));
    }

    #[test]
    fn export_source_map_serializes_with_the_typescript_camel_case_contract() {
        let result = export_portugol_rs("funcao inicio() { seja idade: inteiro = 16; }");
        let value = serde_json::to_value(result).unwrap();
        let mapping = &value["sourceMap"][0];

        assert!(mapping.get("generatedStart").is_some());
        assert!(mapping.get("generatedEnd").is_some());
        assert!(mapping.get("quality").is_some());
        assert!(value.get("source_map").is_none());
    }

    #[test]
    fn format_portugol_rs_returns_utf16_whitespace_edits_and_rejects_invalid_source() {
        let source = "funcao inicio(){escreva(\"😀\");seja x:inteiro=1+2;}";
        let edits = super::format_portugol_rs(source).expect("fonte PORS válida deve formatar");
        assert!(!edits.is_empty());
        let mut units: Vec<u16> = source.encode_utf16().collect();
        for edit in edits.into_iter().rev() {
            units.splice(edit.from..edit.to, edit.insert.encode_utf16());
        }
        let formatted = String::from_utf16(&units).unwrap();
        assert_eq!(
            formatted,
            "funcao inicio() {\n    escreva(\"😀\");\n    seja x: inteiro = 1 + 2;\n}\n"
        );
        assert!(
            super::format_portugol_rs(&formatted)
                .expect("fonte já formatada")
                .is_empty()
        );
        assert!(super::format_portugol_rs("funcao inicio() { seja x: inteiro = ; }").is_none());
    }
}
