use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::pors_run::{
    COMPILATION_LIMIT, ProcessLimitReason, RUNTIME_LIMIT, configure_process_tree, supervise,
};
use crate::{Position, position::LineIndex};

const MAX_CARGO_OUTPUT_BYTES: usize = 1_048_576;

fn validated_manifest(project_root: &Path) -> Result<PathBuf, String> {
    let manifest = project_root.join("Cargo.toml");
    let metadata = std::fs::symlink_metadata(&manifest)
        .map_err(|error| format!("Não foi possível acessar Cargo.toml: {error}"))?;
    if metadata.file_type().is_symlink() {
        return Err("Cargo.toml não pode ser um link simbólico.".into());
    }
    if !metadata.is_file() {
        return Err("Cargo.toml precisa ser um arquivo regular do projeto.".into());
    }
    Ok(manifest)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoProjectRunResult {
    pub status: String,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub phase: Option<String>,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoProjectDiagnostic {
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
    pub rendered: Option<String>,
    pub start: Position,
    pub end: Position,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoProjectCheckResult {
    pub status: String,
    pub diagnostics: Vec<CargoProjectDiagnostic>,
    pub stderr: String,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoRunTarget {
    pub package_name: String,
    pub target_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CargoRunTargets {
    pub targets: Vec<CargoRunTarget>,
    pub default_target: Option<CargoRunTarget>,
}

pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

/// Reads local Cargo workspace metadata without resolving dependencies or running build scripts.
pub fn cargo_run_targets_with_cancel(
    project_root: &Path,
    cancel: &AtomicBool,
) -> Result<CargoRunTargets, String> {
    let project_root = std::fs::canonicalize(project_root)
        .map_err(|error| format!("Não foi possível acessar o projeto Rust: {error}"))?;
    if !project_root.is_dir() {
        return Err("A pasta do projeto Cargo não está disponível.".into());
    }
    let manifest = validated_manifest(&project_root)?;

    let mut command = Command::new("cargo");
    configure_process_tree(&mut command);
    let child = command
        .arg("metadata")
        .arg("--no-deps")
        .arg("--offline")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(&manifest)
        .current_dir(&project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Não foi possível iniciar Cargo; instale Rust/Cargo: {error}"))?;
    let output = supervise(child, COMPILATION_LIMIT, Some(cancel))?;
    if let Some(reason) = output.limit {
        return Err(match reason {
            ProcessLimitReason::Cancelled => "Descoberta de alvos Cargo cancelada.".into(),
            ProcessLimitReason::Time => "A descoberta de alvos Cargo excedeu 20 segundos.".into(),
            ProcessLimitReason::Output | ProcessLimitReason::StringMemory => {
                "A saída de metadata Cargo excedeu 1 MiB.".into()
            }
        });
    }
    if output.status != Some(0) {
        return Err(format!(
            "Não foi possível identificar os alvos do projeto Cargo: {}",
            bounded_output(&output.stderr)
        ));
    }
    let metadata: Value = serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("Cargo retornou metadata inválida: {error}"))?;
    let packages = metadata
        .get("packages")
        .and_then(Value::as_array)
        .ok_or_else(|| "Cargo não retornou os pacotes do workspace.".to_string())?;
    let mut targets = packages
        .iter()
        .flat_map(|package| {
            let package_name = package
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            package
                .get("targets")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter(|target| {
                    target
                        .get("kind")
                        .and_then(Value::as_array)
                        .is_some_and(|kinds| kinds.iter().any(|kind| kind.as_str() == Some("bin")))
                })
                .filter_map(move |target| {
                    Some(CargoRunTarget {
                        package_name: package_name.to_owned(),
                        target_name: target.get("name")?.as_str()?.to_owned(),
                    })
                })
        })
        .collect::<Vec<_>>();
    targets.sort_by(|left, right| {
        (&left.package_name, &left.target_name).cmp(&(&right.package_name, &right.target_name))
    });

    let root_manifest = manifest.to_string_lossy();
    let root_package = packages.iter().find(|package| {
        package.get("manifest_path").and_then(Value::as_str) == Some(root_manifest.as_ref())
    });
    let default_member_ids = metadata
        .get("workspace_default_members")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let default_package = root_package.or_else(|| {
        if default_member_ids.len() != 1 {
            return None;
        }
        let id = default_member_ids[0].as_str()?;
        packages
            .iter()
            .find(|package| package.get("id").and_then(Value::as_str) == Some(id))
    });
    let default_target = default_package.and_then(|package| {
        let package_name = package.get("name")?.as_str()?;
        let package_targets = targets
            .iter()
            .filter(|target| target.package_name == package_name)
            .collect::<Vec<_>>();
        let default_run = package.get("default_run").and_then(Value::as_str);
        let selected = if let Some(default_run) = default_run {
            package_targets
                .iter()
                .find(|target| target.target_name == default_run)
                .copied()
        } else if package_targets.len() == 1 {
            package_targets.first().copied()
        } else {
            None
        }?;
        Some(selected.clone())
    });
    Ok(CargoRunTargets {
        targets,
        default_target,
    })
}

/// Runs Cargo's project-aware compiler diagnostics without starting the program.
/// Cargo may execute build scripts and procedural macros; callers must obtain consent.
pub fn check_cargo_project_with_cancel(
    project_root: &Path,
    source_path: &Path,
    cancel: &AtomicBool,
) -> Result<CargoProjectCheckResult, String> {
    let project_root = std::fs::canonicalize(project_root)
        .map_err(|error| format!("Não foi possível acessar o projeto Rust: {error}"))?;
    let source_path = std::fs::canonicalize(source_path)
        .map_err(|error| format!("Não foi possível acessar o arquivo Rust: {error}"))?;
    if !project_root.is_dir() || !source_path.is_file() || !source_path.starts_with(&project_root) {
        return Err("O arquivo Rust precisa estar dentro do projeto Cargo aberto.".into());
    }
    let manifest = validated_manifest(&project_root)?;

    let mut command = Command::new("cargo");
    configure_process_tree(&mut command);
    let child = command
        .arg("check")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(&manifest)
        .current_dir(&project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Não foi possível iniciar Cargo; instale Rust/Cargo: {error}"))?;
    let output = supervise(child, COMPILATION_LIMIT, Some(cancel))?;
    let rendered = cargo_rendered_diagnostics(&output.stdout);
    let stderr = [bounded_output(&output.stderr), rendered]
        .into_iter()
        .filter(|text| !text.is_empty())
        .collect::<Vec<_>>()
        .join("\n");
    if let Some(reason) = output.limit {
        return Ok(CargoProjectCheckResult {
            status: if reason == ProcessLimitReason::Cancelled {
                "cancelled"
            } else {
                "check_limit"
            }
            .into(),
            diagnostics: Vec::new(),
            stderr,
            reason: Some(
                match reason {
                    ProcessLimitReason::Time => "time",
                    ProcessLimitReason::Output => "output",
                    ProcessLimitReason::StringMemory => "memory",
                    ProcessLimitReason::Cancelled => "cancelled",
                }
                .into(),
            ),
        });
    }
    let diagnostics = cargo_source_diagnostics(&output.stdout, &project_root, &source_path)?;
    Ok(CargoProjectCheckResult {
        status: if output.status == Some(0) {
            "checked"
        } else {
            "check_error"
        }
        .into(),
        diagnostics,
        stderr,
        reason: None,
    })
}

/// Compila e executa o binário padrão de um projeto Cargo sem shell.
/// Cargo e os scripts/dependências do projeto executam com as permissões do usuário.
pub fn run_cargo_project_with_cancel(
    project_root: &Path,
    cancel: &AtomicBool,
) -> Result<CargoProjectRunResult, String> {
    run_cargo_project_target_with_cancel(project_root, None, cancel)
}

pub fn run_cargo_project_target_with_cancel(
    project_root: &Path,
    target: Option<&CargoRunTarget>,
    cancel: &AtomicBool,
) -> Result<CargoProjectRunResult, String> {
    let project_root = std::fs::canonicalize(project_root)
        .map_err(|error| format!("Não foi possível acessar o projeto Rust: {error}"))?;
    if !project_root.is_dir() {
        return Err("A pasta do projeto Cargo não está disponível.".into());
    }
    let manifest = validated_manifest(&project_root)?;
    if let Some(target) = target {
        let available = cargo_run_targets_with_cancel(&project_root, cancel)?;
        if !available.targets.contains(target) {
            return Err("O alvo binário selecionado não pertence a este projeto Cargo.".into());
        }
    }

    let mut build = Command::new("cargo");
    configure_process_tree(&mut build);
    let mut build = build
        .arg("build")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(&manifest);
    if let Some(target) = target {
        build = build
            .arg("--package")
            .arg(&target.package_name)
            .arg("--bin")
            .arg(&target.target_name);
    }
    let child = build
        .current_dir(&project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Não foi possível iniciar Cargo; instale Rust/Cargo: {error}"))?;
    let build = supervise(child, COMPILATION_LIMIT, Some(cancel))?;
    let build_stdout = String::new();
    let build_stderr = bounded_output(&build.stderr);
    let build_messages = cargo_rendered_diagnostics(&build.stdout);
    if let Some(reason) = build.limit {
        return Ok(limit_result(
            reason,
            "compilation",
            build_stdout,
            build_stderr,
        ));
    }
    if build.status != Some(0) {
        return Ok(CargoProjectRunResult {
            status: "build_error".into(),
            stdout: build_stdout,
            stderr: [build_stderr, build_messages]
                .into_iter()
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n"),
            exit_code: build.status,
            phase: Some("compilation".into()),
            reason: None,
        });
    }

    let mut run = Command::new("cargo");
    configure_process_tree(&mut run);
    let mut run = run
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest);
    if let Some(target) = target {
        run = run
            .arg("--package")
            .arg(&target.package_name)
            .arg("--bin")
            .arg(&target.target_name);
    }
    let child = run
        .current_dir(&project_root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Não foi possível iniciar o programa Cargo: {error}"))?;
    let execution = supervise(child, RUNTIME_LIMIT, Some(cancel))?;
    let stdout = bounded_output(&execution.stdout);
    let stderr = bounded_output(&execution.stderr);
    if let Some(reason) = execution.limit {
        return Ok(limit_result(reason, "runtime", stdout, stderr));
    }
    Ok(CargoProjectRunResult {
        status: if execution.status == Some(0) {
            "executed"
        } else {
            "run_error"
        }
        .into(),
        stdout: [build_messages, stdout]
            .into_iter()
            .filter(|text| !text.is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        stderr,
        exit_code: execution.status,
        phase: Some("runtime".into()),
        reason: None,
    })
}

fn bounded_output(bytes: &[u8]) -> String {
    String::from_utf8_lossy(&bytes[..bytes.len().min(MAX_CARGO_OUTPUT_BYTES)]).into_owned()
}

fn cargo_rendered_diagnostics(messages: &[u8]) -> String {
    messages
        .split(|byte| *byte == b'\n')
        .filter_map(|line| serde_json::from_slice::<serde_json::Value>(line).ok())
        .filter(|message| {
            message.get("reason").and_then(|value| value.as_str()) == Some("compiler-message")
        })
        .filter_map(|message| {
            message
                .get("message")
                .and_then(|diagnostic| diagnostic.get("rendered"))
                .and_then(|value| value.as_str())
                .map(str::to_owned)
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn cargo_source_diagnostics(
    messages: &[u8],
    project_root: &Path,
    source_path: &Path,
) -> Result<Vec<CargoProjectDiagnostic>, String> {
    let source = std::fs::read_to_string(source_path)
        .map_err(|error| format!("Não foi possível ler a fonte Rust para mapear spans: {error}"))?;
    let index = LineIndex::new(&source);
    let mut diagnostics = Vec::new();
    for message in messages
        .split(|byte| *byte == b'\n')
        .take(100_000)
        .filter_map(|line| serde_json::from_slice::<Value>(line).ok())
        .filter(|message| message.get("reason").and_then(Value::as_str) == Some("compiler-message"))
        .take(1000)
    {
        let Some(compiler_message) = message.get("message") else {
            continue;
        };
        let primary = compiler_message
            .get("spans")
            .and_then(Value::as_array)
            .and_then(|spans| {
                spans
                    .iter()
                    .find(|span| span.get("is_primary").and_then(Value::as_bool) == Some(true))
            });
        let Some(primary) = primary else {
            continue;
        };
        let Some(file_name) = primary.get("file_name").and_then(Value::as_str) else {
            continue;
        };
        let diagnostic_path = Path::new(file_name);
        let diagnostic_path = if diagnostic_path.is_absolute() {
            diagnostic_path.to_path_buf()
        } else {
            project_root.join(diagnostic_path)
        };
        if std::fs::canonicalize(diagnostic_path).ok().as_deref() != Some(source_path) {
            continue;
        }
        let Some(byte_start) = primary.get("byte_start").and_then(Value::as_u64) else {
            continue;
        };
        let Some(byte_end) = primary.get("byte_end").and_then(Value::as_u64) else {
            continue;
        };
        let Ok(byte_start) = u32::try_from(byte_start) else {
            continue;
        };
        let Ok(byte_end) = u32::try_from(byte_end) else {
            continue;
        };
        diagnostics.push(CargoProjectDiagnostic {
            severity: compiler_message
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_string(),
            code: compiler_message
                .pointer("/code/code")
                .and_then(Value::as_str)
                .map(str::to_owned),
            message: compiler_message
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Erro de compilação Cargo.")
                .to_string(),
            rendered: compiler_message
                .get("rendered")
                .and_then(Value::as_str)
                .map(str::to_owned),
            start: index.position(byte_start),
            end: index.position(byte_end),
        });
    }
    Ok(diagnostics)
}

fn limit_result(
    reason: ProcessLimitReason,
    phase: &str,
    stdout: String,
    stderr: String,
) -> CargoProjectRunResult {
    let cancelled = reason == ProcessLimitReason::Cancelled;
    CargoProjectRunResult {
        status: if cancelled {
            "cancelled"
        } else if phase == "compilation" {
            "build_limit"
        } else {
            "runtime_limit"
        }
        .into(),
        stdout,
        stderr,
        exit_code: None,
        phase: Some(phase.into()),
        reason: Some(
            match reason {
                ProcessLimitReason::Time => "time",
                ProcessLimitReason::Output => "output",
                ProcessLimitReason::StringMemory => "memory",
                ProcessLimitReason::Cancelled => "cancelled",
            }
            .into(),
        ),
    }
}
