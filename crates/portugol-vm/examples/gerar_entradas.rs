//! Gera `tests/corpus/**/*.entrada` para os programas que usam `leia`: cada
//! leitura recebe um valor do tipo pedido, de uma sequência fixa. Só cria o
//! arquivo se o programa pedir entrada e ele ainda não existir (entradas
//! escritas à mão não são sobrescritas). Uso:
//!
//! ```text
//! cargo run -p portugol-vm --example gerar_entradas -- tests/corpus
//! ```

use std::path::{Path, PathBuf};

use portugol_bytecode::ValueType;
use portugol_syntax::{Severity, parse};
use portugol_vm::{Io, Vm};

const MAX_READS: usize = 40;

/// Valores variados por tipo; terminam em "sair" (0, "n", falso) para que
/// menus em laço acabem.
fn answer(ty: ValueType, n: usize) -> String {
    let pick = |values: &[&str]| values[n.min(values.len() - 1)].to_string();
    match ty {
        ValueType::Inteiro => pick(&["5", "3", "8", "2", "7", "1", "4", "6", "0"]),
        ValueType::Real => pick(&["2.5", "7.0", "1.75", "9.5", "3.0", "0.0"]),
        ValueType::Cadeia => pick(&["Maria", "teste", "Portugol", "n"]),
        ValueType::Caracter => pick(&["s", "a", "x", "n"]),
        ValueType::Logico => pick(&["verdadeiro", "falso"]),
        ValueType::Vazio => String::new(),
    }
}

#[derive(Default)]
struct Recorder {
    lines: Vec<String>,
    counts: [usize; 6],
}

impl Io for Recorder {
    fn write(&mut self, _: &str) {}

    fn read_line(&mut self, expected: ValueType) -> Option<String> {
        if self.lines.len() >= MAX_READS {
            return None;
        }
        let slot = expected as usize;
        let line = answer(expected, self.counts[slot]);
        self.counts[slot] += 1;
        self.lines.push(line.clone());
        Some(line)
    }
}

fn main() {
    let dir = std::env::args().nth(1).expect("diretório do corpus");
    let mut files = Vec::new();
    collect(Path::new(&dir), &mut files);
    let mut created = 0;
    for file in files {
        let target = file.with_extension("entrada");
        if target.exists() {
            continue;
        }
        let source = std::fs::read_to_string(&file).unwrap();
        let result = parse(&source);
        let mut diagnostics = result.diagnostics;
        diagnostics.extend(portugol_semantics::analyze(&result.program));
        if diagnostics.iter().any(|d| d.severity == Severity::Error) {
            continue;
        }
        let Ok(module) = portugol_bytecode::compile(&result.program) else {
            continue;
        };
        let mut io = Recorder::default();
        let _ = Vm::new(&module, &mut io)
            .with_seed(42)
            .with_simulated_time(1_704_110_400_000)
            .offline()
            .step(2_000_000);
        if !io.lines.is_empty() {
            std::fs::write(&target, io.lines.join("\n") + "\n").unwrap();
            created += 1;
        }
    }
    println!("{created} arquivo(s) .entrada criados");
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
