//! Testes de regressão sobre o corpus `tests/corpus` (os exemplos do Portugol
//! Studio original).
//!
//! Para cada `programa.por` existe um `programa.saida` com os diagnósticos, a
//! saída do programa, um resumo dos quadros desenhados (Graficos) e como a
//! execução terminou. Um `programa.entrada`
//! opcional fornece as linhas lidas pelo `leia`. A execução é determinística:
//! semente fixa, relógio simulado e limite de instruções.
//!
//! Quando uma mudança altera a saída de propósito, regrave com:
//!
//! ```text
//! PORTUGOL_ATUALIZAR=1 cargo test -p portugol-vm --test regressao
//! ```
//!
//! e revise o diff dos `.saida` antes de commitar.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portugol_syntax::span::line_col;
use portugol_syntax::{Severity, parse};
use portugol_vm::graphics::{InputState, WindowCommand};
use portugol_vm::{BufferIo, Host, Sound, Status, Vm};
use rodio::{Decoder, Source};

/// Limite por programa; jogos com laço infinito param aqui.
const BUDGET: usize = 2_000_000;
/// 2024-01-01 12:00:00 UTC.
const EPOCH_MS: u64 = 1_704_110_400_000;
const SEED: u64 = 42;

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/corpus")
}

fn collect(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).expect("ler tests/corpus").flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect(&path, files);
        } else if path.extension().is_some_and(|e| e == "por") {
            files.push(path);
        }
    }
}

/// Cópia do corpus numa pasta temporária: os programas rodam a partir da
/// própria pasta (como na IDE) e podem gravar arquivos sem sujar o
/// repositório. O caminho aparece como `<pasta>` nas saídas.
fn copy_corpus() -> PathBuf {
    let dir = std::env::temp_dir().join(format!("portugol-regressao-{}", std::process::id()));
    copy_dir(&corpus_dir(), &dir);
    dir
}

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("criar pasta temporária");
    for entry in std::fs::read_dir(from).expect("ler corpus").flatten() {
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), &target).expect("copiar arquivo do corpus");
        }
    }
}

/// Janela gráfica sem tela: aceita os quadros, conta e resume o conteúdo.
/// Teclado e mouse nunca recebem eventos; esperar uma tecla dá erro.
#[derive(Default)]
struct Frames {
    count: usize,
    hash: u64,
}

struct Headless {
    frames: Arc<Mutex<Frames>>,
    input: Arc<InputState>,
}

struct SilentClip {
    duration: Duration,
    position: Duration,
}

impl Sound for SilentClip {
    fn play(&mut self, _looping: bool) -> Result<(), String> {
        Ok(())
    }

    fn pause(&mut self) {}

    fn stop(&mut self) {}

    fn duration(&self) -> Duration {
        self.duration
    }

    fn position(&self) -> Duration {
        self.position
    }

    fn seek(&mut self, position: Duration) -> Result<(), String> {
        self.position = position.min(self.duration);
        Ok(())
    }

    fn set_gain(&mut self, _gain: f32) {}
}

impl Host for Headless {
    fn load_sound(&mut self, path: &Path) -> Result<Box<dyn Sound>, String> {
        let file = std::fs::File::open(path).map_err(|error| error.to_string())?;
        let source = Decoder::try_from(file).map_err(|error| error.to_string())?;
        let samples_per_second =
            u64::from(source.channels().get()) * u64::from(source.sample_rate().get());
        Ok(Box::new(SilentClip {
            duration: Duration::from_secs_f64(source.count() as f64 / samples_per_second as f64),
            position: Duration::ZERO,
        }))
    }

    fn graphics_open(&mut self, _: i32, _: i32) -> Result<(), String> {
        Ok(())
    }

    fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
        let mut frames = self.frames.lock().unwrap();
        frames.count += 1;
        // FNV-1a: o mesmo programa gera os mesmos quadros em qualquer máquina.
        for byte in frame {
            frames.hash = (frames.hash ^ byte as u64).wrapping_mul(0x100_0000_01b3);
        }
        Ok(())
    }

    fn graphics_snapshot(
        &mut self,
        _: Vec<u8>,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>, String> {
        Ok(vec![0; (width * height * 4) as usize])
    }

    fn graphics_window(&mut self, _: WindowCommand) -> Result<(), String> {
        Ok(())
    }

    fn screen_size(&mut self) -> Result<(i32, i32), String> {
        Ok((1920, 1080))
    }

    fn input(&mut self) -> Option<Arc<InputState>> {
        Some(Arc::clone(&self.input))
    }
}

