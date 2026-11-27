//! Comando `portugol migrar`: renomeia arquivos com extensões antigas
//! (como `.pors`) para suas extensões canônicas (como `.fer`).

use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

#[derive(Debug)]
struct MigrationItem {
    from: PathBuf,
    to: PathBuf,
}

fn usage() -> ExitCode {
    eprintln!("uso: portugol migrar [--sim] <pasta | arquivo>...");
    ExitCode::FAILURE
}

pub fn main(args: &[String]) -> ExitCode {
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        println!(
            "uso: portugol migrar [--sim] <pasta | arquivo>...\n\n\
             Renomeia arquivos com extensões antigas (como .pors) para a extensão canônica (.fer).\n\
             Sem --sim, lista as alterações e pede confirmação antes de renomear."
        );
        return if args.is_empty() {
            ExitCode::FAILURE
        } else {
            ExitCode::SUCCESS
        };
    }

    let mut auto_confirm = false;
    let mut targets = Vec::new();
    for arg in args {
        match arg.as_str() {
            "--sim" => auto_confirm = true,
            other if other.starts_with('-') => {
                eprintln!("opção desconhecida: {other}");
                return usage();
            }
            path => targets.push(PathBuf::from(path)),
        }
    }

    if targets.is_empty() {
        return usage();
    }

    let mut items = Vec::new();
    for target in &targets {
        if !target.exists() {
            eprintln!("{}: caminho não encontrado", target.display());
            return ExitCode::FAILURE;
        }
        if let Err(error) = collect_migrations(target, &mut items) {
            eprintln!("{}: erro ao inspecionar: {error}", target.display());
            return ExitCode::FAILURE;
        }
    }

    items.sort_by(|a, b| a.from.cmp(&b.from));
    items.dedup_by(|a, b| a.from == b.from);

    if items.is_empty() {
        println!("Nenhum arquivo com extensão antiga encontrado.");
        return ExitCode::SUCCESS;
    }

    let mut conflicts = 0usize;
    for item in &items {
        if item.to.exists() {
            eprintln!(
                "conflito: {} já existe (não é possível migrar {})",
                item.to.display(),
                item.from.display()
            );
            conflicts += 1;
        }
    }
    if conflicts > 0 {
        return ExitCode::FAILURE;
    }

    println!("Arquivos a migrar ({}):", items.len());
    for item in &items {
        println!("  {} -> {}", item.from.display(), item.to.display());
    }

    if !auto_confirm && !confirm_migration(items.len()) {
        println!("Migração cancelada.");
        return ExitCode::SUCCESS;
    }

    for item in &items {
        if let Err(error) = fs::rename(&item.from, &item.to) {
            eprintln!(
                "{}: não foi possível renomear para {}: {error}",
                item.from.display(),
                item.to.display()
            );
            return ExitCode::FAILURE;
        }
    }

    println!("{} arquivo(s) renomeado(s).", items.len());
    ExitCode::SUCCESS
}

fn collect_migrations(path: &Path, out: &mut Vec<MigrationItem>) -> io::Result<()> {
    if path.is_dir() {
        if let Some(name) = path.file_name().and_then(|n| n.to_str())
            && matches!(name, ".git" | "target" | "node_modules")
        {
            return Ok(());
        }
        for entry in fs::read_dir(path)? {
            let entry = entry?;
            collect_migrations(&entry.path(), out)?;
        }
        return Ok(());
    }
    if let Some(to) = portugol_formatos::canonical_path(path) {
        out.push(MigrationItem {
            from: path.to_path_buf(),
            to,
        });
    }
    Ok(())
}

fn confirm_migration(count: usize) -> bool {
    print!("Renomear {count} arquivo(s)? [s/N] ");
    if io::stdout().flush().is_err() {
        return false;
    }
    let mut line = String::new();
    if io::stdin().lock().read_line(&mut line).is_err() {
        return false;
    }
    let answer = line.trim();
    answer.eq_ignore_ascii_case("s")
        || answer.eq_ignore_ascii_case("sim")
        || answer.eq_ignore_ascii_case("y")
        || answer.eq_ignore_ascii_case("yes")
}
