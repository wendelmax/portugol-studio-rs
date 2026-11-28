//! Janela em que uma tela desktop roda (ADR 0018, seção 5). Cada execução é
//! uma sessão [`ScreenRun`] guardada pelo rótulo da janela (`screen-N`); a
//! janela pede o estado ao abrir e manda cada evento, e a saída de `escreva`
//! e os erros vão para o console da IDE pelo evento `screen-run:output`.

use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU32, Ordering};

use ferro_service::{DesignNode, ScreenEvent, ScreenRun, ScreenStep};
use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder, WindowEvent};

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
    design_source: String,
    code_source: String,
) -> Result<String, String> {
    let design = design_source.clone();
    let (run, step) = tauri::async_runtime::spawn_blocking(move || {
        ScreenRun::start(&design_source, &code_source)
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    let manager = app.state::<ScreenWindows>();
    let label = format!("screen-{}", manager.next.fetch_add(1, Ordering::Relaxed));
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
