use std::path::Path;
use std::process::{Command, Stdio};
use std::sync::atomic::AtomicBool;

use serde::Serialize;

use crate::pors_run::{
    COMPILATION_LIMIT, ProcessLimitReason, RUNTIME_LIMIT, configure_process_tree, supervise,
};

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

pub fn cargo_available() -> bool {
    Command::new("cargo")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
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
    if let Some(reason) = build.limit {
        return Ok(limit_result(
            reason,
            "compilation",
            build_stdout,
            build_stderr,
        ));
    }
    if build.status != Some(0) {
        let diagnostics = cargo_rendered_diagnostics(&build.stdout);
        return Ok(CargoProjectRunResult {
            status: "build_error".into(),
            stdout: build_stdout,
            stderr: [build_stderr, diagnostics]
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
        stdout,
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
