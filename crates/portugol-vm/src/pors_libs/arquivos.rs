#[cfg(not(target_arch = "wasm32"))]
use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Component, Path, PathBuf};
#[cfg(not(target_arch = "wasm32"))]
use std::rc::Rc;

use crate::value::Value;

use super::{PorsRuntime, arity, err_string};
#[cfg(not(target_arch = "wasm32"))]
use super::{check_and_charge, expect_str, ok_value};

/// Limite máximo por arquivo em `Arquivos.*` (2 MiB).
#[cfg(not(target_arch = "wasm32"))]
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
/// Limite máximo de entradas listadas por `Arquivos.listar`.
#[cfg(not(target_arch = "wasm32"))]
const MAX_LIST_ENTRIES: usize = 1_024;

#[cfg(target_arch = "wasm32")]
pub(super) fn call(
    name: &str,
    _args: &[Value],
    _runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    if matches!(
        name,
        "ler_texto"
            | "ler_linhas"
            | "escrever_texto"
            | "acrescentar_texto"
            | "existe"
            | "remover"
            | "listar"
    ) {
        return err_string(
            "Acesso a arquivos não está disponível no ambiente WebAssembly.".to_string(),
            charge_string,
        );
    }
    Err(format!("Função desconhecida em Arquivos: `{name}`."))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn resolve_workspace_path(
    runtime: &PorsRuntime,
    raw: &str,
    allow_root: bool,
) -> Result<(PathBuf, PathBuf), String> {
    let Some(root) = runtime.workspace_dir.as_deref() else {
        return Err("Acesso a arquivos requer um diretório de workspace configurado.".to_string());
    };
    if raw.contains('\0') {
        return Err("Caminho inválido: contém caractere nulo.".to_string());
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() && !allow_root {
        return Err("O caminho do arquivo não pode ser vazio.".to_string());
    }
    let rel = Path::new(trimmed);
    if rel.is_absolute() {
        return Err("Apenas caminhos relativos ao workspace são permitidos.".to_string());
    }
    let mut clean = PathBuf::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(
                    "Caminho fora do workspace não é permitido (`..` ou raiz absoluta)."
                        .to_string(),
                );
            }
        }
    }
    if clean.as_os_str().is_empty() && !allow_root {
        return Err("O caminho deve apontar para um arquivo dentro do workspace.".to_string());
    }
    let canonical_root =
        std::fs::canonicalize(root).map_err(|err| format!("Workspace inacessível: {err}"))?;
    let target = canonical_root.join(&clean);

    // Garante que nenhum ancestral existente escape do workspace via symlink.
    let mut probe = target.as_path();
    while !probe.exists() {
        let Some(parent) = probe.parent() else {
            break;
        };
        probe = parent;
    }
    if probe.exists() {
        let canonical_probe = std::fs::canonicalize(probe)
            .map_err(|err| format!("Não foi possível validar o caminho: {err}"))?;
        if !canonical_probe.starts_with(&canonical_root) {
            return Err("Caminho escapa do workspace (link simbólico bloqueado).".to_string());
        }
    }
    Ok((canonical_root, target))
}

