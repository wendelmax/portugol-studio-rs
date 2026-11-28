use ferro_service::{
    FerroRustRunPhase, FerroRustRunResult, run_ferro_rs, run_ferro_rs_with_cancel,
    run_ferro_rs_with_io,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn compiles_and_runs_supported_ferro_rs_with_real_rustc() {
    let result = run_ferro_rs(
        "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }",
    )
    .unwrap();

    match result {
        ferro_service::FerroRustRunResult::Executed {
            stdout,
            stderr,
            exit_code,
        } => {
            assert_eq!(stdout, "17");
            assert!(stderr.is_empty());
            assert_eq!(exit_code, Some(0));
        }
        other => panic!("esperava execução bem-sucedida, recebido {other:?}"),
    }
}

#[test]
fn compiles_and_runs_ferro_records_with_typed_fields() {
    let source = "registro Pessoa { idade: inteiro, nome: cadeia } funcao inicio() { seja pessoa: Pessoa = Pessoa { idade: 16, nome: \"Ana\" }; escreva(pessoa.idade); escreva(pessoa.nome); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "16Ana"),
        "{result:?}"
    );
}

#[test]
fn compiles_and_runs_ferro_record_field_updates_with_real_rustc() {
    let source = "registro Estado { contador: inteiro } funcao inicio() { seja mutavel estado: Estado = Estado { contador: 2 }; estado.contador += 3; escreva(estado.contador); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "5"),
        "{result:?}"
    );
}

#[test]
fn compiles_and_runs_nested_record_field_updates_with_real_rustc() {
    let source = "registro Conta { mensagem: cadeia, total: inteiro } registro Tela { dados: Conta } funcao inicio() { seja antes: Tela = Tela { dados: Conta { mensagem: \"\", total: 1 } }; seja mutavel t: Tela = antes; t.dados.mensagem = \"ok\"; t.dados.total += 2; escreva(antes.dados.total, t.dados.mensagem, t.dados.total); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "1ok3"),
        "{result:?}"
    );
}

#[test]
fn compiles_and_runs_vectors_of_ferro_records_with_real_rustc() {
    let source = "registro Pessoa { idade: inteiro } funcao inicio() { seja pessoas: vetor<Pessoa> = [Pessoa { idade: 16 }, Pessoa { idade: 17 }]; escreva(pessoas[0].idade); escreva(pessoas[1].idade); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "1617"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_keeps_copied_records_independent_and_charges_string_copies() {
    let source = "registro Pessoa { nome: cadeia } funcao inicio() { seja mutavel original: Pessoa = Pessoa { nome: \"Ana\" }; seja mutavel copia: Pessoa = original; copia.nome = \"Bia\"; escreva(original.nome); escreva(copia.nome); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "AnaBia"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_passes_and_returns_records_by_value() {
    let source = "registro Estado { contador: inteiro } funcao incrementar(mutavel estado: Estado) -> Estado { estado.contador += 1; retorne estado; } funcao inicio() { seja original: Estado = Estado { contador: 4 }; seja proximo: Estado = incrementar(original); escreva(original.contador); escreva(proximo.contador); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "45"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_matches_all_enum_variants_and_binds_payloads() {
    let source = "enum Evento { Incrementar, Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir(7); escolha evento { caso Evento::Incrementar => { escreva(0); } caso Evento::Definir(valor) => { escreva(valor); } } }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "7"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_copies_constructor_fields_and_enum_payloads_from_bindings() {
    let source = "registro Pessoa { nome: cadeia } enum Mensagem { Texto(cadeia) } funcao inicio() { seja nome: cadeia = \"Ana\"; seja pessoa: Pessoa = Pessoa { nome: nome }; escreva(nome); escreva(pessoa.nome); seja texto: cadeia = \"oi\"; seja mensagem: Mensagem = Mensagem::Texto(texto); escreva(texto); escolha mensagem { caso Mensagem::Texto(conteudo) => { escreva(conteudo); } } }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "AnaAnaoioi"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_copies_record_payloads_without_consuming_the_source() {
    let source = "registro Estado { valor: inteiro } enum Envelope { Dados(Estado) } funcao inicio() { seja mutavel original: Estado = Estado { valor: 1 }; seja envelope: Envelope = Envelope::Dados(original); original.valor = 2; escolha envelope { caso Envelope::Dados(estado) => { escreva(estado.valor); } } escreva(original.valor); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "12"),
        "{result:?}"
    );
}

