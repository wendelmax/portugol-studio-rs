//! Executa todos os `.por` de um diretório na VM, sem entrada e com limite de
//! instruções, e classifica o resultado. Uso:
//!
//! ```text
//! cargo run -p portugol-vm --example corpus -- <diretório> [-v]
//! ```
//!
//! Usa o mesmo ambiente da suíte de regressão: host gráfico headless, diretório
//! de trabalho de cada exemplo, tempo simulado e semente fixa. Assim os jogos e
//! exemplos com imagens não são contados como erro de ambiente.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use portugol_syntax::{Severity, parse};
use portugol_vm::graphics::{InputState, WindowCommand};
use portugol_vm::{BufferIo, Host, Status, Vm};

const BUDGET: usize = 20_000_000;
const SEED: u64 = 42;
const EPOCH_MS: u64 = 1_600_000_000_000;

#[derive(Default)]
struct Frames {
    count: usize,
    hash: u64,
}

struct Headless {
    frames: Arc<Mutex<Frames>>,
    input: Arc<InputState>,
}

impl Host for Headless {
    fn graphics_open(&mut self, _: i32, _: i32) -> Result<(), String> {
        Ok(())
    }

    fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
        let mut frames = self.frames.lock().unwrap();
        frames.count += 1;
        for byte in frame {
            frames.hash = (frames.hash ^ byte as u64).wrapping_mul(0x100_0000_01b3);
        }
        Ok(())
    }

    fn graphics_snapshot(&mut self, _: Vec<u8>, width: i32, height: i32) -> Result<Vec<u8>, String> {
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

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let verbose = args.iter().any(|a| a == "-v");
    let dir = args
        .iter()
        .find(|a| !a.starts_with('-'))
        .expect("diretório");

    let mut files = Vec::new();
    collect(Path::new(dir), &mut files);
    files.sort();

    let mut summary: BTreeMap<String, usize> = BTreeMap::new();
    for file in &files {
        let bytes = std::fs::read(file).unwrap();
        let source = String::from_utf8(bytes.clone())
            .unwrap_or_else(|_| bytes.iter().map(|&b| b as char).collect());
        let outcome = classify(&source, file.parent().unwrap_or(Path::new(".")));
        if verbose || outcome.starts_with("ERRO") {
            println!("{}: {outcome}", file.display());
        }
        let key = outcome.split(':').next().unwrap().to_string();
        *summary.entry(key).or_default() += 1;
    }
    println!();
    for (key, count) in summary {
        println!("{count:4}  {key}");
    }
}

fn classify(source: &str, dir: &Path) -> String {
    let result = parse(source);
    let mut diagnostics = result.diagnostics;
    if !diagnostics.iter().any(|d| d.severity == Severity::Error) {
        diagnostics.extend(portugol_semantics::analyze(&result.program));
    }
    if diagnostics.iter().any(|d| d.severity == Severity::Error) {
        return "não analisado (erro de sintaxe/semântica)".to_string();
    }
    let module = match portugol_bytecode::compile(&result.program) {
        Ok(module) => module,
        Err(error) => return format!("ERRO de compilação: {}", error.message),
    };
    let mut io = BufferIo::default();
    let keyboard = Arc::new(InputState::default());
    keyboard.close();
    let host = Headless {
        frames: Arc::new(Mutex::new(Frames {
            count: 0,
            hash: 0xcbf2_9ce4_8422_2325,
        })),
        input: keyboard,
    };
    let mut vm = Vm::new(&module, &mut io)
        .with_host(Box::new(host))
        .with_seed(SEED)
        .with_simulated_time(EPOCH_MS)
        .offline()
        .with_working_dir(dir);
    match vm.step(BUDGET) {
        Ok(Status::Finished) => "executou até o fim".to_string(),
        Ok(Status::Running) => "limite de instruções (laço de jogo/espera)".to_string(),
        Err(error) if error.message.contains("entrada de dados") => {
            "parou no leia (sem entrada)".to_string()
        }
        Err(error) if error.message.contains("ainda não está disponível") => {
            let name = error.message.split('"').nth(1).unwrap_or("");
            let library = name.split('.').next().unwrap_or("");
            format!("biblioteca ainda não portada: {library}")
        }
        Err(error) => format!("ERRO de execução: {}", error.message),
    }
}

fn collect(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        for entry in std::fs::read_dir(path).unwrap().flatten() {
            collect(&entry.path(), files);
        }
    } else if path.extension().is_some_and(|e| e == "por") {
        files.push(path.to_path_buf());
    }
}
