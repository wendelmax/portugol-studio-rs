//! Execução em segundo plano.
//!
//! A VM roda numa thread própria. A saída é acumulada e entregue em lotes
//! (a cada [`FLUSH_INTERVAL`]) por uma segunda thread, para não inundar a
//! ponte com a janela quando o programa escreve num laço apertado. Toda
//! entrega ao observador acontece com o buffer travado, o que mantém a ordem
//! entre saída, `limpa` e pedidos de `leia`.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Receiver, Sender};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use serde::Serialize;

use portugol_bytecode::ValueType;
use portugol_syntax::{Diagnostic, Severity, Span};
use portugol_vm::debug::{DebugCommand, DebugControl, PausedInfo};
use portugol_vm::{FileFilter, Host, Io, Vm};

use crate::position::{LineIndex, Position};
use crate::{DiagnosticInfo, analyze, describe};

const FLUSH_INTERVAL: Duration = Duration::from_millis(30);
/// Acima disso o lote é entregue na hora, sem esperar o intervalo.
const MAX_BUFFER: usize = 64 * 1024;

/// Variável exposta no inspetor, já com posições para a interface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DebugVarInfo {
    pub name: String,
    pub ty: String,
    pub value: String,
    /// `"global"` ou `"local"`.
    pub scope: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DebugFrameInfo {
    pub function: String,
    pub position: Position,
}

/// Estado publicado a cada pausa do depurador.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DebugInfo {
    pub position: Position,
    /// Da função atual para a de entrada.
    pub stack: Vec<DebugFrameInfo>,
    pub variables: Vec<DebugVarInfo>,
}

/// Recebe os eventos de uma execução. Chamado de outras threads.
pub trait RunObserver: Send + Sync + 'static {
    fn output(&self, text: &str);
    fn clear(&self);
    /// O programa parou num `leia`; `expected` é o nome do tipo esperado.
    fn input_requested(&self, expected: &'static str);
    /// A VM parou num ponto de parada ou comando de passo.
    fn paused(&self, _info: DebugInfo) {}
    fn finished(&self, outcome: RunOutcome);
    /// Diálogo de `Arquivos.selecionar_arquivo`. Bloqueia a thread da VM até
    /// o usuário escolher; `None` quando cancela.
    fn select_file(&self, filters: &[FileFilter], accept_all: bool) -> Option<String> {
        let _ = (filters, accept_all);
        None
    }
}

/// Opções de uma execução.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Parâmetros de `funcao inicio(cadeia params[])`.
    pub args: Vec<String>,
    /// Base dos caminhos relativos da biblioteca Arquivos (a pasta do
    /// programa, como no original). `None` usa a pasta atual do processo.
    pub working_dir: Option<std::path::PathBuf>,
    /// Linhas (base 1) com ponto de parada.
    pub breakpoints: Vec<u32>,
}

struct ServiceHost {
    observer: Arc<dyn RunObserver>,
}

