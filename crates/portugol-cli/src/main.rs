mod debug;
mod run;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use portugol_semantics::analyze;
use portugol_syntax::{Diagnostic, Severity, parse};

const PORS_STARTER: &str = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FileKind {
    Pors,
    ClassicPor,
}

pub(crate) fn detect_kind(path: &Path) -> Option<FileKind> {
    let ext = path.extension()?.to_str()?;
    if ext.eq_ignore_ascii_case("pors") {
        Some(FileKind::Pors)
    } else if ext.eq_ignore_ascii_case("por") {
        Some(FileKind::ClassicPor)
    } else {
        None
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().skip(1).collect();

    match args.first().map(String::as_str) {
        Some("executar") => return run::main(&args[1..]),
        Some("depurar") => return debug::main(&args[1..]),
        Some("formatar") => return format_command(&args[1..]),
        Some("novo") => return new_command(&args[1..]),
        Some("exportar-rust") => return export_rust_command(&args[1..]),
        Some("exportar-cargo") => return export_cargo_command(&args[1..]),
        _ => {}
    }

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
        if !path.is_dir() && detect_kind(&path).is_none() {
            eprintln!(
                "{}: extensão não suportada (use .pors ou .por)",
                path.display()
            );
            return ExitCode::FAILURE;
        }
        collect_files(&path, &mut files);
    }

    if files.is_empty() {
        eprintln!("nenhum arquivo .pors ou .por encontrado");
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

        let diagnostics: Vec<Diagnostic> = match detect_kind(&file) {
            Some(FileKind::Pors) => match portugol_rs_frontend::lower(&source) {
                Ok(_) => Vec::new(),
                Err(diagnostics) => diagnostics,
            },
            Some(FileKind::ClassicPor) => {
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
                diagnostics
            }
            None => continue,
        };

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

fn new_command(args: &[String]) -> ExitCode {
    if args.len() != 1 || args[0] == "-h" || args[0] == "--help" {
        eprintln!("uso: portugol novo <arquivo.pors>");
        return ExitCode::FAILURE;
    }
    let mut path = PathBuf::from(&args[0]);
    if path.extension().is_none() {
        path.set_extension("pors");
    }
    if detect_kind(&path) != Some(FileKind::Pors) {
        eprintln!(
            "{}: o comando `novo` cria apenas arquivos .pors",
            path.display()
        );
        return ExitCode::FAILURE;
    }
    if path.exists() {
        eprintln!("{}: o arquivo já existe", path.display());
        return ExitCode::FAILURE;
    }
    if let Some(parent) = path.parent()
        && !parent.as_os_str().is_empty()
        && let Err(error) = fs::create_dir_all(parent)
    {
        eprintln!("{}: não foi possível criar pasta: {error}", path.display());
        return ExitCode::FAILURE;
    }
    match fs::write(&path, PORS_STARTER) {
        Ok(()) => {
            println!("{}: criado", path.display());
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{}: não foi possível gravar: {error}", path.display());
            ExitCode::FAILURE
        }
    }
}

fn format_command(args: &[String]) -> ExitCode {
    let mut check_only = false;
    let mut file: Option<&str> = None;
    for arg in args {
        match arg.as_str() {
            "--check" => check_only = true,
            _ if file.is_none() => file = Some(arg.as_str()),
            other => {
                eprintln!("argumento inesperado: {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(file) = file else {
        eprintln!("uso: portugol formatar [--check] <arquivo.pors | arquivo.por>");
        return ExitCode::FAILURE;
    };
    let path = Path::new(file);
    let Some(kind) = detect_kind(path) else {
        eprintln!("{file}: extensão não suportada (use .pors ou .por)");
        return ExitCode::FAILURE;
    };
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{file}: não foi possível ler: {error}");
            return ExitCode::FAILURE;
        }
    };

    let edits: Option<Vec<(usize, usize, String)>> = match kind {
        FileKind::Pors => portugol_rs_frontend::format(&source).map(|edits| {
            edits
                .into_iter()
                .map(|edit| (edit.from, edit.to, edit.insert))
                .collect()
        }),
        FileKind::ClassicPor => portugol_syntax::format::format(&source).map(|edits| {
            edits
                .into_iter()
                .map(|edit| (edit.from, edit.to, edit.insert))
                .collect()
        }),
    };

    let Some(edits) = edits else {
        eprintln!("{file}: não foi possível formatar (corrija os erros do programa primeiro)");
        return ExitCode::FAILURE;
    };

    if edits.is_empty() {
        println!("{file}: já formatado");
        return ExitCode::SUCCESS;
    }
    if check_only {
        eprintln!("{file}: precisa de formatação");
        return ExitCode::FAILURE;
    }

    let formatted = match apply_utf16_edits(&source, edits) {
        Ok(text) => text,
        Err(error) => {
            eprintln!("{file}: erro interno ao aplicar formatação: {error}");
            return ExitCode::FAILURE;
        }
    };
    match fs::write(path, formatted) {
        Ok(()) => {
            println!("{file}: formatado");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{file}: não foi possível gravar: {error}");
            ExitCode::FAILURE
        }
    }
}

fn export_rust_command(args: &[String]) -> ExitCode {
    if args.len() != 1 || args[0] == "-h" || args[0] == "--help" {
        eprintln!("uso: portugol exportar-rust <arquivo.pors | arquivo.por>");
        return ExitCode::FAILURE;
    }
    let file = &args[0];
    let path = Path::new(file);
    let Some(kind) = detect_kind(path) else {
        eprintln!("{file}: extensão não suportada (use .pors ou .por)");
        return ExitCode::FAILURE;
    };
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{file}: não foi possível ler: {error}");
            return ExitCode::FAILURE;
        }
    };

    match kind {
        FileKind::Pors => match portugol_rust::export_portugol_rs(&source) {
            Ok(code) => {
                print!("{code}");
                ExitCode::SUCCESS
            }
            Err(diagnostics) => {
                for diagnostic in diagnostics {
                    eprintln!("{}", diagnostic.render(&source, file));
                }
                ExitCode::FAILURE
            }
        },
        FileKind::ClassicPor => match portugol_rust::export(&source) {
            Ok(code) => {
                print!("{code}");
                ExitCode::SUCCESS
            }
            Err(diagnostic) => {
                eprintln!("{}", diagnostic.render(&source, file));
                ExitCode::FAILURE
            }
        },
    }
}

fn export_cargo_command(args: &[String]) -> ExitCode {
    if args.len() != 2 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("uso: portugol exportar-cargo <arquivo.pors> <pasta-destino>");
        return ExitCode::FAILURE;
    }
    let file = &args[0];
    let target_dir = Path::new(&args[1]);
    let path = Path::new(file);
    if detect_kind(path) != Some(FileKind::Pors) {
        eprintln!("{file}: `exportar-cargo` aceita apenas arquivos .pors");
        return ExitCode::FAILURE;
    }
    if target_dir.join("Cargo.toml").exists() {
        eprintln!(
            "{}: já existe um Cargo.toml na pasta de destino",
            target_dir.display()
        );
        return ExitCode::FAILURE;
    }
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{file}: não foi possível ler: {error}");
            return ExitCode::FAILURE;
        }
    };
    let package_name = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("programa-pors");
    let project = match portugol_rust::export_portugol_rs_cargo_project(&source, package_name) {
        Ok(project) => project,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.render(&source, file));
            }
            return ExitCode::FAILURE;
        }
    };
    match portugol_rust::write_cargo_project(&project, target_dir) {
        Ok(()) => {
            println!(
                "{}: projeto Cargo \"{}\" gerado",
                target_dir.display(),
                project.package_name
            );
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{}: não foi possível gravar projeto Cargo: {error}",
                target_dir.display()
            );
            ExitCode::FAILURE
        }
    }
}

