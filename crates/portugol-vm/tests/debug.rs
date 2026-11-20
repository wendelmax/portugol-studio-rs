//! Testes do depurador: pausas, comandos de passo e inspeção de estado.

use std::sync::mpsc::{self, Receiver, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use portugol_bytecode::compile;
use portugol_syntax::{Span, parse};
use portugol_vm::debug::{DebugCommand, DebugControl, PausedInfo};
use portugol_vm::{BufferIo, Status, Vm};

const TIMEOUT: Duration = Duration::from_secs(10);

/// Span da linha inteira que contém `snippet`, como a fachada envia à VM.
fn span_of(source: &str, snippet: &str) -> Span {
    let at = source
        .find(snippet)
        .unwrap_or_else(|| panic!("trecho não encontrado: {snippet}"));
    let start = source[..at].rfind('\n').map_or(0, |p| p + 1);
    let end = source[at..].find('\n').map_or(source.len(), |p| at + p);
    Span::new(start as u32, end as u32)
}

struct Session {
    control: DebugControl,
    paused: Receiver<PausedInfo>,
}

impl Session {
    fn new(breakpoints: Vec<Span>) -> Self {
        let (tx, paused) = mpsc::channel();
        let control = DebugControl::new(move |info| {
            let _ = tx.send(info.clone());
        });
        control.set_breakpoints(breakpoints);
        Session { control, paused }
    }

    fn wait(&self) -> PausedInfo {
        match self.paused.recv_timeout(TIMEOUT) {
            Ok(info) => info,
            Err(RecvTimeoutError::Timeout) => panic!("tempo esgotado esperando uma pausa"),
            Err(RecvTimeoutError::Disconnected) => panic!("a VM terminou sem parar"),
        }
    }

    fn command(&self, command: DebugCommand) {
        self.control.command(command);
    }
}

/// Executa num thread próprio: a VM bloqueia dentro de `step` enquanto o
/// depurador está pausado, então os comandos precisam vir de outro thread.
fn spawn(
    source: &str,
    control: DebugControl,
) -> thread::JoinHandle<(Result<Status, String>, String)> {
    let ast = parse(source).program;
    let module = compile(&ast).expect("compilar");
    thread::spawn(move || {
        let mut io = BufferIo::default();
        let mut vm = Vm::new(&module, &mut io).with_debug(control);
        let status = vm.step(1_000_000).map_err(|e| e.message);
        (status, io.output)
    })
}

#[test]
fn ponto_de_parada_pausa_e_expõe_variáveis() {
    let source = r#"
programa {
    funcao inicio() {
        inteiro total = 10
        total = total + 5
        escreva(total)
    }
}
"#;
    let session = Session::new(vec![span_of(source, "escreva(total)")]);
    let vm = spawn(source, session.control.clone());

    let paused = session.wait();
    assert_eq!(paused.stack.first().unwrap().function, "inicio");
    let total = paused
        .variables
        .iter()
        .find(|v| v.name == "total")
        .expect("variável total");
    assert_eq!(total.value, "15");
    assert_eq!(total.ty, "inteiro");
    assert_eq!(total.scope, "local");

    session.command(DebugCommand::Continue);
    let (status, output) = vm.join().unwrap();
    assert_eq!(status.unwrap(), Status::Finished);
    assert_eq!(output, "15");
}

#[test]
fn step_into_entra_na_funcao_e_step_out_volta() {
    let source = r#"
programa {
    funcao soma(inteiro a, inteiro b) {
        retorne a + b
    }
    funcao inicio() {
        inteiro r = soma(2, 3)
        escreva(r)
    }
}
"#;
    // Paramos no comando `escreva`, depois que `soma` já retornou; usamos
    // uma segunda sessão para testar a entrada na função.
    let session = Session::new(vec![span_of(source, "soma(2, 3)")]);
    let vm = spawn(source, session.control.clone());

    let paused = session.wait();
    assert_eq!(
        paused.stack.first().unwrap().function,
        "inicio",
        "a pausa ocorre antes de chamar soma"
    );

    session.command(DebugCommand::StepInto);
    let inside = session.wait();
    assert_eq!(inside.stack.first().unwrap().function, "soma");
    assert_eq!(inside.stack.len(), 3); // soma, inicio e a entrada sintética
    assert_eq!(inside.stack[1].function, "inicio");
    let a = inside.variables.iter().find(|v| v.name == "a").unwrap();
    let b = inside.variables.iter().find(|v| v.name == "b").unwrap();
    assert_eq!((a.value.as_str(), b.value.as_str()), ("2", "3"));

    session.command(DebugCommand::StepOut);
    let back = session.wait();
    assert_eq!(back.stack.first().unwrap().function, "inicio");

    session.command(DebugCommand::Continue);
    let (status, output) = vm.join().unwrap();
    assert_eq!(status.unwrap(), Status::Finished);
    assert_eq!(output, "5");
}

#[test]
fn pedido_de_pausa_funciona_sem_ponto_de_parada() {
    let source = r#"
programa {
    funcao inicio() {
        inteiro i = 0
        enquanto (i < 100000) { i++ }
        escreva(i)
    }
}
"#;
    let session = Session::new(vec![]);
    let control = session.control.clone();
    let vm = spawn(source, control.clone());

    control.request_pause();
    let paused = session.wait();
    assert_eq!(paused.stack.first().unwrap().function, "inicio");
    assert!(paused.variables.iter().any(|v| v.name == "i"));

    control.stop();
    let (status, _) = vm.join().unwrap();
    assert!(status.is_err(), "a VM deve encerrar com interrupção");
}

#[test]
fn vetores_e_matrizes_aparecem_expandidos() {
    let source = r#"
programa {
    funcao inicio() {
        inteiro v[2] = {7, 8}
        escreva(v[0])
    }
}
"#;
    let session = Session::new(vec![span_of(source, "escreva(v[0])")]);
    let vm = spawn(source, session.control.clone());

    let paused = session.wait();
    assert!(
        paused
            .variables
            .iter()
            .any(|v| v.name == "v" && v.value == "2 elemento(s)")
    );
    assert!(
        paused
            .variables
            .iter()
            .any(|v| v.name == "v[0]" && v.value == "7")
    );
    assert!(
        paused
            .variables
            .iter()
            .any(|v| v.name == "v[1]" && v.value == "8")
    );

    session.command(DebugCommand::Continue);
    let (status, _) = vm.join().unwrap();
    assert_eq!(status.unwrap(), Status::Finished);
}
