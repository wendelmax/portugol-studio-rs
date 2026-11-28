//! Exportação de uma tela desktop (`.design` + code-behind) para um projeto
//! Cargo com [Slint](https://slint.dev) (ADR 0018, seção 6, Fase 6).
//!
//! O projeto gerado tem a interface em `ui/<Janela>.slint`, compilada pelo
//! `build.rs` com `slint-build`, e o `src/main.rs` com o code-behind traduzido
//! pelo `ferro-rust` e um adaptador que liga cada callback do Slint ao handler:
//! lê a janela da UI, chama a função e escreve o resultado de volta, como a VM
//! faz na IDE.

use ferro_hir::{Program, Type};
use ferro_tela::slint::{self, DataField, DataRecord, DataType};

/// Um projeto Cargo de tela pronto para gravar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenCargoProject {
    pub package_name: String,
    /// Arquivos do projeto, com o caminho relativo à pasta dele.
    pub files: Vec<(String, String)>,
    /// Imagens usadas pela tela, relativas à pasta do `.design`: devem ser
    /// copiadas para `ui/`, no mesmo caminho relativo.
    pub images: Vec<String>,
}

const SLINT_VERSION: &str = "1";

/// Os serviços do sistema no programa exportado, com a dependência de cada
/// um. O comportamento segue o da IDE: `mensagem` e `confirmar` mostram caixas
/// do sistema, os diálogos de arquivo devolvem `Opcao`, `colar` devolve
/// `Nenhum` quando não há texto, e uma notificação que falha não para a tela.
const SERVICES: &[(&str, &str, &str)] = &[
    (
        "Dialogo",
        "rfd = \"0.15\"",
        r#"
mod __ferro_dialogo {
    fn caixa(texto: &str) -> rfd::MessageDialog {
        rfd::MessageDialog::new().set_title("Mensagem").set_description(texto)
    }

    pub fn mensagem(texto: &str) {
        caixa(texto).set_level(rfd::MessageLevel::Info).show();
    }

    pub fn confirmar(texto: &str) -> bool {
        matches!(
            caixa(texto)
                .set_title("Confirmar")
                .set_buttons(rfd::MessageButtons::YesNo)
                .show(),
            rfd::MessageDialogResult::Yes
        )
    }

    /// `"*.txt;*.csv"` (o `Filtro` do `<DialogoArquivo>`) em extensões.
    fn arquivo(titulo: &str, filtro: &str) -> rfd::FileDialog {
        let extensoes: Vec<String> = filtro
            .split(';')
            .map(|parte| parte.trim().trim_start_matches('*').trim_start_matches('.').to_owned())
            .filter(|extensao| !extensao.is_empty() && extensao != "*")
            .collect();
        let dialogo = rfd::FileDialog::new().set_title(titulo);
        if extensoes.is_empty() {
            dialogo
        } else {
            let nome = extensoes.iter().map(|e| format!("*.{e}")).collect::<Vec<_>>().join(", ");
            dialogo.add_filter(nome, &extensoes)
        }
    }

    pub fn abrir_arquivo(titulo: &str, filtro: &str) -> Option<String> {
        arquivo(titulo, filtro).pick_file().map(|caminho| caminho.display().to_string())
    }

    pub fn salvar_arquivo(titulo: &str, filtro: &str) -> Option<String> {
        arquivo(titulo, filtro).save_file().map(|caminho| caminho.display().to_string())
    }
}
"#,
    ),
    (
        "AreaTransferencia",
        "arboard = \"3\"",
        r#"
mod __ferro_area_transferencia {
    pub fn copiar(texto: &str) {
        if let Ok(mut area) = arboard::Clipboard::new() {
            let _ = area.set_text(texto.to_owned());
        }
    }

    pub fn colar() -> Option<String> {
        arboard::Clipboard::new().ok()?.get_text().ok()
    }
}
"#,
    ),
    (
        "Notificacao",
        "notify-rust = \"4\"",
        r#"
mod __ferro_notificacao {
    pub fn mostrar(titulo: &str, texto: &str) {
        let _ = notify_rust::Notification::new().summary(titulo).body(texto).show();
    }
}
"#,
    ),
];

