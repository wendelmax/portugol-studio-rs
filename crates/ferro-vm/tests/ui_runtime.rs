use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ferro_bytecode::{Function, LocalInfo, Module, NativeRef, Op, ValueType};
use ferro_hir::Span;
use ferro_vm::ui::{UiBudget, UiError, UiEvent, UiNode, UiSession};
use ferro_vm::{BufferIo, RuntimeLimitReason, Value, Vm};

fn local(name: &str, ty: ValueType) -> LocalInfo {
    LocalInfo {
        name: name.into(),
        ty,
        dims: 0,
        by_ref: false,
        ferro_debug: None,
    }
}

/// Constrói um módulo MVU completo:
/// - `iniciar() -> inteiro` retorna `0`
/// - `atualizar(estado: inteiro, tipo: cadeia, id: cadeia, valor: cadeia) -> inteiro`
///   incrementa em `clique("inc")` e zera em `entrada_alterada("reset", "0")`
/// - `visualizar(estado: inteiro) -> inteiro` cria `Coluna([Texto("contagem"), Botao("inc", "+1"), Entrada("reset", "", "digite 0")])`
fn counter_mvu_module() -> Module {
    let span = Span::new(0, 1);
    let strings = vec![
        "inc".to_string(),      // 0
        "clique".to_string(),   // 1
        "contagem".to_string(), // 2
        "+1".to_string(),       // 3
        "reset".to_string(),    // 4
        "".to_string(),         // 5
        "digite 0".to_string(), // 6
    ];
    let natives = vec![
        NativeRef {
            library: "UI".into(),
            name: "texto".into(),
        }, // 0
        NativeRef {
            library: "UI".into(),
            name: "botao".into(),
        }, // 1
        NativeRef {
            library: "UI".into(),
            name: "entrada".into(),
        }, // 2
        NativeRef {
            library: "UI".into(),
            name: "coluna".into(),
        }, // 3
    ];

    let iniciar = Function {
        name: "iniciar".into(),
        arity: 0,
        return_type: ValueType::Inteiro,
        locals: vec![],
        code: vec![Op::PushInt(0), Op::Return],
        spans: vec![span; 2],
        nodes: vec![None; 2],
        span,
    };

    // se (tipo == "clique" && id == "inc") { retorne estado + 1; }
    // se (id == "reset") { retorne 0; }
    // retorne estado;
    let atualizar_code = vec![
        Op::LoadLocal(1),    // 0: tipo
        Op::PushStr(1),      // 1: "clique"
        Op::Eq,              // 2
        Op::JumpIfFalse(12), // 3
        Op::LoadLocal(2),    // 4: id
        Op::PushStr(0),      // 5: "inc"
        Op::Eq,              // 6
        Op::JumpIfFalse(12), // 7
        Op::LoadLocal(0),    // 8: estado
        Op::PushInt(1),      // 9
        Op::Add,             // 10
        Op::Return,          // 11
        Op::LoadLocal(2),    // 12: id
        Op::PushStr(4),      // 13: "reset"
        Op::Eq,              // 14
        Op::JumpIfFalse(18), // 15
        Op::PushInt(0),      // 16
        Op::Return,          // 17
        Op::LoadLocal(0),    // 18: estado
        Op::Return,          // 19
    ];
    let atualizar = Function {
        name: "atualizar".into(),
        arity: 4,
        return_type: ValueType::Inteiro,
        locals: vec![
            local("estado", ValueType::Inteiro),
            local("tipo", ValueType::Cadeia),
            local("id", ValueType::Cadeia),
            local("valor", ValueType::Cadeia),
        ],
        spans: vec![span; atualizar_code.len()],
        nodes: vec![None; atualizar_code.len()],
        code: atualizar_code,
        span,
    };

    // texto_id = UI.texto("contagem:" + estado) -> para manter simples na VM, usa "contagem"
    // botao_id = UI.botao("inc", "+1")
    // entrada_id = UI.entrada("reset", "", "digite 0")
    // filhos = [texto_id, botao_id, entrada_id]
    // retorne UI.coluna(filhos)
    let visualizar_code = vec![
        Op::PushStr(2), // "contagem"
        Op::LoadLocal(0),
        Op::Add, // "contagem0", "contagem1", ...
        Op::CallNative { native: 0, argc: 1 },
        Op::PushStr(0),
        Op::PushStr(3),
        Op::CallNative { native: 1, argc: 2 },
        Op::PushStr(4),
        Op::PushStr(5),
        Op::PushStr(6),
        Op::CallNative { native: 2, argc: 3 },
        Op::ArrayLit(3),
        Op::CallNative { native: 3, argc: 1 },
        Op::Return,
    ];
    let visualizar = Function {
        name: "visualizar".into(),
        arity: 1,
        return_type: ValueType::Inteiro,
        locals: vec![local("estado", ValueType::Inteiro)],
        spans: vec![span; visualizar_code.len()],
        nodes: vec![None; visualizar_code.len()],
        code: visualizar_code,
        span,
    };

    Module {
        functions: vec![iniciar, atualizar, visualizar],
        entry: 0,
        globals: vec![],
        strings,
        natives,
        native_consts: vec![],
    }
}

