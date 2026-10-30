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
use portugol_syntax::{Diagnostic, Severity};
use portugol_vm::{Io, Vm};

use crate::position::{LineIndex, Position};
use crate::{DiagnosticInfo, analyze, describe};

const FLUSH_INTERVAL: Duration = Duration::from_millis(30);
/// Acima disso o lote é entregue na hora, sem esperar o intervalo.
const MAX_BUFFER: usize = 64 * 1024;

/// Recebe os eventos de uma execução. Chamado de outras threads.
pub trait RunObserver: Send + Sync + 'static {
    fn output(&self, text: &str);
    fn clear(&self);
    /// O programa parou num `leia`; `expected` é o nome do tipo esperado.
    fn input_requested(&self, expected: &'static str);
    fn finished(&self, outcome: RunOutcome);
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
}

impl RunHandle {
    /// Responde a um `leia`.
    pub fn send_input(&self, line: String) {
        let _ = self.input.send(Some(line));
    }

    /// Interrompe o programa, inclusive se estiver esperando um `leia`.
    pub fn stop(&self) {
        self.interrupt.store(true, Ordering::Relaxed);
        let _ = self.input.send(None);
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
    args: Vec<String>,
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
    let source = source.to_string();
    let flag = Arc::clone(&interrupt);

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
            let result = Vm::new(&module, &mut io)
                .with_args(args)
                .with_interrupt(Arc::clone(&flag))
                .run();

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
