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
fn parses_nominal_record_definitions_before_the_entry_function() {
    let source = "registro Pessoa { idade: inteiro, nome: cadeia } funcao inicio() {}";
    let program = lower(source).expect("registro declarado antes de inicio deve ser aceito");
    assert_eq!(program.record_types.len(), 1);
    assert_eq!(program.record_types[0].name, "Pessoa");
    assert_eq!(program.record_types[0].fields.len(), 2);
}

#[test]
fn parses_record_construction_and_typed_field_reads() {
    let source = "registro Pessoa { idade: inteiro } funcao inicio() { seja pessoa: Pessoa = Pessoa { idade: 16 }; escreva(pessoa.idade); }";
    let program = lower(source).expect("construção e leitura de campo devem ser tipadas");
    let StmtKind::Declare {
        initializer: Some(value),
        ..
    } = &program.body[0].kind
    else {
        panic!("declaração")
    };
    assert!(matches!(value.kind, ExprKind::RecordLiteral { .. }));
    let StmtKind::Write(values) = &program.body[1].kind else {
        panic!("escreva")
    };
    assert!(matches!(
        values[0].kind,
        ExprKind::Field { field_index: 0, .. }
    ));
    assert_eq!(values[0].ty, Type::Integer);
}

#[test]
fn parses_enum_definitions_with_typed_payloads() {
    let source = "enum Evento { Iniciar, Atualizar(inteiro, cadeia) } funcao inicio() {}";
    let program = lower(source).expect("enum com payloads deve ser aceito");
    assert_eq!(program.enum_types.len(), 1);
    assert_eq!(program.enum_types[0].variants[1].name, "Atualizar");
    assert_eq!(
        program.enum_types[0].variants[1].payload_types,
        [Type::Integer, Type::String]
    );
}

#[test]
fn parses_enum_constructors_and_exhaustive_choice_arms() {
    let source = "enum Evento { Incrementar, Definir(inteiro) } funcao inicio() { seja evento: Evento = Evento::Definir(7); escolha evento { caso Evento::Incrementar => { escreva(0); } caso Evento::Definir(valor) => { escreva(valor); } } }";
    let program = lower(source).expect("construtor e escolha exaustiva devem ser aceitos");
    assert_eq!(program.enum_types.len(), 1);
    assert!(
        matches!(program.body[0].kind, StmtKind::Declare { initializer: Some(ref expression), .. } if matches!(expression.kind, ExprKind::EnumConstructor { .. }))
    );
    assert!(
        matches!(program.body[1].kind, StmtKind::Choose { ref arms, .. } if arms.len() == 2 && arms[1].payload_bindings.len() == 1)
    );
}

#[test]
fn parses_typed_helper_functions_forward_calls_and_returns() {
    let source = "funcao dobro(valor: inteiro) -> inteiro { retorne somar(valor, valor); }\nfuncao somar(a: inteiro, b: inteiro) -> inteiro { retorne a + b; }\nfuncao inicio() { escreva(dobro(21)); }";
    let program = lower(source).expect("funções tipadas e chamadas para frente devem ser válidas");
    assert_eq!(program.body.len(), 1);
}

#[test]
fn parses_mutable_parameter_and_void_helper_return() {
    let source = "funcao incrementar(mutavel valor: inteiro) -> inteiro { valor += 1; retorne valor; }\nfuncao mostrar(valor: inteiro) { escreva(valor); retorne; }\nfuncao inicio() { mostrar(incrementar(4)); }";
    let program = lower(source).expect("parâmetro mutável e retorno vazio devem ser válidos");
    assert!(matches!(program.body[0].kind, StmtKind::Call { .. }));
}

#[test]
fn parses_vectors_with_explicit_element_types_and_length() {
    for (ty, literal) in [
        ("inteiro", "7"),
        ("real", "2.5"),
        ("logico", "verdadeiro"),
        ("cadeia", "\"olá\""),
        ("caracter", "'x'"),
    ] {
        let source = format!(
            "funcao inicio() {{ seja valores: vetor<{ty}> = [{literal}]; escreva(tamanho(valores)); }}"
        );
        let program = lower(&source).expect("vetor primitivo tipado deve ser aceito");
        assert_eq!(program.bindings.len(), 1);
        assert!(format!("{:?}", program.bindings[0].ty).starts_with("Vector"));
    }
}

