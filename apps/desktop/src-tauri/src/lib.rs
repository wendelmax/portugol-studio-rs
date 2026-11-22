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
//! | `debug_*`      | pausa, continua, passos e pontos de parada         |
//! | `read_file`    | lê `.por` em UTF-8 ou ISO-8859-1                   |
//! | `write_file`   | grava em UTF-8                                     |
//!
//! Eventos emitidos durante uma execução (todos com `id`):
//! `run:output` (`text`), `run:clear`, `run:input` (`expected`),
//! `run:paused` (`info`) e `run:finished` (`outcome`).

mod audio;
mod catalog;
mod graphics;

mod alerts;
mod open_files;
mod updates;
mod workspace;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use portugol_service::{
    DebugInfo, DiagnosticInfo, FileFilter, RunHandle, RunObserver, RunOptions, RunOutcome,
};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Runs {
    next_id: AtomicU32,
    current: Mutex<Option<(u32, RunHandle)>>,
    rust_source_cancellations: Mutex<HashMap<u32, Arc<AtomicBool>>>,
    portugol_rs_input_senders: Mutex<HashMap<u32, mpsc::Sender<String>>>,
    portugol_rs_input_receivers: Mutex<HashMap<u32, mpsc::Receiver<String>>>,
}

impl Runs {
    fn register_rust_source(&self) -> Result<u32, String> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
        let mut cancellations = self
            .rust_source_cancellations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        match cancellations.entry(id) {
            std::collections::hash_map::Entry::Vacant(entry) => {
                entry.insert(Arc::new(AtomicBool::new(false)));
            }
            std::collections::hash_map::Entry::Occupied(_) => {
                return Err("Já existe uma execução Rust com este identificador.".into());
            }
        }
        Ok(id)
    }

    fn register_portugol_rs(&self) -> Result<u32, String> {
        let id = self.register_rust_source()?;
        let (sender, receiver) = mpsc::channel();
        self.portugol_rs_input_senders
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, sender);
        self.portugol_rs_input_receivers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, receiver);
        Ok(id)
    }

    fn request_stop(&self, id: u32) -> bool {
        {
            let current = self
                .current
                .lock()
                .unwrap_or_else(|error| error.into_inner());
            if let Some((current_id, handle)) = current.as_ref()
                && *current_id == id
            {
                handle.stop();
                return true;
            }
        }
        let cancellations = self
            .rust_source_cancellations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if let Some(cancel) = cancellations.get(&id) {
            cancel.store(true, Ordering::Release);
            true
        } else {
            false
        }
    }

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
    expected: String,
}

#[derive(Clone, Serialize)]
struct FinishedEvent {
    id: u32,
    outcome: RunOutcome,
}

#[derive(Clone, Serialize)]
struct PausedEvent {
    id: u32,
    info: DebugInfo,
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
                expected: expected.to_owned(),
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

