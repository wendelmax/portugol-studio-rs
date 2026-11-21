use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, RunEvent, State};

pub(crate) struct PendingFiles(Mutex<Vec<String>>);

fn file_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| {
            path.extension().is_some_and(|extension| {
                extension.eq_ignore_ascii_case("por") || extension.eq_ignore_ascii_case("pex")
            }) && path.is_file()
        })
        .filter_map(|path| path.into_os_string().into_string().ok())
        .collect()
}

impl PendingFiles {
    pub(crate) fn from_args() -> Self {
        let paths = std::env::args_os().skip(1).map(PathBuf::from);
        Self(Mutex::new(file_paths(paths)))
    }
}

#[tauri::command]
pub(crate) fn pending_files(files: State<'_, PendingFiles>) -> Vec<String> {
    let mut list = files.0.lock().unwrap_or_else(|error| error.into_inner());
    std::mem::take(&mut *list)
}

pub(crate) fn application_event(app: &AppHandle, event: RunEvent) {
    #[cfg(target_os = "macos")]
    if let RunEvent::Opened { urls } = event {
        use tauri::{Emitter, Manager};
        let paths = file_paths(urls.into_iter().filter_map(|url| url.to_file_path().ok()));
        if !paths.is_empty() {
            let state = app.state::<PendingFiles>();
            {
                let mut list = state.0.lock().unwrap_or_else(|error| error.into_inner());
                list.extend(paths);
            }
            let _ = app.emit("project-files-opened", ());
        }
    }
    #[cfg(not(target_os = "macos"))]
    let _ = (app, event);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_existing_programs_and_exercises_from_the_arguments() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let base = root.join("crates/portugol-service/assets/exercicios");
        let pex = base.join("exercicio01.pex");
        let por = root.join("tests/corpus/exemplos/jogos/pong.por");
        let not_source = root.join("Cargo.toml");
        let paths = file_paths([
            pex.clone(),
            not_source,
            por.clone(),
            base.join("missing.pex"),
        ]);
        assert_eq!(paths, [pex.to_str().unwrap(), por.to_str().unwrap()]);
    }
}
