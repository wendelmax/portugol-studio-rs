#[cfg(test)]
mod tests {
    use super::{
        WorkspaceEntry, create_source_file, delete_relative_file, list_entries, read_relative_file,
        rename_relative_file, restore_saved_workspace, save_last_workspace,
    };
    use std::fs;
    use std::path::Path;
    use std::sync::atomic::{AtomicUsize, Ordering};

    static NEXT_DIR: AtomicUsize = AtomicUsize::new(0);

    struct TempDir(std::path::PathBuf);

    impl TempDir {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!(
                "portugol-workspace-test-{}-{}",
                std::process::id(),
                NEXT_DIR.fetch_add(1, Ordering::Relaxed)
            ));
            fs::create_dir_all(&path).unwrap();
            Self(path)
        }
    }

    impl Drop for TempDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn file_paths(entries: &[WorkspaceEntry], output: &mut Vec<String>) {
        for entry in entries {
            match entry {
                WorkspaceEntry::Folder { path, entries, .. } => {
                    assert!(!Path::new(path).is_absolute());
                    file_paths(entries, output);
                }
                WorkspaceEntry::File { path, .. } => output.push(path.clone()),
            }
        }
    }

    #[test]
    fn lists_supported_files_recursively_and_ignores_other_extensions() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aula/subpasta")).unwrap();
        fs::write(temp.0.join("inicio.POR"), "programa").unwrap();
        fs::write(temp.0.join("aula/exercicio.pEx"), "atividade").unwrap();
        fs::write(temp.0.join("aula/subpasta/rust.pors"), "funcao").unwrap();
        fs::write(temp.0.join("notas.txt"), "ignorar").unwrap();

        let entries = list_entries(&temp.0).unwrap();
        let mut paths = Vec::new();
        file_paths(&entries, &mut paths);
        paths.sort();

        assert_eq!(
            paths,
            [
                "aula/exercicio.pEx",
                "aula/subpasta/rust.pors",
                "inicio.POR"
            ]
        );
    }

    #[test]
    fn reads_a_supported_file_relative_to_the_workspace_root() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aula")).unwrap();
        fs::write(temp.0.join("aula/programa.por"), "escreva(42)").unwrap();

        let content = read_relative_file(&temp.0, Path::new("aula/programa.por")).unwrap();

        assert_eq!(content, "escreva(42)");
    }

    #[test]
    fn rejects_path_traversal_and_unsupported_files() {
        let temp = TempDir::new();
        let root = temp.0.join("projeto");
        fs::create_dir_all(&root).unwrap();
        fs::write(temp.0.join("fora.por"), "segredo").unwrap();
        fs::write(temp.0.join("notas.txt"), "não suportado").unwrap();

        assert!(read_relative_file(&root, Path::new("../fora.por")).is_err());
        assert!(read_relative_file(&temp.0, Path::new("notas.txt")).is_err());
    }

    #[test]
    fn only_saves_existing_portugol_source_files() {
        let temp = TempDir::new();
        fs::write(temp.0.join("programa.por"), "antes").unwrap();
        fs::write(temp.0.join("atividade.pex"), "exercício").unwrap();

        super::write_relative_file(&temp.0, Path::new("programa.por"), "depois").unwrap();

        assert_eq!(
            fs::read_to_string(temp.0.join("programa.por")).unwrap(),
            "depois"
        );
        assert!(
            super::write_relative_file(&temp.0, Path::new("atividade.pex"), "alterado").is_err()
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("atividade.pex")).unwrap(),
            "exercício"
        );
    }

    #[test]
    fn creates_empty_portugol_and_portugol_rs_files_exclusively() {
        let temp = TempDir::new();

        create_source_file(&temp.0, "ola.por").unwrap();
        create_source_file(&temp.0, "ponte.pors").unwrap();

        assert_eq!(fs::read_to_string(temp.0.join("ola.por")).unwrap(), "");
        assert_eq!(fs::read_to_string(temp.0.join("ponte.pors")).unwrap(), "");
    }

    #[test]
    fn rejects_invalid_names_and_never_overwrites_an_existing_file() {
        let temp = TempDir::new();
        fs::write(temp.0.join("aula.por"), "conteúdo existente").unwrap();

        for invalid in [
            "",
            ".",
            "..",
            "../fora.por",
            "subpasta\\programa.por",
            "dados.pex",
        ] {
            assert!(create_source_file(&temp.0, invalid).is_err(), "{invalid}");
        }
        assert!(create_source_file(&temp.0, "aula.por").is_err());
        assert_eq!(
            fs::read_to_string(temp.0.join("aula.por")).unwrap(),
            "conteúdo existente"
        );
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_replace_a_symlink_when_creating_a_source_file() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("segredo"), "não alterar").unwrap();
        symlink(outside.0.join("segredo"), temp.0.join("novo.por")).unwrap();

        assert!(create_source_file(&temp.0, "novo.por").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("segredo")).unwrap(),
            "não alterar"
        );
    }

    #[test]
    fn renames_a_source_file_without_changing_its_contents() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aulas")).unwrap();
        fs::write(temp.0.join("aulas/antigo.por"), "escreva(42)").unwrap();

        let renamed =
            rename_relative_file(&temp.0, Path::new("aulas/antigo.por"), "novo.por").unwrap();

        let expected = fs::canonicalize(temp.0.join("aulas/novo.por")).unwrap();
        assert_eq!(renamed, expected);
        assert_eq!(fs::read_to_string(&renamed).unwrap(), "escreva(42)");
        assert!(!temp.0.join("aulas/antigo.por").exists());
    }

    #[test]
    fn preserves_original_extension_and_rejects_path_changes() {
        let temp = TempDir::new();
        fs::write(temp.0.join("programa.pors"), "funcao inicio() {}").unwrap();

        for invalid in [
            "novo.por",
            "../fora.pors",
            "pasta\\novo.pors",
            "anotacao.txt",
        ] {
            assert!(
                rename_relative_file(&temp.0, Path::new("programa.pors"), invalid).is_err(),
                "{invalid}"
            );
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("programa.pors")).unwrap(),
            "funcao inicio() {}"
        );
    }

    #[test]
    fn rename_collision_never_overwrites_either_file() {
        let temp = TempDir::new();
        fs::write(temp.0.join("antes.por"), "origem").unwrap();
        fs::write(temp.0.join("depois.por"), "destino").unwrap();

        assert!(rename_relative_file(&temp.0, Path::new("antes.por"), "depois.por").is_err());
        assert_eq!(
            fs::read_to_string(temp.0.join("antes.por")).unwrap(),
            "origem"
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("depois.por")).unwrap(),
            "destino"
        );
    }

    #[test]
    fn deletes_supported_files_and_preserves_neighbors() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("aulas")).unwrap();
        fs::write(temp.0.join("aulas/classico.por"), "programa").unwrap();
        fs::write(temp.0.join("aulas/ponte.pors"), "funcao inicio() {}").unwrap();
        fs::write(temp.0.join("aulas/exercicio.pex"), "enunciado").unwrap();
        fs::write(temp.0.join("aulas/manter.por"), "manter").unwrap();

        for name in ["classico.por", "ponte.pors", "exercicio.pex"] {
            let relative = Path::new("aulas").join(name);
            let expected = fs::canonicalize(temp.0.join("aulas").join(name)).unwrap();
            let deleted = delete_relative_file(&temp.0, &relative).unwrap();
            assert_eq!(deleted, expected);
            assert!(!temp.0.join("aulas").join(name).exists());
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("aulas/manter.por")).unwrap(),
            "manter"
        );
    }

    #[test]
    fn delete_rejects_traversal_directories_missing_and_unsupported_files() {
        let temp = TempDir::new();
        fs::create_dir_all(temp.0.join("pasta.por")).unwrap();
        fs::write(temp.0.join("notas.txt"), "manter").unwrap();
        fs::write(temp.0.join("programa.por"), "manter").unwrap();

        for relative in [
            Path::new("../fora.por"),
            Path::new("pasta.por"),
            Path::new("ausente.por"),
            Path::new("notas.txt"),
        ] {
            assert!(
                delete_relative_file(&temp.0, relative).is_err(),
                "{relative:?}"
            );
        }
        assert_eq!(
            fs::read_to_string(temp.0.join("programa.por")).unwrap(),
            "manter"
        );
        assert_eq!(
            fs::read_to_string(temp.0.join("notas.txt")).unwrap(),
            "manter"
        );
        assert!(temp.0.join("pasta.por").is_dir());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_delete_a_symlink_outside_the_workspace() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("programa.por"), "fora").unwrap();
        symlink(outside.0.join("programa.por"), temp.0.join("ligacao.por")).unwrap();

        assert!(delete_relative_file(&temp.0, Path::new("ligacao.por")).is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("programa.por")).unwrap(),
            "fora"
        );
        assert!(temp.0.join("ligacao.por").exists());
    }

    #[cfg(unix)]
    #[test]
    fn refuses_to_rename_a_symlink_outside_the_workspace() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("programa.por"), "fora").unwrap();
        symlink(outside.0.join("programa.por"), temp.0.join("ligacao.por")).unwrap();

        assert!(rename_relative_file(&temp.0, Path::new("ligacao.por"), "novo.por").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("programa.por")).unwrap(),
            "fora"
        );
        assert!(temp.0.join("ligacao.por").exists());
    }

    #[test]
    fn saves_the_canonical_workspace_path_for_the_next_session() {
        let config = TempDir::new();
        let root = TempDir::new();
        fs::create_dir_all(root.0.join("Aulas com acento")).unwrap();
        let workspace = root.0.join("Aulas com acento");

        save_last_workspace(&config.0, &workspace).unwrap();

        assert_eq!(
            super::load_last_workspace(&config.0).unwrap(),
            Some(fs::canonicalize(workspace).unwrap())
        );
    }

    #[test]
    fn restores_a_saved_workspace_with_a_fresh_session_id() {
        let config = TempDir::new();
        let root = TempDir::new();
        fs::write(root.0.join("programa.pors"), "funcao inicio() {}").unwrap();
        save_last_workspace(&config.0, &root.0).unwrap();

        let session = super::WorkspaceSession::default();
        let restored = restore_saved_workspace(&session, &config.0)
            .unwrap()
            .unwrap();

        assert_eq!(
            restored.name,
            root.0.file_name().unwrap().to_string_lossy().into_owned()
        );
        assert_eq!(restored.id, "workspace-1");
        assert!(
            matches!(&restored.entries[0], WorkspaceEntry::File { name, .. } if name == "programa.pors")
        );
    }

    #[test]
    fn no_saved_workspace_returns_none_without_an_error() {
        let config = TempDir::new();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
    }

    #[test]
    fn a_removed_workspace_is_forgotten_after_restore() {
        let config = TempDir::new();
        let root = TempDir::new();
        save_last_workspace(&config.0, &root.0).unwrap();
        fs::remove_dir_all(&root.0).unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(super::load_last_workspace(&config.0).unwrap().is_none());
    }

    #[test]
    fn malformed_saved_state_is_discarded_without_blocking_startup() {
        let config = TempDir::new();
        let saved_path = config.0.join(super::LAST_WORKSPACE_FILE);
        fs::write(&saved_path, "not valid JSON").unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(!saved_path.exists());
    }

    #[test]
    fn relative_saved_paths_are_discarded() {
        let config = TempDir::new();
        let saved_path = config.0.join(super::LAST_WORKSPACE_FILE);
        fs::write(&saved_path, serde_json::to_vec("relative/project").unwrap()).unwrap();
        let session = super::WorkspaceSession::default();

        assert!(
            restore_saved_workspace(&session, &config.0)
                .unwrap()
                .is_none()
        );
        assert!(!saved_path.exists());
    }

    #[cfg(unix)]
    #[test]
    fn skips_symlinks_and_rejects_symlink_reads() {
        use std::os::unix::fs::symlink;

        let temp = TempDir::new();
        let outside = TempDir::new();
        fs::write(outside.0.join("segredo.por"), "segredo").unwrap();
        symlink(outside.0.join("segredo.por"), temp.0.join("link.por")).unwrap();
        symlink(&outside.0, temp.0.join("pasta-link")).unwrap();

        assert!(list_entries(&temp.0).unwrap().is_empty());
        assert!(read_relative_file(&temp.0, Path::new("link.por")).is_err());
        assert!(super::write_relative_file(&temp.0, Path::new("link.por"), "alterado").is_err());
        assert_eq!(
            fs::read_to_string(outside.0.join("segredo.por")).unwrap(),
            "segredo"
        );
    }
}
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Mutex};