/// Relatório textual de uma execução, no formato dos `.saida`.
fn report(source: &str, input: &str, dir: &Path, user: &Path) -> String {
    let mut out = String::new();
    let result = parse(source);
    let mut diagnostics = result.diagnostics;
    if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
        diagnostics.extend(portugol_semantics::analyze(&result.program));
    }

    out.push_str("# diagnósticos\n");
    for d in &diagnostics {
        let (line, column) = line_col(source, d.span.start);
        let _ = writeln!(
            out,
            "{} {} {line}:{column} {}",
            d.severity, d.code, d.message
        );
    }

    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        out.push_str("# resultado\nnão executado\n");
        return out;
    }

    let module = match portugol_bytecode::compile(&result.program) {
        Ok(module) => module,
        Err(error) => {
            let _ = writeln!(out, "# resultado\nerro de compilação: {}", error.message);
            return out;
        }
    };

    let frames = Arc::new(Mutex::new(Frames {
        count: 0,
        hash: 0xcbf2_9ce4_8422_2325,
    }));
    let keyboard = Arc::new(InputState::default());
    keyboard.close();
    let host = Headless {
        frames: Arc::clone(&frames),
        input: keyboard,
    };
    let mut io = BufferIo::new(input);
    let outcome = Vm::new(&module, &mut io)
        .with_host(Box::new(host))
        .with_seed(SEED)
        .with_simulated_time(EPOCH_MS)
        .offline()
        .with_working_dir(dir)
        .with_user_dir(user)
        .step(BUDGET);
    out.push_str("# saída\n");
    out.push_str(&io.output);
    if !io.output.ends_with('\n') {
        out.push('\n');
    }
    let frames = frames.lock().unwrap();
    if frames.count > 0 {
        let _ = writeln!(
            out,
            "# gráficos\n{} quadro(s), resumo {:016x}",
            frames.count, frames.hash
        );
    }
    out.push_str("# resultado\n");
    let outcome = outcome.map_err(|mut error| {
        error.message = error
            .message
            .replace(&dir.display().to_string(), "<pasta>")
            .replace(&user.display().to_string(), "<usuário>");
        if cfg!(windows) {
            error.message = error.message.replace('\\', "/");
        }
        error
    });
    match outcome {
        Ok(Status::Finished) => out.push_str("fim\n"),
        Ok(Status::Running) => out.push_str("limite de instruções\n"),
        Err(error) => {
            let (line, column) = line_col(source, error.span.start);
            let _ = writeln!(out, "erro {line}:{column}: {}", error.message);
        }
    }
    out
}

#[test]
fn corpus_do_portugol_studio() {
    let mut files = Vec::new();
    collect(&corpus_dir(), &mut files);
    files.sort();
    assert!(
        files.len() >= 196,
        "corpus incompleto: {} arquivos",
        files.len()
    );

    let update = std::env::var_os("PORTUGOL_ATUALIZAR").is_some();
    let mut failures = Vec::new();
    let copy = copy_corpus();
    let user = copy.join(".usuario");
    std::fs::create_dir_all(&user).expect("criar diretório do usuário temporário");

    for file in &files {
        let source = std::fs::read_to_string(file).expect("ler .por");
        let input = std::fs::read_to_string(file.with_extension("entrada")).unwrap_or_default();
        let program_dir = copy
            .join(file.strip_prefix(corpus_dir()).unwrap())
            .parent()
            .unwrap()
            .to_path_buf();
        let actual = report(&source, &input, &program_dir, &user);
        let expected_path = file.with_extension("saida");

        if update {
            std::fs::write(&expected_path, &actual).expect("gravar .saida");
            continue;
        }
        let expected = std::fs::read_to_string(&expected_path).unwrap_or_default();
        if expected != actual {
            failures.push(format!(
                "{}\n{}",
                file.strip_prefix(corpus_dir()).unwrap_or(file).display(),
                first_difference(&expected, &actual)
            ));
        }
    }

    let _ = std::fs::remove_dir_all(&copy);
    assert!(
        failures.is_empty(),
        "{} de {} programas mudaram de comportamento \
         (se for intencional, rode com PORTUGOL_ATUALIZAR=1):\n\n{}",
        failures.len(),
        files.len(),
        failures.join("\n\n")
    );
}

fn first_difference(expected: &str, actual: &str) -> String {
    let mut expected_lines = expected.lines();
    let mut actual_lines = actual.lines();
    let mut line = 1;
    loop {
        match (expected_lines.next(), actual_lines.next()) {
            (Some(e), Some(a)) if e == a => line += 1,
            (None, None) => return "  (diferença só em quebras de linha)".to_string(),
            (e, a) => {
                return format!(
                    "  linha {line}\n  esperado: {}\n  obtido:   {}",
                    e.unwrap_or("<fim>"),
                    a.unwrap_or("<fim>")
                );
            }
        }
    }
}