fn apply_utf16_edits(source: &str, edits: Vec<(usize, usize, String)>) -> Result<String, String> {
    let mut units: Vec<u16> = source.encode_utf16().collect();
    for (from, to, insert) in edits.into_iter().rev() {
        units.splice(from..to, insert.encode_utf16());
    }
    String::from_utf16(&units).map_err(|error| error.to_string())
}

fn print_usage() {
    println!(
        "portugol {}\n\
         \n\
         Uso:\n\
           portugol [opções] <arquivo.pors | arquivo.por | diretório>...   valida os arquivos\n\
           portugol novo <arquivo.pors>                                    cria um modelo .pors\n\
           portugol formatar [--check] <arquivo.pors | arquivo.por>        formata o código-fonte\n\
           portugol exportar-rust <arquivo.pors | arquivo.por>             imprime o Rust gerado\n\
           portugol exportar-cargo <arquivo.pors> <pasta>                  gera um projeto Cargo pronto\n\
           portugol executar [--bytecode] <arquivo.pors | arquivo.por> [-- parâmetros...]\n\
                                                                           executa um programa na VM\n\
           portugol depurar [--ponto <linha>]... <arquivo.pors>            depura um programa .pors interativamente\n\
         \n\
         Opções da validação:\n\
           -q, --quiet   só mostra arquivos com erro\n\
           --syntax-only só valida a sintaxe de arquivos .por (desliga a análise semântica)\n\
           -h, --help    mostra esta ajuda",
        env!("CARGO_PKG_VERSION")
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
    } else if detect_kind(path).is_some() {
        files.push(path.to_path_buf());
    }
}

/// Os exemplos históricos do Portugol Studio estão em ISO-8859-1; aceitamos
/// UTF-8 (com ou sem BOM) e caímos para Latin-1 quando necessário.
pub(crate) fn read_source(path: &Path) -> std::io::Result<String> {
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
