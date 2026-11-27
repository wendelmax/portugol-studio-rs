use portugol_rs_frontend::lower;

#[test]
fn rejects_assignment_to_immutable_binding_at_its_span() {
    let source = "funcao inicio() { seja idade: inteiro = 16; idade = 17; }";
    let errors = lower(source).unwrap_err();
    let error = errors
        .iter()
        .find(|d| d.code == "pors.immutable-assignment")
        .unwrap();
    assert_eq!(
        &source[error.span.start as usize..error.span.end as usize],
        "idade"
    );
}

#[test]
fn rejects_field_assignment_through_an_immutable_record_binding() {
    let source = "registro P { x: inteiro } funcao inicio() { seja p: P = P { x: 1 }; p.x = 2; }";
    let errors = lower(source).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|error| error.code == "pors.immutable-assignment")
    );
}

#[test]
fn rejects_non_exhaustive_and_duplicate_enum_cases() {
    let non_exhaustive = "enum Evento { Iniciar, Parar } funcao inicio() { seja evento: Evento = Evento::Iniciar; escolha evento { caso Evento::Iniciar => {} } }";
    assert!(
        lower(non_exhaustive)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.non-exhaustive-choice")
    );

    let duplicate = "enum Evento { Iniciar, Parar } funcao inicio() { seja evento: Evento = Evento::Iniciar; escolha evento { caso Evento::Iniciar => {} caso Evento::Iniciar => {} caso Evento::Parar => {} } }";
    assert!(
        lower(duplicate)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.duplicate-case")
    );
}

#[test]
fn rejects_enum_constructor_and_case_payload_mismatches() {
    let constructor_arity = "enum Evento { Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir; }";
    assert!(
        lower(constructor_arity)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.enum-payload-arity")
    );

    let constructor_type = "enum Evento { Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir(\"x\"); }";
    assert!(
        lower(constructor_type)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.enum-payload-type")
    );

    let case_arity = "enum Evento { Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir(1); escolha evento { caso Evento::Definir => {} } }";
    assert!(
        lower(case_arity)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.case-payload-arity")
    );
}

#[test]
fn rejects_incomplete_duplicate_and_mistyped_record_construction() {
    let incomplete =
        "registro P { x: inteiro, y: inteiro } funcao inicio() { seja p: P = P { x: 1 }; }";
    assert!(
        lower(incomplete)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.missing-field")
    );

    let duplicate = "registro P { x: inteiro } funcao inicio() { seja p: P = P { x: 1, x: 2 }; }";
    assert!(
        lower(duplicate)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.field-redeclared")
    );

    let mismatch = "registro P { x: inteiro } funcao inicio() { seja p: P = P { x: verdadeiro }; }";
    assert!(
        lower(mismatch)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.field-type-mismatch")
    );
}

#[test]
fn rejects_unknown_name_and_type_mismatch() {
    let unknown = "funcao inicio() { escreva(faltante); }";
    assert!(
        lower(unknown)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.unknown-name")
    );

    let mismatch = "funcao inicio() { seja mutavel idade: inteiro = 16; idade = verdadeiro; }";
    assert!(
        lower(mismatch)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.type-mismatch")
    );
}

#[test]
fn rejects_wrong_function_arity_and_argument_type() {
    let arity = "funcao dobro(valor: inteiro) -> inteiro { retorne valor * 2; } funcao inicio() { escreva(dobro()); }";
    assert!(
        lower(arity)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.call-arity")
    );

    let argument_type = "funcao dobro(valor: inteiro) -> inteiro { retorne valor * 2; } funcao inicio() { escreva(dobro(verdadeiro)); }";
    assert!(
        lower(argument_type)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.call-type-mismatch")
    );
}

#[test]
fn rejects_non_void_functions_without_a_return_on_every_path() {
    let source = "funcao escolher(condicao: logico) -> inteiro { se (condicao) { retorne 1; } } funcao inicio() {}";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.missing-return")
    );
}

#[test]
fn rejects_using_void_function_as_a_value() {
    let source = "funcao mostrar() { retorne; } funcao inicio() { escreva(mostrar()); }";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.void-value")
    );
}

