use std::fs;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::Serialize;

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
const MAX_SOURCE_BYTES: usize = 1_048_576;

#[derive(Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortugolRsRunResult {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
}

struct TemporaryDirectory(PathBuf);

impl TemporaryDirectory {
    fn create() -> Result<Self, String> {
        let root = std::env::temp_dir();
        for _ in 0..32 {
            let id = NEXT_TEMP_ID.fetch_add(1, Ordering::Relaxed);
            let path = root.join(format!("portugol-rs-{}-{id}", std::process::id()));
            match fs::create_dir(&path) {
                Ok(()) => {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).map_err(
                            |error| {
                                let _ = fs::remove_dir(&path);
                                format!(
                                    "Não foi possível proteger os arquivos temporários: {error}"
                                )
                            },
                        )?;
                    }
                    return Ok(Self(path));
                }
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => continue,
                Err(error) => {
                    return Err(format!(
                        "Não foi possível criar a pasta temporária: {error}"
                    ));
                }
            }
        }
        Err("Não foi possível reservar uma pasta temporária para a execução.".into())
    }
}

impl Drop for TemporaryDirectory {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

pub fn rust_toolchain_available() -> bool {
    Command::new("rustc")
        .arg("--version")
        .output()
        .is_ok_and(|output| output.status.success())
}

/// Compila e executa apenas o Rust gerado pelo subset validado de Portugol-RS.
pub fn run_portugol_rs(source: &str) -> Result<PortugolRsRunResult, String> {
    if source.len() > MAX_SOURCE_BYTES {
        return Err("O arquivo Portugol-RS excede o limite de 1 MiB para compilação.".into());
    }
    if !rust_toolchain_available() {
        return Err(
            "Rustc não foi encontrado. Instale a toolchain Rust para executar `.pors`.".into(),
        );
    }
    let export = crate::export_portugol_rs(source);
    let generated = export.code.ok_or_else(|| match export.diagnostic {
        Some(diagnostic) => format!(
            "{} (linha {}, coluna {}).",
            diagnostic.message, diagnostic.start.line, diagnostic.start.column
        ),
        None => "Corrija os erros do Portugol-RS antes de executar.".to_string(),
    })?;

    let temporary = TemporaryDirectory::create()?;
    let source_path = temporary.0.join("program.rs");
    let executable_path = temporary.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&source_path, generated)
        .map_err(|error| format!("Não foi possível gravar o Rust temporário: {error}"))?;

    let compilation = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--crate-name=portugol_rs_program")
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .map_err(|error| format!("Não foi possível iniciar rustc: {error}"))?;
    if !compilation.status.success() {
        let stderr = String::from_utf8_lossy(&compilation.stderr);
        return Err(format!("Falha ao compilar o Rust gerado:\n{stderr}"));
    }

    let execution = Command::new(&executable_path)
        .output()
        .map_err(|error| format!("Não foi possível executar o programa compilado: {error}"))?;
    Ok(PortugolRsRunResult {
        stdout: String::from_utf8_lossy(&execution.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&execution.stderr).into_owned(),
        exit_code: execution.status.code(),
    })
}
