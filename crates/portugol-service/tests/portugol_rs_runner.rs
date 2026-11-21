use portugol_service::run_portugol_rs;

#[test]
fn compiles_and_runs_supported_portugol_rs_with_real_rustc() {
    let result = run_portugol_rs(
        "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }",
    )
    .unwrap();

    assert_eq!(result.stdout, "17");
    assert!(result.stderr.is_empty());
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn rejects_portugol_rs_constructs_outside_the_supported_subset() {
    let error = run_portugol_rs("funcao inicio() { enquanto (verdadeiro) { } }").unwrap_err();

    assert!(error.contains("enquanto"));
}

#[test]
fn returns_the_real_exit_code_and_stderr_from_a_rust_runtime_failure() {
    let result =
        run_portugol_rs("funcao inicio() { seja divisor: inteiro = 0; escreva(1 / divisor); }")
            .unwrap();

    assert_ne!(result.exit_code, Some(0));
    assert!(result.stderr.contains("panicked"));
}
