use portugol_hir as hir;

#[test]
fn compiles_hir_to_a_vm_function_with_binding_names_and_source_spans() {
    let span = hir::Span::new(0, 8);
    let program = hir::Program {
        record_types: vec![],
        enum_types: vec![],
        bindings: vec![hir::Binding {
            id: hir::BindingId(0),
            source_name: "idade".into(),
            ty: hir::Type::Integer,
            mutability: hir::Mutability::Mutable,
            is_const: false,
            span,
        }],
        body: vec![
            hir::Stmt {
                kind: hir::StmtKind::Declare {
                    binding: hir::BindingId(0),
                    initializer: Some(hir::Expr {
                        kind: hir::ExprKind::Integer(16),
                        ty: hir::Type::Integer,
                        span,
                    }),
                },
                span,
            },
            hir::Stmt {
                kind: hir::StmtKind::Assign {
                    binding: hir::BindingId(0),
                    op: hir::AssignOp::Add,
                    value: hir::Expr {
                        kind: hir::ExprKind::Integer(1),
                        ty: hir::Type::Integer,
                        span,
                    },
                },
                span,
            },
            hir::Stmt {
                kind: hir::StmtKind::Write(vec![hir::Expr {
                    kind: hir::ExprKind::Binding(hir::BindingId(0)),
                    ty: hir::Type::Integer,
                    span,
                }]),
                span,
            },
        ],
        functions: vec![],
        span,
    };

    let module = portugol_bytecode::compile_pors(&program).expect("HIR PORS deve compilar");
    let function = module.function("inicio").expect("função de entrada");
    assert_eq!(function.locals[0].name, "idade");
    assert!(function.code.len() >= 8);
    assert!(function.spans.iter().all(|s| s.start == 0 && s.end == 8));
}

#[test]
fn compiles_boolean_short_circuit_and_loop_control_to_jumps() {
    let span = hir::Span::new(0, 12);
    let program = hir::Program {
        record_types: vec![],
        enum_types: vec![],
        bindings: vec![],
        body: vec![hir::Stmt {
            kind: hir::StmtKind::While {
                condition: hir::Expr {
                    kind: hir::ExprKind::Boolean(false),
                    ty: hir::Type::Boolean,
                    span,
                },
                body: vec![hir::Stmt {
                    kind: hir::StmtKind::LoopControl {
                        control: hir::LoopControl::Break,
                    },
                    span,
                }],
            },
            span,
        }],
        functions: vec![],
        span,
    };
    let module = portugol_bytecode::compile_pors(&program).expect("loop deve compilar");
    assert!(
        module
            .function("inicio")
            .unwrap()
            .code
            .iter()
            .any(|op| matches!(op, portugol_bytecode::Op::Jump(_)))
    );
}