#[cfg(not(target_arch = "wasm32"))]
pub(super) fn call(
    name: &str,
    args: &[Value],
    runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "ler_texto" => {
            arity(args, 1, "Arquivos.ler_texto")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let meta = match std::fs::metadata(&target) {
                Ok(meta) => meta,
                Err(err) => {
                    return err_string(
                        format!("Não foi possível abrir '{path}': {err}"),
                        charge_string,
                    );
                }
            };
            if !meta.is_file() {
                return err_string(format!("'{path}' não é um arquivo regular."), charge_string);
            }
            if meta.len() as usize > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excede o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            check_and_charge(meta.len() as usize, charge_string)?;
            match std::fs::read_to_string(&target) {
                Ok(content) => Ok(ok_value(Value::str(content))),
                Err(err) => err_string(format!("Erro ao ler '{path}': {err}"), charge_string),
            }
        }
        "ler_linhas" => {
            arity(args, 1, "Arquivos.ler_linhas")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let meta = match std::fs::metadata(&target) {
                Ok(meta) => meta,
                Err(err) => {
                    return err_string(
                        format!("Não foi possível abrir '{path}': {err}"),
                        charge_string,
                    );
                }
            };
            if !meta.is_file() {
                return err_string(format!("'{path}' não é um arquivo regular."), charge_string);
            }
            if meta.len() as usize > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excede o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            check_and_charge(meta.len() as usize, charge_string)?;
            match std::fs::read_to_string(&target) {
                Ok(content) => {
                    let lines: Vec<Value> = content.lines().map(Value::str).collect();
                    Ok(ok_value(Value::Array(Rc::new(RefCell::new(lines)))))
                }
                Err(err) => err_string(format!("Erro ao ler '{path}': {err}"), charge_string),
            }
        }
        "escrever_texto" => {
            arity(args, 2, "Arquivos.escrever_texto")?;
            let path = expect_str(&args[0])?;
            let content = expect_str(&args[1])?;
            if content.len() > MAX_FILE_BYTES {
                return err_string(
                    format!("O conteúdo excede o limite de {MAX_FILE_BYTES} bytes por arquivo."),
                    charge_string,
                );
            }
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            if let Some(parent) = target.parent()
                && let Err(err) = std::fs::create_dir_all(parent)
            {
                return err_string(
                    format!("Não foi possível criar diretório para '{path}': {err}"),
                    charge_string,
                );
            }
            match std::fs::write(&target, content) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(format!("Erro ao gravar '{path}': {err}"), charge_string),
            }
        }
        "acrescentar_texto" => {
            arity(args, 2, "Arquivos.acrescentar_texto")?;
            let path = expect_str(&args[0])?;
            let content = expect_str(&args[1])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let existing_len = std::fs::metadata(&target)
                .map(|m| m.len() as usize)
                .unwrap_or(0);
            if existing_len.saturating_add(content.len()) > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excederia o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            if let Some(parent) = target.parent()
                && let Err(err) = std::fs::create_dir_all(parent)
            {
                return err_string(
                    format!("Não foi possível criar diretório para '{path}': {err}"),
                    charge_string,
                );
            }
            use std::io::Write;
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target);
            match file.and_then(|mut handle| handle.write_all(content.as_bytes())) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(
                    format!("Erro ao acrescentar em '{path}': {err}"),
                    charge_string,
                ),
            }
        }
        "existe" => {
            arity(args, 1, "Arquivos.existe")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, true) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            Ok(ok_value(Value::Bool(target.exists())))
        }
        "remover" => {
            arity(args, 1, "Arquivos.remover")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            match std::fs::remove_file(&target) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(format!("Erro ao remover '{path}': {err}"), charge_string),
            }
        }
        "listar" => {
            arity(args, 1, "Arquivos.listar")?;
            let dir = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, dir, true) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let entries = match std::fs::read_dir(&target) {
                Ok(entries) => entries,
                Err(err) => {
                    return err_string(format!("Erro ao listar '{dir}': {err}"), charge_string);
                }
            };
            let mut names = Vec::new();
            let mut total_bytes = 0usize;
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        return err_string(format!("Erro ao listar '{dir}': {err}"), charge_string);
                    }
                };
                if names.len() >= MAX_LIST_ENTRIES {
                    return err_string(
                        format!("Diretório excede o limite de {MAX_LIST_ENTRIES} entradas."),
                        charge_string,
                    );
                }
                let item = entry.file_name().to_string_lossy().into_owned();
                total_bytes = total_bytes.saturating_add(item.len());
                names.push(item);
            }
            names.sort();
            check_and_charge(total_bytes, charge_string)?;
            let values: Vec<Value> = names.into_iter().map(Value::str).collect();
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(values)))))
        }
        other => Err(format!("Função desconhecida em Arquivos: `{other}`.")),
    }
}