#[test]
fn rejects_duplicate_and_unknown_functions() {
    let duplicate = "funcao ajuda() {} funcao ajuda() {} funcao inicio() {}";
    assert!(
        lower(duplicate)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.function-redeclared")
    );
    let unknown = "funcao inicio() { ausente(); }";
    assert!(
        lower(unknown)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.unknown-function")
    );
}

#[test]
fn rejects_return_values_that_disagree_with_the_function_signature() {
    for source in [
        "funcao valor() -> inteiro { retorne; } funcao inicio() {}",
        "funcao vazio() { retorne 1; } funcao inicio() {}",
        "funcao valor() -> inteiro { retorne verdadeiro; } funcao inicio() {}",
    ] {
        assert!(lower(source).is_err(), "{source}");
    }
}

#[test]
fn rejects_heterogeneous_vector_elements_and_non_integer_indexes() {
    let heterogeneous = "funcao inicio() { seja valores: vetor<inteiro> = [1, verdadeiro]; }";
    assert!(
        lower(heterogeneous)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.vector-element-type")
    );
    let non_integer =
        "funcao inicio() { seja valores: vetor<inteiro> = [1, 2]; escreva(valores[0.5]); }";
    assert!(
        lower(non_integer)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.index-type")
    );
}

#[test]
fn rejects_mutating_an_immutable_vector() {
    let source = "funcao inicio() { seja valores: vetor<inteiro> = [1, 2]; valores[0] = 3; }";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.immutable-assignment")
    );
}

#[test]
fn rejects_tamanho_for_non_vector_values() {
    let source = "funcao inicio() { escreva(tamanho(1)); }";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.length-type")
    );
}

#[test]
fn resolves_shadowed_names_to_distinct_bindings() {
    let source = "funcao inicio() { seja valor: inteiro = 1; se (verdadeiro) { seja valor: inteiro = 2; escreva(valor); } escreva(valor); }";
    let program = lower(source).expect("sombreamento em bloco é permitido");
    assert_eq!(program.bindings.len(), 2);
    assert_ne!(program.bindings[0].id, program.bindings[1].id);
    let portugol_hir::StmtKind::If { then_branch, .. } = &program.body[1].kind else {
        panic!("esperava condicional")
    };
    let portugol_hir::StmtKind::Write(inner) = &then_branch[1].kind else {
        panic!("esperava escrita interna")
    };
    assert!(
        matches!(inner[0].kind, portugol_hir::ExprKind::Binding(id) if id == program.bindings[1].id)
    );
    let portugol_hir::StmtKind::Write(outer) = &program.body[2].kind else {
        panic!("esperava escrita externa")
    };
    assert!(
        matches!(outer[0].kind, portugol_hir::ExprKind::Binding(id) if id == program.bindings[0].id)
    );
}

#[test]
fn all_declarations_require_an_initializer() {
    let source = "funcao inicio() { seja mutavel idade: inteiro; }";
    assert!(!lower(source).is_ok());
}

#[test]
fn rejects_reserved_words_as_variable_names() {
    let source = "funcao inicio() { seja mutavel seja: inteiro = 1; }";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.reserved-name")
    );
    for keyword in ["pare", "continue"] {
        let source = format!("funcao inicio() {{ seja mutavel {keyword}: inteiro = 1; }}");
        assert!(
            lower(&source)
                .unwrap_err()
                .iter()
                .any(|d| d.code == "pors.reserved-name"),
            "{keyword} não deve ser usado como identificador"
        );
    }
}

#[test]
fn enforces_boolean_conditions_and_loop_scope() {
    let non_boolean = "funcao inicio() { enquanto (1) {} }";
    assert!(
        lower(non_boolean)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.condition-type")
    );

    let escaped = "funcao inicio() { enquanto (falso) { seja item: inteiro = 1; } escreva(item); }";
    assert!(
        lower(escaped)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.unknown-name")
    );

    let immutable =
        "funcao inicio() { enquanto (verdadeiro) { seja item: inteiro = 1; item = 2; } }";
    assert!(
        lower(immutable)
            .unwrap_err()
            .iter()
            .any(|d| d.code == "pors.immutable-assignment")
    );
}