#[test]
fn compiles_and_runs_ferro_helper_functions_with_real_rustc() {
    let result = run_ferro_rs("funcao dobro(valor: inteiro) -> inteiro { retorne valor * 2; } funcao inicio() { escreva(dobro(21)); }").unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "42")
    );
}

#[test]
fn compiles_and_runs_ferro_vectors_with_value_semantics() {
    let source = "funcao alterar(mutavel valores: vetor<inteiro>) { valores[0] = 9; } funcao criar() -> vetor<inteiro> { seja mutavel valores: vetor<inteiro> = [3]; retorne valores; } funcao inicio() { seja mutavel original: vetor<inteiro> = [1, 2]; alterar(original); seja mutavel copia: vetor<inteiro> = original; copia[1] = 8; seja retorno: vetor<inteiro> = criar(); escreva(original[0]); escreva(copia[1]); escreva(retorno[0]); escreva(tamanho(copia)); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "1832"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_reports_vector_index_bounds_in_portuguese() {
    let result = run_ferro_rs(
        "funcao inicio() { seja valores: vetor<inteiro> = [1]; seja indice: inteiro = -1; escreva(valores[indice]); }",
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. } if code != 0 && stderr.contains("índice de vetor fora do intervalo")),
        "{result:?}"
    );
}

#[test]
fn generated_rust_copies_vectors_of_strings_without_changing_values() {
    let source = "funcao inicio() { seja original: vetor<cadeia> = [\"olá\"]; seja copia: vetor<cadeia> = original; escreva(original[0]); escreva(copia[0]); }";
    let result = run_ferro_rs(source).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "oláolá"),
        "{result:?}"
    );
}

#[test]
fn pauses_at_leia_requests_typed_input_and_resumes_the_program() {
    let cancel = AtomicBool::new(false);
    let mut streamed = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja mutavel idade: inteiro = 0; escreva(\"idade? \"); leia(idade); escreva(idade); }",
        &cancel,
        |expected| {
            assert_eq!(expected, "inteiro");
            Ok("42".to_owned())
        },
        |text| streamed.push(text.to_owned()),
    )
    .unwrap();

    assert_eq!(streamed.concat(), "idade? ");
    assert!(
        matches!(result, FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "42")
    );
}

#[test]
fn generated_rust_requests_a_string_when_read_is_an_expression() {
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { escreva(leia()); }",
        &cancel,
        |expected| {
            requests.push(expected.to_owned());
            Ok("Ada".to_owned())
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["cadeia"]);
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "Ada"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_requests_typed_input_for_read_expression() {
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: inteiro = leia(); escreval(n); }",
        &cancel,
        |expected| {
            requests.push(expected.to_owned());
            Ok("36".to_owned())
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["inteiro"]);
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "36\n"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_reports_invalid_typed_expression_input() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: inteiro = leia(); escreval(n); }",
        &cancel,
        |_| Ok("abc".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(
            result,
            FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. }
                if code != 0 && stderr.to_lowercase().contains("inteiro")
        ),
        "{result:?}"
    );
}

#[test]
fn generated_rust_declares_read_helper_for_leia_in_para_header() {
    let cancel = AtomicBool::new(false);
    let mut inputs = ["abc", "a"].into_iter();
    let mut streamed = Vec::new();
    let result = run_ferro_rs_with_io(
        "funcao inicio() { para (seja mutavel i: inteiro = Texto.numero_caracteres(leia()); i < 4; i += 1) { escreva(i); } para (seja mutavel j: inteiro = 0; j < 1; j += Texto.numero_caracteres(leia())) { escreva(j); } }",
        &cancel,
        |expected| {
            assert_eq!(expected, "cadeia");
            Ok(inputs.next().expect("duas entradas").to_owned())
        },
        |text| streamed.push(text.to_owned()),
    )
    .unwrap();
    assert!(
        matches!(
            result,
            FerroRustRunResult::Executed {
                exit_code: Some(0),
                ..
            }
        ),
        "{result:?}"
    );
    assert_eq!(streamed.concat(), "30");
    assert_eq!(inputs.len(), 0);
}