use serde::Serialize;
use tauri::{AppHandle, Manager, State};
use tauri_plugin_dialog::DialogExt;

#[derive(Default)]
struct WorkspaceState {
    next_id: u64,
    roots: HashMap<String, PathBuf>,
    ids: HashMap<PathBuf, String>,
}

#[derive(Default, Clone)]
pub struct WorkspaceSession(Arc<Mutex<WorkspaceState>>);

impl WorkspaceSession {
    fn register(&self, root: PathBuf) -> String {
        let mut state = self.0.lock().unwrap_or_else(|error| error.into_inner());
        if let Some(id) = state.ids.get(&root) {
            return id.clone();
        }
        state.next_id += 1;
        let next_id = state.next_id;
        let id = format!("workspace-{next_id}");
        state.ids.insert(root.clone(), id.clone());
        state.roots.insert(id.clone(), root);
        id
    }

    fn root(&self, id: &str) -> Result<PathBuf, String> {
        self.0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .roots
            .get(id)
            .cloned()
            .ok_or_else(|| {
                "Esta pasta não está mais aberta. Abra-a novamente para continuar.".into()
            })
    }
}

#[derive(Serialize, Clone, Debug, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum WorkspaceEntry {
    Folder {
        name: String,
        path: String,
        entries: Vec<WorkspaceEntry>,
    },
    File {
        name: String,
        path: String,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceListing {
    id: String,
    name: String,
    entries: Vec<WorkspaceEntry>,
}

const LAST_WORKSPACE_FILE: &str = "last-workspace.json";

fn save_last_workspace(config_dir: &Path, root: &Path) -> Result<(), String> {
    let root = fs::canonicalize(root)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    let root = root.to_str().ok_or_else(|| {
        "O caminho do projeto não pode ser representado nesta interface.".to_string()
    })?;
    let encoded = serde_json::to_vec(root)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    fs::create_dir_all(config_dir)
        .map_err(|error| format!("Não foi possível acessar as preferências da IDE: {error}"))?;

    let saved_path = config_dir.join(LAST_WORKSPACE_FILE);
    let temporary_path = config_dir.join(format!("{LAST_WORKSPACE_FILE}.tmp"));
    fs::write(&temporary_path, encoded)
        .map_err(|error| format!("Não foi possível salvar a pasta do projeto: {error}"))?;
    match fs::rename(&temporary_path, &saved_path) {
        Ok(()) => Ok(()),
        Err(error) if saved_path.exists() => {
            fs::remove_file(&saved_path).map_err(|remove_error| {
                format!("Não foi possível atualizar a pasta salva: {remove_error}")
            })?;
            fs::rename(&temporary_path, &saved_path).map_err(|rename_error| {
                format!("Não foi possível salvar a pasta do projeto: {rename_error}; {error}")
            })
        }
        Err(error) => Err(format!(
            "Não foi possível salvar a pasta do projeto: {error}"
        )),
    }
}

fn load_last_workspace(config_dir: &Path) -> Result<Option<PathBuf>, String> {
    let saved_path = config_dir.join(LAST_WORKSPACE_FILE);
    let encoded = match fs::read(&saved_path) {
        Ok(encoded) => encoded,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => {
            return Err(format!(
                "Não foi possível ler as preferências da IDE: {error}"
            ));
        }
    };
    let Ok(path) = serde_json::from_slice::<String>(&encoded) else {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    };
    if path.is_empty() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    let path = PathBuf::from(path);
    if !path.is_absolute() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    Ok(Some(path))
}

fn clear_last_workspace(config_dir: &Path) -> Result<(), String> {
    match fs::remove_file(config_dir.join(LAST_WORKSPACE_FILE)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(format!(
            "Não foi possível limpar a preferência de projeto: {error}"
        )),
    }
}

fn restore_saved_workspace(
    session: &WorkspaceSession,
    config_dir: &Path,
) -> Result<Option<WorkspaceListing>, String> {
    let Some(saved_root) = load_last_workspace(config_dir)? else {
        return Ok(None);
    };
    let Ok(root) = fs::canonicalize(saved_root) else {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    };
    if !root.is_dir() {
        clear_last_workspace(config_dir)?;
        return Ok(None);
    }
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let id = session.register(root);
    Ok(Some(WorkspaceListing { id, name, entries }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFile {
    path: String,
    relative_path: String,
    source: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFilePath {
    path: String,
    relative_path: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceRenameResult {
    listing: WorkspaceListing,
    file: WorkspaceFilePath,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceDeleteResult {
    listing: WorkspaceListing,
    file: WorkspaceFilePath,
}

fn supported_source(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "por" | "pors" | "pex"
            )
        })
}

fn supported_program(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| matches!(extension.to_ascii_lowercase().as_str(), "por" | "pors"))
}

fn relative_path(root: &Path, path: &Path) -> Result<String, String> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| "O caminho está fora da pasta aberta.".to_string())?;
    relative
        .to_str()
        .map(|path| path.replace(std::path::MAIN_SEPARATOR, "/"))
        .ok_or_else(|| "O nome do arquivo não pode ser representado nesta interface.".to_string())
}

fn list_folder(root: &Path, folder: &Path) -> Result<Vec<WorkspaceEntry>, String> {
    let entries = fs::read_dir(folder)
        .map_err(|error| format!("Não foi possível listar a pasta: {error}"))?;
    let mut ordered = BTreeMap::new();
    for entry in entries {
        let entry =
            entry.map_err(|error| format!("Não foi possível ler uma entrada da pasta: {error}"))?;
        let file_type = entry
            .file_type()
            .map_err(|error| format!("Não foi possível inspecionar uma entrada: {error}"))?;
        if file_type.is_symlink() || entry.file_name().to_str().is_none() {
            continue;
        }
        let path = entry.path();
        if file_type.is_dir() {
            let children = list_folder(root, &path)?;
            if !children.is_empty() {
                ordered.insert(entry.file_name(), (path, Some(children)));
            }
        } else if file_type.is_file() && supported_source(&path) {
            ordered.insert(entry.file_name(), (path, None));
        }
    }

    ordered
        .into_values()
        .map(|(path, children)| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| {
                    "O nome do arquivo não pode ser representado nesta interface.".to_string()
                })?
                .to_string();
            let path = relative_path(root, &path)?;
            Ok(match children {
                Some(entries) => WorkspaceEntry::Folder {
                    name,
                    path,
                    entries,
                },
                None => WorkspaceEntry::File { name, path },
            })
        })
        .collect()
}

