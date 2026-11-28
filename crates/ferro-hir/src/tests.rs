use super::*;

#[test]
fn records_binding_identity_mutability_and_source_spans() {
    let span = Span::new(3, 8);
    let id = BindingId(7);
    let binding = Binding {
        id,
        source_name: "idade".into(),
        ty: Type::Integer,
        mutability: Mutability::Mutable,
        is_const: false,
        span,
    };
    let expr = Expr {
        kind: ExprKind::Binding(id),
        ty: Type::Integer,
        span,
    };
    let program = Program {
        record_types: vec![],
        enum_types: vec![],
        bindings: vec![binding],
        body: vec![Stmt {
            kind: StmtKind::Declare {
                binding: id,
                initializer: Some(expr),
            },
            span,
        }],
        functions: vec![],
        span,
    };
    assert_eq!(program.bindings[0].id, id);
    assert_eq!(program.bindings[0].mutability, Mutability::Mutable);
    assert_eq!(program.body[0].span, span);
}

#[test]
fn shadowed_names_can_have_distinct_bindings() {
    let outer = BindingId(0);
    let inner = BindingId(1);
    assert_ne!(outer, inner);
}