fn assert_int(value: &Value, expected: i32) {
    assert!(
        matches!(value, Value::Int(actual) if *actual == expected),
        "esperava inteiro {expected}, recebido {value:?}"
    );
}

#[test]
fn runs_headless_mvu_lifecycle_with_history_and_canonical_json() {
    let module = counter_mvu_module();
    let mut io = BufferIo::default();
    let interrupt = Arc::new(AtomicBool::new(false));

    let mut session = UiSession::start(&module, &mut io, UiBudget::default(), interrupt)
        .expect("sessão MVU deve iniciar");
    assert_int(session.state(), 0);
    assert_eq!(session.history().len(), 1);
    assert_eq!(
        session.tree().to_json(),
        r#"{"kind":"coluna","children":[{"kind":"texto","content":"contagem0"},{"kind":"botao","id":"inc","label":"+1"},{"kind":"entrada","id":"reset","value":"","placeholder":"digite 0"}]}"#
    );

    let snap1 = session
        .dispatch(UiEvent::Click { id: "inc".into() }, &mut io)
        .expect("primeiro clique");
    assert_eq!(snap1.step, 1);
    assert_int(&snap1.state, 1);
    assert!(snap1.tree.to_json().contains("\"content\":\"contagem1\""));

    let snap2 = session
        .dispatch(UiEvent::Click { id: "inc".into() }, &mut io)
        .expect("segundo clique");
    assert_eq!(snap2.step, 2);
    assert_int(&snap2.state, 2);

    let snap3 = session
        .dispatch(
            UiEvent::InputChanged {
                id: "reset".into(),
                value: "0".into(),
            },
            &mut io,
        )
        .expect("evento de entrada");
    assert_eq!(snap3.step, 3);
    assert_int(&snap3.state, 0);

    // Histórico MVU preserva as transições anteriores para inspeção/time-travel.
    assert_eq!(session.history().len(), 4);
    assert_int(&session.history()[0].state, 0);
    assert_int(&session.history()[2].state, 2);
    assert_int(&session.history()[3].state, 0);
}

#[test]
fn invokes_ferro_functions_reentrantly_with_per_event_step_budget() {
    let source = r#"
funcao proximo(estado: inteiro, delta: inteiro) -> inteiro {
    retorne estado + delta;
}

funcao travar(estado: inteiro) -> inteiro {
    enquanto (verdadeiro) {
    }
    retorne estado;
}

funcao inicio() {
}
"#;
    let hir = ferro_frontend::lower(source).expect("fonte FERRO válida");
    let module = ferro_bytecode::compile_ferro(&hir).expect("bytecode FERRO");
    let mut io = BufferIo::default();
    let mut vm = Vm::new(&module, &mut io);

    let mut estado = Value::Int(10);
    for _ in 0..5 {
        estado = vm
            .call_function("proximo", vec![estado, Value::Int(3)], 500)
            .expect("chamada FERRO dentro do orçamento");
    }
    assert_int(&estado, 25);

    let err = vm
        .call_function("travar", vec![estado], 200)
        .expect_err("laço infinito deve estourar o orçamento de passos do evento");
    assert_eq!(err.limit, Some(RuntimeLimitReason::Time));
}

#[test]
fn enforces_node_and_depth_limits_and_explicit_cancellation() {
    let budget = UiBudget {
        max_nodes: 3,
        max_depth: 2,
        ..UiBudget::default()
    };

    let too_wide = UiNode::Column {
        children: vec![
            UiNode::Text {
                content: "a".into(),
            },
            UiNode::Text {
                content: "b".into(),
            },
            UiNode::Text {
                content: "c".into(),
            },
        ],
    };
    assert_eq!(
        too_wide.validate(&budget),
        Err(UiError::NodeLimitExceeded { limit: 3 })
    );

    let too_deep = UiNode::Column {
        children: vec![UiNode::Row {
            children: vec![UiNode::Text {
                content: "x".into(),
            }],
        }],
    };
    assert_eq!(
        too_deep.validate(&budget),
        Err(UiError::DepthLimitExceeded { limit: 2 })
    );

    let module = counter_mvu_module();
    let mut io = BufferIo::default();
    let interrupt = Arc::new(AtomicBool::new(false));
    let mut session = UiSession::start(
        &module,
        &mut io,
        UiBudget::default(),
        Arc::clone(&interrupt),
    )
    .expect("iniciar sessão");
    interrupt.store(true, Ordering::Relaxed);
    assert!(matches!(
        session.dispatch(UiEvent::Click { id: "inc".into() }, &mut io),
        Err(UiError::Cancelled)
    ));
}