fn list_entries(root: &Path) -> Result<Vec<WorkspaceEntry>, String> {
    if !root.is_dir() {
        return Err("A pasta selecionada não está disponível.".into());
    }
    list_folder(root, root)
}

fn checked_file(root: &Path, relative: &Path, allow_exercise: bool) -> Result<PathBuf, String> {
    if relative.as_os_str().is_empty()
        || relative.is_absolute()
        || relative
            .components()
            .any(|component| !matches!(component, Component::Normal(_)))
    {
        return Err("O caminho do arquivo é inválido.".into());
    }
    let root = fs::canonicalize(root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let mut candidate = root.clone();
    let components: Vec<_> = relative.components().collect();
    for (index, component) in components.iter().enumerate() {
        let Component::Normal(name) = component else {
            return Err("O caminho do arquivo é inválido.".into());
        };
        candidate.push(name);
        let metadata = fs::symlink_metadata(&candidate)
            .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
        if metadata.file_type().is_symlink() {
            return Err("Links simbólicos não podem ser abertos pelo explorador.".into());
        }
        if index + 1 < components.len() && !metadata.is_dir() {
            return Err("O caminho do arquivo é inválido.".into());
        }
    }
    let metadata =
        fs::metadata(&candidate).map_err(|_| "O arquivo não está mais disponível.".to_string())?;
    if !metadata.is_file()
        || !(supported_program(&candidate) || allow_exercise && is_exercise(&candidate))
    {
        return Err("O explorador só abre arquivos .por, .pors e .pex.".into());
    }
    let canonical = fs::canonicalize(&candidate)
        .map_err(|_| "O arquivo não está mais disponível.".to_string())?;
    if !canonical.starts_with(&root) {
        return Err("O arquivo está fora da pasta aberta.".into());
    }
    Ok(canonical)
}

fn is_exercise(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("pex"))
}

