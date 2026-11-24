use std::slice;

use portugol_bytecode::{Function, LocalInfo, Module, NativeRef, Op, ValueType};
use portugol_syntax::Span;
use portugol_syntax::ast::Type;
use portugol_vm::ui::UiBudget;
use portugol_wasm::{
    RunBudget, SessionStore, format_pors_json, pors_alloc, pors_dealloc, pors_format,
    pors_result_len, pors_result_ptr, pors_run, pors_ui_close, pors_ui_start, pors_validate,
    run_pors_json, validate_pors_json,
};

fn local(name: &str, ty: Type) -> LocalInfo {
    LocalInfo {
        name: name.into(),
        ty,
        dims: 0,
        by_ref: false,
        pors_debug: None,
    }
}

fn build_counter_ui_module() -> Module {
    let span = Span::new(0, 1);

    // iniciar() -> inteiro { retorne 0 }
    let iniciar = Function {
        name: "iniciar".into(),
        arity: 0,
        return_type: ValueType::Inteiro,
        locals: vec![],
        code: vec![Op::PushInt(0), Op::Return],
        spans: vec![span; 2],
        span,
    };

    // atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia, valor: cadeia) -> inteiro
    // Se alvo == "inc" -> estado + 1; se alvo == "nome" e valor == "reset" -> 0; senao estado
    let atualizar = Function {
        name: "atualizar".into(),
        arity: 4,
        return_type: ValueType::Inteiro,
        locals: vec![
            local("estado", Type::Inteiro),
            local("tipo", Type::Cadeia),
            local("alvo", Type::Cadeia),
            local("valor", Type::Cadeia),
        ],
        code: vec![
            // if alvo == "inc" (string 0) -> estado + 1
            Op::LoadLocal(2),
            Op::PushStr(0),
            Op::Eq,
            Op::JumpIfFalse(8),
            Op::LoadLocal(0),
            Op::PushInt(1),
            Op::Add,
            Op::Return,
            // if valor == "reset" (string 1) -> 0
            Op::LoadLocal(3),
            Op::PushStr(1),
            Op::Eq,
            Op::JumpIfFalse(14),
            Op::PushInt(0),
            Op::Return,
            // else estado
            Op::LoadLocal(0),
            Op::Return,
        ],
        spans: vec![span; 16],
        span,
    };

    // visualizar(estado: inteiro) -> inteiro
    // texto("Total: " + estado), botao("inc", "+1"), entrada("nome", "", "Digite"), coluna([...])
    let visualizar = Function {
        name: "visualizar".into(),
        arity: 1,
        return_type: ValueType::Inteiro,
        locals: vec![local("estado", Type::Inteiro)],
        code: vec![
            // UI.texto("Total: " + estado) -> native 0
            Op::PushStr(2),
            Op::LoadLocal(0),
            Op::Add,
            Op::CallNative { native: 0, argc: 1 },
            // UI.botao("inc", "+1") -> native 1
            Op::PushStr(0),
            Op::PushStr(3),
            Op::CallNative { native: 1, argc: 2 },
            // UI.entrada("nome", "", "Digite") -> native 2
            Op::PushStr(4),
            Op::PushStr(5),
            Op::PushStr(6),
            Op::CallNative { native: 2, argc: 3 },
            // UI.coluna([texto, botao, entrada]) -> native 3
            Op::ArrayLit(3),
            Op::CallNative { native: 3, argc: 1 },
            Op::Return,
        ],
        spans: vec![span; 14],
        span,
    };

    Module {
        functions: vec![iniciar, atualizar, visualizar],
        entry: 0,
        globals: vec![],
        strings: vec![
            "inc".into(),     // 0
            "reset".into(),   // 1
            "Total: ".into(), // 2
            "+1".into(),      // 3
            "nome".into(),    // 4
            "".into(),        // 5
            "Digite".into(),  // 6
        ],
        natives: vec![
            NativeRef {
                library: "UI".into(),
                name: "texto".into(),
            },
            NativeRef {
                library: "UI".into(),
                name: "botao".into(),
            },
            NativeRef {
                library: "UI".into(),
                name: "entrada".into(),
            },
            NativeRef {
                library: "UI".into(),
                name: "coluna".into(),
            },
        ],
        native_consts: vec![],
    }
}