#[test]
fn escapes_json_strings_and_supports_all_adr_0017_components() {
    let tree = UiNode::List {
        items: vec![UiNode::Row {
            children: vec![
                UiNode::Text {
                    content: "Olá \"FERRO\"\n\\🦀".into(),
                },
                UiNode::Button {
                    id: "ok".into(),
                    label: "Enviar\t!".into(),
                },
            ],
        }],
    };
    assert_eq!(
        tree.to_json(),
        r#"{"kind":"lista","items":[{"kind":"linha","children":[{"kind":"texto","content":"Olá \"FERRO\"\n\\🦀"},{"kind":"botao","id":"ok","label":"Enviar\t!"}]}]}"#
    );
    assert_eq!(
        UiEvent::InputChanged {
            id: "nome".into(),
            value: "Ana \"B\"".into(),
        }
        .to_json(),
        r#"{"kind":"entrada_alterada","id":"nome","value":"Ana \"B\""}"#
    );
}

#[test]
fn bounds_mvu_history_and_string_memory_per_event() {
    let module = counter_mvu_module();
    let mut io = BufferIo::default();
    let budget = UiBudget {
        max_history: 2,
        ..UiBudget::default()
    };
    let mut session = UiSession::start(&module, &mut io, budget, Arc::new(AtomicBool::new(false)))
        .expect("iniciar sessão");
    for _ in 0..4 {
        session
            .dispatch(UiEvent::Click { id: "inc".into() }, &mut io)
            .expect("clique dentro do orçamento");
    }
    assert_eq!(session.history().len(), 2);
    assert_eq!(session.history()[0].step, 3);
    assert_eq!(session.history()[1].step, 4);

    let source = r#"
funcao explodir(texto: cadeia) -> cadeia {
    seja mutavel acc: cadeia = texto;
    enquanto (verdadeiro) {
        acc += acc;
    }
    retorne acc;
}
funcao inicio() {}
"#;
    let hir = ferro_frontend::lower(source).expect("fonte FERRO válida");
    let mem_module = ferro_bytecode::compile_ferro(&hir).expect("bytecode FERRO");
    let mut vm = Vm::new(&mem_module, &mut io).with_string_memory_limit(512);
    let err = vm
        .call_function("explodir", vec![Value::str("x")], 100_000)
        .expect_err("limite de memória de cadeias deve encerrar o evento");
    assert_eq!(err.limit, Some(RuntimeLimitReason::StringMemory));
}

#[test]
fn runs_end_to_end_ferro_source_with_records_enums_and_ui_calls_in_ui_session() {
    let source = r#"
enum Acao { Pronto, Editado }

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    acao: Acao
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "Visitante", acao: Acao::Pronto };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, acao: Acao::Editado };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, acao: Acao::Editado };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel selo: cadeia = "pronto";
    escolha estado.acao {
        caso Acao::Pronto => { selo = "pronto"; }
        caso Acao::Editado => { selo = "editado"; }
    }
    retorne UI.coluna([
        UI.texto("Olá, ", estado.nome, " | total=", estado.contagem, " | ", selo),
        UI.linha([
            UI.botao("inc", "+1"),
            UI.entrada("nome", estado.nome, "Digite seu nome")
        ]),
        UI.lista([UI.texto("Passo ", estado.contagem)])
    ]);
}
"#;
    let hir = ferro_frontend::lower(source).expect("fonte FERRO visual válida");
    let module = ferro_bytecode::compile_ferro(&hir).expect("bytecode FERRO visual válido");
    let mut io = BufferIo::default();
    let mut session = UiSession::start(
        &module,
        &mut io,
        UiBudget::default(),
        Arc::new(AtomicBool::new(false)),
    )
    .expect("iniciar sessão MVU a partir de fonte .fer");

    assert!(
        session
            .tree()
            .to_json()
            .contains("\"content\":\"Olá, Visitante | total=0 | pronto\"")
    );

    let after_click = session
        .dispatch(UiEvent::Click { id: "inc".into() }, &mut io)
        .expect("despachar clique");
    assert!(
        after_click
            .tree
            .to_json()
            .contains("\"content\":\"Olá, Visitante | total=1 | editado\"")
    );
    assert!(
        after_click.tree.to_json().contains(
            "\"kind\":\"lista\",\"items\":[{\"kind\":\"texto\",\"content\":\"Passo 1\"}]"
        )
    );

    let after_input = session
        .dispatch(
            UiEvent::InputChanged {
                id: "nome".into(),
                value: "Ada".into(),
            },
            &mut io,
        )
        .expect("despachar entrada_alterada");
    assert!(
        after_input
            .tree
            .to_json()
            .contains("\"content\":\"Olá, Ada | total=1 | editado\"")
    );
    assert!(after_input.tree.to_json().contains(
        "\"kind\":\"entrada\",\"id\":\"nome\",\"value\":\"Ada\",\"placeholder\":\"Digite seu nome\""
    ));
}