#[test]
fn parses_string_real_boolean_and_character_inputs() {
    for (ty, value) in [
        ("cadeia", "Olá, Rust"),
        ("real", "3.5"),
        ("logico", "verdadeiro"),
        ("caracter", "ç"),
    ] {
        let source = format!(
            "funcao inicio() {{ seja mutavel valor: {ty} = {}; leia(valor); escreva(valor); }}",
            match ty {
                "cadeia" => "\"\"".to_owned(),
                "real" => "0.0".to_owned(),
                "logico" => "falso".to_owned(),
                "caracter" => "'a'".to_owned(),
                _ => unreachable!(),
            }
        );
        let cancel = AtomicBool::new(false);
        let result =
            run_ferro_rs_with_io(&source, &cancel, |_| Ok(value.to_owned()), |_| {}).unwrap();
        assert!(matches!(
            result,
            FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == value
        ));
    }
}

#[test]
fn generated_rust_trims_integer_input_before_conversion() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: inteiro = leia(); escreval(n + 1); }",
        &cancel,
        |_| Ok(" 36".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "37\n"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_trims_boolean_input_before_conversion() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: logico = leia(); escreval(n); }",
        &cancel,
        |_| Ok(" verdadeiro".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "verdadeiro\n"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_trims_character_input_before_conversion() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: caracter = leia(); escreval(n); }",
        &cancel,
        |_| Ok(" A ".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "A\n"),
        "{result:?}"
    );
}

#[test]
fn generated_rust_rejects_multi_character_input_for_a_character_read() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: caracter = leia(); escreval(n); }",
        &cancel,
        |_| Ok("AB".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. }
            if code != 0 && stderr.to_lowercase().contains("caracter")),
        "{result:?}"
    );
}

#[test]
fn generated_rust_rejects_unknown_boolean_input_with_surrounding_spaces() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja n: logico = leia(); escreval(n); }",
        &cancel,
        |_| Ok(" talvez ".to_owned()),
        |_| {},
    )
    .unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. }
            if code != 0 && stderr.to_lowercase().contains("logico")),
        "{result:?}"
    );
}

#[test]
fn generated_rust_and_vm_agree_on_trimmed_scalar_reads() {
    for (source, input, expected) in [
        (
            "funcao inicio() { seja n: inteiro = leia(); escreval(n + 1); }",
            " 36",
            "37\n",
        ),
        (
            "funcao inicio() { seja n: logico = leia(); escreval(n); }",
            " verdadeiro",
            "verdadeiro\n",
        ),
        (
            "funcao inicio() { seja n: caracter = leia(); escreval(n); }",
            " A ",
            "A\n",
        ),
    ] {
        let program = ferro_frontend::lower(source).unwrap();
        let module = ferro_bytecode::compile_ferro(&program).unwrap();
        let mut io = ferro_vm::BufferIo::new(input);
        ferro_vm::run(&module, &mut io).expect("VM deve aceitar a entrada");

        let cancel = AtomicBool::new(false);
        let result =
            run_ferro_rs_with_io(source, &cancel, |_| Ok(input.to_owned()), |_| {}).unwrap();
        assert!(
            matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == expected),
            "{result:?}"
        );
        assert_eq!(io.output, expected);
    }
}

#[test]
fn generated_rust_and_vm_agree_on_multi_character_caracter_rejection() {
    let source = "funcao inicio() { seja n: caracter = leia(); escreval(n); }";
    let program = ferro_frontend::lower(source).unwrap();
    let module = ferro_bytecode::compile_ferro(&program).unwrap();
    let mut io = ferro_vm::BufferIo::new("AB");
    let vm_error = ferro_vm::run(&module, &mut io).expect_err("VM deve rejeitar a entrada");
    assert!(format!("{vm_error:?}").contains("caracter"), "{vm_error:?}");

    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(source, &cancel, |_| Ok("AB".to_owned()), |_| {}).unwrap();
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stderr, exit_code: Some(code), .. }
            if code != 0 && stderr.to_lowercase().contains("caracter")),
        "{result:?}"
    );
}

