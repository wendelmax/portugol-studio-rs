use portugol_service::{
    PortugolRsRunPhase, PortugolRsRunResult, run_portugol_rs, run_portugol_rs_with_cancel,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn compiles_and_runs_supported_portugol_rs_with_real_rustc() {
    let result = run_portugol_rs(
        "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }",
    )
    .unwrap();

    match result {
        portugol_service::PortugolRsRunResult::Executed {
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
fn reports_typed_cancellation_during_portugol_rs_compilation() {
    let cancel = AtomicBool::new(true);
    let result = run_portugol_rs_with_cancel("funcao inicio() { escreva(\"ready\"); }", &cancel)
        .expect("cancelamento deve ser resultado tipado");

    assert!(matches!(
        result,
        PortugolRsRunResult::Cancelled {
            phase: PortugolRsRunPhase::Compilation,
            ..
        }
    ));
}

#[test]
fn cancels_a_portugol_rs_job_after_starting_its_real_compiler_or_runtime() {
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = Arc::clone(&cancel);
    let worker = std::thread::spawn(move || {
        run_portugol_rs_with_cancel(
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
    assert!(matches!(result, PortugolRsRunResult::Cancelled { .. }));
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

#[test]
fn tracks_string_literals_clones_and_concatenation_without_changing_values() {
    let result = run_portugol_rs(
        "funcao inicio() { seja origem: cadeia = \"Olá\"; seja resultado: cadeia = origem + \" mundo\"; escreva(resultado); }",
    )
    .unwrap();
    assert!(
        matches!(result, portugol_service::PortugolRsRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "Olá mundo")
    );
}

#[test]
fn counted_loop_step_runs_after_body_shadowing_scope_ends() {
    let source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { seja mutavel i: inteiro = 9; escreva(i); } }";
    let result = run_portugol_rs(source).unwrap();
    assert!(matches!(
        result,
        portugol_service::PortugolRsRunResult::Executed {
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
        let result = run_portugol_rs(&source).unwrap();
        assert!(
            matches!(
                result,
                portugol_service::PortugolRsRunResult::Executed {
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
        run_portugol_rs("funcao inicio() { seja texto: cadeia = 'a' + 'b'; escreva(texto); }")
            .unwrap();
    assert!(
        matches!(result, portugol_service::PortugolRsRunResult::Executed { stdout, exit_code: Some(0), .. } if stdout == "ab")
    );
}

#[test]
fn stops_an_infinite_portugol_rs_loop_with_a_typed_runtime_limit() {
    let result = run_portugol_rs("funcao inicio() { enquanto (verdadeiro) { } }").unwrap();
    assert!(matches!(
        result,
        portugol_service::PortugolRsRunResult::RuntimeLimit {
            reason: portugol_service::PortugolRsRuntimeLimitReason::Time,
            ..
        }
    ));
}

#[test]
fn caps_combined_runtime_output() {
    let result =
        run_portugol_rs("funcao inicio() { enquanto (verdadeiro) { escreva(123); } }").unwrap();
    match result {
        portugol_service::PortugolRsRunResult::RuntimeLimit {
            stdout,
            stderr,
            reason: portugol_service::PortugolRsRuntimeLimitReason::Output,
        } => {
            assert!(stdout.len() + stderr.len() <= 1_048_576);
            assert!(!stdout.is_empty());
        }
        other => panic!("esperava limite combinado de saída, recebido {other:?}"),
    }
}

#[test]
fn limits_cumulative_pors_string_allocations() {
    let result = run_portugol_rs("funcao inicio() { seja mutavel texto: cadeia = \"x\"; enquanto (verdadeiro) { texto += texto; } }").unwrap();
    assert!(matches!(
        result,
        portugol_service::PortugolRsRunResult::RuntimeLimit {
            reason: portugol_service::PortugolRsRuntimeLimitReason::StringMemory,
            ..
        }
    ));
}

#[test]
fn returns_the_real_exit_code_and_stderr_from_a_rust_runtime_failure() {
    let result =
        run_portugol_rs("funcao inicio() { seja divisor: inteiro = 0; escreva(1 / divisor); }")
            .unwrap();

    match result {
        portugol_service::PortugolRsRunResult::Executed {
            stderr, exit_code, ..
        } => {
            assert_ne!(exit_code, Some(0));
            assert!(stderr.contains("panicked"));
        }
        other => panic!("falha em runtime não deve ser erro de compilação: {other:?}"),
    }
}
