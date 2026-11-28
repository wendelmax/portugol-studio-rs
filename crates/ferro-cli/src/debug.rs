//! Subcomando `depurar`: depurador interativo de linha de comando para
//! programas Ferro (`.fer`), com breakpoints, passo a passo e inspeção
//! de variáveis/pilha na VM.

use std::collections::BTreeSet;
use std::io::{self, BufRead, Write};
use std::path::Path;
use std::process::ExitCode;
use std::sync::mpsc;
use std::thread;

use ferro_hir::{Diagnostic, Span, line_col};
use ferro_vm::debug::{DebugCommand, DebugControl, PausedInfo};
use ferro_vm::{StdIo, Vm};

use crate::{is_pors_file, read_source, report_unsupported_extension};

enum Event {
    Paused(PausedInfo),
    Finished,
    Error(ferro_vm::RuntimeError),
}

pub fn main(args: &[String]) -> ExitCode {
    if args.is_empty() || args.iter().any(|arg| arg == "-h" || arg == "--help") {
        eprintln!("uso: portugol depurar [--ponto <linha>]... <arquivo.fer>");
        return ExitCode::SUCCESS;
    }

    let mut breakpoints = BTreeSet::new();
    let mut file: Option<&str> = None;
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--ponto" | "-b" => {
                let Some(raw) = args.get(index + 1) else {
                    eprintln!("esperado número da linha após `{}`", args[index]);
                    return ExitCode::FAILURE;
                };
                match raw.parse::<usize>() {
                    Ok(line) if line >= 1 => {
                        breakpoints.insert(line);
                    }
                    _ => {
                        eprintln!("linha de ponto de parada inválida: `{raw}`");
                        return ExitCode::FAILURE;
                    }
                }
                index += 2;
            }
            arg if file.is_none() => {
                file = Some(arg);
                index += 1;
            }
            other => {
                eprintln!("argumento inesperado: {other}");
                return ExitCode::FAILURE;
            }
        }
    }

    let Some(file) = file else {
        eprintln!("uso: portugol depurar [--ponto <linha>]... <arquivo.fer>");
        return ExitCode::FAILURE;
    };

    let path = Path::new(file);
    if !is_pors_file(path) {
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

    let program = match ferro_frontend::lower(&source) {
        Ok(program) => program,
        Err(diagnostics) => {
            for diagnostic in diagnostics {
                eprintln!("{}", diagnostic.render(&source, file));
            }
            return ExitCode::FAILURE;
        }
    };
    let module = match ferro_bytecode::compile_pors(&program) {
        Ok(module) => module,
        Err(error) => {
            let span = Span::new(error.span.start, error.span.end);
            let diagnostic = Diagnostic::error("pors.compile-error", error.message, span);
            eprintln!("{}", diagnostic.render(&source, file));
            return ExitCode::FAILURE;
        }
    };

    let line_spans = compute_line_spans(&source);
    let (tx, rx) = mpsc::channel::<Event>();
    let pause_tx = tx.clone();
    let control = DebugControl::new(move |info| {
        let _ = pause_tx.send(Event::Paused(info.clone()));
    });
    control.set_breakpoints(resolve_breakpoints(&breakpoints, &line_spans));
    control.request_pause();

    let vm_control = control.clone();
    let worker_dir = path
        .parent()
        .filter(|dir| !dir.as_os_str().is_empty())
        .map(Path::to_path_buf);
    let worker = thread::spawn(move || {
        let mut io = StdIo;
        let mut vm = Vm::new(&module, &mut io).with_debug(vm_control);
        if let Some(dir) = worker_dir {
            vm = vm.with_pors_workspace_dir(dir);
        }
        match vm.run() {
            Ok(()) => {
                let _ = tx.send(Event::Finished);
            }
            Err(error) => {
                let _ = tx.send(Event::Error(error));
            }
        }
    });

    println!("Depurador Ferro — {file} (digite `ajuda` para comandos)");
    let stdin = io::stdin();
    let mut lines = stdin.lock().lines();
    let mut exit_code = ExitCode::SUCCESS;

    while let Ok(event) = rx.recv() {
        match event {
            Event::Finished => {
                println!("\n[depurador] execução finalizada.");
                break;
            }
            Event::Error(error) => {
                println!();
                let diagnostic = Diagnostic::error("exec.error", error.message, error.span);
                eprintln!("{}", diagnostic.render(&source, file));
                exit_code = ExitCode::FAILURE;
                break;
            }
            Event::Paused(info) => {
                let (line, col) = line_col(&source, info.span.start);
                let snippet = source
                    .lines()
                    .nth(line.saturating_sub(1))
                    .unwrap_or("")
                    .trim();
                println!("\n[pausa] {file}:{line}:{col} | {snippet}");
                loop {
                    print!("(pors-dbg) ");
                    let _ = io::stdout().flush();
                    let Some(Ok(raw)) = lines.next() else {
                        control.stop();
                        let _ = worker.join();
                        return exit_code;
                    };
                    let trimmed = raw.trim();
                    let mut parts = trimmed.split_whitespace();
                    let cmd = parts.next().unwrap_or("");
                    match cmd {
                        "" | "p" | "passo" | "n" => {
                            control.command(DebugCommand::StepOver);
                            break;
                        }
                        "e" | "entrar" | "s" => {
                            control.command(DebugCommand::StepInto);
                            break;
                        }
                        "sair" | "o" => {
                            control.command(DebugCommand::StepOut);
                            break;
                        }
                        "c" | "continuar" => {
                            control.command(DebugCommand::Continue);
                            break;
                        }
                        "v" | "variaveis" => {
                            if info.variables.is_empty() {
                                println!("(nenhuma variável ativa)");
                            } else {
                                for var in &info.variables {
                                    println!("  {} : {} = {}", var.name, var.ty, var.value);
                                }
                            }
                        }
                        "pilha" | "bt" => {
                            for frame in &info.stack {
                                let (fl, fc) = line_col(&source, frame.offset);
                                println!("  {} ({file}:{fl}:{fc})", frame.function);
                            }
                        }
                        "b" | "ponto" => {
                            if let Some(arg) = parts.next()
                                && let Ok(bp_line) = arg.parse::<usize>()
                                && bp_line >= 1
                                && bp_line <= line_spans.len()
                            {
                                if !breakpoints.insert(bp_line) {
                                    breakpoints.remove(&bp_line);
                                    println!("ponto de parada removido da linha {bp_line}");
                                } else {
                                    println!("ponto de parada definido na linha {bp_line}");
                                }
                                control.set_breakpoints(resolve_breakpoints(
                                    &breakpoints,
                                    &line_spans,
                                ));
                            } else {
                                println!(
                                    "uso: ponto <1..={}> (ativos: {:?})",
                                    line_spans.len(),
                                    breakpoints
                                );
                            }
                        }
                        "l" | "listar" => {
                            let start_line = line.saturating_sub(2).max(1);
                            let end_line = (line + 2).min(line_spans.len());
                            for ln in start_line..=end_line {
                                let marker = if ln == line { "=>" } else { "  " };
                                let bp = if breakpoints.contains(&ln) { "*" } else { " " };
                                let text = source.lines().nth(ln - 1).unwrap_or("");
                                println!("{marker}{bp} {ln:4} | {text}");
                            }
                        }
                        "h" | "ajuda" => {
                            println!(
                                "Comandos:\n  p / passo        avança uma linha (step over)\n  e / entrar       entra na função (step into)\n  sair             sai da função atual (step out)\n  c / continuar    continua até o próximo ponto de parada ou fim\n  b / ponto <N>    alterna ponto de parada na linha N\n  v / variaveis    lista variáveis ativas no escopo\n  pilha            mostra a pilha de chamadas\n  l / listar       exibe o código ao redor da linha atual\n  q / encerrar     encerra a sessão de depuração"
                            );
                        }
                        "q" | "encerrar" => {
                            control.stop();
                            let _ = worker.join();
                            return ExitCode::SUCCESS;
                        }
                        other => {
                            println!("comando desconhecido: `{other}` (digite `ajuda`)");
                        }
                    }
                }
            }
        }
    }

    let _ = worker.join();
    exit_code
}

fn compute_line_spans(source: &str) -> Vec<Span> {
    let mut spans = Vec::new();
    let mut offset = 0usize;
    for chunk in source.split_inclusive('\n') {
        let len = chunk.len();
        spans.push(Span::new(offset as u32, (offset + len) as u32));
        offset += len;
    }
    if spans.is_empty() {
        spans.push(Span::new(0, source.len() as u32));
    }
    spans
}

fn resolve_breakpoints(lines: &BTreeSet<usize>, line_spans: &[Span]) -> Vec<Span> {
    lines
        .iter()
        .filter_map(|&line| line_spans.get(line.saturating_sub(1)).copied())
        .collect()
}