#[test]
fn cancellation_interrupts_a_run_waiting_for_input() {
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = Arc::clone(&cancel);
    let (requested, request) = std::sync::mpsc::channel();
    let worker = std::thread::spawn(move || {
        run_ferro_rs_with_io(
            "funcao inicio() { seja mutavel valor: inteiro = 0; leia(valor); }",
            &worker_cancel,
            |expected| {
                requested.send(expected.to_owned()).unwrap();
                while !worker_cancel.load(Ordering::Acquire) {
                    std::thread::sleep(std::time::Duration::from_millis(10));
                }
                Ok(String::new())
            },
            |_| {},
        )
    });
    assert_eq!(
        request
            .recv_timeout(std::time::Duration::from_secs(5))
            .unwrap(),
        "inteiro"
    );
    cancel.store(true, Ordering::Release);
    let result = worker.join().unwrap().unwrap();
    assert!(matches!(
        result,
        FerroRustRunResult::Cancelled {
            phase: FerroRustRunPhase::Runtime,
            ..
        }
    ));
}

#[test]
fn cancellation_requested_by_input_handler_wins_before_forwarding_the_value() {
    let cancel = AtomicBool::new(false);
    let result = run_ferro_rs_with_io(
        "funcao inicio() { seja mutavel valor: inteiro = 0; leia(valor); escreva(valor); }",
        &cancel,
        |_| {
            cancel.store(true, Ordering::Release);
            Ok("7".to_owned())
        },
        |_| {},
    )
    .unwrap();

    assert!(matches!(
        result,
        FerroRustRunResult::Cancelled {
            phase: FerroRustRunPhase::Runtime,
            stdout,
            ..
        } if stdout.is_empty()
    ));
}

#[test]
fn rejects_an_input_line_larger_than_one_mib() {
    let cancel = AtomicBool::new(false);
    let oversized = "x".repeat(1_048_577);
    let error = run_ferro_rs_with_io(
        "funcao inicio() { seja mutavel valor: cadeia = \"\"; leia(valor); }",
        &cancel,
        |_| Ok(oversized.clone()),
        |_| {},
    )
    .unwrap_err();
    assert!(error.contains("1 MiB por linha"));
}

#[test]
fn reports_typed_cancellation_during_ferro_rs_compilation() {
    let cancel = AtomicBool::new(true);
    let result = run_ferro_rs_with_cancel("funcao inicio() { escreva(\"ready\"); }", &cancel)
        .expect("cancelamento deve ser resultado tipado");

    assert!(matches!(
        result,
        FerroRustRunResult::Cancelled {
            phase: FerroRustRunPhase::Compilation,
            ..
        }
    ));
}

#[test]
fn cancels_a_ferro_rs_job_after_starting_its_real_compiler_or_runtime() {
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = Arc::clone(&cancel);
    let worker = std::thread::spawn(move || {
        run_ferro_rs_with_cancel(
            "funcao inicio() { enquanto (verdadeiro) { } }",
            &worker_cancel,
        )
    });
    let started = std::time::Instant::now();
    std::thread::sleep(std::time::Duration::from_secs(1));
    cancel.store(true, Ordering::Release);

    let result = worker
        .join()
        .expect("worker deve concluir")
        .expect("cancelamento deve ser retorno tipado");
    assert!(matches!(result, FerroRustRunResult::Cancelled { .. }));
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

#[test]
fn tracks_string_literals_clones_and_concatenation_without_changing_values() {
    let result = run_ferro_rs(
        "funcao inicio() { seja origem: cadeia = \"Olá\"; seja resultado: cadeia = origem + \" mundo\"; escreva(resultado); }",
    )
    .unwrap();
    assert!(
        matches!(result, ferro_service::FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "Olá mundo")
    );
}

#[test]
fn counted_loop_step_runs_after_body_shadowing_scope_ends() {
    let source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { seja mutavel i: inteiro = 9; escreva(i); } }";
    let result = run_ferro_rs(source).unwrap();
    assert!(matches!(
        result,
        ferro_service::FerroRustRunResult::Executed {
            stdout,
            exit_code: Some(0),
            ..
        } if stdout == "99"
    ));
}

