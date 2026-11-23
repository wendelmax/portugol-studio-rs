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

fn run_outcome(rx: &mpsc::Receiver<Event>) -> RunOutcome {
    loop {
        match rx
            .recv_timeout(Duration::from_secs(6))
            .expect("resultado da VM")
        {
            Event::Output(_) => {}
            Event::Finished(outcome) => return outcome,
            Event::Paused(_) => panic!("não havia ponto de parada"),
            Event::Input(_) => panic!("não havia leia"),
        }
    }
}

fn run_vm_to_output(source: &str) -> (String, RunOutcome) {
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(source, RunOptions::default(), observer).expect("iniciar VM");
    let mut output = String::new();
    let outcome = loop {
        match rx
            .recv_timeout(Duration::from_secs(5))
            .expect("evento da VM")
        {
            Event::Output(text) => output.push_str(&text),
            Event::Finished(outcome) => break outcome,
            Event::Paused(_) => panic!("não havia ponto de parada"),
            Event::Input(_) => panic!("não havia leia"),
        }
    };
    handle.join();
    (output, outcome)
}

#[test]
fn pors_debugger_returns_located_diagnostics_before_starting_the_vm() {
    let (source, code) = (
        "funcao inicio() { escreva(faltante); }",
        "pors.unknown-name",
    );
    let (tx, _rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let Err(diagnostics) = start_portugol_rs(source, RunOptions::default(), observer) else {
        panic!("fonte não suportada não deveria iniciar")
    };
    assert_eq!(diagnostics[0].code, code);
    assert_eq!(diagnostics[0].start.line, 1);
}

#[test]
fn pors_debugger_executes_typed_helper_functions_and_forward_calls() {
    let source = "funcao dobro(valor: inteiro) -> inteiro { retorne somar(valor, valor); } funcao somar(a: inteiro, b: inteiro) -> inteiro { retorne a + b; } funcao inicio() { escreva(dobro(21)); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "42");
}

#[test]
fn pors_debugger_executes_record_field_updates_in_the_vm() {
    let source = "registro Estado { contador: inteiro } funcao inicio() { seja mutavel estado: Estado = Estado { contador: 2 }; estado.contador += 3; escreva(estado.contador); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "5");
}

#[test]
fn pors_debugger_reads_records_stored_in_vectors() {
    let source = "registro Pessoa { idade: inteiro } funcao inicio() { seja pessoas: vetor<Pessoa> = [Pessoa { idade: 16 }, Pessoa { idade: 17 }]; escreva(pessoas[0].idade); escreva(pessoas[1].idade); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "1617");
}

#[test]
fn pors_debugger_keeps_copied_records_independent() {
    let source = "registro Pessoa { nome: cadeia } funcao inicio() { seja mutavel original: Pessoa = Pessoa { nome: \"Ana\" }; seja mutavel copia: Pessoa = original; copia.nome = \"Bia\"; escreva(original.nome); escreva(copia.nome); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "AnaBia");
}

#[test]
fn pors_debugger_passes_and_returns_records_by_value() {
    let source = "registro Estado { contador: inteiro } funcao incrementar(mutavel estado: Estado) -> Estado { estado.contador += 1; retorne estado; } funcao inicio() { seja original: Estado = Estado { contador: 4 }; seja proximo: Estado = incrementar(original); escreva(original.contador); escreva(proximo.contador); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "45");
}

#[test]
fn pors_debugger_matches_all_enum_variants_and_binds_payloads() {
    let source = "enum Evento { Incrementar, Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir(7); escolha evento { caso Evento::Incrementar => { escreva(0); } caso Evento::Definir(valor) => { escreva(valor); } } }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "7");
}

#[test]
fn pors_debugger_expands_records_and_shows_enum_variant_payloads() {
    let source = "registro Detalhe { valor: inteiro }\nregistro Estado { contador: inteiro, detalhe: Detalhe }\nenum Evento { Incrementar, Definir(inteiro) }\nfuncao inicio() {\n seja estado: Estado = Estado { contador: 3, detalhe: Detalhe { valor: 9 } };\n seja evento: Evento = Evento::Definir(7);\n escolha evento { caso Evento::Incrementar => {} caso Evento::Definir(valor) => {} }\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![7],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("programa depurável");
    let Event::Paused(info) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa na escolha")
    else {
        panic!("esperava pausa")
    };
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "estado" && variable.ty == "Estado")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "estado.contador" && variable.value == "3")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "estado.detalhe.valor" && variable.value == "9")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "evento" && variable.value == "Evento::Definir")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "evento.Definir[0]" && variable.value == "7")
    );
    handle.stop();
    handle.join();
}

