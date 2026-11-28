mod debug;
mod desktop;
mod run;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use ferro_hir::{Diagnostic, Severity};

const FERRO_STARTER: &str = "funcao inicio() {\n    seja mutavel idade: inteiro = 16;\n    idade += 1;\n    escreva(idade);\n}\n";

use ferro_formatos::Kind;

pub(crate) fn is_ferro_file(path: &Path) -> bool {
    ferro_formatos::kind_of(path, &[Kind::Source]).is_some()
}

pub(crate) fn is_legacy_file(path: &Path) -> bool {
    ferro_formatos::is_retired_path(path)
}

pub(crate) fn report_unsupported_extension(path: &Path) {
    if is_legacy_file(path) {
        eprintln!(
            "{}: arquivos legados .por/.pex não são suportados no Ferro Studio; use arquivos {}",
            path.display(),
            ferro_formatos::describe(&[Kind::Source])
        );
    } else {
        eprintln!(
            "{}: extensão não suportada (use {})",
            path.display(),
            ferro_formatos::describe_alternatives(&[Kind::Source, Kind::Design, Kind::Manifest])
        );
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

    for arg in &args {
        match arg.as_str() {
            "-q" | "--quiet" => quiet = true,
            _ => paths.push(PathBuf::from(arg)),
        }
    }

    let mut files = Vec::new();
    for path in paths {
        if !path.is_dir() && !is_ferro_file(&path) && !desktop::is_markup_file(&path) {
            report_unsupported_extension(&path);
            return ExitCode::FAILURE;
        }
        collect_files(&path, &mut files);
    }

    if files.is_empty() {
        eprintln!(
            "nenhum arquivo {} encontrado",
            ferro_formatos::describe_alternatives(&[Kind::Source, Kind::Design, Kind::Manifest])
        );
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

        let diagnostics: Vec<Diagnostic> = if desktop::is_markup_file(&file) {
            desktop::diagnose(&file, &source)
        } else {
            // O code-behind de uma tela (`Nome.fer` ao lado de `Nome.design`)
            // enxerga os tipos da tela e não precisa de `inicio`.
            let options = desktop::code_behind_design(&file)
                .map(|design| ferro_tela::codebehind::lower_options(&design))
                .unwrap_or_default();
            match ferro_frontend::lower_with_warnings(&source, &options) {
                Ok((_, warnings)) => warnings,
                Err(diagnostics) => diagnostics,
            }
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
    if args.first().is_some_and(|arg| arg == "--desktop") {
        return desktop::new_project(&args[1..]);
    }
    if args.len() != 1 || args[0] == "-h" || args[0] == "--help" {
        eprintln!(
            "uso: ferro novo <arquivo.fer>\n     ferro novo --desktop <pasta> [--nome \"Nome do aplicativo\"] [--exemplo contador]"
        );
        return ExitCode::FAILURE;
    }
    let mut path = PathBuf::from(&args[0]);
    if path.extension().is_none() {
        path.set_extension(ferro_formatos::extension(Kind::Source));
    }
    if !is_ferro_file(&path) {
        eprintln!(
            "{}: o comando `novo` cria apenas arquivos {}",
            path.display(),
            ferro_formatos::describe(&[Kind::Source])
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
    match fs::write(&path, FERRO_STARTER) {
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
        eprintln!("uso: ferro formatar [--check] <arquivo.fer | .design | .ferproj>");
        return ExitCode::FAILURE;
    };
    let path = Path::new(file);
    if desktop::is_markup_file(path) {
        return match read_source(path) {
            Ok(source) => desktop::format_markup(path, &source, check_only),
            Err(error) => {
                eprintln!("{file}: não foi possível ler: {error}");
                ExitCode::FAILURE
            }
        };
    }
    if !is_ferro_file(path) {
        report_unsupported_extension(path);
        return ExitCode::FAILURE;
    }
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{file}: não foi possível ler: {error}");
            return ExitCode::FAILURE;
        }
    };

    let options = desktop::code_behind_design(path)
        .map(|design| ferro_tela::codebehind::lower_options(&design))
        .unwrap_or_default();
    let edits: Option<Vec<(usize, usize, String)>> = ferro_frontend::format_with(&source, &options)
        .map(|edits| {
            edits
                .into_iter()
                .map(|edit| (edit.from, edit.to, edit.insert))
                .collect()
        });

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
        eprintln!("uso: ferro exportar-rust <arquivo.fer>");
        return ExitCode::FAILURE;
    }
    let file = &args[0];
    let path = Path::new(file);
    if !is_ferro_file(path) {
        report_unsupported_extension(path);
        return ExitCode::FAILURE;
    }
    let source = match read_source(path) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{file}: não foi possível ler: {error}");
            return ExitCode::FAILURE;
        }
    };

    match ferro_rust::export_ferro_rs(&source) {
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
    }
}

