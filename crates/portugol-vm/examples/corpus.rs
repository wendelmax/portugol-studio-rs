//! Executa todos os `.por` de um diretório na VM, sem entrada e com limite de
//! instruções, e classifica o resultado. Uso:
//!
//! ```text
//! cargo run -p portugol-vm --example corpus -- <diretório> [-v]
//! ```

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use portugol_syntax::{Severity, parse};
use portugol_vm::{BufferIo, Status, Vm};

const BUDGET: usize = 20_000_000;

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
        let outcome = classify(&source);
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

fn classify(source: &str) -> String {
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
    let mut vm = Vm::new(&module, &mut io).with_seed(42);
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