#[cfg(test)]
fn read_relative_file(root: &Path, relative: &Path) -> Result<String, String> {
    let path = checked_file(root, relative, true)?;
    let bytes =
        fs::read(path).map_err(|error| format!("Não foi possível ler o arquivo: {error}"))?;
    Ok(super::decode_text(bytes))
}

fn write_relative_file(root: &Path, relative: &Path, source: &str) -> Result<(), String> {
    if !supported_program(relative) {
        return Err("Apenas arquivos .por e .pors podem ser salvos pelo explorador.".into());
    }
    let path = checked_file(root, relative, false)?;
    fs::write(path, source).map_err(|error| format!("Não foi possível salvar o arquivo: {error}"))
}

fn create_source_file(root: &Path, file_name: &str) -> Result<(), String> {
    let relative = Path::new(file_name);
    if file_name.trim().is_empty()
        || file_name == "."
        || file_name == ".."
        || file_name.contains('/')
        || file_name.contains('\\')
        || relative.components().count() != 1
        || !matches!(relative.components().next(), Some(Component::Normal(_)))
    {
        return Err("Informe um nome de arquivo sem pastas.".into());
    }
    if !supported_program(relative) {
        return Err("O arquivo deve usar a extensão .por ou .pors.".into());
    }

    let root = fs::canonicalize(root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    if !root.is_dir() {
        return Err("A pasta aberta não está mais disponível.".into());
    }
    let path = root.join(relative);
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map(|_| ())
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::AlreadyExists {
                "Já existe um arquivo com esse nome.".into()
            } else {
                format!("Não foi possível criar o arquivo: {error}")
            }
        })
}