#[test]
fn parses_vector_index_reads_writes_and_function_parameters() {
    let source = "funcao primeiro(mutavel valores: vetor<inteiro>) -> inteiro { valores[0] = 11; retorne valores[0]; } funcao inicio() { seja mutavel notas: vetor<inteiro> = [7, 8]; notas[1] = 10; escreva(primeiro(notas)); escreva(tamanho(notas)); }";
    let program = lower(source).expect("vetor indexável e parâmetro por valor devem ser aceitos");
    assert_eq!(program.functions.len(), 1);
    assert!(format!("{:?}", program.functions[0].bindings[0].ty).starts_with("Vector"));
}

#[test]
fn parses_reading_into_a_mutable_binding() {
    let source = "funcao inicio() { seja mutavel idade: inteiro = 0; leia(idade); }";
    let program = lower(source).expect("leia deve aceitar uma variável mutável");
    assert!(
        matches!(program.body[1].kind, StmtKind::Read { binding, .. } if binding == program.bindings[0].id)
    );
}

#[test]
fn rejects_reading_into_an_immutable_binding() {
    let diagnostics =
        lower("funcao inicio() { seja idade: inteiro = 0; leia(idade); }").unwrap_err();
    assert!(
        diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == "pors.immutable-assignment")
    );
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

#[test]
fn parses_break_and_continue_inside_nested_loop_blocks() {
    let program = lower(
        "funcao inicio() { enquanto (verdadeiro) { se (verdadeiro) { pare; } } para (seja mutavel i: inteiro = 0; i < 1; i += 1) { se (verdadeiro) { continue; } } }",
    )
    .expect("pare e continue devem ser válidos dentro de blocos aninhados em laços");

    let StmtKind::While { body, .. } = &program.body[0].kind else {
        panic!("esperava enquanto");
    };
    let StmtKind::If { then_branch, .. } = &body[0].kind else {
        panic!("esperava se aninhado em enquanto");
    };
    assert!(matches!(
        then_branch[0].kind,
        StmtKind::LoopControl {
            control: portugol_hir::LoopControl::Break
        }
    ));

    let StmtKind::For { body, .. } = &program.body[1].kind else {
        panic!("esperava para");
    };
    let StmtKind::If { then_branch, .. } = &body[0].kind else {
        panic!("esperava se aninhado em para");
    };
    assert!(matches!(
        then_branch[0].kind,
        StmtKind::LoopControl {
            control: portugol_hir::LoopControl::Continue
        }
    ));
}

#[test]
fn rejects_break_and_continue_outside_loops_at_the_keyword_span() {
    for keyword in ["pare", "continue"] {
        let source = format!("funcao inicio() {{ se (verdadeiro) {{ {keyword}; }} }}");
        let errors = lower(&source).unwrap_err();
        let error = errors
            .iter()
            .find(|diagnostic| diagnostic.code == "pors.loop-control-outside-loop")
            .unwrap_or_else(|| panic!("esperava diagnóstico de controle fora de laço: {keyword}"));
        assert_eq!(
            &source[error.span.start as usize..error.span.end as usize],
            keyword
        );
    }
}

#[test]
fn parses_declarative_ui_mvu_program_with_records_enums_and_all_components() {
    let source = r#"
enum Modo { Normal, Destaque }

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    modo: Modo
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "", modo: Modo::Normal };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, modo: Modo::Destaque };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, modo: estado.modo };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel rotulo: cadeia = "Normal";
    escolha estado.modo {
        caso Modo::Normal => { rotulo = "Normal"; }
        caso Modo::Destaque => { rotulo = "Destaque"; }
    }
    retorne UI.coluna([
        UI.texto("Total: ", estado.contagem, " (", rotulo, ")"),
        UI.linha([
            UI.botao("inc", "+1"),
            UI.entrada("nome", estado.nome, "Seu nome")
        ]),
        UI.lista([UI.texto("Item ", 1)])
    ]);
}
"#;
    let program = lower(source).expect("programa visual MVU com UI.* deve ser aceito");
    assert!(program.body.is_empty());
    assert_eq!(program.functions.len(), 3);
    assert_eq!(program.record_types.len(), 1);
    assert_eq!(program.enum_types.len(), 1);
}

#[test]
fn validates_ui_host_calls_and_mvu_entry_signatures() {
    let unknown = "funcao inicio() { seja no: inteiro = UI.canvas([1]); }";
    let errors = lower(unknown).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.unknown-ui-component"));

    let bad_arity = "funcao inicio() { seja no: inteiro = UI.botao(\"so_um\"); }";
    let errors = lower(bad_arity).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.ui-call-arity"));

    let bad_type = "funcao inicio() { seja no: inteiro = UI.coluna([\"texto\"]); }";
    let errors = lower(bad_type).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.ui-call-type-mismatch")
    );

    let bad_mvu = r#"
