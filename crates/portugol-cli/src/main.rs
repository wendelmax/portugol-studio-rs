use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use portugol_semantics::analyze;
use portugol_syntax::{Severity, parse};

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        print_usage();
        return ExitCode::SUCCESS;
    }

    let mut paths = Vec::new();
    let mut quiet = false;
    let mut syntax_only = false;

    for arg in &args {
        match arg.as_str() {
            "-q" | "--quiet" => quiet = true,
            "--syntax-only" => syntax_only = true,
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    let mut files = Vec::new();
    for path in paths {
        collect_files(&path, &mut files);
    }

    if files.is_empty() {
        eprintln!("nenhum arquivo .por encontrado");
        return ExitCode::FAILURE;
    }

    let mut total = 0usize;
    let mut failed = 0usize;
    let mut total_errors = 0usize;
    let mut total_warnings = 0usize;

    for file in files {
        total += 1;

        let source = match read_source(&file) {
            Ok(source) => source,
            Err(error) => {
                eprintln!("{}: não foi possível ler: {}", file.display(), error);
                failed += 1;
                continue;
            }
        };

        let result = parse(&source);
        let mut diagnostics = result.diagnostics;

        // Com erro de sintaxe a AST pode estar incompleta; a semântica geraria
        // diagnósticos em cascata. Só roda quando a sintaxe está limpa.
        if !syntax_only
            && !diagnostics
                .iter()
                .any(|diagnostic| diagnostic.severity == Severity::Error)
        {
            diagnostics.extend(analyze(&result.program));
        }

        let errors = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .count();
        let warnings = diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Warning)
            .count();

        total_errors += errors;
        total_warnings += warnings;

        if errors > 0 {
            failed += 1;
            println!("{}: {} erro(s)", file.display(), errors);
            for diagnostic in &diagnostics {
                println!(
                    "  {}",
                    diagnostic.render(&source, &file.display().to_string())
                );
            }
        } else if !quiet {
            println!("{}: ok", file.display());
        }
    }

    println!();
    println!(
        "{} arquivo(s), {} com erro(s), {} erro(s), {} aviso(s)",
        total, failed, total_errors, total_warnings
    );

    if failed == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn print_usage() {
    println!(
        "portugol-parse 0.1.0\n\
         \n\
         Uso:\n\
           portugol-parse [opções] <arquivo.por | diretório>...\n\
         \n\
         Opções:\n\
           -q, --quiet   só mostra arquivos com erro\n\
           --syntax-only só valida a sintaxe (desliga a análise semântica)\n\
           -h, --help    mostra esta ajuda"
    );
}

fn collect_files(path: &Path, files: &mut Vec<PathBuf>) {
    if path.is_dir() {
        let Ok(entries) = fs::read_dir(path) else {
            return;
        };
        for entry in entries.flatten() {
            collect_files(&entry.path(), files);
        }
    } else if path.extension().is_some_and(|ext| ext == "por") {
        files.push(path.to_path_buf());
    }
}

/// Os exemplos históricos do Portugol Studio estão em ISO-8859-1; aceitamos
/// UTF-8 (com ou sem BOM) e caímos para Latin-1 quando necessário.
fn read_source(path: &Path) -> std::io::Result<String> {
    let bytes = fs::read(path)?;

    match String::from_utf8(bytes.clone()) {
        Ok(mut text) => {
            if text.starts_with('\u{feff}') {
                text.remove(0);
            }
            Ok(text)
        }
        Err(_) => Ok(bytes.iter().map(|&byte| byte as char).collect()),
    }
}
