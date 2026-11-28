//! Janela em que uma tela desktop roda (ADR 0018, seção 5). Cada execução é
//! uma sessão [`ScreenRun`] guardada pelo rótulo da janela (`screen-N`); a
//! janela pede o estado ao abrir e manda cada evento, e a saída de `escreva`
//! e os erros vão para o console da IDE pelo evento `screen-run:output`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use ferro_service::{DesignNode, FileFilter, Host, ScreenEvent, ScreenRun, ScreenStep};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_dialog::{DialogExt, MessageDialogButtons, MessageDialogKind};
use tauri_plugin_notification::NotificationExt;

/// Diálogos do sistema (`Dialogo`) presos à janela da tela. É chamado da
/// thread que trata o evento, nunca da thread principal.
struct ScreenHost {
    app: AppHandle,
    label: String,
}

impl ScreenHost {
    fn window_title(&self) -> String {
        self.app
            .get_webview_window(&self.label)
            .and_then(|window| window.title().ok())
            .unwrap_or_else(|| "Tela".to_owned())
    }

    fn message(&self, text: &str) -> tauri_plugin_dialog::MessageDialogBuilder<tauri::Wry> {
        let dialog = self.app.dialog().message(text).title(self.window_title());
        match self.app.get_webview_window(&self.label) {
            Some(window) => dialog.parent(&window),
            None => dialog,
        }
    }
}

impl Host for ScreenHost {
    fn show_message(&mut self, text: &str) -> Result<(), String> {
        self.message(text)
            .kind(MessageDialogKind::Info)
            .buttons(MessageDialogButtons::OkCustom("OK".into()))
            .blocking_show();
        Ok(())
    }

    fn confirm(&mut self, text: &str) -> Result<bool, String> {
        Ok(self
            .message(text)
            .kind(MessageDialogKind::Warning)
            .buttons(MessageDialogButtons::OkCancelCustom(
                "Sim".into(),
                "Não".into(),
            ))
            .blocking_show())
    }

    fn clipboard_write(&mut self, text: &str) -> Result<(), String> {
        self.app
            .clipboard()
            .write_text(text.to_owned())
            .map_err(|error| format!("Não foi possível copiar: {error}"))
    }

    fn clipboard_read(&mut self) -> Result<Option<String>, String> {
        // Sem texto (área vazia ou com imagem), o plugin devolve erro: é `Nenhum`.
        Ok(self
            .app
            .clipboard()
            .read_text()
            .ok()
            .filter(|text| !text.is_empty()))
    }

    fn notify(&mut self, title: &str, text: &str) -> Result<(), String> {
        let title = if title.trim().is_empty() {
            self.window_title()
        } else {
            title.to_owned()
        };
        self.app
            .notification()
            .builder()
            .title(title)
            .body(text)
            .show()
            .map_err(|error| format!("Não foi possível mostrar a notificação: {error}"))
    }

    fn file_dialog(
        &mut self,
        save: bool,
        title: &str,
        filters: &[FileFilter],
    ) -> Result<Option<String>, String> {
        let default_title = if save {
            "Salvar arquivo"
        } else {
            "Abrir arquivo"
        };
        let mut dialog = self
            .app
            .dialog()
            .file()
            .set_title(if title.trim().is_empty() {
                default_title
            } else {
                title
            });
        if let Some(window) = self.app.get_webview_window(&self.label) {
            dialog = dialog.set_parent(&window);
        }
        for filter in filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
            dialog = dialog.add_filter(&filter.description, &extensions);
        }
        let chosen = if save {
            dialog.blocking_save_file()
        } else {
            dialog.blocking_pick_file()
        };
        Ok(chosen
            .and_then(|path| path.into_path().ok())
            .map(|path| path.display().to_string()))
    }
}

struct Session {
    run: ScreenRun,
    design: String,
    title: String,
}

#[derive(Default)]
pub struct ScreenWindows {
    next: AtomicU32,
    sessions: Mutex<HashMap<String, Session>>,
}

/// O que a janela recebe ao abrir: a árvore do `.design` para o layout e o
/// estado atual da janela.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenState {
    tree: Option<DesignNode>,
    step: ScreenStep,
}

/// Mensagem para o console da IDE.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
struct ScreenOutput {
    label: String,
    text: String,
    error: bool,
}

fn emit_output(app: &AppHandle, label: &str, text: String, error: bool) {
    if text.is_empty() {
        return;
    }
    let _ = app.emit_to(
        "main",
        "screen-run:output",
        ScreenOutput {
            label: label.to_owned(),
            text,
            error,
        },
    );
}

