//! Comandos do CLI para projetos desktop PORS (ADR 0018): criar o projeto,
//! validar e formatar `.design` e `.ferproj`.

use std::fs;
use std::path::Path;
use std::process::ExitCode;

use ferro_hir::Diagnostic;
use ferro_tela::project;

use ferro_formatos::Kind;

pub(crate) fn is_design_file(path: &Path) -> bool {
    ferro_formatos::kind_of(path, &[Kind::Design]).is_some()
}

/// Arquivos de marcação de projetos desktop: `.design` e `.ferproj`.
pub(crate) fn is_markup_file(path: &Path) -> bool {
    ferro_formatos::classify_path(path).is_some_and(|m| m.kind().is_markup())
}

/// Texto do `.design` com o mesmo nome, quando `path` é o code-behind de uma
/// tela (`Nome.fer` ao lado de `Nome.design`).
pub(crate) fn code_behind_design(path: &Path) -> Option<String> {
    let design = path.with_extension(ferro_tela::FILE_EXTENSION);
    design
        .is_file()
        .then(|| fs::read_to_string(design).ok())
        .flatten()
}

/// Diagnósticos de um `.design` ou `.ferproj`. No manifesto, também confere
/// se a janela inicial existe na pasta dele.
pub(crate) fn diagnose(path: &Path, source: &str) -> Vec<Diagnostic> {
    if is_design_file(path) {
        return ferro_tela::analyze(source).diagnostics;
    }
    let analysis = project::analyze_manifest(source);
    let mut diagnostics = analysis.diagnostics;
    if let Some(manifest) = analysis.manifest {
        let folder = path.parent().filter(|p| !p.as_os_str().is_empty());
        let designs = design_files(folder.unwrap_or(Path::new(".")));
        let names: Vec<&str> = designs.iter().map(String::as_str).collect();
        diagnostics.extend(project::check_start_window(&manifest, &names));
    }
    diagnostics
}

fn design_files(folder: &Path) -> Vec<String> {
    let Ok(entries) = fs::read_dir(folder) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter(|entry| entry.path().is_file() && is_design_file(&entry.path()))
        .filter_map(|entry| entry.file_name().to_str().map(str::to_owned))
        .collect()
}

/// `portugol novo --desktop <pasta> [--nome "Nome do App"]`
pub(crate) fn new_project(args: &[String]) -> ExitCode {
    let usage = || {
        eprintln!("uso: portugol novo --desktop <pasta> [--nome \"Nome do aplicativo\"]");
        ExitCode::FAILURE
    };
    let mut folder: Option<&str> = None;
    let mut name: Option<&str> = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--nome" => {
                let Some(value) = args.get(index + 1) else {
                    return usage();
                };
                name = Some(value);
                index += 2;
            }
            "-h" | "--help" => return usage(),
            other if folder.is_none() => {
                folder = Some(other);
                index += 1;
            }
            _ => return usage(),
        }
    }
    let Some(folder) = folder else {
        return usage();
    };
    let folder = Path::new(folder);
    let name = match name {
        Some(name) => name.to_owned(),
        None => match folder.file_name().and_then(|n| n.to_str()) {
            Some(name) => name.to_owned(),
            None => {
                eprintln!("{}: informe o nome com --nome", folder.display());
                return ExitCode::FAILURE;
            }
        },
    };

    let template = match project::desktop_template(&name) {
        Ok(template) => template,
        Err(message) => {
            eprintln!("{message}");
            return ExitCode::FAILURE;
        }
    };

    if folder.exists() {
        let empty = fs::read_dir(folder).is_ok_and(|mut entries| entries.next().is_none());
        if !folder.is_dir() || !empty {
            eprintln!(
                "{}: a pasta já existe e não está vazia; escolha outra pasta",
                folder.display()
            );
            return ExitCode::FAILURE;
        }
    }

    let result = (|| -> std::io::Result<()> {
        fs::create_dir_all(folder)?;
        for directory in &template.directories {
            fs::create_dir_all(folder.join(directory))?;
        }
        for file in &template.files {
            fs::write(folder.join(&file.path), &file.content)?;
        }
        Ok(())
    })();
    match result {
        Ok(()) => {
            println!("{}: projeto \"{}\" criado", folder.display(), name.trim());
            for file in &template.files {
                println!("  {}", file.path);
            }
            for directory in &template.directories {
                println!("  {directory}/");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!(
                "{}: não foi possível criar o projeto: {error}",
                folder.display()
            );
            ExitCode::FAILURE
        }
    }
}

/// Formata um `.design` ou `.ferproj`.
pub(crate) fn format_markup(path: &Path, source: &str, check_only: bool) -> ExitCode {
    let file = path.display();
    let formatted = match ferro_tela::format_source(source) {
        Ok(formatted) => formatted,
        Err(diagnostic) => {
            eprintln!("{}", diagnostic.render(source, &file.to_string()));
            eprintln!("{file}: não foi possível formatar (corrija o erro de sintaxe primeiro)");
            return ExitCode::FAILURE;
        }
    };
    if formatted == source {
        println!("{file}: já formatado");
        return ExitCode::SUCCESS;
    }
    if check_only {
        eprintln!("{file}: precisa de formatação");
        return ExitCode::FAILURE;
    }
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
