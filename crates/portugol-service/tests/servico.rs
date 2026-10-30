use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use portugol_service::{Level, RunObserver, RunOutcome, check, start};

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Output(String),
    Clear,
    Input(&'static str),
    Finished(RunOutcome),
}

struct Recorder(Mutex<Sender<Event>>);

impl RunObserver for Recorder {
    fn output(&self, text: &str) {
        let _ = self.0.lock().unwrap().send(Event::Output(text.to_string()));
    }
    fn clear(&self) {
        let _ = self.0.lock().unwrap().send(Event::Clear);
    }
    fn input_requested(&self, expected: &'static str) {
        let _ = self.0.lock().unwrap().send(Event::Input(expected));
    }
    fn finished(&self, outcome: RunOutcome) {
        let _ = self.0.lock().unwrap().send(Event::Finished(outcome));
    }
}

fn recorder() -> (Arc<Recorder>, mpsc::Receiver<Event>) {
    let (tx, rx) = mpsc::channel();
    (Arc::new(Recorder(Mutex::new(tx))), rx)
}

fn next(rx: &mpsc::Receiver<Event>) -> Event {
    rx.recv_timeout(Duration::from_secs(5)).expect("evento")
}

/// Junta saídas consecutivas (o agrupamento em lotes não é determinístico).
fn collect_until_finished(rx: &mpsc::Receiver<Event>) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();
    loop {
        let event = next(rx);
        let done = matches!(event, Event::Finished(_));
        match (events.last_mut(), event) {
            (Some(Event::Output(acc)), Event::Output(text)) => acc.push_str(&text),
            (_, event) => events.push(event),
        }
        if done {
            return events;
        }
    }
}

#[test]
fn check_devolve_posicoes_do_editor() {
    // Identificadores só aceitam [a-zA-Z_], como na gramática original; o
    // acento na cadeia testa a conversão para UTF-16.
    let source = "programa {\n funcao inicio() {\n  escreva(\"ação😀\", x)\n }\n}";
    let diagnostics = check(source);
    assert_eq!(diagnostics.len(), 1);
    let d = &diagnostics[0];
    assert_eq!(d.severity, Level::Error);
    assert_eq!(d.code, "sem.symbol-not-declared");
    assert_eq!((d.start.line, d.start.column), (3, 20));
    let utf16: Vec<u16> = source.encode_utf16().collect();
    assert_eq!(utf16[d.start.offset as usize], 'x' as u16);
    assert_eq!(d.end.offset - d.start.offset, 1);
}

#[test]
fn executa_com_leia_e_saida() {
    let (observer, rx) = recorder();
    let handle = start(
        r#"programa { funcao inicio() {
            inteiro n
            escreva("n? ")
            leia(n)
            escreva(n * 2)
        } }"#,
        Vec::new(),
        observer,
    )
    .unwrap();

    assert_eq!(next(&rx), Event::Output("n? ".to_string()));
    assert_eq!(next(&rx), Event::Input("inteiro"));
    handle.send_input("21".to_string());
    assert_eq!(
        collect_until_finished(&rx),
        [Event::Output("42".into()), Event::Finished(RunOutcome::Finished)]
    );
    handle.join();
}

#[test]
fn limpa_descarta_saida_anterior() {
    let (observer, rx) = recorder();
    start(
        r#"programa { funcao inicio() { escreva("a") limpa() escreva("b") } }"#,
        Vec::new(),
        observer,
    )
    .unwrap()
    .join();
    assert_eq!(
        collect_until_finished(&rx),
        [
            Event::Clear,
            Event::Output("b".into()),
            Event::Finished(RunOutcome::Finished)
        ]
    );
}

#[test]
fn parar_laco_infinito_e_leia() {
    let (observer, rx) = recorder();
    let handle = start(
        "programa { funcao inicio() { enquanto (verdadeiro) { } } }",
        Vec::new(),
        observer,
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(50));
    handle.stop();
    assert_eq!(next(&rx), Event::Finished(RunOutcome::Stopped));
    handle.join();

    let (observer, rx) = recorder();
    let handle = start(
        "programa { funcao inicio() { cadeia s leia(s) } }",
        Vec::new(),
        observer,
    )
    .unwrap();
    assert_eq!(next(&rx), Event::Input("cadeia"));
    handle.stop();
    assert_eq!(next(&rx), Event::Finished(RunOutcome::Stopped));
}

#[test]
fn erro_de_execucao_tem_posicao_e_pilha() {
    let (observer, rx) = recorder();
    start(
        "programa {\n funcao inicio() {\n  inteiro z = 0\n  escreva(1 / z)\n }\n}",
        Vec::new(),
        observer,
    )
    .unwrap()
    .join();
    let Event::Finished(RunOutcome::Error {
        message,
        start,
        trace,
        ..
    }) = next(&rx)
    else {
        panic!("esperava erro");
    };
    assert_eq!(message, "Foi efetuada uma divisão por zero.");
    assert_eq!(start.line, 4);
    assert_eq!(trace[0].function, "inicio");
}

#[test]
fn programa_com_erro_nao_executa() {
    let (observer, _rx) = recorder();
    let Err(errors) = start("programa { funcao inicio() { x = 1 } }", Vec::new(), observer) else {
        panic!("não deveria executar");
    };
    assert_eq!(errors[0].code, "sem.symbol-not-declared");
}