#[test]
fn tracks_shadowed_binding_lifetimes_with_visibility_opcodes() {
    let outer_span = hir::Span::new(0, 10);
    let inner_span = hir::Span::new(20, 30);
    let program = hir::Program {
        record_types: vec![],
        enum_types: vec![],
        bindings: vec![
            hir::Binding {
                id: hir::BindingId(0),
                source_name: "valor".into(),
                ty: hir::Type::Integer,
                mutability: hir::Mutability::Mutable,
                is_const: false,
                span: outer_span,
            },
            hir::Binding {
                id: hir::BindingId(1),
                source_name: "valor".into(),
                ty: hir::Type::Integer,
                mutability: hir::Mutability::Mutable,
                is_const: false,
                span: inner_span,
            },
        ],
        body: vec![
            hir::Stmt {
                kind: hir::StmtKind::Declare {
                    binding: hir::BindingId(0),
                    initializer: Some(hir::Expr {
                        kind: hir::ExprKind::Integer(1),
                        ty: hir::Type::Integer,
                        span: outer_span,
                    }),
                },
                span: outer_span,
            },
            hir::Stmt {
                kind: hir::StmtKind::If {
                    condition: hir::Expr {
                        kind: hir::ExprKind::Boolean(true),
                        ty: hir::Type::Boolean,
                        span: inner_span,
                    },
                    then_branch: vec![
                        hir::Stmt {
                            kind: hir::StmtKind::Declare {
                                binding: hir::BindingId(1),
                                initializer: Some(hir::Expr {
                                    kind: hir::ExprKind::Integer(2),
                                    ty: hir::Type::Integer,
                                    span: inner_span,
                                }),
                            },
                            span: inner_span,
                        },
                        hir::Stmt {
                            kind: hir::StmtKind::Write(vec![hir::Expr {
                                kind: hir::ExprKind::Binding(hir::BindingId(1)),
                                ty: hir::Type::Integer,
                                span: inner_span,
                            }]),
                            span: inner_span,
                        },
                    ],
                    else_branch: None,
                },
                span: inner_span,
            },
        ],
        functions: vec![],
        span: hir::Span::new(0, 30),
    };

    let function = portugol_bytecode::compile_pors(&program)
        .expect("sombreamento deve preservar escopos no debugger")
        .function("inicio")
        .unwrap()
        .clone();
    assert!(function.code.iter().any(|op| matches!(
        op,
        portugol_bytecode::Op::SetLocalActive {
            local: 0,
            active: true
        }
    )));
    assert!(function.code.iter().any(|op| matches!(
        op,
        portugol_bytecode::Op::SetLocalActive {
            local: 1,
            active: false
        }
    )));
}

#[test]
fn compiles_string_concatenation_and_continue_with_mapped_source_spans() {
    let span = hir::Span::new("ação ".len() as u32, "ação escreva".len() as u32);
    let concat = hir::Expr {
        kind: hir::ExprKind::Binary {
            op: hir::BinaryOp::Add,
            left: Box::new(hir::Expr {
                kind: hir::ExprKind::String("olá ".into()),
                ty: hir::Type::String,
                span,
            }),
            right: Box::new(hir::Expr {
                kind: hir::ExprKind::String("Rust".into()),
                ty: hir::Type::String,
                span,
            }),
        },
        ty: hir::Type::String,
        span,
    };
    let program = hir::Program {
        record_types: vec![],
        enum_types: vec![],
        bindings: vec![],
        body: vec![
            hir::Stmt {
                kind: hir::StmtKind::Write(vec![concat]),
                span,
            },
            hir::Stmt {
                kind: hir::StmtKind::While {
                    condition: hir::Expr {
                        kind: hir::ExprKind::Boolean(false),
                        ty: hir::Type::Boolean,
                        span,
                    },
                    body: vec![hir::Stmt {
                        kind: hir::StmtKind::LoopControl {
                            control: hir::LoopControl::Continue,
                        },
                        span,
                    }],
                },
                span,
            },
        ],
        functions: vec![],
        span,
    };

    let function = portugol_bytecode::compile_pors(&program)
        .expect("concatenação e continue suportados")
        .function("inicio")
        .unwrap()
        .clone();
    assert!(
        function
            .code
            .iter()
            .any(|op| matches!(op, portugol_bytecode::Op::Add))
    );
    assert!(
        function
            .code
            .iter()
            .any(|op| matches!(op, portugol_bytecode::Op::Jump(_)))
    );
    assert!(
        function
            .spans
            .iter()
            .any(|mapped| mapped.start == span.start)
    );
}

#[test]
fn maps_interpolation_parts_and_read_expression_to_hir_nodes() {
    let source = "funcao inicio() { escreva(\"Olá, {leia()}!\"); }";
    let program = portugol_rs_frontend::lower(source).expect("fonte válida");
    let module = portugol_bytecode::compile_pors(&program).expect("bytecode válido");
    let entry = &module.functions[module.entry as usize];
    let labels: Vec<_> = entry.nodes.iter().flatten().map(|n| n.node).collect();
    assert!(labels.contains(&"InterpolatedString.Literal"));
    assert!(labels.contains(&"InterpolatedString.Expression"));
    assert!(labels.contains(&"Read"));
}
