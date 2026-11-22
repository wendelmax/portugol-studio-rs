use portugol_hir::{AssignOp, ExprKind, Mutability, StmtKind, Type};
use portugol_rs_frontend::lower;

const SOURCE: &str =
    "funcao inicio() { seja mutavel idade: inteiro = 16; idade += 1; escreva(idade); }";

#[test]
fn parses_the_initial_portugol_rs_program_into_typed_hir() {
    let program = lower(SOURCE).expect("o exemplo inicial deve ser válido");
    assert_eq!(program.bindings.len(), 1);
    assert_eq!(program.bindings[0].source_name, "idade");
    assert_eq!(program.bindings[0].ty, Type::Integer);
    assert_eq!(program.bindings[0].mutability, Mutability::Mutable);
    assert_eq!(program.body.len(), 3);
    assert!(matches!(program.body[0].kind, StmtKind::Declare { .. }));
    assert!(matches!(program.body[1].kind, StmtKind::Assign { .. }));
    let StmtKind::Write(values) = &program.body[2].kind else {
        panic!("esperava escreva")
    };
    assert!(matches!(values[0].kind, ExprKind::Binding(id) if id == program.bindings[0].id));
}

#[test]
fn malformed_syntax_returns_a_localized_diagnostic() {
    let source = "funcao inicio() { seja idade: inteiro = 16 }";
    let diagnostics = lower(source).unwrap_err();
    assert!(!diagnostics.is_empty());
    assert!(
        diagnostics
            .iter()
            .any(|d| d.span.start as usize <= source.len())
    );
}

#[test]
fn keeps_utf8_byte_spans_for_unknown_names() {
    let source = "funcao inicio() { escreva(\"ação\"); escreva(inexistente); }";
    let diagnostics = lower(source).unwrap_err();
    let error = diagnostics
        .iter()
        .find(|d| d.code == "pors.unknown-name")
        .unwrap();
    assert_eq!(
        &source[error.span.start as usize..error.span.end as usize],
        "inexistente"
    );
}

#[test]
fn preserves_operator_precedence_in_hir() {
    let source = "funcao inicio() { escreva(1 + 2 * 3); }";
    let program = lower(source).unwrap();
    let StmtKind::Write(values) = &program.body[0].kind else {
        panic!("esperava escreva")
    };
    let ExprKind::Binary {
        op: portugol_hir::BinaryOp::Add,
        right,
        ..
    } = &values[0].kind
    else {
        panic!("esperava soma no nível externo")
    };
    assert!(matches!(
        right.kind,
        ExprKind::Binary {
            op: portugol_hir::BinaryOp::Multiply,
            ..
        }
    ));
}

#[test]
fn reports_unclosed_literals_and_comments_without_panicking() {
    for source in [
        "funcao inicio() { escreva(\"sem fim); }",
        "funcao inicio() { /* sem fim",
    ] {
        let diagnostics = lower(source).unwrap_err();
        assert!(!diagnostics.is_empty(), "{source}");
    }
}

#[test]
fn parses_while_loop_with_typed_condition_and_mutable_binding() {
    let program = lower(
        "funcao inicio() { seja mutavel i: inteiro = 0; enquanto (i < 3) { i += 1; } escreva(i); }",
    )
    .expect("enquanto deve gerar HIR tipada");
    let StmtKind::While { condition, body } = &program.body[1].kind else {
        panic!("esperava enquanto");
    };
    assert_eq!(condition.ty, Type::Boolean);
    assert!(matches!(body[0].kind, StmtKind::Assign { .. }));
}

#[test]
fn parses_counted_loop_with_typed_initializer_condition_step_and_body() {
    let program = lower(
        "funcao inicio() { para (seja mutavel i: inteiro = 0; i < 3; i += 1) { escreva(i); } }",
    )
    .expect("para deve gerar HIR tipada");
    let StmtKind::For {
        initializer,
        condition,
        step,
        body,
    } = &program.body[0].kind
    else {
        panic!("esperava para");
    };
    assert!(
        matches!(initializer.kind, StmtKind::Declare { binding, .. } if binding == program.bindings[0].id)
    );
    assert_eq!(condition.ty, Type::Boolean);
    assert!(
        matches!(step.kind, StmtKind::Assign { binding, op: AssignOp::Add, .. } if binding == program.bindings[0].id)
    );
    assert!(matches!(body[0].kind, StmtKind::Write(_)));
}