#[test]
fn validates_and_formats_pors_sources_with_structured_json() {
    let valid = "funcao inicio() {\n    seja x: inteiro = 1;\n    escreva(x);\n}\n";
    assert_eq!(
        validate_pors_json(valid),
        "{\"ok\":true,\"diagnostics\":[]}"
    );

    let formatted_already = format_pors_json(valid);
    assert!(formatted_already.contains("\"ok\":true"));
    assert!(formatted_already.contains("\"changed\":false"));

    let compact = "funcao inicio(){escreva(\"olá 🦀\");seja x:inteiro=2+3;escreva(x);}";
    let formatted = format_pors_json(compact);
    assert!(formatted.contains("\"ok\":true"));
    assert!(formatted.contains("\"changed\":true"));
    assert!(formatted.contains("seja x: inteiro = 2 + 3;"));

    let invalid = "funcao inicio() { escreva(desconhecido); }";
    let invalid_json = validate_pors_json(invalid);
    assert!(invalid_json.contains("\"ok\":false"));
    assert!(invalid_json.contains("pors.unknown-name"));
}

#[test]
fn executes_pors_with_input_vectors_and_escapes_json_stdout() {
    let source = r#"
funcao somar(itens: vetor<inteiro>) -> inteiro {
    seja mutavel total: inteiro = 0;
    para (seja mutavel i: inteiro = 0; i < tamanho(itens); i += 1) {
        total += itens[i];
    }
    retorne total;
}

funcao inicio() {
    seja mutavel nome: cadeia = "";
    seja mutavel bonus: inteiro = 0;
    leia(nome);
    leia(bonus);
    seja notas: vetor<inteiro> = [10, 20];
    escreva("Aluno \"", nome, "\": ", somar(notas) + bonus);
}
"#;
    let json = run_pors_json(source, "Ada\n12\n", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"Aluno \\\"Ada\\\": 42\",\"diagnostics\":[]}"
    );
}

#[test]
fn enforces_step_string_memory_and_output_budgets_in_run_pors() {
    let infinite_loop = "funcao inicio() { enquanto (verdadeiro) {} }";
    let step_err = run_pors_json(
        infinite_loop,
        "",
        RunBudget {
            max_steps: 500,
            ..RunBudget::default()
        },
    );
    assert!(step_err.contains("\"ok\":false"));
    assert!(step_err.contains("\"limit\":\"steps\""));

    let string_growth =
        "funcao inicio() { seja mutavel s: cadeia = \"a\"; enquanto (verdadeiro) { s += s; } }";
    let mem_err = run_pors_json(
        string_growth,
        "",
        RunBudget {
            max_steps: 100_000,
            max_string_bytes: 512,
            ..RunBudget::default()
        },
    );
    assert!(mem_err.contains("\"ok\":false"));
    assert!(mem_err.contains("\"limit\":\"string_memory\""));

    let output_flood = "funcao inicio() { enquanto (verdadeiro) { escreva(\"0123456789\"); } }";
    let out_err = run_pors_json(
        output_flood,
        "",
        RunBudget {
            max_steps: 100_000,
            max_output_bytes: 32,
            ..RunBudget::default()
        },
    );
    assert!(out_err.contains("\"ok\":false"));
    assert!(out_err.contains("\"limit\":\"output\""));
}

#[test]
fn drives_owned_mvu_ui_session_lifecycle_and_serializes_tree_json() {
    let mut store = SessionStore::new();
    let started = store.start_module(build_counter_ui_module(), UiBudget::default());
    assert!(started.contains("\"ok\":true"));
    assert!(started.contains("\"session\":1"));
    assert!(started.contains("\"step\":0"));
    assert!(started.contains("\"kind\":\"coluna\""));
    assert!(started.contains("\"content\":\"Total: 0\""));
    assert!(started.contains("\"kind\":\"botao\",\"id\":\"inc\",\"label\":\"+1\""));
    assert!(started.contains("\"kind\":\"entrada\",\"id\":\"nome\""));
    assert_eq!(store.active_count(), 1);

    let after_click = store.dispatch(1, "clique", "inc", "");
    assert!(after_click.contains("\"ok\":true"));
    assert!(after_click.contains("\"step\":1"));
    assert!(after_click.contains("\"content\":\"Total: 1\""));

    let after_input = store.dispatch(1, "entrada_alterada", "nome", "reset");
    assert!(after_input.contains("\"ok\":true"));
    assert!(after_input.contains("\"step\":2"));
    assert!(after_input.contains("\"content\":\"Total: 0\""));

    let snap = store.snapshot(1);
    assert!(snap.contains("\"ok\":true"));
    assert!(snap.contains("\"step\":2"));
    assert!(snap.contains("\"history_len\":3"));

    let bad_event = store.dispatch(1, "arrastar", "inc", "");
    assert!(bad_event.contains("\"ok\":false"));
    assert!(bad_event.contains("\"code\":\"invalid_event\""));

    assert!(store.close(1));
    assert!(!store.close(1));
    assert_eq!(store.active_count(), 0);

    let missing = store.snapshot(1);
    assert!(missing.contains("\"ok\":false"));
    assert!(missing.contains("\"code\":\"session_not_found\""));
}