funcao iniciar() -> inteiro { retorne 0; }
funcao atualizar(estado: inteiro, alvo: inteiro) -> inteiro { retorne estado + alvo; }
funcao visualizar(estado: inteiro) -> inteiro { retorne UI.texto(estado); }
"#;
    let errors = lower(bad_mvu).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.ui-entry-signature"));
}

#[test]
fn parses_builtin_option_result_exhaustive_choose_and_try_propagation() {
    let source = r#"
registro Ponto { x: inteiro, y: inteiro }

funcao buscar_ponto(id: inteiro) -> Opcao<Ponto> {
    se (id == 1) {
        retorne Opcao::Algum(Ponto { x: 3, y: 4 });
    }
    retorne Opcao::Nenhum;
}

funcao soma_x(id: inteiro) -> Opcao<inteiro> {
    seja x: inteiro = buscar_ponto(id)?.x;
    retorne Opcao::Algum(x + 1);
}

funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("divisao por zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao descrever_divisao(a: inteiro, b: inteiro) -> Resultado<cadeia, cadeia> {
    seja q: inteiro = dividir(a, b)?;
    se (q > 0) {
        retorne Resultado::Ok("positivo");
    }
    retorne Resultado::Ok("nao positivo");
}

funcao inicio() {
    seja itens: vetor<Opcao<inteiro>> = [Opcao::Algum(10), Opcao::Nenhum];
    seja primeiro: Opcao<inteiro> = itens[0];
    escolha soma_x(1) {
        caso Opcao::Algum(v) => { escreva(v); }
        caso Opcao::Nenhum => { escreva("vazio"); }
    }
    escolha descrever_divisao(10, 2) {
        caso Resultado::Ok(msg) => { escreva(msg); }
        caso Resultado::Erro(err) => { escreva(err); }
    }
    escolha primeiro {
        caso Opcao::Algum(n) => { escreva(n); }
        caso Opcao::Nenhum => { escreva(0); }
    }
}
"#;
    let program = lower(source).expect("programa com Opcao, Resultado e ? deve ser aceito");
    assert_eq!(program.record_types.len(), 1);
    assert!(
        program
            .enum_types
            .iter()
            .any(|e| matches!(e.builtin, Some(portugol_hir::BuiltinEnum::Option(_))))
    );
    assert!(program.enum_types.iter().any(|e| matches!(
        e.builtin,
        Some(portugol_hir::BuiltinEnum::Result {
            ok: portugol_hir::Type::Integer,
            err: portugol_hir::Type::String
        })
    )));
    assert!(program.enum_types.iter().any(|e| matches!(
        e.builtin,
        Some(portugol_hir::BuiltinEnum::Result {
            ok: portugol_hir::Type::String,
            err: portugol_hir::Type::String
        })
    )));
}

#[test]
fn validates_option_result_inference_exhaustiveness_try_rules_and_reserved_names() {
    let bad_option_infer = "funcao inicio() { escreva(Opcao::Nenhum); }";
    let errors = lower(bad_option_infer).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.option-type-inference")
    );

    let bad_result_infer = "funcao inicio() { escreva(Resultado::Ok(1)); }";
    let errors = lower(bad_result_infer).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.result-type-inference")
    );

    let try_non_builtin = "funcao f() -> Opcao<inteiro> { seja x: inteiro = 1?; retorne Opcao::Algum(x); } funcao inicio() {}";
    let errors = lower(try_non_builtin).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.try-invalid-type"));

    let try_in_inicio = "funcao inicio() { seja x: inteiro = Opcao::Algum(1)?; escreva(x); }";
    let errors = lower(try_in_inicio).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.try-return-mismatch"));

    let try_err_mismatch = r#"
funcao f() -> Resultado<inteiro, cadeia> { retorne Resultado::Erro("falha"); }
funcao g() -> Resultado<inteiro, inteiro> { seja x: inteiro = f()?; retorne Resultado::Ok(x); }
funcao inicio() {}
"#;
    let errors = lower(try_err_mismatch).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.try-return-mismatch"));

    let non_exhaustive = r#"
funcao inicio() {
    seja o: Opcao<inteiro> = Opcao::Algum(1);
    escolha o {
        caso Opcao::Algum(v) => { escreva(v); }
    }
}
"#;
    let errors = lower(non_exhaustive).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.non-exhaustive-choice")
    );

    let reserved_enum = "enum Opcao { A } funcao inicio() {}";
    let errors = lower(reserved_enum).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.reserved-name"));

    let reserved_record = "registro Resultado { x: inteiro } funcao inicio() {}";
    let errors = lower(reserved_record).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.reserved-name"));
}
