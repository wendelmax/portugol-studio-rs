//! Subcomando `executar`: análise completa, compilação e execução na VM
//! para programas Portugol-RS (`.pors`).

use std::path::Path;
use std::process::ExitCode;

use portugol_hir::{Diagnostic, Span};
use portugol_vm::{StdIo, Vm};

use crate::{is_pors_file, read_source, report_unsupported_extension};

pub fn main(args: &[String]) -> ExitCode {
    let (args, program_args) = match args.iter().position(|a| a == "--") {
        Some(at) => (&args[..at], args[at + 1..].to_vec()),
        None => (args, Vec::new()),
    };
    let mut listing = false;
    let mut file = None;
    for arg in args {
        match arg.as_str() {
            "--bytecode" => listing = true,
            _ if file.is_none() => file = Some(arg.as_str()),
            other => {
                eprintln!("argumento inesperado: {other}");
                return ExitCode::FAILURE;
            }
        }
    }
    let Some(file) = file else {
        eprintln!("uso: portugol executar [--bytecode] <arquivo.fer>");
        return ExitCode::FAILURE;
    };

    let path = Path::new(file);
    if !is_pors_file(path) {
        report_unsupported_extension(path);
        return ExitCode::FAILURE;
    }
    if !program_args.is_empty() {
        eprintln!(
            "{file}: parâmetros de linha de comando ainda não são suportados em funcao inicio() de .pors"
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

    let program = match portugol_rs_frontend::lower(&source) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.render(&source, file));
            }
            return ExitCode::FAILURE;
        }
    };
    let module = match portugol_bytecode::compile_pors(&program) {
        Ok(module) => module,
        Err(error) => {
            let span = Span::new(error.span.start, error.span.end);
            let diagnostic = Diagnostic::error("pors.compile-error", error.message, span);
            eprintln!("{}", diagnostic.render(&source, file));
            return ExitCode::FAILURE;
        }
    };

    if listing {
        print_listing(&module);
        return ExitCode::SUCCESS;
    }

    let mut io = StdIo;
    let mut vm = Vm::new(&module, &mut io);
    match vm.run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            println!();
            let diagnostic = Diagnostic::error("exec.error", error.message, error.span);
            eprintln!("{}", diagnostic.render(&source, file));
            for (function, span) in error.trace.iter().skip(1) {
                let (line, column) = portugol_hir::line_col(&source, span.start);
                eprintln!("  chamado em {function} ({file}:{line}:{column})");
            }
            ExitCode::FAILURE
        }
    }
}

fn print_listing(module: &portugol_bytecode::Module) {
    for (index, function) in module.functions.iter().enumerate() {
        println!(
            "#{index} {} (parâmetros: {}, locais: {})",
            function.name,
            function.arity,
            function.locals.len()
        );
        for (ip, op) in function.code.iter().enumerate() {
            println!("  {ip:5}  {op:?}");
        }
        println!();
    }
    if !module.strings.is_empty() {
        println!("cadeias:");
        for (index, text) in module.strings.iter().enumerate() {
            println!("  {index:5}  {text:?}");
        }
    }
}
