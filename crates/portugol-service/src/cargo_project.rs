use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use serde::Serialize;
use serde_json::Value;

use crate::pors_run::{
    COMPILATION_LIMIT, ProcessLimitReason, RUNTIME_LIMIT, configure_process_tree, supervise,
};
use crate::{Position, position::LineIndex};

const MAX_CARGO_OUTPUT_BYTES: usize = 1_048_576;

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

pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
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
    let manifest = project_root.join("Cargo.toml");
    let metadata = std::fs::symlink_metadata(&manifest)
        .map_err(|error| format!("Não foi possível acessar Cargo.toml: {error}"))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        return Err("Cargo.toml precisa ser um arquivo regular do projeto.".into());
    }

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
    let project_root = std::fs::canonicalize(project_root)
        .map_err(|error| format!("Não foi possível acessar o projeto Rust: {error}"))?;
    if !project_root.is_dir() {
        return Err("A pasta do projeto Cargo não está disponível.".into());
    }
    let manifest = project_root.join("Cargo.toml");
    if !manifest.is_file() {
        return Err("Não há Cargo.toml na pasta do projeto Rust.".into());
    }

    let mut build = Command::new("cargo");
    configure_process_tree(&mut build);
    let child = build
        .arg("build")
        .arg("--message-format=json")
        .arg("--manifest-path")
        .arg(&manifest)
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
    let child = run
        .arg("run")
        .arg("--quiet")
        .arg("--manifest-path")
        .arg(&manifest)
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