fn rename_relative_file(root: &Path, relative: &Path, new_name: &str) -> Result<PathBuf, String> {
    let source = checked_file(root, relative, true)?;
    let name = Path::new(new_name);
    if new_name.trim().is_empty()
        || new_name == "."
        || new_name == ".."
        || new_name.contains('/')
        || new_name.contains('\\')
        || name.components().count() != 1
        || !matches!(name.components().next(), Some(Component::Normal(_)))
    {
        return Err("Informe um nome de arquivo sem pastas.".into());
    }
    if new_name.ends_with('.') || !supported_source(name) {
        return Err("O arquivo deve manter uma extensão .por, .pors ou .pex.".into());
    }

    let source_extension = source
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| "A extensão original do arquivo não é válida.".to_string())?;
    let new_extension = name
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| "A extensão do arquivo não é válida.".to_string())?;
    if !source_extension.eq_ignore_ascii_case(new_extension) {
        return Err(format!(
            "A extensão deve continuar sendo .{source_extension}."
        ));
    }

    let parent = source
        .parent()
        .ok_or_else(|| "O arquivo selecionado não pode ser renomeado.".to_string())?;
    let destination = parent.join(name);
    if source == destination {
        return Ok(source);
    }
    fs::hard_link(&source, &destination).map_err(|error| {
        if error.kind() == std::io::ErrorKind::AlreadyExists {
            "Já existe um arquivo com esse nome.".into()
        } else {
            format!("Não foi possível renomear o arquivo: {error}")
        }
    })?;
    if let Err(error) = fs::remove_file(&source) {
        let _ = fs::remove_file(&destination);
        return Err(format!("Não foi possível renomear o arquivo: {error}"));
    }
    fs::canonicalize(&destination)
        .map_err(|error| format!("Não foi possível acessar o arquivo renomeado: {error}"))
}

