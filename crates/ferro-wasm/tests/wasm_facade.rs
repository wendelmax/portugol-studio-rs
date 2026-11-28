use std::slice;

use ferro_wasm::{
    RunBudget, export_rust_ferro_json, ferro_alloc, ferro_dealloc, ferro_export_rust, ferro_format,
    ferro_result_len, ferro_result_ptr, ferro_run, ferro_validate, format_ferro_json,
    run_ferro_json, validate_ferro_json,
};

#[test]
fn validates_and_formats_ferro_sources_with_structured_json() {
    let valid = "funcao inicio() {\n    seja x: inteiro = 1;\n    escreva(x);\n}\n";
    assert_eq!(
        validate_ferro_json(valid),
        "{\"ok\":true,\"diagnostics\":[]}"
    );

    let formatted_already = format_ferro_json(valid);
    assert!(formatted_already.contains("\"ok\":true"));
    assert!(formatted_already.contains("\"changed\":false"));

    let compact = "funcao inicio(){escreva(\"olá 🦀\");seja x:inteiro=2+3;escreva(x);}";
    let formatted = format_ferro_json(compact);
    assert!(formatted.contains("\"ok\":true"));
    assert!(formatted.contains("\"changed\":true"));
    assert!(formatted.contains("seja x: inteiro = 2 + 3;"));

    let invalid = "funcao inicio() { escreva(desconhecido); }";
    let invalid_json = validate_ferro_json(invalid);
    assert!(invalid_json.contains("\"ok\":false"));
    assert!(invalid_json.contains("ferro.unknown-name"));
}

#[test]
fn executes_ferro_with_input_vectors_and_escapes_json_stdout() {
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
    let json = run_ferro_json(source, "Ada\n12\n", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"Aluno \\\"Ada\\\": 42\",\"diagnostics\":[]}"
    );
}

#[test]
fn enforces_step_string_memory_and_output_budgets_in_run_ferro() {
    let infinite_loop = "funcao inicio() { enquanto (verdadeiro) {} }";
    let step_err = run_ferro_json(
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
    let mem_err = run_ferro_json(
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
    let out_err = run_ferro_json(
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
fn exercises_exported_c_abi_memory_and_result_buffer() {
    let source = "funcao inicio() { escreva(20 + 22); }";
    let src_bytes = source.as_bytes();
    let ptr = ferro_alloc(src_bytes.len());
    assert!(!ptr.is_null());

    // SAFETY: `ptr` foi alocado com exatamente `src_bytes.len()` bytes.
    unsafe {
        std::ptr::copy_nonoverlapping(src_bytes.as_ptr(), ptr, src_bytes.len());
    }

    let validate_len = unsafe { ferro_validate(ptr, src_bytes.len()) };
    assert_eq!(validate_len, ferro_result_len());
    let validate_out = unsafe { slice::from_raw_parts(ferro_result_ptr(), validate_len) };
    assert_eq!(
        std::str::from_utf8(validate_out).unwrap(),
        "{\"ok\":true,\"diagnostics\":[]}"
    );

    let format_len = unsafe { ferro_format(ptr, src_bytes.len()) };
    let format_out = unsafe { slice::from_raw_parts(ferro_result_ptr(), format_len) };
    assert!(
        std::str::from_utf8(format_out)
            .unwrap()
            .contains("\"changed\":true")
    );

    let run_len = unsafe { ferro_run(ptr, src_bytes.len(), std::ptr::null(), 0, 0, 0, 0) };
    let run_out = unsafe { slice::from_raw_parts(ferro_result_ptr(), run_len) };
    assert_eq!(
        std::str::from_utf8(run_out).unwrap(),
        "{\"ok\":true,\"stdout\":\"42\",\"diagnostics\":[]}"
    );

    unsafe {
        ferro_dealloc(ptr, src_bytes.len());
    }
}

#[test]
fn executes_ferro_core_libraries_in_zero_dep_wasm_facade() {
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
    let json = run_ferro_json(source, "", RunBudget::default());
    assert_eq!(json, "{\"ok\":true,\"stdout\":\"0042\",\"diagnostics\":[]}");
}

#[test]
fn executes_ferro_util_and_calendario_in_zero_dep_wasm_facade() {
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
    let json = run_ferro_json(source, "", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"29|verdadeiro|verdadeiro\",\"diagnostics\":[]}"
    );
}

#[test]
fn executes_ferro_json_in_zero_dep_wasm_facade() {
    let source = r#"
funcao resumo() -> Resultado<cadeia, cadeia> {
    seja mutavel doc: cadeia = "{}";
    doc = Json.definir_cadeia(doc, "curso", "FERRO")?;
    doc = Json.definir_inteiro(doc, "versao", 2)?;
    doc = Json.definir_json(doc, "modulos", "[\"Texto\",\"Json\"]")?;
    seja curso: cadeia = Json.obter_cadeia(doc, "curso")?;
    seja versao: inteiro = Json.obter_inteiro(doc, "versao")?;
    seja mods: vetor<cadeia> = Json.obter_lista_cadeia(doc, "modulos")?;
    retorne Resultado::Ok(curso + "-" + Tipos.inteiro_para_cadeia(versao) + ":" + mods[1]);
}

funcao inicio() {
    escolha resumo() {
        caso Resultado::Ok(s) => { escreva(s, "|", Json.valido("[]")); }
        caso Resultado::Erro(e) => { escreva("ERR:", e); }
    }
}
"#;
    let json = run_ferro_json(source, "", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"FERRO-2:Json|verdadeiro\",\"diagnostics\":[]}"
    );
}

#[test]
fn returns_typed_error_for_ferro_internet_in_wasm_facade() {
    let source = r#"
funcao inicio() {
    Internet.definir_tempo_limite(1000);
    escolha Internet.obter_texto("https://example.com") {
        caso Resultado::Ok(_) => { escreva("OK"); }
        caso Resultado::Erro(_) => { escreva("WASM_ERR"); }
    }
}
"#;
    let json = run_ferro_json(source, "", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"WASM_ERR\",\"diagnostics\":[]}"
    );
}

#[test]
fn exports_ferro_to_rust_through_wasm_facade_and_ffi() {
    let source = "funcao inicio() { seja idade: inteiro = 18; escreva(idade); }";
    let json = export_rust_ferro_json(source);
    assert!(json.starts_with("{\"ok\":true,\"code\":\""), "{json}");
    assert!(json.contains("fn main()"), "{json}");
    assert!(json.contains("pt_idade: i32 = 18_i32"), "{json}");

    let invalid = export_rust_ferro_json("funcao inicio() { seja idade: inteiro = ; }");
    assert!(invalid.starts_with("{\"ok\":false,"), "{invalid}");
    assert!(invalid.contains("\"code\":\"diagnostics\""), "{invalid}");

    let bytes = source.as_bytes();
    let ptr = ferro_alloc(bytes.len());
    assert!(!ptr.is_null());
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
        let len = ferro_export_rust(ptr, bytes.len());
        assert_eq!(len, ferro_result_len());
        let out = std::str::from_utf8(slice::from_raw_parts(ferro_result_ptr(), len)).unwrap();
        assert!(out.contains("\"ok\":true"), "{out}");
        assert!(out.contains("fn main()"), "{out}");
        ferro_dealloc(ptr, bytes.len());
    }
}

#[test]
fn executes_canonical_syntax_evolution_program_through_wasm_facade() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let json = run_ferro_json(source, "Ada\n", RunBudget::default());
    assert_eq!(
        json,
        "{\"ok\":true,\"stdout\":\"Olá, Ada!\\n\",\"diagnostics\":[]}"
    );
}