#[test]
fn pors_debugger_exposes_helper_parameters_and_call_frames() {
    let source = "funcao dobrar(valor: inteiro) -> inteiro {\n retorne valor * 2;\n}\nfuncao inicio() {\n seja base: inteiro = 21;\n escreva(dobrar(base));\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![2],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("programa depurável");
    let Event::Paused(info) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa dentro da função")
    else {
        panic!("esperava pausa")
    };
    assert_eq!(
        info.stack.first().map(|frame| frame.function.as_str()),
        Some("dobrar")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "valor" && variable.value == "21")
    );
    handle.continue_run();
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
    handle.join();
}

#[test]
fn pors_debugger_executes_vector_indexing_and_tamanho() {
    let source = "funcao inicio() {\n seja mutavel valores: vetor<inteiro> = [2, 4];\n valores[1] = 5;\n escreva(valores[0] + valores[1]);\n escreva(tamanho(valores));\n}";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "72");
}

#[test]
fn pors_debugger_exposes_vector_elements_in_a_paused_snapshot() {
    let source =
        "funcao inicio() {\n seja valores: vetor<inteiro> = [7, 8];\n escreva(valores[0]);\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![3],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("programa com vetor depurável");
    let Event::Paused(info) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa antes de acessar vetor")
    else {
        panic!("esperava pausa")
    };
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "valores[0]" && variable.value == "7")
    );
    assert!(
        info.variables
            .iter()
            .any(|variable| variable.name == "valores[1]" && variable.value == "8")
    );
    handle.continue_run();
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
    handle.join();
}

#[test]
fn pors_debugger_localizes_vector_out_of_bounds_runtime_error() {
    let source =
        "funcao inicio() {\n seja valores: vetor<inteiro> = [7];\n escreva(valores[1]);\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(source, RunOptions::default(), observer)
        .expect("programa deve iniciar antes do bounds check");
    let RunOutcome::Error { start, message, .. } = run_outcome(&rx) else {
        panic!("esperava erro de índice")
    };
    assert_eq!(start.line, 3);
    assert!(message.contains("fora do intervalo"));
    handle.join();
}

#[test]
fn pors_vector_parameter_mutation_does_not_change_the_callers_copy() {
    let source = "funcao alterar(mutavel valores: vetor<inteiro>) -> inteiro { valores[0] = 9; retorne valores[0]; } funcao inicio() { seja mutavel original: vetor<inteiro> = [1]; escreva(alterar(original)); escreva(original[0]); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "91");
}

#[test]
fn pors_vector_assignment_and_return_keep_independent_values() {
    let source = "funcao criar() -> vetor<inteiro> { seja mutavel valores: vetor<inteiro> = [3]; retorne valores; } funcao inicio() { seja mutavel original: vetor<inteiro> = [1, 2]; seja mutavel copia: vetor<inteiro> = original; copia[1] = 8; seja retorno: vetor<inteiro> = criar(); escreva(original[1]); escreva(copia[1]); escreva(retorno[0]); }";
    let (output, outcome) = run_vm_to_output(source);
    assert_eq!(outcome, RunOutcome::Finished);
    assert_eq!(output, "283");
}

#[test]
fn pors_debugger_maps_semantic_diagnostics_to_utf16_editor_spans() {
    let source = "funcao inicio() { escreva(\"🦀\"); escreva(faltante); }";
    let expected_start = source
        .split("faltante")
        .next()
        .unwrap()
        .encode_utf16()
        .count() as u32;
    let (tx, _rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let Err(diagnostics) = start_portugol_rs(source, RunOptions::default(), observer) else {
        panic!("nome ausente não deveria iniciar a VM")
    };

    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "pors.unknown-name")
        .expect("diagnóstico de nome desconhecido");
    assert_eq!(diagnostic.start.offset, expected_start);
    assert_eq!(
        diagnostic.end.offset,
        expected_start + "faltante".encode_utf16().count() as u32
    );
}