/// Serviços que o code-behind usa, pelo lexer (comentários não contam).
fn used_services(code: &str) -> Vec<&'static (&'static str, &'static str, &'static str)> {
    let names: Vec<&str> = SERVICES.iter().map(|(name, _, _)| *name).collect();
    let uses = ferro_frontend::module_uses(code, &names);
    SERVICES
        .iter()
        .filter(|(name, _, _)| uses.iter().any(|(module, _)| module == name))
        .collect()
}

fn data_type(ty: Type, program: &Program, path: &mut Vec<String>) -> Result<DataType, String> {
    let record = |id| {
        program
            .record_types
            .iter()
            .find(|definition| definition.id == id)
            .ok_or_else(|| "registro desconhecido".to_string())
    };
    Ok(match ty {
        Type::Integer => DataType::Int,
        Type::Real => DataType::Float,
        Type::Boolean => DataType::Bool,
        Type::String => DataType::String,
        Type::VectorInteger => DataType::List(Box::new(DataType::Int)),
        Type::VectorReal => DataType::List(Box::new(DataType::Float)),
        Type::VectorBoolean => DataType::List(Box::new(DataType::Bool)),
        Type::VectorString => DataType::List(Box::new(DataType::String)),
        Type::Record(id) => {
            let definition = record(id)?;
            DataType::Struct(
                definition.name.clone(),
                data_fields(definition, program, path)?,
            )
        }
        Type::VectorRecord(id) => {
            let definition = record(id)?;
            DataType::List(Box::new(DataType::Struct(
                definition.name.clone(),
                data_fields(definition, program, path)?,
            )))
        }
        _ => {
            return Err(format!(
                "O campo `{}` dos dados tem um tipo que a exportação para Slint ainda não aceita (use inteiro, real, logico, cadeia, registros e vetores deles).",
                path.join(".")
            ));
        }
    })
}

fn data_fields(
    definition: &ferro_hir::RecordDef,
    program: &Program,
    path: &mut Vec<String>,
) -> Result<Vec<DataField>, String> {
    definition
        .fields
        .iter()
        .map(|field| {
            path.push(field.name.clone());
            let ty = data_type(field.ty, program, path);
            path.pop();
            Ok(DataField {
                name: field.name.clone(),
                ty: ty?,
            })
        })
        .collect()
}

/// O registro de `Dados` da tela, com os tipos do code-behind.
fn data_record(program: &Program, window: &str) -> Result<Option<DataRecord>, String> {
    let Some(window) = program.record_types.iter().find(|r| r.name == window) else {
        return Ok(None);
    };
    let Some(field) = window.fields.iter().find(|f| f.name == "dados") else {
        return Ok(None);
    };
    let mut path = vec!["dados".to_owned()];
    match data_type(field.ty, program, &mut path)? {
        DataType::Struct(name, fields) => Ok(Some(DataRecord { name, fields })),
        _ => Ok(None),
    }
}

fn render_errors(source: &str, file: &str, diagnostics: &[ferro_hir::Diagnostic]) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| diagnostic.render(source, file))
        .collect::<Vec<_>>()
        .join("\n")
}