    fn paused(&self, info: DebugInfo) {
        let _ = self
            .app
            .emit("run:paused", PausedEvent { id: self.id, info });
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

#[tauri::command]
fn inspect(source: String) -> portugol_service::Inspection {
    portugol_service::inspect(&source)
}

#[tauri::command]
fn inspect_portugol_rs(source: String) -> portugol_service::Inspection {
    portugol_service::inspect_portugol_rs(&source)
}

#[tauri::command]
async fn export_rust(source: String) -> Result<portugol_service::RustExport, String> {
    tauri::async_runtime::spawn_blocking(move || portugol_service::export_rust(&source))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn export_portugol_rs(source: String) -> Result<portugol_service::RustExport, String> {
    tauri::async_runtime::spawn_blocking(move || portugol_service::export_portugol_rs(&source))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn format_rust_source(source: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || portugol_service::format_rust_source(&source))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn rustfmt_available() -> bool {
    portugol_service::rustfmt_available()
}

#[tauri::command]
fn rust_toolchain_available() -> bool {
    portugol_service::rust_toolchain_available()
}

#[tauri::command]
async fn check_rust_source(
    source: String,
) -> Result<Vec<portugol_service::RustSourceCompileDiagnostic>, String> {
    tauri::async_runtime::spawn_blocking(move || portugol_service::check_rust_source(&source))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn run_portugol_rs(
    app: AppHandle,
    runs: State<'_, Runs>,
    id: u32,
    source: String,
) -> Result<portugol_service::PortugolRsRunResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução Portugol-RS não está mais registrada.".to_string())?;
    let receiver = runs
        .portugol_rs_input_receivers
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id)
        .ok_or_else(|| "O canal de entrada Portugol-RS não está mais registrado.".to_string())?;
    let input_app = app.clone();
    let output_app = app;
    let joined = tauri::async_runtime::spawn_blocking(move || {
        portugol_service::run_portugol_rs_with_io(
            &source,
            &cancellation,
            |expected| {
                input_app
                    .emit(
                        "run:input",
                        InputEvent {
                            id,
                            expected: expected.to_owned(),
                        },
                    )
                    .map_err(|error| error.to_string())?;
                loop {
                    match receiver.recv_timeout(std::time::Duration::from_millis(50)) {
                        Ok(line) => return Ok(line),
                        Err(mpsc::RecvTimeoutError::Timeout)
                            if !cancellation.load(std::sync::atomic::Ordering::Acquire) => {}
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            return Ok(String::new());
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => {
                            return Err("O canal de entrada Portugol-RS foi encerrado.".into());
                        }
                    }
                }
            },
            |text| {
                let _ = output_app.emit("run:output", OutputEvent { id, text });
            },
        )
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    runs.portugol_rs_input_senders
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
fn register_rust_source_run(runs: State<'_, Runs>) -> Result<u32, String> {
    runs.register_rust_source()
}

#[tauri::command]
fn register_portugol_rs_run(runs: State<'_, Runs>) -> Result<u32, String> {
    runs.register_portugol_rs()
}

#[tauri::command]
async fn run_rust_source(
    runs: State<'_, Runs>,
    id: u32,
    source: String,
    args: Vec<String>,
) -> Result<portugol_service::RustSourceRunResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução Rust não está mais registrada.".to_string())?;
    let joined = tauri::async_runtime::spawn_blocking(move || {
        portugol_service::run_rust_source_with_args_and_cancel(&source, &args, &cancellation)
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
fn cargo_toolchain_available() -> bool {
    portugol_service::cargo_available()
}

#[tauri::command]
async fn run_cargo_project(
    runs: State<'_, Runs>,
    workspace: State<'_, workspace::WorkspaceSession>,
    id: u32,
    workspace_id: String,
    relative_source: String,
) -> Result<portugol_service::CargoProjectRunResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução Cargo não está mais registrada.".to_string())?;
    let project_root =
        match workspace::cargo_project_for_source(&workspace, &workspace_id, &relative_source) {
            Ok(Some(root)) => root,
            Ok(None) => {
                runs.rust_source_cancellations
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .remove(&id);
                return Err("Não foi encontrado Cargo.toml acima deste arquivo Rust.".into());
            }
            Err(error) => {
                runs.rust_source_cancellations
                    .lock()
                    .unwrap_or_else(|error| error.into_inner())
                    .remove(&id);
                return Err(error);
            }
        };
    let joined = tauri::async_runtime::spawn_blocking(move || {
        portugol_service::run_cargo_project_with_cancel(&project_root, &cancellation)
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
fn format_source(source: String) -> Option<Vec<portugol_service::TextEdit>> {
    portugol_service::format_source(&source)
}

#[tauri::command]
fn include_library(
    source: String,
    name: String,
) -> Result<Option<portugol_service::TextEdit>, String> {
    portugol_service::include_library(&source, &name)
}

#[tauri::command]
fn exercise_catalog() -> Vec<portugol_service::Exercise> {
    portugol_service::exercises()
}

#[tauri::command]
fn inspect_exercise(contents: String) -> Result<portugol_service::Exercise, String> {
    portugol_service::inspect_exercise(&contents)
}

#[tauri::command]
async fn check_exercise(
    id: u32,
    source: String,
    contents: Option<String>,
) -> Result<portugol_service::ExerciseResult, String> {
    tauri::async_runtime::spawn_blocking(move || match contents {
        Some(raw) => portugol_service::check_imported_exercise(&raw, &source),
        None => portugol_service::check_exercise(id, &source),
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn rename_target(source: String, offset: u32) -> Result<portugol_service::RenameTarget, String> {
    portugol_service::rename_target(&source, offset)
}

#[tauri::command]
fn rename_target_portugol_rs(
    source: String,
    offset: u32,
) -> Result<portugol_service::RenameTarget, String> {
    portugol_service::rename_target_portugol_rs(&source, offset)
}

#[tauri::command]
fn resolve_symbol(source: String, offset: u32) -> Result<portugol_service::SymbolLocation, String> {
    portugol_service::resolve_symbol(&source, offset)
}

#[tauri::command]
fn resolve_symbol_portugol_rs(
    source: String,
    offset: u32,
) -> Result<portugol_service::SymbolLocation, String> {
    portugol_service::resolve_symbol_portugol_rs(&source, offset)
}

#[tauri::command]
fn rename_symbol(
    source: String,
    offset: u32,
    name: String,
) -> Result<Vec<portugol_service::RenameEdit>, String> {
    portugol_service::rename_symbol(&source, offset, &name)
}

#[tauri::command]
fn rename_symbol_portugol_rs(
    source: String,
    offset: u32,
    name: String,
) -> Result<Vec<portugol_service::RenameEdit>, String> {
    portugol_service::rename_symbol_portugol_rs(&source, offset, &name)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum ProjectPage {
    BugReport,
    Releases,
    Contribute,
}

#[tauri::command]
async fn open_project_page(page: ProjectPage) -> Result<(), String> {
    let url = match page {
        ProjectPage::BugReport => "https://github.com/wendelmax/portugol-studio-rs/issues/new",
        ProjectPage::Releases => "https://github.com/wendelmax/portugol-studio-rs/releases",
        ProjectPage::Contribute => "https://github.com/wendelmax/portugol-studio-rs",
    };
    let mut command;
    #[cfg(target_os = "linux")]
    {
        command = std::process::Command::new("xdg-open");
        command.arg(url);
    }
    #[cfg(target_os = "macos")]
    {
        command = std::process::Command::new("open");
        command.arg(url);
    }
    #[cfg(target_os = "windows")]
    {
        command = std::process::Command::new("rundll32.exe");
        command.args(["url.dll,FileProtocolHandler", url]);
    }
    let status = tauri::async_runtime::spawn_blocking(move || command.status())
        .await
        .map_err(|error| error.to_string())?
        .map_err(|error| error.to_string())?;
    if status.success() {
        Ok(())
    } else {
        Err("Não foi possível abrir o navegador.".into())
    }
}

/// Inicia uma execução, parando a anterior se ainda estiver rodando.
#[tauri::command]
fn run(
    app: AppHandle,
    runs: State<'_, Runs>,
    source: String,
    args: Vec<String>,
    path: Option<String>,
    breakpoints: Vec<u32>,
    pause_on_start: Option<bool>,
) -> Result<u32, Vec<DiagnosticInfo>> {
    let id = runs.next_id.fetch_add(1, Ordering::Relaxed) + 1;
    let mut current = runs.current.lock().unwrap_or_else(|e| e.into_inner());
    if let Some((_, previous)) = current.take() {
        previous.stop();
    }
    let observer: Arc<dyn RunObserver> = Arc::new(EventSink {
        app: app.clone(),
        id,
    });
    let host_observer = Arc::clone(&observer);
    // Como no original, caminhos relativos partem da pasta do programa.
    let working_dir = path
        .as_deref()
        .and_then(|p| std::path::Path::new(p).parent())
        .map(std::path::Path::to_path_buf);
    let options = RunOptions {
        args,
        working_dir,
        breakpoints,
        pause_on_start: pause_on_start.unwrap_or_default(),
    };
    let handle = portugol_service::start_with_host(&source, options, observer, move |interrupt| {
        Box::new(graphics::DesktopHost::new(
            app,
            id,
            host_observer,
            interrupt,
        ))
    })?;
    *current = Some((id, handle));
    Ok(id)
}

#[tauri::command]
fn send_input(runs: State<'_, Runs>, id: u32, line: String) {
    let mut line = Some(line);
    runs.with_current(id, |handle| {
        handle.send_input(line.take().unwrap_or_default())
    });
    if let Some(line) = line
        && let Some(sender) = runs
            .portugol_rs_input_senders
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
    {
        let _ = sender.send(line);
    }
}

#[tauri::command]
fn stop(runs: State<'_, Runs>, id: u32) -> bool {
    runs.request_stop(id)
}

#[tauri::command]
fn debug_pause(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::pause);
}

#[tauri::command]
fn debug_continue(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::continue_run);
}

#[tauri::command]
fn debug_step_over(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::step_over);
}

#[tauri::command]
fn debug_step_into(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::step_into);
}

#[tauri::command]
fn debug_step_out(runs: State<'_, Runs>, id: u32) {
    runs.with_current(id, RunHandle::step_out);
}

#[tauri::command]
fn debug_set_breakpoints(runs: State<'_, Runs>, id: u32, source: String, lines: Vec<u32>) {
    runs.with_current(id, |handle| handle.set_breakpoints(&lines, &source));
}

/// Os exemplos históricos do Portugol Studio estão em ISO-8859-1; aceitamos
/// UTF-8 (com ou sem BOM) e caímos para Latin-1 quando necessário.
#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let bytes = std::fs::read(&path).map_err(|e| format!("não foi possível ler {path}: {e}"))?;
    Ok(decode_text(bytes))
}

fn decode_text(bytes: Vec<u8>) -> String {
    match String::from_utf8(bytes) {
        Ok(text) => text
            .strip_prefix('\u{feff}')
            .map(str::to_string)
            .unwrap_or(text),
        Err(error) => error.into_bytes().iter().map(|&b| b as char).collect(),
    }
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
        .manage(workspace::WorkspaceSession::default())
        .manage(open_files::PendingFiles::from_args())
        .manage(graphics::GraphicsWindows::default())
        .invoke_handler(tauri::generate_handler![
            check,
            inspect,
            inspect_portugol_rs,
            export_rust,
            export_portugol_rs,
            format_rust_source,
            rustfmt_available,
            rust_toolchain_available,
            check_rust_source,
            run_portugol_rs,
            register_rust_source_run,
            register_portugol_rs_run,
            run_rust_source,
            cargo_toolchain_available,
            run_cargo_project,
            format_source,
            include_library,
            exercise_catalog,
            check_exercise,
            inspect_exercise,
            rename_target,
            rename_target_portugol_rs,
            rename_symbol,
            rename_symbol_portugol_rs,
            resolve_symbol,
            resolve_symbol_portugol_rs,
            open_project_page,
            updates::check_updates,
            alerts::project_alerts,
            run,
            send_input,
            stop,
            debug_pause,
            debug_continue,
            debug_step_over,
            debug_step_into,
            debug_step_out,
            debug_set_breakpoints,
            graphics::graphics_next,
            graphics::graphics_reply,
            graphics::graphics_input,
            read_file,
            open_files::pending_files,
            catalog::example_catalog,
            catalog::open_example,
            catalog::example_preview,
            write_file,
            workspace::workspace_open,
            workspace::workspace_cargo_project_available,
            workspace::workspace_read_file,
            workspace::workspace_write_file,
            workspace::workspace_create_file,
            workspace::workspace_restore,
            workspace::workspace_rename_file,
            workspace::workspace_delete_file,
            workspace::workspace_export
        ])
        .build(tauri::generate_context!())
        .expect("erro ao iniciar a IDE")
        .run(open_files::application_event);
}

#[cfg(test)]
mod rust_source_run_tests {
    use super::Runs;
    use std::sync::atomic::Ordering;

    #[test]
    fn stop_targets_only_a_registered_rust_run() {
        let runs = Runs::default();
        let id = runs.register_rust_source().expect("id deve ser registrado");

        assert!(!runs.request_stop(id + 1));
        assert!(runs.request_stop(id));
        let cancellations = runs
            .rust_source_cancellations
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        assert!(cancellations[&id].load(Ordering::Acquire));
    }

    #[test]
    fn each_rust_run_gets_a_unique_id_and_stop_acceptance() {
        let runs = Runs::default();
        let first = runs.register_rust_source().unwrap();
        let second = runs.register_rust_source().unwrap();

        assert_ne!(first, second);
        assert!(runs.request_stop(first));
        assert!(!runs.request_stop(u32::MAX));
    }

    #[test]
    fn portugol_rs_run_registers_a_stoppable_input_channel() {
        let runs = Runs::default();
        let id = runs.register_portugol_rs().unwrap();
        let sender = runs
            .portugol_rs_input_senders
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
            .unwrap()
            .clone();
        let receiver = runs
            .portugol_rs_input_receivers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&id)
            .unwrap();

        sender.send("42".into()).unwrap();
        assert_eq!(receiver.recv().unwrap(), "42");
        assert!(runs.request_stop(id));
    }
}