#[test]
fn keeps_loop_shadowed_bindings_distinct_and_accepts_loop_control() {
    let source = "funcao inicio() { seja valor: inteiro = 1; enquanto (falso) { seja valor: inteiro = 2; escreva(valor); } escreva(valor); }";
    let program = lower(source).expect("sombreamento dentro do loop deve ser aceito");
    assert_eq!(program.bindings.len(), 2);
    assert_ne!(program.bindings[0].id, program.bindings[1].id);
    let source =
        "funcao inicio() { enquanto (verdadeiro) { se (verdadeiro) { pare; } continue; } }";
    assert!(
        lower(source).is_ok(),
        "controles são válidos dentro do laço"
    );
}

#[test]
fn counted_loop_control_does_not_escape_its_scope() {
    let source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { escreva(i); } escreva(i); }";
    assert!(
        lower(source)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.unknown-name")
    );
}

#[test]
fn counted_loop_rejects_non_boolean_condition_and_non_numeric_control() {
    let condition = "funcao inicio() { para (seja mutavel i: inteiro = 0; i; i += 1) {} }";
    assert!(
        lower(condition)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.condition-type")
    );

    let control = "funcao inicio() { para (seja mutavel ativo: logico = verdadeiro; ativo; ativo = falso) {} }";
    assert!(
        lower(control)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.for-control-type")
    );
}

#[test]
fn counted_loop_requires_the_seja_keyword_before_its_control_declaration() {
    let source = "funcao inicio() { para (banana mutavel i: inteiro = 0; i < 2; i += 1) {} }";
    let errors = lower(source).unwrap_err();
    assert!(errors.iter().any(|diagnostic| {
        diagnostic.code == "pors.expected-keyword" && diagnostic.message.contains("Esperado `seja`")
    }));
}

#[test]
fn counted_loop_step_must_compound_assign_its_mutable_control() {
    let wrong_binding = "funcao inicio() { seja mutavel j: inteiro = 0; para (seja mutavel i: inteiro = 0; i < 2; j += 1) {} }";
    assert!(
        lower(wrong_binding)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.for-step-binding")
    );

    let plain_assignment =
        "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i = i + 1) {} }";
    assert!(
        lower(plain_assignment)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.for-step-operator")
    );

    let immutable = "funcao inicio() { para (seja i: inteiro = 0; i < 2; i += 1) {} }";
    assert!(
        lower(immutable)
            .unwrap_err()
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.immutable-assignment")
    );
}

#[test]
fn nested_counted_loops_keep_shadowed_control_bindings_separate() {
    let source = "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { para (seja mutavel i: inteiro = 0; i < 2; i += 1) { escreva(i); } escreva(i); } }";
    let program = lower(source).expect("loops internos podem sombrear controles externos");
    assert_eq!(program.bindings.len(), 2);
    assert_ne!(program.bindings[0].id, program.bindings[1].id);
}

#[test]
fn rejects_read_expression_in_an_integer_context_at_its_span() {
    let source = "funcao inicio() { seja idade: inteiro = leia(); }";
    let errors = lower(source).unwrap_err();
    let error = errors.iter().find(|d| d.code == "pors.type-mismatch").unwrap();
    assert_eq!(&source[error.span.start as usize..error.span.end as usize], "leia()");
}

#[test]
fn rejects_invalid_interpolation_content_at_the_inner_span() {
    let source = "funcao inicio() { escreva(\"{1 +}\"); }";
    let errors = lower(source).unwrap_err();
    let error = errors.iter().find(|d| d.code == "pors.interpolation-invalid").unwrap();
    assert_eq!(&source[error.span.start as usize..error.span.end as usize], "1 +");
}

#[test]
fn rejects_trailing_expression_in_a_function_without_return_type() {
    let source = "funcao inicio() { 1 + 1 }";
    let errors = lower(source).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.implicit-return-without-type"));
}