/// Projeto Cargo de uma tela. `name` vira o nome do pacote. Erros de sintaxe
/// ou de tipo no `.design` ou no code-behind, e recursos que a exportação
/// ainda não traduz, voltam como texto para mostrar ao aluno.
pub fn export_screen_cargo_project(
    design_source: &str,
    code_source: &str,
    name: &str,
) -> Result<ScreenCargoProject, String> {
    let document = ferro_tela::parse(design_source).map_err(|diagnostics| {
        render_errors(
            design_source,
            "tela.design",
            std::slice::from_ref(&diagnostics),
        )
    })?;
    let (screen, design_diagnostics) = ferro_tela::codebehind::screen_type(&document);
    if design_diagnostics
        .iter()
        .any(|d| d.severity == ferro_hir::Severity::Error)
    {
        return Err(render_errors(
            design_source,
            "tela.design",
            &design_diagnostics,
        ));
    }
    let window = screen.window.name.clone();
    let file = format!("{window}.fer");
    let options = ferro_tela::codebehind::lower_options(design_source);
    let program = ferro_frontend::lower_with(code_source, &options)
        .map_err(|diagnostics| render_errors(code_source, &file, &diagnostics))?;
    let generated = crate::export_screen_code(code_source, &options)
        .map_err(|diagnostics| render_errors(code_source, &file, &diagnostics))?;
    let data = data_record(&program, &window)?;

    let exported = slint::export_screen(&document, &screen, data.as_ref());
    for handler in &exported.callbacks {
        if !program.functions.iter().any(|f| f.source_name == *handler) {
            return Err(format!(
                "A tela liga o evento a `{handler}`, mas o code-behind não tem essa função."
            ));
        }
    }
    let adapter = slint::rust_adapter(
        &document,
        &screen,
        data.as_ref(),
        &exported,
        &crate::rust_name,
    );
    // O código gerado termina com um `main` vazio (o code-behind não tem
    // `inicio`); o adaptador traz o `main` da janela.
    let code = generated.code;
    let code = code
        .trim_end()
        .strip_suffix("fn main() {\n}")
        .unwrap_or(&code)
        .trim_end();
    let services = used_services(code_source);
    let runtime: String = services.iter().map(|(_, _, code)| *code).collect();
    let main_rs = format!(
        "// Gerado pelo Ferro Studio a partir de {window}.fer e {window}.design.\n#![allow(dead_code, non_camel_case_types, non_snake_case, unused)]\n{code}\n{runtime}{adapter}"
    );
    let dependencies: String = services
        .iter()
        .map(|(_, dependency, _)| format!("{dependency}\n"))
        .collect();

    let package_name = crate::cargo_package_name(name);
    let cargo_toml = format!(
        "[package]\nname = \"{package_name}\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nslint = \"{SLINT_VERSION}\"\n{dependencies}\n[build-dependencies]\nslint-build = \"{SLINT_VERSION}\"\n"
    );
    let build_rs = format!(
        "fn main() {{\n    slint_build::compile(\"ui/{window}.slint\").expect(\"a interface da tela não compilou\");\n}}\n"
    );
    let readme = format!(
        "# {package_name}\n\nProjeto Rust gerado pelo Ferro Studio a partir da tela `{window}`.\n\n## Como executar\n\n```bash\ncargo run\n```\n\nA primeira compilação baixa e compila o Slint e leva alguns minutos.\n\n## Arquivos\n\n- `ui/{window}.slint`: a interface, traduzida do `.design`.\n- `src/main.rs`: o code-behind traduzido para Rust e o adaptador que liga os eventos da interface às funções da tela.\n\nPara mudar a tela ou o código, edite o projeto no Ferro Studio e exporte de novo.\n\n## Licença do Slint\n\nA interface usa o [Slint](https://slint.dev), distribuído sob GPLv3, sob a *Royalty-free License* ou sob licença comercial. Para distribuir este programa sem as obrigações da GPLv3, siga a *Royalty-free License* do Slint, que pede a atribuição \"Made with Slint\" (por exemplo, na tela Sobre ou na documentação). Confira o texto vigente em <https://slint.dev/terms-and-conditions> antes de publicar.\n"
    );
    Ok(ScreenCargoProject {
        package_name,
        files: vec![
            ("Cargo.toml".into(), cargo_toml),
            ("build.rs".into(), build_rs),
            (format!("ui/{window}.slint"), exported.slint),
            ("src/main.rs".into(), main_rs),
            ("README.md".into(), readme),
        ],
        images: exported.images,
    })
}