impl Host for ServiceHost {
    fn select_file(
        &mut self,
        filters: &[FileFilter],
        accept_all: bool,
    ) -> Result<Option<String>, String> {
        Ok(self.observer.select_file(filters, accept_all))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TraceFrame {
    pub function: String,
    pub line: u32,
    pub column: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", rename_all = "lowercase")]
pub enum RunOutcome {
    Finished,
    Stopped,
    Error {
        message: String,
        start: Position,
        end: Position,
        trace: Vec<TraceFrame>,
    },
}

/// Controle de uma execução em andamento.
pub struct RunHandle {
    input: Sender<Option<String>>,
    interrupt: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
    debug: DebugControl,
}

impl RunHandle {
    /// Responde a um `leia`.
    pub fn send_input(&self, line: String) {
        let _ = self.input.send(Some(line));
    }

    /// Interrompe o programa, inclusive se estiver parado no depurador ou
    /// esperando um `leia`.
    pub fn stop(&self) {
        self.debug.stop();
        self.interrupt.store(true, Ordering::Relaxed);
        let _ = self.input.send(None);
    }

    /// Atualiza as linhas com ponto de parada durante a execução.
    pub fn set_breakpoints(&self, lines: &[u32], source: &str) {
        let index = LineIndex::new(source);
        self.debug
            .set_breakpoints(lines.iter().map(|&line| index.line_span(line)).collect());
    }

    /// Pede uma pausa na próxima instrução.
    pub fn pause(&self) {
        self.debug.request_pause();
    }

    pub fn continue_run(&self) {
        self.debug.command(DebugCommand::Continue);
    }

    pub fn step_over(&self) {
        self.debug.command(DebugCommand::StepOver);
    }

    pub fn step_into(&self) {
        self.debug.command(DebugCommand::StepInto);
    }

    pub fn step_out(&self) {
        self.debug.command(DebugCommand::StepOut);
    }

    pub fn is_finished(&self) -> bool {
        self.thread.as_ref().is_none_or(|t| t.is_finished())
    }

    /// Espera a execução terminar.
    pub fn join(mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

/// Analisa, compila e começa a executar. Erros de análise ou compilação
/// voltam como diagnósticos e nada é executado.
pub fn start(
    source: &str,
    options: RunOptions,
    observer: Arc<dyn RunObserver>,
) -> Result<RunHandle, Vec<DiagnosticInfo>> {
    let index = LineIndex::new(source);
    let (diagnostics, program) = analyze(source);
    let Some(program) = program else {
        return Err(diagnostics
            .iter()
            .filter(|d| d.severity == Severity::Error)
            .map(|d| describe(&index, d))
            .collect());
    };
    let module = portugol_bytecode::compile(&program).map_err(|error| {
        let diagnostic = Diagnostic::error("compile.error", error.message, error.span);
        vec![describe(&index, &diagnostic)]
    })?;

    let (input_tx, input_rx) = mpsc::channel();
    let interrupt = Arc::new(AtomicBool::new(false));
    let source = Arc::new(source.to_string());
    let flag = Arc::clone(&interrupt);

    let breakpoints: Vec<Span> = options
        .breakpoints
        .iter()
        .map(|&line| index.line_span(line))
        .collect();
    let debug = {
        let source = Arc::clone(&source);
        let observer = Arc::clone(&observer);
        DebugControl::new(move |info: &PausedInfo| {
            let index = LineIndex::new(&source);
            observer.paused(DebugInfo {
                position: index.position(info.span.start),
                stack: info
                    .stack
                    .iter()
                    .map(|frame| DebugFrameInfo {
                        function: frame.function.clone(),
                        position: index.position(frame.offset),
                    })
                    .collect(),
                variables: info
                    .variables
                    .iter()
                    .map(|variable| DebugVarInfo {
                        name: variable.name.clone(),
                        ty: variable.ty.clone(),
                        value: variable.value.clone(),
                        scope: variable.scope.to_string(),
                    })
                    .collect(),
            });
        })
    };
    debug.set_breakpoints(breakpoints);
    let handle_debug = debug.clone();

    let thread = thread::Builder::new()
        .name("portugol-vm".to_string())
        .spawn(move || {
            let output = Arc::new(Output {
                buffer: Mutex::new(String::new()),
                observer: Arc::clone(&observer),
            });
            let done = Arc::new(AtomicBool::new(false));
            let flusher = spawn_flusher(Arc::clone(&output), Arc::clone(&done));

            let mut io = ServiceIo {
                output: Arc::clone(&output),
                input: input_rx,
            };
            let mut vm = Vm::new(&module, &mut io)
                .with_args(options.args)
                .with_interrupt(Arc::clone(&flag))
                .with_debug(debug)
                .with_host(Box::new(ServiceHost {
                    observer: Arc::clone(&observer),
                }));
            if let Some(dir) = options.working_dir {
                vm = vm.with_working_dir(dir);
            }
            let result = vm.run();

            done.store(true, Ordering::Relaxed);
            let _ = flusher.join();
            output.flush();

            let outcome = match result {
                Ok(()) => RunOutcome::Finished,
                Err(_) if flag.load(Ordering::Relaxed) => RunOutcome::Stopped,
                Err(error) => {
                    let index = LineIndex::new(&source);
                    RunOutcome::Error {
                        message: error.message,
                        start: index.position(error.span.start),
                        end: index.position(error.span.end.max(error.span.start)),
                        trace: error
                            .trace
                            .iter()
                            .map(|(function, span)| {
                                let position = index.position(span.start);
                                TraceFrame {
                                    function: function.clone(),
                                    line: position.line,
                                    column: position.column,
                                }
                            })
                            .collect(),
                    }
                }
            };
            observer.finished(outcome);
        })
        .expect("criar a thread da VM");

    Ok(RunHandle {
        input: input_tx,
        interrupt,
        thread: Some(thread),
        debug: handle_debug,
    })
}

struct Output {
    buffer: Mutex<String>,
    observer: Arc<dyn RunObserver>,
}

impl Output {
    fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap_or_else(|e| e.into_inner());
        if !buffer.is_empty() {
            self.observer.output(&buffer);
            buffer.clear();
        }
    }
}

fn spawn_flusher(output: Arc<Output>, done: Arc<AtomicBool>) -> JoinHandle<()> {
    thread::spawn(move || {
        while !done.load(Ordering::Relaxed) {
            thread::sleep(FLUSH_INTERVAL);
            output.flush();
        }
    })
}

struct ServiceIo {
    output: Arc<Output>,
    input: Receiver<Option<String>>,
}

impl Io for ServiceIo {
    fn write(&mut self, text: &str) {
        let full = {
            let mut buffer = self.output.buffer.lock().unwrap_or_else(|e| e.into_inner());
            buffer.push_str(text);
            buffer.len() >= MAX_BUFFER
        };
        if full {
            self.output.flush();
        }
    }

    fn read_line(&mut self, expected: ValueType) -> Option<String> {
        {
            let mut buffer = self.output.buffer.lock().unwrap_or_else(|e| e.into_inner());
            if !buffer.is_empty() {
                self.output.observer.output(&buffer);
                buffer.clear();
            }
            self.output.observer.input_requested(expected.name());
        }
        self.input.recv().ok().flatten()
    }

    fn clear(&mut self) {
        let mut buffer = self.output.buffer.lock().unwrap_or_else(|e| e.into_inner());
        // O que ainda não foi entregue seria apagado mesmo.
        buffer.clear();
        self.output.observer.clear();
    }
}