#[test]
fn counted_loop_division_and_remainder_by_zero_preserve_runtime_failure() {
    for step in ["i /= 0", "i %= 0"] {
        let source =
            format!("funcao inicio() {{ para (seja mutavel i: inteiro = 1; i < 2; {step}) {{}} }}");
        let result = run_ferro_rs(&source).unwrap();
        assert!(
            matches!(
                result,
                ferro_service::FerroRustRunResult::Executed {
                    stderr,
                    exit_code: Some(code),
                    ..
                } if code != 0 && stderr.contains("panicked")
            ),
            "{step} must retain the Rust runtime failure"
        );
    }
}

#[test]
fn charges_character_concatenation_to_the_string_budget() {
    let result =
        run_ferro_rs("funcao inicio() { seja texto: cadeia = 'a' + 'b'; escreva(texto); }")
            .unwrap();
    assert!(
        matches!(result, ferro_service::FerroRustRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "ab")
    );
}

#[test]
fn stops_an_infinite_ferro_rs_loop_with_a_typed_runtime_limit() {
    let result = run_ferro_rs("funcao inicio() { enquanto (verdadeiro) { } }").unwrap();
    assert!(matches!(
        result,
        ferro_service::FerroRustRunResult::RuntimeLimit {
            reason: ferro_service::FerroRustRuntimeLimitReason::Time,
            ..
        }
    ));
}

#[test]
fn caps_combined_runtime_output() {
    let result =
        run_ferro_rs("funcao inicio() { enquanto (verdadeiro) { escreva(123); } }").unwrap();
    match result {
        ferro_service::FerroRustRunResult::RuntimeLimit {
            stdout,
            stderr,
            reason: ferro_service::FerroRustRuntimeLimitReason::Output,
        } => {
            assert!(stdout.len() + stderr.len() <= 1_048_576);
            assert!(!stdout.is_empty());
        }
        other => panic!("esperava limite combinado de saída, recebido {other:?}"),
    }
}

#[test]
fn limits_cumulative_ferro_string_allocations() {
    let result = run_ferro_rs("funcao inicio() { seja mutavel texto: cadeia = \"x\"; enquanto (verdadeiro) { texto += texto; } }").unwrap();
    assert!(matches!(
        result,
        ferro_service::FerroRustRunResult::RuntimeLimit {
            reason: ferro_service::FerroRustRuntimeLimitReason::StringMemory,
            ..
        }
    ));
}

#[test]
fn returns_the_real_exit_code_and_stderr_from_a_rust_runtime_failure() {
    let result =
        run_ferro_rs("funcao inicio() { seja divisor: inteiro = 0; escreva(1 / divisor); }")
            .unwrap();

    match result {
        ferro_service::FerroRustRunResult::Executed {
            stderr, exit_code, ..
        } => {
            assert_ne!(exit_code, Some(0));
            assert!(stderr.contains("panicked"));
        }
        other => panic!("falha em runtime não deve ser erro de compilação: {other:?}"),
    }
}

#[test]
fn compiles_and_runs_canonical_syntax_evolution_program_with_real_rustc() {
    let source = include_str!("../../../tests/fixtures/ferro-syntax-evolution.fer");
    let cancel = AtomicBool::new(false);
    let mut requests = Vec::new();
    let result = run_ferro_rs_with_io(
        source,
        &cancel,
        |expected| {
            requests.push(expected.to_owned());
            Ok("Ada".to_owned())
        },
        |_| {},
    )
    .unwrap();
    assert_eq!(requests, ["cadeia"]);
    assert!(
        matches!(result, FerroRustRunResult::Executed { ref stdout, exit_code: Some(0), .. } if stdout == "Olá, Ada!\n"),
        "{result:?}"
    );
}