fn title_of(step: &ScreenStep) -> String {
    step.snapshot["janela"]["titulo"]
        .as_str()
        .filter(|title| !title.is_empty())
        .unwrap_or("Tela")
        .to_owned()
}

fn size_of(step: &ScreenStep) -> (f64, f64) {
    let read = |field: &str, default: f64| {
        step.snapshot["janela"][field]
            .as_f64()
            .filter(|n| *n >= 1.0)
            .unwrap_or(default)
    };
    (read("largura", 400.0), read("altura", 300.0))
}

/// Inicia a tela numa janela própria; devolve o rótulo da janela.
#[tauri::command]
pub async fn screen_run_start(
    app: AppHandle,
    session: tauri::State<'_, crate::workspace::WorkspaceSession>,
    design_source: String,
    code_source: String,
    workspace_id: Option<String>,
    design_path: Option<String>,
) -> Result<String, String> {
    let design = design_source.clone();
    // Numa pasta de projeto, as permissões do .ferproj valem e o `Arquivos`
    // pode receber a pasta do projeto.
    let project = match (workspace_id.as_deref(), design_path.as_deref()) {
        (Some(id), Some(path)) => {
            Some(crate::workspace::screen_project(session.inner(), id, path)?)
        }
        _ => None,
    };
    let (mut run, step) = tauri::async_runtime::spawn_blocking(move || {
        ScreenRun::start(&design_source, &code_source)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;
    if let Some((folder, Some(manifest))) = project {
        let permissions = ferro_tela::project::analyze_manifest(&manifest)
            .manifest
            .map(|manifest| manifest.permissions)
            .unwrap_or_default();
        run.set_project(permissions, Some(folder));
    }

    let manager = app.state::<ScreenWindows>();
    let label = format!("screen-{}", manager.next.fetch_add(1, Ordering::Relaxed));
    let host_app = app.clone();
    let host_label = label.clone();
    run.set_host(Box::new(move || {
        Box::new(ScreenHost {
            app: host_app.clone(),
            label: host_label.clone(),
        })
    }));
    let title = title_of(&step);
    let (width, height) = size_of(&step);
    emit_output(&app, &label, step.output.clone(), false);
    manager
        .sessions
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .insert(
            label.clone(),
            Session {
                run,
                design,
                title: title.clone(),
            },
        );

    let url = WebviewUrl::App(format!("screen.html?sessao={label}").into());
    let window = WebviewWindowBuilder::new(&app, &label, url)
        .title(&title)
        .inner_size(width, height)
        .resizable(true)
        .build();
    let window = match window {
        Ok(window) => window,
        Err(error) => {
            manager
                .sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&label);
            return Err(error.to_string());
        }
    };
    let closing_app = app.clone();
    let closing_label = label.clone();
    window.on_window_event(move |event| {
        if matches!(event, WindowEvent::Destroyed) {
            closing_app
                .state::<ScreenWindows>()
                .sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(&closing_label);
            let _ = closing_app.emit_to("main", "screen-run:closed", closing_label.clone());
        }
    });
    Ok(label)
}

/// Estado inicial pedido pela janela da tela quando ela carrega.
#[tauri::command]
pub fn screen_run_state(
    windows: tauri::State<'_, ScreenWindows>,
    label: String,
) -> Result<ScreenState, String> {
    let sessions = windows.sessions.lock().unwrap_or_else(|e| e.into_inner());
    let session = sessions
        .get(&label)
        .ok_or_else(|| "A execução desta tela já terminou.".to_string())?;
    Ok(ScreenState {
        tree: ferro_service::design_tree(&session.design),
        step: session.run.current_step(),
    })
}

/// Um evento da janela: aplica o valor, chama a função ligada e devolve o
/// novo estado. Erros vão também para o console da IDE.
#[tauri::command]
pub async fn screen_run_dispatch(
    app: AppHandle,
    label: String,
    event: ScreenEvent,
) -> Result<ScreenStep, String> {
    let worker_app = app.clone();
    let worker_label = label.clone();
    let result = tauri::async_runtime::spawn_blocking(move || {
        let windows = worker_app.state::<ScreenWindows>();
        let mut sessions = windows.sessions.lock().unwrap_or_else(|e| e.into_inner());
        let session = sessions
            .get_mut(&worker_label)
            .ok_or_else(|| "A execução desta tela já terminou.".to_string())?;
        let step = session.run.dispatch(&event)?;
        let title = title_of(&step);
        let renamed = (title != session.title).then(|| {
            session.title = title.clone();
            title
        });
        Ok::<_, String>((step, renamed))
    })
    .await
    .map_err(|error| error.to_string())?;
    match result {
        Ok((step, renamed)) => {
            emit_output(&app, &label, step.output.clone(), false);
            if let (Some(title), Some(window)) = (renamed, app.get_webview_window(&label)) {
                let _ = window.set_title(&title);
            }
            Ok(step)
        }
        Err(error) => {
            emit_output(&app, &label, format!("{error}\n"), true);
            Err(error)
        }
    }
}

/// Exportar uma tela (ADR 0018, seção 6): pede a pasta e grava o projeto
/// Cargo com a interface em Slint; as imagens da tela vão para `ui/`.
#[tauri::command]
pub async fn screen_export_cargo(
    app: AppHandle,
    session: tauri::State<'_, crate::workspace::WorkspaceSession>,
    design_source: String,
    code_source: String,
    workspace_id: String,
    design_path: String,
) -> Result<Option<ScreenExportResult>, String> {
    let (folder, _) =
        crate::workspace::screen_project(session.inner(), &workspace_id, &design_path)?;
    let name = folder
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("tela-ferro")
        .to_owned();
    let project = ferro_rust::export_screen_cargo_project(&design_source, &code_source, &name)?;
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Exportar a tela como projeto Cargo")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let destination = selected
            .into_path()
            .map_err(|error| format!("Não foi possível acessar a pasta selecionada: {error}"))?;
        write_screen_project(&project, &folder, &destination)?;
        Ok(Some(ScreenExportResult {
            package_name: project.package_name,
            project_dir: destination.display().to_string(),
        }))
    })
    .await
    .map_err(|error| error.to_string())?
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreenExportResult {
    package_name: String,
    project_dir: String,
}