#[test]
fn exercises_exported_c_abi_memory_and_result_buffer() {
    let source = "funcao inicio() { escreva(20 + 22); }";
    let src_bytes = source.as_bytes();
    let ptr = pors_alloc(src_bytes.len());
    assert!(!ptr.is_null());

    // SAFETY: `ptr` foi alocado com exatamente `src_bytes.len()` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(src_bytes.as_ptr(), ptr, src_bytes.len());
    }

    let validate_len = unsafe { pors_validate(ptr, src_bytes.len()) };
    assert_eq!(validate_len, pors_result_len());
    let validate_out = unsafe { slice::from_raw_parts(pors_result_ptr(), validate_len) };
    assert_eq!(
        std::str::from_utf8(validate_out).unwrap(),
        "{\"ok\":true,\"diagnostics\":[]}"
    );

    let format_len = unsafe { pors_format(ptr, src_bytes.len()) };
    let format_out = unsafe { slice::from_raw_parts(pors_result_ptr(), format_len) };
    assert!(
        std::str::from_utf8(format_out)
            .unwrap()
            .contains("\"changed\":true")
    );

    let run_len = unsafe { pors_run(ptr, src_bytes.len(), std::ptr::null(), 0, 0, 0, 0) };
    let run_out = unsafe { slice::from_raw_parts(pors_result_ptr(), run_len) };
    assert_eq!(
        std::str::from_utf8(run_out).unwrap(),
        "{\"ok\":true,\"stdout\":\"42\",\"diagnostics\":[]}"
    );

    // Um programa comum sem iniciar/atualizar/visualizar devolve erro estruturado.
    let ui_len = unsafe { pors_ui_start(ptr, src_bytes.len(), 0, 0, 0) };
    let ui_out = unsafe { slice::from_raw_parts(pors_result_ptr(), ui_len) };
    assert!(
        std::str::from_utf8(ui_out)
            .unwrap()
            .contains("\"code\":\"missing_function\"")
    );
    assert_eq!(pors_ui_close(999), 0);

    unsafe {
        pors_dealloc(ptr, src_bytes.len());
    }
}

#[test]
fn runs_pors_source_mvu_app_end_to_end_through_wasm_session_store() {
    let source = r#"
registro Estado {
    contagem: inteiro,
    nome: cadeia
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "" };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    retorne UI.coluna([
        UI.texto("Contador (", estado.nome, "): ", estado.contagem),
        UI.botao("inc", "+1"),
        UI.entrada("nome", estado.nome, "Nome")
    ]);
}
"#;
    let mut store = SessionStore::new();
    let started = store.start_pors(source, UiBudget::default());
    assert!(started.contains("\"ok\":true"), "{started}");
    assert!(started.contains("\"session\":1"));
    assert!(started.contains("\"content\":\"Contador (): 0\""));

    let clicked = store.dispatch(1, "clique", "inc", "");
    assert!(clicked.contains("\"ok\":true"), "{clicked}");
    assert!(clicked.contains("\"step\":1"));
    assert!(clicked.contains("\"content\":\"Contador (): 1\""));

    let typed = store.dispatch(1, "entrada_alterada", "nome", "Grace");
    assert!(typed.contains("\"ok\":true"), "{typed}");
    assert!(typed.contains("\"step\":2"));
    assert!(typed.contains("\"content\":\"Contador (Grace): 1\""));
    assert!(typed.contains("\"value\":\"Grace\""));

    assert!(store.close(1));
}

#[test]
fn executes_pors_core_libraries_in_zero_dep_wasm_facade() {
    let source = r#"
funcao dobro_parse(texto: cadeia) -> Resultado<inteiro, cadeia> {
    seja n: inteiro = Tipos.para_inteiro(Texto.aparar(texto))?;
    retorne Resultado::Ok(Matematica.maior_inteiro(n * 2, 10));
}

funcao inicio() {
    escolha dobro_parse(" 21 ") {
        caso Resultado::Ok(v) => { escreva(Texto.preencher_a_esquerda('0', 4, Tipos.inteiro_para_cadeia(v))); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let json = run_pors_json(source, "", RunBudget::default());
    assert_eq!(json, "{\"ok\":true,\"stdout\":\"0042\",\"diagnostics\":[]}");
}

#[test]
fn executes_pors_util_and_calendario_in_zero_dep_wasm_facade() {
    let source = r#"
funcao inicio() {
    Util.definir_semente(7);
    Util.aguarde(250);
    escolha Calendario.dias_no_mes(2024, Calendario.MES_FEVEREIRO) {
        caso Resultado::Ok(dias) => { escreva(dias, "|", Calendario.eh_bissexto(2024), "|", Util.tempo_decorrido() >= 0); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let json = run_pors_json(source, "", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"29|verdadeiro|verdadeiro\",\"diagnostics\":[]}"
    );
}