fn delete_relative_file(root: &Path, relative: &Path) -> Result<PathBuf, String> {
    let path = checked_file(root, relative, true)?;
    fs::remove_file(&path)
        .map_err(|error| format!("Não foi possível excluir o arquivo: {error}"))?;
    Ok(path)
}

#[tauri::command]
pub async fn workspace_open(
    app: AppHandle,
    session: State<'_, WorkspaceSession>,
) -> Result<Option<WorkspaceListing>, String> {
    let session = session.inner().clone();
    tauri::async_runtime::spawn_blocking(move || {
        let Some(selected) = app
            .dialog()
            .file()
            .set_title("Abrir pasta do projeto")
            .blocking_pick_folder()
        else {
            return Ok(None);
        };
        let selected = selected
            .into_path()
            .map_err(|error| format!("Não foi possível abrir a pasta selecionada: {error}"))?;
        let root = fs::canonicalize(selected)
            .map_err(|error| format!("Não foi possível acessar a pasta selecionada: {error}"))?;
        if !root.is_dir() {
            return Err("A seleção não é uma pasta.".into());
        }
        let entries = list_entries(&root)?;
        let name = root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("Projeto")
            .to_string();
        if let Ok(config_dir) = app.path().app_config_dir()
            && let Err(error) = save_last_workspace(&config_dir, &root)
        {
            eprintln!("{error}");
        }
        let id = session.register(root);
        Ok(Some(WorkspaceListing { id, name, entries }))
    })
    .await
    .map_err(|error| format!("Não foi possível abrir a pasta do projeto: {error}"))?
}