#[test]
fn pors_debugger_reports_type_mismatch_before_starting_the_vm() {
    let source = "funcao inicio() { seja idade: inteiro = \"dez\"; }";
    let (tx, _rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let Err(diagnostics) = start_portugol_rs(source, RunOptions::default(), observer) else {
        panic!("inicializador incompatível não deveria iniciar a VM")
    };

    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.code == "pors.type-mismatch")
        .expect("diagnóstico de tipos incompatíveis");
    assert_eq!(diagnostic.start.line, 1);
    assert!(diagnostic.end.offset > diagnostic.start.offset);
}

#[test]
fn pors_debugger_shows_only_the_binding_active_in_each_scope() {
    let source = "funcao inicio() {\n seja valor: inteiro = 1;\n se (verdadeiro) {\n seja valor: inteiro = 2;\n escreva(valor);\n }\n escreva(valor);\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![5, 7],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("programa com shadowing deve ser depurável");

    let Event::Paused(inner) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa interna")
    else {
        panic!("esperava pausa no bloco interno")
    };
    let visible: Vec<_> = inner
        .variables
        .iter()
        .filter(|v| v.name == "valor")
        .collect();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].value, "2");
    handle.continue_run();

    let Event::Paused(outer) = rx
        .recv_timeout(Duration::from_secs(5))
        .expect("pausa externa")
    else {
        panic!("esperava pausa no escopo externo")
    };
    let visible: Vec<_> = outer
        .variables
        .iter()
        .filter(|v| v.name == "valor")
        .collect();
    assert_eq!(visible.len(), 1);
    assert_eq!(visible[0].value, "1");
    handle.continue_run();
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
    handle.join();
}

#[test]
fn pors_debugger_restores_shadowed_binding_after_counted_loop() {
    let source = "funcao inicio() {\n seja mutavel i: inteiro = 9;\n para (seja mutavel i: inteiro = 0; i < 1; i += 1) {\n escreva(i);\n }\n escreva(i);\n}";
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        source,
        RunOptions {
            breakpoints: vec![4, 6],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("laço com binding sombreado deve ser depurável");

    for expected in ["0", "9"] {
        let Event::Paused(info) = rx
            .recv_timeout(Duration::from_secs(5))
            .expect("pausa no laço ou depois dele")
        else {
            panic!("esperava breakpoint")
        };
        let visible: Vec<_> = info.variables.iter().filter(|v| v.name == "i").collect();
        assert_eq!(visible.len(), 1);
        assert_eq!(visible[0].value, expected);
        handle.continue_run();
    }
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
    handle.join();
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
fn pors_debugger_applies_string_memory_limit_to_chain_input() {
    const STRING_LIMIT: usize = 16 * 1024 * 1024;
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        "funcao inicio() {\n seja mutavel texto: cadeia = \"\";\n leia(texto);\n}",
        RunOptions::default(),
        observer,
    )
    .expect("iniciar debugger com leitura de cadeia");

    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5))
            .expect("pedido de entrada"),
        Event::Input("cadeia")
    );
    handle.send_input("x".repeat(STRING_LIMIT + 1));

    assert_eq!(
        run_outcome(&rx),
        RunOutcome::RuntimeLimit {
            reason: portugol_service::RunLimitReason::StringMemory,
        }
    );
    handle.join();
}

