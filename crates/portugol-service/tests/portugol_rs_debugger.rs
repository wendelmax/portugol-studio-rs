use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portugol_service::{DebugInfo, RunObserver, RunOptions, RunOutcome, start_portugol_rs};

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Output(String),
    Input(&'static str),
    Paused(DebugInfo),
    Finished(RunOutcome),
}

struct Recorder(Mutex<Sender<Event>>);

impl RunObserver for Recorder {
    fn output(&self, text: &str) {
        let _ = self.0.lock().unwrap().send(Event::Output(text.to_owned()));
    }
    fn clear(&self) {}
    fn input_requested(&self, expected: &'static str) {
        let _ = self.0.lock().unwrap().send(Event::Input(expected));
    }
    fn paused(&self, info: DebugInfo) {
        let _ = self.0.lock().unwrap().send(Event::Paused(info));
    }
    fn finished(&self, outcome: RunOutcome) {
        let _ = self.0.lock().unwrap().send(Event::Finished(outcome));
    }
}

#[test]
fn pors_debugger_reuses_typed_input_channel_without_repeating_pause() {
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let source = "funcao inicio() {\n seja mutavel resposta: inteiro = 0;\n leia(resposta);\n escreva(resposta);\n}";
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![3],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("compilar e iniciar depuração");

    let Event::Paused(_) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("breakpoint antes de leia")
    else {
        panic!("esperava pausa")
    };
    handle.continue_run();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5))
            .expect("pedido de entrada"),
        Event::Input("inteiro")
    );
    handle.send_input("42".into());
    let mut output = String::new();
    loop {
        match rx
            .recv_timeout(Duration::from_secs(5))
            .expect("evento final")
        {
            Event::Output(text) => output.push_str(&text),
            Event::Finished(RunOutcome::Finished) => break,
            Event::Finished(other) => panic!("resultado inesperado: {other:?}"),
            Event::Paused(_) => panic!("não deve repetir a pausa ao aguardar leia"),
            Event::Input(_) => panic!("não deve repetir a solicitação de entrada"),
        }
    }
    assert_eq!(output, "42");
    handle.join();
}

#[test]
fn pors_vm_debug_output_matches_generated_rust_for_core_statements() {
    if !portugol_service::rust_toolchain_available() {
        eprintln!("rustc indisponível; teste diferencial ignorado neste ambiente");
        return;
    }
    let source = "funcao inicio() {\n seja mutavel total: inteiro = 0;\n para (seja mutavel n: inteiro = 1; n <= 4; n += 1) {\n total += n;\n }\n se (total == 10 && verdadeiro) {\n escreva(total);\n } senao {\n escreva(0);\n }\n}";
    let generated = portugol_service::run_portugol_rs(source).expect("compilar Rust gerado");
    let portugol_service::PortugolRsRunResult::Executed {
        stdout,
        exit_code: Some(0),
        ..
    } = generated
    else {
        panic!("Rust gerado não concluiu: {generated:?}");
    };

    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(source, RunOptions::default(), observer).expect("iniciar VM");
    let mut vm_output = String::new();
    loop {
        match rx
            .recv_timeout(Duration::from_secs(5))
            .expect("evento final")
        {
            Event::Output(text) => vm_output.push_str(&text),
            Event::Finished(RunOutcome::Finished) => break,
            Event::Finished(other) => panic!("VM retornou erro: {other:?}"),
            Event::Paused(_) => panic!("não havia breakpoint"),
            Event::Input(_) => panic!("não havia leia"),
        }
    }
    handle.join();
    assert_eq!(vm_output, stdout);
}

#[test]
fn pors_debugger_stop_interrupts_a_waiting_read() {
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        "funcao inicio() {\n seja mutavel valor: inteiro = 0;\n leia(valor);\n}",
        RunOptions::default(),
        observer,
    )
    .expect("iniciar debugger");
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5))
            .expect("pedido de entrada"),
        Event::Input("inteiro")
    );
    handle.stop();
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5))
            .expect("resultado interrompido"),
        Event::Finished(RunOutcome::Stopped)
    );
    handle.join();
}

#[test]
fn pors_debugger_executes_in_vm_and_publishes_real_variables() {
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let source =
        "funcao inicio() {\n seja mutavel idade: inteiro = 16;\n idade += 1;\n escreva(idade);\n}";
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![3],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("compilar e iniciar depuração");

    let Event::Paused(info) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa inicial")
    else {
        panic!("esperava evento de pausa")
    };
    assert_eq!(info.stack[0].function, "inicio");
    assert_eq!(
        info.variables
            .iter()
            .find(|v| v.name == "idade")
            .unwrap()
            .value,
        "16"
    );
    handle.step_over();
    let Event::Paused(after_step) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa após avanço")
    else {
        panic!("esperava uma pausa após step over")
    };
    assert_eq!(after_step.position.line, 4);
    assert_eq!(
        after_step
            .variables
            .iter()
            .find(|v| v.name == "idade")
            .unwrap()
            .value,
        "17"
    );
    handle.continue_run();

    let mut output = String::new();
    loop {
        match rx
            .recv_timeout(Duration::from_secs(5))
            .expect("evento final")
        {
            Event::Output(text) => output.push_str(&text),
            Event::Finished(RunOutcome::Finished) => break,
            Event::Finished(other) => panic!("resultado inesperado: {other:?}"),
            Event::Paused(_) => panic!("pausa inesperada"),
            Event::Input(_) => panic!("entrada inesperada"),
        }
    }
    assert_eq!(output, "17");
    handle.join();
}
