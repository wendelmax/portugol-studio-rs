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
