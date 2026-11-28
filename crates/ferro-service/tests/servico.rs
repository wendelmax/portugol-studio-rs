use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use ferro_service::{
    DebugInfo, Host, Level, RunObserver, RunOptions, RunOutcome, SymbolKind, graphics,
    inspect_portugol_rs, start_portugol_rs, start_portugol_rs_with_host,
};

#[derive(Debug, Clone, PartialEq)]
enum Event {
    Output(String),
    Clear,
    Input(&'static str),
    Paused(DebugInfo),
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
    fn paused(&self, info: DebugInfo) {
        let _ = self.0.lock().unwrap().send(Event::Paused(info));
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
fn inspect_portugol_rs_devolve_posicoes_utf16_do_editor() {
    let source = "funcao inicio() {\n  escreva(\"ação😀\", x);\n}";
    let inspection = inspect_portugol_rs(source);
    assert_eq!(inspection.diagnostics.len(), 1);
    let d = &inspection.diagnostics[0];
    assert_eq!(d.severity, Level::Error);
    assert!(d.code.starts_with("pors."), "código inesperado: {}", d.code);
    assert_eq!((d.start.line, d.start.column), (2, 20));
    let utf16: Vec<u16> = source.encode_utf16().collect();
    assert_eq!(utf16[d.start.offset as usize], 'x' as u16);
    assert_eq!(d.end.offset - d.start.offset, 1);
}

#[test]
fn executa_com_leia_e_saida() {
    let (observer, rx) = recorder();
    let handle = start_portugol_rs(
        r#"funcao inicio() {
            escreva("n? ");
            seja mutavel n: inteiro = leia();
            escreva(n * 2);
        }"#,
        RunOptions::default(),
        observer,
    )
    .unwrap();

    assert_eq!(next(&rx), Event::Output("n? ".to_string()));
    assert_eq!(next(&rx), Event::Input("inteiro"));
    handle.send_input("21".to_string());
    assert_eq!(
        collect_until_finished(&rx),
        [
            Event::Output("42".into()),
            Event::Finished(RunOutcome::Finished)
        ]
    );
    handle.join();
}

#[test]
fn parar_laco_infinito_e_leia() {
    let (observer, rx) = recorder();
    let handle = start_portugol_rs(
        "funcao inicio() { enquanto (verdadeiro) { } }",
        RunOptions::default(),
        observer,
    )
    .unwrap();
    std::thread::sleep(Duration::from_millis(50));
    handle.stop();
    assert_eq!(next(&rx), Event::Finished(RunOutcome::Stopped));
    handle.join();

    let (observer, rx) = recorder();
    let handle = start_portugol_rs(
        "funcao inicio() { seja s = leia(); }",
        RunOptions::default(),
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
    start_portugol_rs(
        "funcao inicio() {\n  seja z: inteiro = 0;\n  escreva(1 / z);\n}",
        RunOptions::default(),
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
    assert_eq!(start.line, 3);
    assert_eq!(trace[0].function, "inicio");
}

#[test]
fn depurador_pausa_continua_e_termina() {
    let source = "funcao inicio() {\n seja x: inteiro = 1;\n escreva(x);\n}";
    let (observer, rx) = recorder();
    let options = RunOptions {
        breakpoints: vec![3], // a linha do escreva
        ..RunOptions::default()
    };
    let handle = start_portugol_rs(source, options, observer).expect("compilar");

    let Event::Paused(info) = next(&rx) else {
        panic!("esperava uma pausa no depurador");
    };
    assert_eq!(info.position.line, 3);
    assert_eq!(info.stack[0].function, "inicio");
    let x = info
        .variables
        .iter()
        .find(|v| v.name == "x")
        .expect("variável x");
    assert_eq!(x.value, "1");
    assert_eq!(x.scope, "local");

    handle.continue_run();
    let events = collect_until_finished(&rx);
    assert!(
        events
            .iter()
            .any(|event| matches!(event, Event::Output(text) if text == "1")),
        "saída: {events:?}"
    );
    assert!(matches!(
        events.last(),
        Some(Event::Finished(RunOutcome::Finished))
    ));
    handle.join();
}

#[test]
fn programa_com_erro_nao_executa() {
    let (observer, _rx) = recorder();
    let Err(errors) = start_portugol_rs(
        "funcao inicio() { x = 1; }",
        RunOptions::default(),
        observer,
    ) else {
        panic!("não deveria executar");
    };
    assert!(errors[0].code.starts_with("pors."));
}

#[test]
fn depuracao_inicia_pausada_sem_ponto_de_parada() {
    let (observer, rx) = recorder();
    let handle = start_portugol_rs(
        "funcao inicio() { escreva(42); }",
        RunOptions {
            pause_on_start: true,
            ..RunOptions::default()
        },
        observer,
    )
    .expect("compilar");
    let Event::Paused(info) = next(&rx) else {
        panic!("esperava pausa antes da primeira instrução");
    };
    assert_eq!(info.stack[0].function, "inicio");
    handle.continue_run();
    let events = collect_until_finished(&rx);
    assert!(matches!(
        events.last(),
        Some(Event::Finished(RunOutcome::Finished))
    ));
    handle.join();
}

#[test]
fn executa_graficos_com_o_host_da_ide() {
    struct HostGrafico(Arc<Mutex<Vec<&'static str>>>);

    impl Host for HostGrafico {
        fn graphics_open(&mut self, width: i32, height: i32) -> Result<(), String> {
            assert_eq!((width, height), (640, 480));
            self.0.lock().unwrap().push("abriu");
            Ok(())
        }

        fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
            assert_eq!(frame.first(), Some(&graphics::op::LOAD_FONT));
            assert_eq!(frame[frame.len() - 9], graphics::op::POINT);
            self.0.lock().unwrap().push("desenhou");
            Ok(())
        }

        fn graphics_close(&mut self) {
            self.0.lock().unwrap().push("fechou");
        }
    }

    let (observer, rx) = recorder();
    let operations = Arc::new(Mutex::new(Vec::new()));
    let host_operations = Arc::clone(&operations);
    start_portugol_rs_with_host(
        r"funcao inicio() {
            Graficos.iniciar_modo_grafico(falso);
            Graficos.desenhar_ponto(2, 3);
            Graficos.renderizar();
            Graficos.encerrar_modo_grafico();
        }",
        RunOptions::default(),
        observer,
        move |_| Box::new(HostGrafico(host_operations)),
    )
    .expect("compilar")
    .join();
    assert!(matches!(next(&rx), Event::Finished(RunOutcome::Finished)));
    assert_eq!(*operations.lock().unwrap(), ["abriu", "desenhou", "fechou"]);
}

#[test]
fn arvore_estrutural_inclui_funcoes_parametros_e_posicoes_utf16() {
    let source = r#"funcao saudacao(nome: cadeia) -> cadeia {
        // 😀
        seja prefixo: cadeia = "Olá, ";
        retorne prefixo + nome;
    }

    funcao inicio() {
        seja resultado: cadeia = saudacao("Ferro");
        escreva(resultado);
    }"#;
    let result = inspect_portugol_rs(source);
    assert!(result.diagnostics.is_empty(), "{:?}", result.diagnostics);
    let helper = result
        .symbols
        .iter()
        .find(|symbol| symbol.name == "saudacao")
        .expect("função auxiliar");
    assert_eq!(helper.kind, SymbolKind::Function);
    assert_eq!(helper.ty, "cadeia");
    assert_eq!(
        helper
            .children
            .iter()
            .map(|s| (s.name.as_str(), s.kind, s.ty))
            .collect::<Vec<_>>(),
        [
            ("nome", SymbolKind::Parameter, "cadeia"),
            ("prefixo", SymbolKind::Variable, "cadeia"),
        ]
    );
    let before_prefixo = source.split_once("prefixo").unwrap().0;
    assert_eq!(
        helper.children[1].position.offset as usize,
        before_prefixo.encode_utf16().count()
    );
}
