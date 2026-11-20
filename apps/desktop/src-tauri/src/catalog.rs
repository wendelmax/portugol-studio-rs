use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use tauri::{AppHandle, Manager, ipc::Response, path::BaseDirectory};

use super::decode_text;

#[derive(Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CatalogEntry {
    Folder {
        name: String,
        entries: Vec<CatalogEntry>,
    },
    File {
        name: String,
        file: String,
        description: String,
        preview: Option<String>,
    },
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OpenedExample {
    source: String,
    work_path: String,
}

fn assets(app: &AppHandle) -> Result<PathBuf, String> {
    app.path()
        .resolve("exemplos", BaseDirectory::Resource)
        .map_err(|error| error.to_string())
}

fn root_relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root)
        .expect("caminho dentro do catálogo")
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

fn labels(path: &Path) -> String {
    path.file_stem()
        .unwrap_or_default()
        .to_string_lossy()
        .replace('_', " ")
}

fn read_index(folder: &Path) -> io::Result<Vec<HashMap<String, String>>> {
    let text = match fs::read(folder.join("index.properties")) {
        Ok(bytes) => decode_text(bytes),
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    let mut entries = BTreeMap::<usize, HashMap<_, _>>::new();
    for line in text.lines() {
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        if let Some((index, key)) = key
            .trim()
            .strip_prefix("item")
            .and_then(|key| key.split_once('.'))
            && let Ok(index) = index.parse::<usize>()
        {
            entries
                .entry(index)
                .or_default()
                .insert(key.to_string(), value.trim().to_string());
        }
    }
    Ok(entries.into_values().collect())
}

fn preview(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            matches!(
                extension.to_ascii_lowercase().as_str(),
                "png" | "jpg" | "jpeg" | "gif"
            )
        })
}

fn source_file(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("por"))
}

fn normal_name(name: &str) -> bool {
    let mut components = Path::new(name).components();
    matches!(components.next(), Some(Component::Normal(_))) && components.next().is_none()
}

fn tree(root: &Path, folder: &Path) -> io::Result<Vec<CatalogEntry>> {
    let mut children = BTreeMap::new();
    for entry in fs::read_dir(folder)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let path = entry.path();
        if ty.is_dir() || ty.is_file() && source_file(&path) {
            children.insert(
                entry.file_name().to_string_lossy().into_owned(),
                (path, ty.is_dir()),
            );
        }
    }

    let mut ordered = Vec::new();
    for mut entry in read_index(folder)? {
        let Some(file) = entry.remove("dir").or_else(|| entry.remove("file")) else {
            continue;
        };
        if let Some((path, directory)) = children.remove(&file) {
            ordered.push((path, directory, entry));
        }
    }
    ordered.extend(
        children
            .into_values()
            .map(|(path, directory)| (path, directory, HashMap::new())),
    );

    let mut result = Vec::new();
    for (path, directory, mut entry) in ordered {
        let name = entry.remove("name").unwrap_or_else(|| labels(&path));
        if directory {
            let entries = tree(root, &path)?;
            if !entries.is_empty() {
                result.push(CatalogEntry::Folder { name, entries });
            }
        } else {
            let image = entry
                .get("hasImage")
                .is_some_and(|value| value.eq_ignore_ascii_case("true"))
                .then(|| entry.remove("image"))
                .flatten()
                .filter(|name| normal_name(name))
                .map(|name| folder.join(name))
                .filter(|path| preview(path) && path.is_file())
                .map(|path| root_relative(root, &path));
            result.push(CatalogEntry::File {
                name,
                file: root_relative(root, &path),
                description: entry.remove("description").unwrap_or_default(),
                preview: image,
            });
        }
    }
    Ok(result)
}