#[tauri::command]
pub async fn workspace_restore(
    app: AppHandle,
    session: State<'_, WorkspaceSession>,
) -> Result<Option<WorkspaceListing>, String> {
    let session = session.inner().clone();
    let config_dir = app
        .path()
        .app_config_dir()
        .map_err(|error| format!("Não foi possível localizar as preferências da IDE: {error}"))?;
    tauri::async_runtime::spawn_blocking(move || restore_saved_workspace(&session, &config_dir))
        .await
        .map_err(|error| format!("Não foi possível restaurar o último projeto: {error}"))?
}

#[tauri::command]
pub fn workspace_read_file(
    workspace_id: String,
    relative_path: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceFile, String> {
    let root = session.root(&workspace_id)?;
    let relative = Path::new(&relative_path);
    let absolute = checked_file(&root, relative, true)?;
    let bytes =
        fs::read(&absolute).map_err(|error| format!("Não foi possível ler o arquivo: {error}"))?;
    let source = super::decode_text(bytes);
    let path = absolute.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let relative_path = relative
        .to_str()
        .ok_or_else(|| "O caminho do arquivo é inválido.".to_string())?
        .replace('\\', "/");
    Ok(WorkspaceFile {
        path,
        relative_path,
        source,
    })
}

#[tauri::command]
pub fn workspace_write_file(
    workspace_id: String,
    relative_path: String,
    source: String,
    session: State<'_, WorkspaceSession>,
) -> Result<(), String> {
    let root = session.root(&workspace_id)?;
    write_relative_file(&root, Path::new(&relative_path), &source)
}

#[tauri::command]
pub fn workspace_create_file(
    workspace_id: String,
    file_name: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceListing, String> {
    let root = session.root(&workspace_id)?;
    create_source_file(&root, &file_name)?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    Ok(WorkspaceListing {
        id: workspace_id,
        name,
        entries,
    })
}

#[tauri::command]
pub fn workspace_rename_file(
    workspace_id: String,
    relative_path: String,
    new_name: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceRenameResult, String> {
    let root = session.root(&workspace_id)?;
    let new_path = rename_relative_file(&root, Path::new(&relative_path), &new_name)?;
    let root = fs::canonicalize(&root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let path = new_path.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let new_relative_path = self::relative_path(&root, &new_path)?;
    Ok(WorkspaceRenameResult {
        listing: WorkspaceListing {
            id: workspace_id,
            name,
            entries,
        },
        file: WorkspaceFilePath {
            path,
            relative_path: new_relative_path,
        },
    })
}

#[tauri::command]
pub fn workspace_delete_file(
    workspace_id: String,
    relative_path: String,
    session: State<'_, WorkspaceSession>,
) -> Result<WorkspaceDeleteResult, String> {
    let root = session.root(&workspace_id)?;
    let deleted_path = delete_relative_file(&root, Path::new(&relative_path))?;
    let root = fs::canonicalize(&root)
        .map_err(|_| "A pasta aberta não está mais disponível.".to_string())?;
    let entries = list_entries(&root)?;
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("Projeto")
        .to_string();
    let path = deleted_path.to_str().map(str::to_string).ok_or_else(|| {
        "O caminho do arquivo não pode ser representado nesta interface.".to_string()
    })?;
    let deleted_relative_path = self::relative_path(&root, &deleted_path)?;
    Ok(WorkspaceDeleteResult {
        listing: WorkspaceListing {
            id: workspace_id,
            name,
            entries,
        },
        file: WorkspaceFilePath {
            path,
            relative_path: deleted_relative_path,
        },
    })
}
