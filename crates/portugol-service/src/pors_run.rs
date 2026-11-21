use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde_json::Value;

use crate::{Position, SourceMapQuality, position::LineIndex};
use portugol_rust::{GeneratedRust, SourceMapQuality as RustSourceMapQuality, SourceMapSegment};

static NEXT_TEMP_ID: AtomicU64 = AtomicU64::new(1);
const MAX_SOURCE_BYTES: usize = 1_048_576;
const MAX_RUNTIME_OUTPUT_BYTES: usize = 1_048_576;
const RUNTIME_LIMIT: Duration = Duration::from_secs(3);
const STRING_BUDGET_EXIT_CODE: i32 = 78;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "status",
    rename_all = "snake_case",
    rename_all_fields = "camelCase"
)]
pub enum PortugolRsRunResult {
    Executed {
        stdout: String,
        stderr: String,
        exit_code: Option<i32>,
    },
    CompileError {
        diagnostics: Vec<PortugolRsCompileDiagnostic>,
        stderr: String,
    },
    RuntimeLimit {
        stdout: String,
        stderr: String,
        reason: PortugolRsRuntimeLimitReason,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortugolRsRuntimeLimitReason {
    Time,
    Output,
    StringMemory,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortugolRsCompileDiagnostic {
    pub severity: String,
    pub code: Option<String>,
    pub message: String,
    pub rendered: Option<String>,
    pub generated_start: Option<Position>,
    pub generated_end: Option<Position>,
    pub source_start: Option<Position>,
    pub source_end: Option<Position>,
    pub quality: SourceMapQuality,
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
    let generated = portugol_rust::export_portugol_rs_with_map(source).map_err(|diagnostics| {
        diagnostics.first().map_or_else(
            || "Corrija os erros do Portugol-RS antes de executar.".to_string(),
            |diagnostic| {
                let index = LineIndex::new(source);
                let diagnostic = crate::describe(&index, diagnostic);
                format!(
                    "{} (linha {}, coluna {}).",
                    diagnostic.message, diagnostic.start.line, diagnostic.start.column
                )
            },
        )
    })?;

    compile_and_run_generated(&generated, source)
}

fn compile_and_run_generated(
    generated: &GeneratedRust,
    source: &str,
) -> Result<PortugolRsRunResult, String> {
    let temporary = TemporaryDirectory::create()?;
    let source_path = temporary.0.join("program.rs");
    let executable_path = temporary.0.join(if cfg!(windows) {
        "program.exe"
    } else {
        "program"
    });
    fs::write(&source_path, &generated.code)
        .map_err(|error| format!("Não foi possível gravar o Rust temporário: {error}"))?;

    let compilation = Command::new("rustc")
        .arg("--edition=2021")
        .arg("--error-format=json")
        .arg("--crate-name=portugol_rs_program")
        .arg(&source_path)
        .arg("-o")
        .arg(&executable_path)
        .output()
        .map_err(|error| format!("Não foi possível iniciar rustc: {error}"))?;
    if !compilation.status.success() {
        let stderr = String::from_utf8_lossy(&compilation.stderr).into_owned();
        let diagnostics =
            parse_rustc_diagnostics(&stderr, &generated.code, source, &generated.mappings);
        return Ok(PortugolRsRunResult::CompileError {
            diagnostics,
            stderr,
        });
    }

    let execution = Command::new(&executable_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("Não foi possível executar o programa compilado: {error}"))?;
    let execution = supervise(execution)?;
    let stdout = String::from_utf8_lossy(&execution.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&execution.stderr).into_owned();
    if let Some(reason) = execution.limit {
        return Ok(PortugolRsRunResult::RuntimeLimit {
            stdout,
            stderr,
            reason,
        });
    }
    Ok(PortugolRsRunResult::Executed {
        stdout,
        stderr,
        exit_code: execution.status,
    })
}

struct SupervisedOutput {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    status: Option<i32>,
    limit: Option<PortugolRsRuntimeLimitReason>,
}

fn supervise(mut child: Child) -> Result<SupervisedOutput, String> {
    let stdout = child
        .stdout
        .take()
        .ok_or("stdout do processo não foi capturado")?;
    let stderr = child
        .stderr
        .take()
        .ok_or("stderr do processo não foi capturado")?;
    let (sender, receiver) = mpsc::channel::<(bool, Vec<u8>)>();
    let captured = Arc::new(AtomicUsize::new(0));
    let exceeded = Arc::new(AtomicBool::new(false));
    let pipe_error = Arc::new(Mutex::new(None));
    let out_thread = pipe_reader(
        stdout,
        true,
        sender.clone(),
        captured.clone(),
        exceeded.clone(),
        pipe_error.clone(),
    );
    let err_thread = pipe_reader(
        stderr,
        false,
        sender,
        captured,
        exceeded.clone(),
        pipe_error.clone(),
    );
    let started = Instant::now();
    let mut stdout_bytes = Vec::new();
    let mut stderr_bytes = Vec::new();
    let mut limit = None;
    let status = loop {
        while let Ok((is_stdout, bytes)) = receiver.try_recv() {
            if is_stdout {
                stdout_bytes.extend(bytes);
            } else {
                stderr_bytes.extend(bytes);
            }
        }
        if exceeded.load(Ordering::Relaxed) {
            limit = Some(PortugolRsRuntimeLimitReason::Output);
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("Não foi possível recolher o processo: {error}"))?
                .code();
        }
        if pipe_error
            .lock()
            .expect("erro do pipe deve estar acessível")
            .is_some()
        {
            let _ = child.kill();
            let _ = child.wait();
            break None;
        }
        if started.elapsed() >= RUNTIME_LIMIT {
            limit = Some(PortugolRsRuntimeLimitReason::Time);
            let _ = child.kill();
            break child
                .wait()
                .map_err(|error| format!("Não foi possível recolher o processo: {error}"))?
                .code();
        }
        if let Some(status) = child
            .try_wait()
            .map_err(|error| format!("Não foi possível acompanhar o processo: {error}"))?
        {
            break status.code();
        }
        thread::sleep(Duration::from_millis(5));
    };
    out_thread
        .join()
        .map_err(|_| "A captura de stdout falhou inesperadamente.".to_string())?;
    err_thread
        .join()
        .map_err(|_| "A captura de stderr falhou inesperadamente.".to_string())?;
    if let Some(error) = pipe_error
        .lock()
        .expect("erro do pipe deve estar acessível")
        .take()
    {
        return Err(error);
    }
    while let Ok((is_stdout, bytes)) = receiver.try_recv() {
        if is_stdout {
            stdout_bytes.extend(bytes);
        } else {
            stderr_bytes.extend(bytes);
        }
    }
    if limit.is_none() && status == Some(STRING_BUDGET_EXIT_CODE) {
        limit = Some(PortugolRsRuntimeLimitReason::StringMemory);
    }
    Ok(SupervisedOutput {
        stdout: stdout_bytes,
        stderr: stderr_bytes,
        status,
        limit,
    })
}

fn pipe_reader<R: Read + Send + 'static>(
    mut pipe: R,
    is_stdout: bool,
    sender: mpsc::Sender<(bool, Vec<u8>)>,
    captured: Arc<AtomicUsize>,
    exceeded: Arc<AtomicBool>,
    pipe_error: Arc<Mutex<Option<String>>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 8192];
        loop {
            let read = match pipe.read(&mut buffer) {
                Ok(read) => read,
                Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(error) => {
                    *pipe_error
                        .lock()
                        .expect("erro do pipe deve estar acessível") = Some(format!(
                        "Não foi possível capturar a saída do processo: {error}"
                    ));
                    break;
                }
            };
            if read == 0 {
                break;
            }
            let previous = captured.fetch_add(read, Ordering::Relaxed);
            let allowed = read.min(MAX_RUNTIME_OUTPUT_BYTES.saturating_sub(previous));
            if allowed < read {
                exceeded.store(true, Ordering::Relaxed);
            }
            if allowed > 0
                && sender
                    .send((is_stdout, buffer[..allowed].to_vec()))
                    .is_err()
            {
                break;
            }
            if allowed < read {
                break;
            }
        }
    })
}