/// Grava o projeto; recusa uma pasta que já tem `Cargo.toml` e imagens fora
/// da pasta da tela.
fn write_screen_project(
    project: &ferro_rust::ScreenCargoProject,
    screen_folder: &std::path::Path,
    destination: &std::path::Path,
) -> Result<(), String> {
    if destination.join("Cargo.toml").exists() {
        return Err("A pasta escolhida já tem um Cargo.toml; escolha uma pasta vazia.".into());
    }
    let write = |relative: &str, content: &[u8]| -> Result<(), String> {
        let target = destination.join(relative);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        std::fs::write(&target, content)
            .map_err(|error| format!("Não foi possível gravar {}: {error}", target.display()))
    };
    for (relative, content) in &project.files {
        write(relative, content.as_bytes())?;
    }
    let screen_folder = std::fs::canonicalize(screen_folder).map_err(|error| error.to_string())?;
    for image in &project.images {
        let source = std::fs::canonicalize(screen_folder.join(image))
            .map_err(|_| format!("A imagem `{image}` da tela não foi encontrada."))?;
        if !source.starts_with(&screen_folder) {
            return Err(format!("A imagem `{image}` está fora da pasta da tela."));
        }
        let bytes = std::fs::read(&source).map_err(|error| error.to_string())?;
        write(&format!("ui/{image}"), &bytes)?;
    }
    Ok(())
}

#[cfg(test)]
mod export_tests {
    #[test]
    fn grava_o_projeto_com_as_imagens_e_nao_sobrescreve() {
        let base = std::env::temp_dir().join(format!("ferro-exportar-tela-{}", std::process::id()));
        let screen = base.join("App");
        std::fs::create_dir_all(screen.join("recursos")).unwrap();
        std::fs::write(screen.join("recursos/logo.png"), [1u8, 2, 3]).unwrap();
        let design = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"100\" Altura=\"100\">\n    <Imagem Nome=\"img\" X=\"0\" Y=\"0\" Arquivo=\"recursos/logo.png\" />\n</Janela>\n";
        let project = ferro_rust::export_screen_cargo_project(design, "", "App").unwrap();
        let destination = base.join("saida");
        super::write_screen_project(&project, &screen, &destination).unwrap();
        assert!(destination.join("ui/J.slint").is_file());
        assert_eq!(
            std::fs::read(destination.join("ui/recursos/logo.png")).unwrap(),
            [1, 2, 3]
        );
        let again = super::write_screen_project(&project, &screen, &destination).unwrap_err();
        assert!(again.contains("Cargo.toml"), "{again}");

        let outside = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"100\" Altura=\"100\">\n    <Imagem Nome=\"img\" X=\"0\" Y=\"0\" Arquivo=\"../segredo.png\" />\n</Janela>\n";
        std::fs::write(base.join("segredo.png"), [9u8]).unwrap();
        let project = ferro_rust::export_screen_cargo_project(outside, "", "App").unwrap();
        let error =
            super::write_screen_project(&project, &screen, &base.join("saida2")).unwrap_err();
        assert!(error.contains("fora da pasta"), "{error}");
        std::fs::remove_dir_all(&base).unwrap();
    }
}