fn export_cargo_command(args: &[String]) -> ExitCode {
    if args.len() != 2 || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("uso: ferro exportar-cargo <arquivo.fer|tela.design> <pasta-destino>");
        return ExitCode::FAILURE;
    }
    let file = &args[0];
    let target_dir = Path::new(&args[1]);
    let path = Path::new(file);
    if ferro_formatos::kind_of(path, &[Kind::Design]).is_some() {
        return export_screen_command(path, target_dir);
    }
    if !is_ferro_file(path) {
        report_unsupported_extension(path);
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
        .unwrap_or("programa-ferro");
    let project = match ferro_rust::export_ferro_rs_cargo_project(&source, package_name) {
        Ok(project) => project,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.render(&source, file));
            }
            return ExitCode::FAILURE;
        }
    };
    match ferro_rust::write_cargo_project(&project, target_dir) {
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

/// `exportar-cargo` de uma tela: o `.design` e o code-behind ao lado dele
/// viram um projeto Cargo com Slint (ADR 0018, seção 6).
fn export_screen_command(design_path: &Path, target_dir: &Path) -> ExitCode {
    if target_dir.join("Cargo.toml").exists() {
        eprintln!(
            "{}: já existe um Cargo.toml na pasta de destino",
            target_dir.display()
        );
        return ExitCode::FAILURE;
    }
    let code_path = design_path.with_extension("fer");
    let (design, code) = match (read_source(design_path), read_source(&code_path)) {
        (Ok(design), Ok(code)) => (design, code),
        (Err(error), _) => {
            eprintln!("{}: não foi possível ler: {error}", design_path.display());
            return ExitCode::FAILURE;
        }
        (_, Err(error)) => {
            eprintln!(
                "{}: não foi possível ler o código da tela: {error}",
                code_path.display()
            );
            return ExitCode::FAILURE;
        }
    };
    let name = design_path
        .parent()
        .and_then(|folder| folder.file_name())
        .and_then(|name| name.to_str())
        .or_else(|| design_path.file_stem().and_then(|stem| stem.to_str()))
        .unwrap_or("tela-ferro");
    let project = match ferro_rust::export_screen_cargo_project(&design, &code, name) {
        Ok(project) => project,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };
    let folder = design_path.parent().unwrap_or(Path::new("."));
    let written = (|| -> std::io::Result<()> {
        for (relative, content) in &project.files {
            let target = target_dir.join(relative);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(target, content)?;
        }
        for image in &project.images {
            let target = target_dir.join("ui").join(image);
            if let Some(parent) = target.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(folder.join(image), target)?;
        }
        Ok(())
    })();
    match written {
        Ok(()) => {
            println!(
                "{}: projeto Cargo \"{}\" gerado com a tela em Slint",
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
        "ferro {}\n\
         \n\
         Uso:\n\
           ferro [opções] <arquivo | diretório>...            valida .fer, .design e .ferproj\n\
           ferro novo <arquivo.fer>                           cria um modelo .fer\n\
           ferro novo --desktop <pasta> [--nome <nome>]       cria um projeto desktop\n\
           ferro novo --desktop <pasta> --exemplo contador    cria o projeto de exemplo\n\
           ferro formatar [--check] <arquivo>                 formata .fer, .design ou .ferproj\n\
           ferro exportar-rust <arquivo.fer>                  imprime o Rust gerado\n\
           ferro exportar-cargo <arquivo.fer> <pasta>         gera um projeto Cargo pronto\n\
           ferro exportar-cargo <tela.design> <pasta>         gera um projeto Cargo com a tela em Slint\n\
           ferro executar [--bytecode] <arquivo.fer>          executa um programa na VM\n\
           ferro depurar [--ponto <linha>]... <arquivo.fer>   depura um programa interativamente\n\
         \n\
         Opções da validação:\n\
           -q, --quiet   só mostra arquivos com erro\n\
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
    } else if is_ferro_file(path) || desktop::is_markup_file(path) {
        files.push(path.to_path_buf());
    }
}

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
