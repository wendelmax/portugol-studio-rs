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
fn keeps_loop_shadowed_bindings_distinct_and_rejects_break_continue() {
    let source = "funcao inicio() { seja valor: inteiro = 1; enquanto (falso) { seja valor: inteiro = 2; escreva(valor); } escreva(valor); }";
    let program = lower(source).expect("sombreamento dentro do loop deve ser aceito");
    assert_eq!(program.bindings.len(), 2);
    assert_ne!(program.bindings[0].id, program.bindings[1].id);
    for keyword in ["pare", "continue"] {
        let source = format!("funcao inicio() {{ enquanto (verdadeiro) {{ {keyword}; }} }}");
        assert!(lower(&source).is_err(), "{keyword} ainda não é suportado");
    }
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
