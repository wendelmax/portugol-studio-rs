//! Ponte entre a janela e o núcleo (`ferro-service`).
//!
//! Comandos chamados pelo frontend:
//!
//! | comando                  | o que faz                                          |
//! |--------------------------|----------------------------------------------------|
//! | `inspect_ferro_rs`    | diagnósticos e símbolos de `.fer`                 |
//! | `debug_ferro_rs`      | compila e executa `.fer` na VM; devolve o id      |
//! | `send_input`             | responde a um `leia`                               |
//! | `stop`                   | interrompe a execução                              |
//! | `debug_*`                | pausa, continua, passos e pontos de parada         |
//! | `read_file`              | lê `.fer`, `.rs` ou `.json` em UTF-8              |
//! | `write_file`             | grava `.fer` ou `.rs` em UTF-8                    |
//!
//! Eventos emitidos durante uma execução (todos com `id`):
//! `run:output` (`text`), `run:clear`, `run:input` (`expected`),
//! `run:paused` (`info`) e `run:finished` (`outcome`).

mod audio;
mod graphics;

mod alerts;
mod open_files;
mod screen_window;
mod updates;
mod workspace;

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Mutex, mpsc};

use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use ferro_service::{
    DebugInfo, DiagnosticInfo, FileFilter, RunHandle, RunObserver, RunOptions, RunOutcome,
};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct Runs {
    next_id: AtomicU32,
    current: Mutex<Option<(u32, RunHandle)>>,
    rust_source_cancellations: Mutex<HashMap<u32, Arc<AtomicBool>>>,
    ferro_rs_input_senders: Mutex<HashMap<u32, mpsc::Sender<String>>>,
    ferro_rs_input_receivers: Mutex<HashMap<u32, mpsc::Receiver<String>>>,
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

    fn register_ferro_rs(&self) -> Result<u32, String> {
        let id = self.register_rust_source()?;
        let (sender, receiver) = mpsc::channel();
        self.ferro_rs_input_senders
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .insert(id, sender);
        self.ferro_rs_input_receivers
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
fn inspect_ferro_rs(source: String) -> ferro_service::Inspection {
    ferro_service::inspect_ferro_rs(&source)
}

/// Diagnósticos e estrutura de um `.design` ou `.ferproj`; o tipo vem da
/// extensão de `path`.
/// Com `check_code`, um `.design` também tem as ligações de eventos
/// conferidas com o code-behind (`code_source`; `None` quando a tela ainda
/// não tem `.fer`).
#[tauri::command]
fn inspect_markup(
    source: String,
    path: String,
    check_code: Option<bool>,
    code_source: Option<String>,
) -> Result<ferro_service::Inspection, String> {
    let kind = ferro_service::MarkupKind::from_path(&path)
        .ok_or_else(|| "O arquivo não é um .design nem um .ferproj.".to_string())?;
    let code = match (check_code.unwrap_or(false), code_source.as_deref()) {
        (false, _) => ferro_service::CodeBehind::Unknown,
        (true, None) => ferro_service::CodeBehind::Missing,
        (true, Some(code)) => ferro_service::CodeBehind::Source(code),
    };
    Ok(ferro_service::inspect_markup_with_code(&source, kind, code))
}

/// Code-behind de uma tela, analisado com os tipos do `.design` dela.
#[tauri::command]
fn inspect_code_behind(
    source: String,
    design_source: String,
    manifest_source: Option<String>,
) -> ferro_service::Inspection {
    ferro_service::inspect_code_behind_in_project(
        &source,
        &design_source,
        manifest_source.as_deref(),
    )
}

#[tauri::command]
fn format_markup(source: String) -> Option<Vec<ferro_service::TextEdit>> {
    ferro_service::format_markup(&source)
}

/// Árvore de um `.design` para o designer visual; `None` com erro de sintaxe.
#[tauri::command]
fn design_tree(source: String) -> Option<ferro_service::DesignNode> {
    ferro_service::design_tree(&source)
}

/// Aplica edições do designer e devolve o novo texto do `.design`.
#[tauri::command]
fn edit_design(source: String, edits: Vec<ferro_service::DesignEdit>) -> Result<String, String> {
    ferro_service::edit_design(&source, edits)
}

/// Campos do registro de `Dados` de uma tela, para os dados de exemplo.
#[tauri::command]
fn screen_data_fields(design_source: String, code_source: String) -> Vec<ferro_service::DataField> {
    ferro_service::screen_data_fields(&design_source, &code_source)
}

/// Projeto desktop de exemplo, para a lista de exemplos da IDE.
#[derive(serde::Serialize)]
struct DesktopExample {
    id: &'static str,
    name: &'static str,
    description: &'static str,
}

/// Projetos desktop de exemplo (o contador, por exemplo).
#[tauri::command]
fn desktop_example_projects() -> Vec<DesktopExample> {
    ferro_tela::project::EXAMPLE_PROJECTS
        .iter()
        .map(|example| DesktopExample {
            id: example.id,
            name: example.name,
            description: example.description,
        })
        .collect()
}

/// Controles e atributos do formato `.design`, para a caixa de ferramentas.
#[tauri::command]
fn design_schema() -> ferro_service::DesignSchema {
    ferro_service::design_schema()
}

#[tauri::command]
async fn export_ferro_rs(source: String) -> Result<ferro_service::RustExport, String> {
    tauri::async_runtime::spawn_blocking(move || ferro_service::export_ferro_rs(&source))
        .await
        .map_err(|error| error.to_string())
}

#[tauri::command]
async fn format_rust_source(source: String) -> Result<String, String> {
    tauri::async_runtime::spawn_blocking(move || ferro_service::format_rust_source(&source))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
fn rustfmt_available() -> bool {
    ferro_service::rustfmt_available()
}

#[tauri::command]
fn rust_toolchain_available() -> bool {
    ferro_service::rust_toolchain_available()
}

#[tauri::command]
async fn check_rust_source(
    source: String,
) -> Result<Vec<ferro_service::RustSourceCompileDiagnostic>, String> {
    tauri::async_runtime::spawn_blocking(move || ferro_service::check_rust_source(&source))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
async fn run_ferro_rs(
    app: AppHandle,
    runs: State<'_, Runs>,
    id: u32,
    source: String,
) -> Result<ferro_service::FerroRustRunResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução Ferro não está mais registrada.".to_string())?;
    let receiver = runs
        .ferro_rs_input_receivers
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id)
        .ok_or_else(|| "O canal de entrada Ferro não está mais registrado.".to_string())?;
    let input_app = app.clone();
    let output_app = app;
    let joined = tauri::async_runtime::spawn_blocking(move || {
        ferro_service::run_ferro_rs_with_io(
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
                            return Err("O canal de entrada Ferro foi encerrado.".into());
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
    runs.ferro_rs_input_senders
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
fn register_ferro_rs_run(runs: State<'_, Runs>) -> Result<u32, String> {
    runs.register_ferro_rs()
}

#[tauri::command]
async fn run_rust_source(
    runs: State<'_, Runs>,
    id: u32,
    source: String,
    args: Vec<String>,
) -> Result<ferro_service::RustSourceRunResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução Rust não está mais registrada.".to_string())?;
    let joined = tauri::async_runtime::spawn_blocking(move || {
        ferro_service::run_rust_source_with_args_and_cancel(&source, &args, &cancellation)
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
    ferro_service::cargo_available()
}

#[tauri::command]
async fn run_cargo_project(
    runs: State<'_, Runs>,
    workspace: State<'_, workspace::WorkspaceSession>,
    id: u32,
    workspace_id: String,
    relative_source: String,
    target: Option<ferro_service::CargoRunTarget>,
    args: Vec<String>,
) -> Result<ferro_service::CargoProjectRunResult, String> {
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
        ferro_service::run_cargo_project_target_with_args_and_cancel(
            &project_root,
            target.as_ref(),
            &args,
            &cancellation,
        )
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
async fn cargo_project_run_targets(
    runs: State<'_, Runs>,
    workspace: State<'_, workspace::WorkspaceSession>,
    id: u32,
    workspace_id: String,
    relative_source: String,
) -> Result<ferro_service::CargoRunTargets, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A descoberta de alvos Cargo não está mais registrada.".to_string())?;
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
        ferro_service::cargo_run_targets_with_cancel(&project_root, &cancellation)
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
async fn check_cargo_project(
    runs: State<'_, Runs>,
    workspace: State<'_, workspace::WorkspaceSession>,
    id: u32,
    workspace_id: String,
    relative_source: String,
) -> Result<ferro_service::CargoProjectCheckResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A análise Cargo não está mais registrada.".to_string())?;
    let (project_root, source_path) =
        match workspace::cargo_project_and_source(&workspace, &workspace_id, &relative_source) {
            Ok(Some(paths)) => paths,
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
        ferro_service::check_cargo_project_with_cancel(&project_root, &source_path, &cancellation)
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
async fn test_cargo_project(
    runs: State<'_, Runs>,
    workspace: State<'_, workspace::WorkspaceSession>,
    id: u32,
    workspace_id: String,
    relative_source: String,
) -> Result<ferro_service::CargoProjectTestResult, String> {
    let cancellation = runs
        .rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .get(&id)
        .cloned()
        .ok_or_else(|| "A execução de testes Cargo não está mais registrada.".to_string())?;
    let project_root =
        match workspace::cargo_project_and_source(&workspace, &workspace_id, &relative_source) {
            Ok(Some((root, _))) => root,
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
        ferro_service::test_cargo_project_with_cancel(&project_root, &cancellation)
    })
    .await;
    runs.rust_source_cancellations
        .lock()
        .unwrap_or_else(|error| error.into_inner())
        .remove(&id);
    joined.map_err(|error| error.to_string())?
}

#[tauri::command]
fn format_ferro_rs(source: String) -> Option<Vec<ferro_service::TextEdit>> {
    ferro_service::format_ferro_rs(&source)
}

/// Formata o code-behind de uma tela com os tipos do `.design` dela.
#[tauri::command]
fn format_code_behind(
    source: String,
    design_source: String,
) -> Option<Vec<ferro_service::TextEdit>> {
    ferro_service::format_code_behind(&source, &design_source)
}

#[tauri::command]
fn rename_target_ferro_rs(
    source: String,
    offset: u32,
) -> Result<ferro_service::RenameTarget, String> {
    ferro_service::rename_target_ferro_rs(&source, offset)
}

#[tauri::command]
fn resolve_symbol_ferro_rs(
    source: String,
    offset: u32,
) -> Result<ferro_service::SymbolLocation, String> {
    ferro_service::resolve_symbol_ferro_rs(&source, offset)
}

#[tauri::command]
fn rename_symbol_ferro_rs(
    source: String,
    offset: u32,
    name: String,
) -> Result<Vec<ferro_service::RenameEdit>, String> {
    ferro_service::rename_symbol_ferro_rs(&source, offset, &name)
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
        ProjectPage::BugReport => "https://github.com/wendelmax/ferro-studio/issues/new",
        ProjectPage::Releases => "https://github.com/wendelmax/ferro-studio/releases",
        ProjectPage::Contribute => "https://github.com/wendelmax/ferro-studio",
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

/// Executa ou depura Ferro na VM educacional, sem depender de `rustc`.
/// Com `pause_on_start` falso é a execução normal: para apenas nos pontos de
/// parada.
#[tauri::command]
fn debug_ferro_rs(
    app: AppHandle,
    runs: State<'_, Runs>,
    source: String,
    breakpoints: Vec<u32>,
    pause_on_start: Option<bool>,
    path: Option<String>,
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
    // `Arquivos.*` e ativos multimídia ficam confinados à pasta do programa.
    let working_dir = path
        .as_deref()
        .and_then(|p| std::path::Path::new(p).parent())
        .map(std::path::Path::to_path_buf);
    let options = RunOptions {
        breakpoints,
        pause_on_start: pause_on_start.unwrap_or(true),
        working_dir,
        ..RunOptions::default()
    };
    let handle =
        ferro_service::start_ferro_rs_with_host(&source, options, observer, move |interrupt| {
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
            .ferro_rs_input_senders
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

const MAX_STANDALONE_FILE_BYTES: u64 = 16 * 1024 * 1024;
/// O que pode ser lido fora de um workspace: programas, exercícios e JSON de
/// exercício importado com outro nome.
fn standalone_readable(path: &std::path::Path) -> bool {
    use ferro_formatos::Kind;
    ferro_formatos::classify_path(path)
        .is_some_and(|m| m.kind().is_program() || m.kind() == Kind::Lesson)
        || path
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("json"))
}

/// O que pode ser gravado fora de um workspace: só programas.
fn standalone_writable(path: &std::path::Path) -> bool {
    ferro_formatos::classify_path(path).is_some_and(|m| m.kind().is_program())
}

fn validate_standalone_file_path<'a>(
    raw_path: &'a str,
    allowed: fn(&std::path::Path) -> bool,
    action: &str,
) -> Result<&'a std::path::Path, String> {
    if raw_path.trim().is_empty() || raw_path.contains('\0') {
        return Err(format!("Caminho inválido para {action}."));
    }
    let path = std::path::Path::new(raw_path);
    if ferro_formatos::is_retired_path(path) {
        return Err(format!(
            "Arquivos legados .por e .pex não são suportados no Ferro Studio. Utilize arquivos {}.",
            ferro_formatos::describe(&[
                ferro_formatos::Kind::Source,
                ferro_formatos::Kind::Rust,
                ferro_formatos::Kind::Lesson
            ])
        ));
    }
    if !allowed(path) {
        return Err(format!(
            "Tipo de arquivo não suportado para {action}: {raw_path}"
        ));
    }
    Ok(path)
}

/// Os exemplos clássicos estão em ISO-8859-1; aceitamos
/// UTF-8 (com ou sem BOM) e caímos para Latin-1 quando necessário.
#[tauri::command]
fn read_file(path: String) -> Result<String, String> {
    let target = validate_standalone_file_path(&path, standalone_readable, "leitura")?;
    let metadata =
        std::fs::metadata(target).map_err(|e| format!("não foi possível ler {path}: {e}"))?;
    if !metadata.is_file() {
        return Err(format!(
            "não foi possível ler {path}: o caminho não é um arquivo regular"
        ));
    }
    if metadata.len() > MAX_STANDALONE_FILE_BYTES {
        return Err(format!(
            "não foi possível ler {path}: o arquivo excede o limite de {} MiB",
            MAX_STANDALONE_FILE_BYTES / (1024 * 1024)
        ));
    }
    let bytes = std::fs::read(target).map_err(|e| format!("não foi possível ler {path}: {e}"))?;
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
    let target = validate_standalone_file_path(&path, standalone_writable, "gravação")?;
    if (text.len() as u64) > MAX_STANDALONE_FILE_BYTES {
        return Err(format!(
            "não foi possível gravar {path}: o conteúdo excede o limite de {} MiB",
            MAX_STANDALONE_FILE_BYTES / (1024 * 1024)
        ));
    }
    if let Ok(metadata) = std::fs::metadata(target)
        && !metadata.is_file()
    {
        return Err(format!(
            "não foi possível gravar {path}: o destino não é um arquivo regular"
        ));
    }
    std::fs::write(target, text).map_err(|e| format!("não foi possível gravar {path}: {e}"))
}

#[tauri::command]
fn ferro_example_catalog() -> Vec<ferro_service::FerroExample> {
    ferro_service::ferro_examples().to_vec()
}

#[tauri::command]
fn ferro_exercise_catalog() -> Vec<ferro_service::FerroExerciseSummary> {
    ferro_service::ferro_exercises()
}

#[tauri::command]
async fn check_ferro_exercise(
    id: Option<String>,
    source: String,
    exercise_json: Option<String>,
) -> Result<ferro_service::FerroExerciseEvaluation, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if let Some(raw_json) = exercise_json {
            ferro_service::check_imported_ferro_exercise(&raw_json, &source)
        } else if let Some(id) = id {
            ferro_service::check_ferro_exercise(&id, &source)
        } else {
            Err("Informe o identificador ou o JSON do exercício .fer.".into())
        }
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
fn inspect_ferro_exercise(raw_json: String) -> Result<ferro_service::FerroExerciseSummary, String> {
    ferro_service::inspect_ferro_exercise(&raw_json)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run_app() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
        .manage(Runs::default())
        .manage(workspace::WorkspaceSession::default())
        .manage(open_files::PendingFiles::from_args())
        .manage(graphics::GraphicsWindows::default())
        .manage(screen_window::ScreenWindows::default())
        .invoke_handler(tauri::generate_handler![
            inspect_ferro_rs,
            inspect_markup,
            inspect_code_behind,
            format_markup,
            design_tree,
            edit_design,
            design_schema,
            desktop_example_projects,
            screen_data_fields,
            export_ferro_rs,
            format_rust_source,
            rustfmt_available,
            rust_toolchain_available,
            check_rust_source,
            run_ferro_rs,
            register_rust_source_run,
            register_ferro_rs_run,
            run_rust_source,
            cargo_toolchain_available,
            cargo_project_run_targets,
            run_cargo_project,
            check_cargo_project,
            test_cargo_project,
            format_ferro_rs,
            format_code_behind,
            ferro_example_catalog,
            ferro_exercise_catalog,
            check_ferro_exercise,
            inspect_ferro_exercise,
            rename_target_ferro_rs,
            rename_symbol_ferro_rs,
            resolve_symbol_ferro_rs,
            open_project_page,
            updates::check_updates,
            alerts::project_alerts,
            debug_ferro_rs,
            send_input,
            stop,
            debug_pause,
            debug_continue,
            debug_step_over,
            debug_step_into,
            debug_step_out,
            debug_set_breakpoints,
            screen_window::screen_run_start,
            screen_window::screen_export_cargo,
            screen_window::screen_run_state,
            screen_window::screen_run_dispatch,
            graphics::graphics_next,
            graphics::graphics_reply,
            graphics::graphics_input,
            read_file,
            open_files::pending_files,
            write_file,
            workspace::workspace_open,
            workspace::workspace_create_desktop_project,
            workspace::workspace_refresh,
            workspace::workspace_cargo_project_available,
            workspace::workspace_read_file,
            workspace::workspace_write_file,
            workspace::workspace_create_file,
            workspace::workspace_restore,
            workspace::workspace_rename_file,
            workspace::workspace_delete_file,
            workspace::workspace_export,
            workspace::export_cargo_project
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
    fn ferro_rs_run_registers_a_stoppable_input_channel() {
        let runs = Runs::default();
        let id = runs.register_ferro_rs().unwrap();
        let sender = runs
            .ferro_rs_input_senders
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .get(&id)
            .unwrap()
            .clone();
        let receiver = runs
            .ferro_rs_input_receivers
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .remove(&id)
            .unwrap();

        sender.send("42".into()).unwrap();
        assert_eq!(receiver.recv().unwrap(), "42");
        assert!(runs.request_stop(id));
    }

    #[test]
    fn standalone_file_commands_allow_supported_extensions_and_reject_others() {
        let dir = std::env::temp_dir().join(format!(
            "ferro-standalone-file-guard-{}",
            std::process::id()
        ));
        std::fs::create_dir_all(&dir).unwrap();

        let ferro = dir.join("exemplo.fer");
        let legacy_por = dir.join("legado.por");
        let legacy_pex = dir.join("legado.pex");
        let secret = dir.join("segredo.env");
        let script = dir.join("shell.sh");
        let exercise = dir.join("ex.json");
        std::fs::write(&legacy_por, "programa {}").unwrap();
        std::fs::write(&legacy_pex, "{}").unwrap();
        std::fs::write(&secret, "SECRET=1").unwrap();
        std::fs::write(&exercise, "{}").unwrap();

        super::write_file(ferro.display().to_string(), "funcao inicio() {}".into()).unwrap();
        assert_eq!(
            super::read_file(ferro.display().to_string()).unwrap(),
            "funcao inicio() {}"
        );
        assert_eq!(
            super::read_file(exercise.display().to_string()).unwrap(),
            "{}"
        );

        assert!(
            super::read_file(legacy_por.display().to_string())
                .unwrap_err()
                .contains("não são suportados")
        );
        assert!(
            super::read_file(legacy_pex.display().to_string())
                .unwrap_err()
                .contains("não são suportados")
        );
        assert!(super::write_file(legacy_por.display().to_string(), "programa {}".into()).is_err());
        assert!(super::read_file(secret.display().to_string()).is_err());
        assert!(super::write_file(script.display().to_string(), "echo hi".into()).is_err());
        assert!(super::write_file(exercise.display().to_string(), "{}".into()).is_err());
        assert!(super::read_file(dir.display().to_string()).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