fn parse_rustc_diagnostics(
    stderr: &str,
    generated: &str,
    source: &str,
    mappings: &[SourceMapSegment],
) -> Vec<PortugolRsCompileDiagnostic> {
    let generated_index = LineIndex::new(generated);
    let source_index = LineIndex::new(source);
    stderr
        .lines()
        .take(1000)
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .filter(|value| {
            value.get("$message_type").and_then(Value::as_str) == Some("diagnostic")
                || value.get("message").and_then(Value::as_str).is_some()
                    && value.get("level").and_then(Value::as_str).is_some()
        })
        .take(100)
        .map(|value| {
            let message = value
                .get("message")
                .and_then(Value::as_str)
                .unwrap_or("Falha ao compilar o Rust gerado.")
                .to_string();
            let severity = value
                .get("level")
                .and_then(Value::as_str)
                .unwrap_or("error")
                .to_string();
            let code = value
                .pointer("/code/code")
                .and_then(Value::as_str)
                .map(str::to_string);
            let rendered = value
                .get("rendered")
                .and_then(Value::as_str)
                .map(str::to_string);
            let primary = value
                .get("spans")
                .and_then(Value::as_array)
                .and_then(|spans| {
                    spans
                        .iter()
                        .find(|span| span.get("is_primary").and_then(Value::as_bool) == Some(true))
                })
                .and_then(|span| {
                    Some((
                        u32::try_from(span.get("byte_start")?.as_u64()?).ok()?,
                        u32::try_from(span.get("byte_end")?.as_u64()?).ok()?,
                    ))
                });
            let (generated_start, generated_end, source_start, source_end, quality) =
                if let Some((start, end)) = primary {
                    let mapping = mappings
                        .iter()
                        .filter(|mapping| {
                            mapping.generated.start <= start && mapping.generated.end >= end
                        })
                        .min_by_key(|mapping| mapping.generated.end - mapping.generated.start);
                    let source_range = mapping.and_then(|mapping| mapping.source);
                    (
                        Some(generated_index.position(start)),
                        Some(generated_index.position(end)),
                        source_range.map(|range| source_index.position(range.start)),
                        source_range.map(|range| source_index.position(range.end)),
                        mapping.map_or(SourceMapQuality::GeneratedOnly, |mapping| {
                            match mapping.quality {
                                RustSourceMapQuality::Exact => SourceMapQuality::Exact,
                                RustSourceMapQuality::Approximate => SourceMapQuality::Approximate,
                                RustSourceMapQuality::GeneratedOnly => {
                                    SourceMapQuality::GeneratedOnly
                                }
                            }
                        }),
                    )
                } else {
                    (None, None, None, None, SourceMapQuality::GeneratedOnly)
                };
            PortugolRsCompileDiagnostic {
                severity,
                code,
                message,
                rendered,
                generated_start,
                generated_end,
                source_start,
                source_end,
                quality,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{SourceMapQuality, parse_rustc_diagnostics};
    use portugol_rust::{
        GeneratedRust, SourceMapQuality as RustSourceMapQuality, SourceMapSegment, SourceRange,
    };

    #[test]
    fn maps_rustc_primary_span_to_utf16_source_positions() {
        let generated_code = "fn main() { let value = String::from(\"😀\"); }\n";
        let source = "funcao inicio() { seja valor: cadeia = \"😀\"; }";
        let generated_start = generated_code.find("\"😀\"").unwrap() as u32;
        let generated_end = generated_start + "\"😀\"".len() as u32;
        let source_start = source.find("\"😀\"").unwrap() as u32;
        let source_end = source_start + "\"😀\"".len() as u32;
        let stderr = format!(
            r#"{{"$message_type":"diagnostic","message":"type mismatch","level":"error","code":{{"code":"E0308"}},"spans":[{{"byte_start":{generated_start},"byte_end":{generated_end},"is_primary":true}}],"rendered":"error[E0308]: type mismatch"}}"#
        );
        let mappings = [SourceMapSegment {
            generated: SourceRange {
                start: generated_start,
                end: generated_end,
            },
            source: Some(SourceRange {
                start: source_start,
                end: source_end,
            }),
            quality: RustSourceMapQuality::Exact,
        }];
        let diagnostics = parse_rustc_diagnostics(&stderr, generated_code, source, &mappings);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].code.as_deref(), Some("E0308"));
        assert_eq!(diagnostics[0].quality, SourceMapQuality::Exact);
        assert_eq!(
            diagnostics[0].source_start.unwrap().offset as usize,
            source[..source_start as usize].encode_utf16().count()
        );
        assert_eq!(
            diagnostics[0].source_end.unwrap().offset as usize,
            source[..source_end as usize].encode_utf16().count()
        );
        assert_eq!(
            diagnostics[0].generated_end.unwrap().offset
                - diagnostics[0].generated_start.unwrap().offset,
            "\"😀\"".encode_utf16().count() as u32
        );
        assert_eq!(
            diagnostics[0].rendered.as_deref(),
            Some("error[E0308]: type mismatch")
        );
    }

    #[test]
    fn diagnostics_without_a_primary_span_remain_generated_only() {
        let diagnostics = parse_rustc_diagnostics(
            r#"{"message":"warning without span","level":"warning","spans":[]}"#,
            "fn main() {}\n",
            "funcao inicio() {}",
            &[],
        );
        assert_eq!(diagnostics.len(), 1);
        assert!(diagnostics[0].source_start.is_none());
        assert!(diagnostics[0].generated_start.is_none());
        assert_eq!(diagnostics[0].quality, SourceMapQuality::GeneratedOnly);
    }

    #[test]
    fn chooses_the_smallest_enclosing_mapping_for_nested_source_ranges() {
        let generated = "fn main() { let x = 1 + 2; }";
        let start = generated.find("1 + 2").unwrap() as u32;
        let end = start + "1 + 2".len() as u32;
        let stderr = format!(
            r#"{{"message":"ambiguous expression","level":"error","spans":[{{"byte_start":{start},"byte_end":{end},"is_primary":true}}]}}"#
        );
        let mappings = [
            SourceMapSegment {
                generated: SourceRange {
                    start: 0,
                    end: generated.len() as u32,
                },
                source: Some(SourceRange { start: 0, end: 40 }),
                quality: RustSourceMapQuality::Approximate,
            },
            SourceMapSegment {
                generated: SourceRange { start, end },
                source: Some(SourceRange { start: 15, end: 20 }),
                quality: RustSourceMapQuality::Exact,
            },
        ];
        let diagnostics = parse_rustc_diagnostics(
            &stderr,
            generated,
            "0123456789012345678901234567890123456789",
            &mappings,
        );
        assert_eq!(diagnostics[0].quality, SourceMapQuality::Exact);
        assert_eq!(diagnostics[0].source_start.unwrap().offset, 15);
    }

    #[test]
    fn malformed_lines_are_ignored_and_unknown_json_fields_are_tolerated() {
        let stderr = "this is not JSON\n{\"message\":\"known diagnostic\",\"level\":\"error\",\"futureField\":42}\n";
        let diagnostics = parse_rustc_diagnostics(stderr, "fn main() {}\n", "", &[]);
        assert_eq!(diagnostics.len(), 1);
        assert_eq!(diagnostics[0].message, "known diagnostic");
        assert_eq!(diagnostics[0].quality, SourceMapQuality::GeneratedOnly);
    }

    #[test]
    fn compile_rejection_is_a_typed_outcome_with_raw_stderr() {
        let code = "fn main() { let _: i32 = \"no inteiro\"; }\n".to_owned();
        let start = code.find("\"no inteiro\"").unwrap() as u32;
        let end = start + "\"no inteiro\"".len() as u32;
        let source = "funcao inicio() { seja valor: inteiro = 1; }";
        let generated = GeneratedRust {
            code,
            mappings: vec![SourceMapSegment {
                generated: SourceRange { start, end },
                source: Some(SourceRange { start: 0, end: 9 }),
                quality: RustSourceMapQuality::Approximate,
            }],
        };
        let result = super::compile_and_run_generated(&generated, source).unwrap();
        match result {
            super::PortugolRsRunResult::CompileError {
                diagnostics,
                stderr,
            } => {
                assert!(!stderr.is_empty());
                assert!(!diagnostics.is_empty());
                assert_eq!(diagnostics[0].quality, SourceMapQuality::Approximate);
                assert!(diagnostics[0].source_start.is_some());
            }
            super::PortugolRsRunResult::Executed { .. } => {
                panic!("código Rust inválido não deveria executar")
            }
            super::PortugolRsRunResult::RuntimeLimit { .. } => {
                panic!("rejeição de compilação não deveria atingir limite de runtime")
            }
        }
    }

    #[test]
    fn tagged_run_result_serializes_the_typescript_contract() {
        let result = super::PortugolRsRunResult::Executed {
            stdout: "17".into(),
            stderr: String::new(),
            exit_code: Some(0),
        };
        let value = serde_json::to_value(result).unwrap();
        assert_eq!(value["status"], "executed");
        assert_eq!(value["exitCode"], 0);
        assert!(value.get("exit_code").is_none());

        let limited = super::PortugolRsRunResult::RuntimeLimit {
            stdout: "parcial".into(),
            stderr: String::new(),
            reason: super::PortugolRsRuntimeLimitReason::StringMemory,
        };
        let value = serde_json::to_value(limited).unwrap();
        assert_eq!(value["status"], "runtime_limit");
        assert_eq!(value["reason"], "string_memory");
        assert_eq!(value["stdout"], "parcial");
    }
}