#[test]
fn pors_vm_debug_output_matches_generated_rust_for_core_statements() {
    if !portugol_service::rust_toolchain_available() {
        eprintln!("rustc indisponível; teste diferencial ignorado neste ambiente");
        return;
    }
    let sources = [
        (
            "funcao inicio() {\n seja mutavel total: inteiro = 0;\n para (seja mutavel n: inteiro = 1; n <= 4; n += 1) {\n total += n;\n }\n se (total == 10 && verdadeiro) {\n escreva(total);\n } senao {\n escreva(0);\n }\n}",
            "10",
        ),
        (
            "funcao inicio() { escreva(2147483647 + 1); }",
            "-2147483648",
        ),
        (
            "funcao inicio() { escreva(\"Olá \" + \"Rust\"); }",
            "Olá Rust",
        ),
        (
            "funcao inicio() {\n seja mutavel n: inteiro = 0;\n enquanto (n < 3) {\n n += 1;\n se (n == 2) { continue; }\n escreva(n);\n }\n}",
            "13",
        ),
        (
            "funcao inicio() { se (falso && verdadeiro) { escreva(1); } senao { escreva(2); } }",
            "2",
        ),
        (
            "funcao repetir(texto: cadeia) -> cadeia { retorne texto + texto; } funcao inicio() { seja original: cadeia = \"oi\"; escreva(repetir(original)); escreva(original); }",
            "oioioi",
        ),
        (
            "funcao fatorial(n: inteiro) -> inteiro { se (n <= 1) { retorne 1; } senao { retorne n * fatorial(n - 1); } } funcao inicio() { escreva(fatorial(5)); }",
            "120",
        ),
        (
            "registro Pessoa { nome: cadeia } enum Mensagem { Texto(cadeia) } funcao inicio() { seja nome: cadeia = \"Ana\"; seja pessoa: Pessoa = Pessoa { nome: nome }; escreva(nome); escreva(pessoa.nome); seja texto: cadeia = \"oi\"; seja mensagem: Mensagem = Mensagem::Texto(texto); escreva(texto); escolha mensagem { caso Mensagem::Texto(conteudo) => { escreva(conteudo); } } }",
            "AnaAnaoioi",
        ),
        (
            "registro Estado { valor: inteiro } enum Envelope { Dados(Estado) } funcao inicio() { seja mutavel original: Estado = Estado { valor: 1 }; seja envelope: Envelope = Envelope::Dados(original); original.valor = 2; escolha envelope { caso Envelope::Dados(estado) => { escreva(estado.valor); } } escreva(original.valor); }",
            "12",
        ),
    ];
    for (source, expected) in sources {
        let generated = portugol_service::run_portugol_rs(source).expect("compilar Rust gerado");
        let portugol_service::PortugolRsRunResult::Executed {
            stdout,
            exit_code: Some(0),
            ..
        } = generated
        else {
            panic!("Rust gerado não concluiu: {generated:?}");
        };
        let (vm_output, outcome) = run_vm_to_output(source);
        assert_eq!(outcome, RunOutcome::Finished);
        assert_eq!(stdout, expected);
        assert_eq!(vm_output, stdout);
    }
}

#[test]
fn pors_vm_and_generated_rust_match_across_binary_operator_families() {
    if !portugol_service::rust_toolchain_available() {
        eprintln!("rustc indisponível; teste diferencial ignorado neste ambiente");
        return;
    }
    let cases = [
        (
            "funcao inicio() { escreva(7 - 10); escreva(\"|\"); escreva(4 * 3); escreva(\"|\"); escreva(20 / 3); escreva(\"|\"); escreva(20 % 3); }",
            "-3|12|6|2",
        ),
        (
            "funcao inicio() { seja mutavel valor: inteiro = 15; valor -= 4; valor *= 2; valor /= 3; valor %= 4; escreva(valor); }",
            "3",
        ),
        ("funcao inicio() { escreva(1.5 + 2.25); }", "3.75"),
        (
            "funcao inicio() { se (\"Portugol\" == \"Portugol\" && \"x\" != \"y\") { escreva(\"strings\"); } senao { escreva(\"mismatch\"); } }",
            "strings",
        ),
        (
            "funcao inicio() { se (2 < 3 && 3 <= 3 && 4 > 3 && 4 >= 4 && 5 == 5 && 5 != 6) { escreva(\"comparisons\"); } senao { escreva(\"mismatch\"); } }",
            "comparisons",
        ),
        (
            "funcao inicio() { se (verdadeiro || 1 / 0 == 0) { escreva(\"or\"); } se (falso && 1 / 0 == 0) { escreva(\"mismatch\"); } se (!falso) { escreva(\"|and\"); } }",
            "or|and",
        ),
    ];

    for (source, expected) in cases {
        let generated = portugol_service::run_portugol_rs(source).expect("compilar Rust gerado");
        let portugol_service::PortugolRsRunResult::Executed {
            stdout,
            exit_code: Some(0),
            ..
        } = generated
        else {
            panic!("Rust gerado não concluiu: {generated:?}");
        };
        let (vm_output, outcome) = run_vm_to_output(source);
        assert_eq!(outcome, RunOutcome::Finished);
        assert_eq!(stdout, expected);
        assert_eq!(vm_output, stdout);
    }
}

