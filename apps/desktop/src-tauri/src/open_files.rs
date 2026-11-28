use std::path::PathBuf;
use std::sync::Mutex;

use tauri::{AppHandle, RunEvent, State};

pub(crate) struct PendingFiles(Mutex<Vec<String>>);

/// Arquivos que a IDE abre quando o sistema os entrega (duplo clique,
/// "Abrir com"): programas e exercícios.
fn opens_from_system(path: &std::path::Path) -> bool {
    ferro_formatos::classify_path(path)
        .is_some_and(|m| m.kind().is_program() || m.kind() == ferro_formatos::Kind::Lesson)
}

fn file_paths(paths: impl IntoIterator<Item = PathBuf>) -> Vec<String> {
    paths
        .into_iter()
        .filter(|path| opens_from_system(path) && path.is_file())
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
    fn keeps_existing_ferro_rust_and_lesson_files_from_arguments() {
        let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let lesson = std::env::temp_dir().join(format!("licao-{}.licao", std::process::id()));
        let ferro = std::env::temp_dir().join(format!("Ferro á {}.FER", std::process::id()));
        let rust = std::env::temp_dir().join(format!("Ferro {}.RS", std::process::id()));
        let unsupported = std::env::temp_dir().join(format!("arquivo-{}.zzz", std::process::id()));
        let legacy_por = std::env::temp_dir().join(format!("legado-{}.por", std::process::id()));
        let legacy_pex = std::env::temp_dir().join(format!("legado-{}.pex", std::process::id()));
        std::fs::write(&lesson, "{}").unwrap();
        std::fs::write(&ferro, "funcao inicio() {}").unwrap();
        std::fs::write(&rust, "fn main() {}").unwrap();
        std::fs::write(&unsupported, "funcao inicio() {}").unwrap();
        std::fs::write(&legacy_por, "programa { funcao inicio() {} }").unwrap();
        std::fs::write(&legacy_pex, "{}").unwrap();
        let not_source = root.join("Cargo.toml");
        let paths = file_paths([
            lesson.clone(),
            not_source,
            unsupported.clone(),
            legacy_por.clone(),
            legacy_pex.clone(),
            ferro.clone(),
            rust.clone(),
            root.join("missing.licao"),
        ]);
        assert_eq!(
            paths,
            [
                lesson.to_str().unwrap(),
                ferro.to_str().unwrap(),
                rust.to_str().unwrap()
            ]
        );
        std::fs::remove_file(lesson).unwrap();
        std::fs::remove_file(ferro).unwrap();
        std::fs::remove_file(rust).unwrap();
        std::fs::remove_file(unsupported).unwrap();
        std::fs::remove_file(legacy_por).unwrap();
        std::fs::remove_file(legacy_pex).unwrap();
    }
}
