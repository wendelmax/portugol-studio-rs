//! Ponte entre a janela e o núcleo (`portugol-service`).
//!
//! Comandos chamados pelo frontend:
//!
//! | comando        | o que faz                                          |
//! |----------------|----------------------------------------------------|
//! | `check`        | diagnósticos de sintaxe + semântica                |
//! | `run`          | compila e executa; devolve o id da execução        |
//! | `send_input`   | responde a um `leia`                               |
//! | `stop`         | interrompe a execução                              |
//! | `read_file`    | lê `.por` em UTF-8 ou ISO-8859-1                   |
//! | `write_file`   | grava em UTF-8                                     |
//!
//! Eventos emitidos durante uma execução (todos com `id`):
//! `run:output` (`text`), `run:clear`, `run:input` (`expected`) e
//! `run:finished` (`outcome`).

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use portugol_service::{
    DiagnosticInfo, FileFilter, RunHandle, RunObserver, RunOptions, RunOutcome,
};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Runs {
    next_id: AtomicU32,
    current: Mutex<Option<(u32, RunHandle)>>,
}

impl Runs {
    fn with_current(&self, id: u32, action: impl FnOnce(&RunHandle)) {
        let current = self.current.lock().unwrap_or_else(|e| e.into_inner());
        if let Some((current_id, handle)) = current.as_ref()
            && *current_id == id
        {
            action(handle);
        }
    }
}

#[derive(Clone, Serialize)]
struct OutputEvent<'a> {
    id: u32,
    text: &'a str,
}

#[derive(Clone, Serialize)]
struct IdEvent {
    id: u32,
}

#[derive(Clone, Serialize)]
struct InputEvent {
    id: u32,
    expected: &'static str,
}

#[derive(Clone, Serialize)]
struct FinishedEvent {
    id: u32,
    outcome: RunOutcome,
}

struct EventSink {
    app: AppHandle,
    id: u32,
}

impl RunObserver for EventSink {
    fn output(&self, text: &str) {
        let _ = self
            .app
            .emit("run:output", OutputEvent { id: self.id, text });
    }

    fn clear(&self) {
        let _ = self.app.emit("run:clear", IdEvent { id: self.id });
    }

    fn input_requested(&self, expected: &'static str) {
        let _ = self.app.emit(
            "run:input",
            InputEvent {
                id: self.id,
                expected,
            },
        );
    }

    fn select_file(&self, filters: &[FileFilter], accept_all: bool) -> Option<String> {
        let mut dialog = self.app.dialog().file().set_title("Selecionar arquivo");
        for filter in filters {
            let extensions: Vec<&str> = filter.extensions.iter().map(String::as_str).collect();
            dialog = dialog.add_filter(&filter.description, &extensions);
        }
        if accept_all {
            dialog = dialog.add_filter("Todos os arquivos", &["*"]);
        }
        // Chamado da thread da VM, nunca da thread principal.
        dialog
            .blocking_pick_file()
            .and_then(|path| path.into_path().ok())
            .map(|path| path.display().to_string())
    }

    fn finished(&self, outcome: RunOutcome) {
        let _ = self.app.emit(
            "run:finished",
            FinishedEvent {
                id: self.id,
                outcome,
            },
        );
    }
}

#[tauri::command]
fn check(source: String) -> Vec<DiagnosticInfo> {
    portugol_service::check(&source)
}

/// Inicia uma execução, parando a anterior se ainda estiver rodando.
#[tauri::command]
fn run(
    app: AppHandle,
    runs: State<'_, Runs>,
    source: String,
    args: Vec<String>,
    path: Option<String>,
) -> Result<u32, Vec<DiagnosticInfo>> {
    let id = runs.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let mut current = runs.current.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, previous)) = current.take() {
        previous.stop();
    }
    let observer = Arc::new(EventSink { app, id });
    // Como no original, caminhos relativos partem da pasta do programa.
    let working_dir = path
        .as_deref()
        .and_then(|p| std::path::Path::new(p).parent())
        .map(std::path::Path::to_path_buf);
    let options = RunOptions { args, working_dir };
    let handle = portugol_service::start(&source, options, observer)?;
    *current = Some((id, handle));
    Ok(id)
}

#[tauri::command]
fn send_input(runs: State<'_, Runs>, id: u32, line: String) {
    runs.with_current(id, |handle| handle.send_input(line));
}

#[tauri::command]
fn stop(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::stop);
}

/// Os exemplos históricos do Portugol Studio estão em ISO-8859-1; aceitamos
/// UTF-8 (com ou sem BOM) e caímos para Latin-1 quando necessário.
#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("não foi possível ler {path}: {e}"))?;
    Ok(match String::from_utf8(bytes) {
        Ok(text) => text
            .strip_prefix('\u{feff}')
            .map(str::to_string)
            .unwrap_or(text),
        Err(error) => error.into_bytes().iter().map(|&b| b as char).collect(),
    })
}

#[tauri::command]
fn write_file(path: String, text: String) -> Result<(), String> {
    std::fs::write(&path, text).map_err(|e| format!("não foi possível gravar {path}: {e}"))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .manage(Runs::default())
        .invoke_handler(tauri::generate_handler![
            check, run, send_input, stop, read_file, write_file
        ])
        .run(tauri::generate_context!())
        .expect("erro ao iniciar a IDE");
}