#[test]
fn pors_debugger_locates_bitwise_operators_outside_the_supported_subset() {
    let source = "funcao inicio() { escreva(6 & 3); }";
    let (tx, _rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let Err(diagnostics) = start_portugol_rs(source, RunOptions::default(), observer) else {
        panic!("operador bit a bit ainda não faz parte da gramática .pors")
    };
    let ampersand = source.find('&').unwrap() as u32;
    assert_eq!(diagnostics[0].code, "pors.invalid-character");
    assert_eq!(diagnostics[0].start.offset, ampersand);
    assert_eq!(diagnostics[0].end.offset, ampersand + 1);
}

#[test]
fn pors_vm_and_generated_rust_agree_on_division_failure_source_line() {
    if !portugol_service::rust_toolchain_available() {
        eprintln!("rustc indisponível; teste diferencial ignorado neste ambiente");
        return;
    }
    let source = "funcao inicio() {\n seja mutavel zero: inteiro = 0;\n escreva(1 / zero);\n}";
    let generated = portugol_service::run_portugol_rs(source).expect("executar Rust gerado");
    assert!(matches!(
        generated,
        portugol_service::PortugolRsRunResult::Executed { exit_code: Some(code), .. } if code != 0
    ));
    let (_, outcome) = run_vm_to_output(source);
    let RunOutcome::Error { start, .. } = outcome else {
        panic!("VM devia informar erro de execução: {outcome:?}");
    };
    assert_eq!(start.line, 3);
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
fn pors_debugger_returns_typed_output_string_and_active_time_limits() {
    let cases = [
        (
            "funcao inicio() { enquanto (verdadeiro) { escreva(\"x\"); } }",
            portugol_service::RunLimitReason::Output,
        ),
        (
            "funcao inicio() { seja mutavel texto: cadeia = \"\"; enquanto (verdadeiro) { texto += \"x\"; } }",
            portugol_service::RunLimitReason::StringMemory,
        ),
        (
            "funcao inicio() { enquanto (verdadeiro) { } }",
            portugol_service::RunLimitReason::Time,
        ),
    ];
    for (source, expected) in cases {
        let (tx, rx) = mpsc::channel();
        let observer = Arc::new(Recorder(Mutex::new(tx)));
        let handle = start_portugol_rs(source, RunOptions::default(), observer)
            .expect("iniciar programa Portugol-RS");
        assert_eq!(
            run_outcome(&rx),
            RunOutcome::RuntimeLimit { reason: expected },
            "limite esperado: {expected:?}"
        );
        handle.join();
    }
}

#[test]
fn pors_debugger_excludes_pause_and_input_wait_from_runtime_budget() {
    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        "funcao inicio() {\n escreva(42);\n}",
        RunOptions {
            breakpoints: vec![2],
            ..RunOptions::default()
        },
        observer,
    )
    .expect("iniciar debugger");
    assert!(matches!(
        rx.recv_timeout(Duration::from_secs(5)).expect("breakpoint"),
        Event::Paused(_)
    ));
    std::thread::sleep(Duration::from_millis(3_100));
    handle.continue_run();
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
    handle.join();

    let (tx, rx) = mpsc::channel();
    let observer = Arc::new(Recorder(Mutex::new(tx)));
    let handle = start_portugol_rs(
        "funcao inicio() {\n seja mutavel valor: inteiro = 0;\n leia(valor);\n escreva(valor);\n}",
        RunOptions::default(),
        observer,
    )
    .expect("iniciar debugger");
    assert_eq!(
        rx.recv_timeout(Duration::from_secs(5))
            .expect("pedido de leia"),
        Event::Input("inteiro")
    );
    std::thread::sleep(Duration::from_millis(3_100));
    handle.send_input("42".into());
    assert_eq!(run_outcome(&rx), RunOutcome::Finished);
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