fn find_file(
    root: &Path,
    relative: &str,
    valid_kind: fn(&Path) -> bool,
) -> Result<PathBuf, String> {
    let path = Path::new(relative);
    if path.as_os_str().is_empty()
        || path
            .components()
            .any(|part| !matches!(part, Component::Normal(_)))
        || !valid_kind(path)
    {
        return Err("O recurso não pertence ao catálogo.".into());
    }
    let canonical_root = root.canonicalize().map_err(|error| error.to_string())?;
    let target = canonical_root
        .join(path)
        .canonicalize()
        .map_err(|error| error.to_string())?;
    if !target.starts_with(canonical_root) || !target.is_file() {
        return Err("O recurso não pertence ao catálogo.".into());
    }
    Ok(target)
}

fn copy_tree(source: &Path, target: &Path) -> io::Result<()> {
    for entry in fs::read_dir(source)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let destination = target.join(entry.file_name());
        if ty.is_dir() {
            fs::create_dir(&destination)?;
            copy_tree(&entry.path(), &destination)?;
        } else if ty.is_file() {
            let path = entry.path();
            // Entradas e saídas gravadas pertencem à suíte de regressão.
            if !path
                .extension()
                .is_some_and(|extension| extension == "entrada" || extension == "saida")
            {
                fs::copy(path, destination)?;
            }
        }
    }
    Ok(())
}

fn prepare_workspace(app: &AppHandle, root: &Path) -> Result<PathBuf, String> {
    let cache = app
        .path()
        .app_cache_dir()
        .map_err(|error| error.to_string())?;
    let workspace = cache.join(format!("exemplos-{}", app.package_info().version));
    if workspace.is_dir() {
        return Ok(workspace);
    }
    fs::create_dir_all(&cache).map_err(|error| error.to_string())?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = cache.join(format!(".exemplos-{}-{nonce}", std::process::id()));
    fs::create_dir(&staging).map_err(|error| error.to_string())?;
    let result = copy_tree(root, &staging).and_then(|()| fs::rename(&staging, &workspace));
    if result.is_err() {
        let _ = fs::remove_dir_all(staging);
        // Outra janela ou instância pode ter terminado a cópia primeiro.
        if !workspace.is_dir() {
            result.map_err(|error| error.to_string())?;
        }
    }
    Ok(workspace)
}

#[tauri::command]
pub async fn example_catalog(app: AppHandle) -> Result<Vec<CatalogEntry>, String> {
    let root = assets(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        tree(&root, &root).map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn open_example(app: AppHandle, file: String) -> Result<OpenedExample, String> {
    let root = assets(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let source = find_file(&root, &file, source_file)?;
        let source = decode_text(fs::read(source).map_err(|error| error.to_string())?);
        let workspace = prepare_workspace(&app, &root)?;
        Ok(OpenedExample {
            source,
            work_path: workspace.join(file).to_string_lossy().into_owned(),
        })
    })
    .await
    .map_err(|error| error.to_string())?
}

#[tauri::command]
pub async fn example_preview(app: AppHandle, file: String) -> Result<Response, String> {
    let root = assets(&app)?;
    tauri::async_runtime::spawn_blocking(move || {
        let path = find_file(&root, &file, preview)?;
        fs::read(path)
            .map(Response::new)
            .map_err(|error| error.to_string())
    })
    .await
    .map_err(|error| error.to_string())?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_includes_original_categories_and_nested_programs() {
        fn count(entries: &[CatalogEntry]) -> usize {
            entries
                .iter()
                .map(|entry| match entry {
                    CatalogEntry::Folder { entries, .. } => count(entries),
                    CatalogEntry::File { .. } => 1,
                })
                .sum()
        }

        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../../tests/corpus/exemplos");
        let entries = tree(&root, &root).unwrap();
        assert!(
            matches!(&entries[0], CatalogEntry::Folder { name, .. } if name == "Entrada e Saída")
        );
        assert_eq!(count(&entries), 119);
        assert!(find_file(&root, "jogos/moon_lander.por", source_file).is_ok());
        assert!(find_file(&root, "../ajuda/recursos/exemplos/index.por", source_file).is_err());
    }
}
