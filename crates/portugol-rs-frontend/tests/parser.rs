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
fn recovers_syntax_diagnostics_across_independent_function_bodies() {
    let source = "funcao primeira() { seja a: inteiro = ; }\nfuncao segunda() { seja b: inteiro = ; }\nfuncao terceira() { seja c: inteiro = ; }\nfuncao inicio() {}";
    let diagnostics = lower(source).expect_err("as três funções têm erros de sintaxe");

    assert_eq!(diagnostics.len(), 3, "{diagnostics:#?}");
    let semicolons: Vec<_> = source
        .match_indices(';')
        .map(|(offset, _)| offset as u32)
        .collect();
    let starts: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.span.start)
        .collect();
    assert_eq!(starts, semicolons);
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "pors.expected-expression")
    );
}

#[test]
fn does_not_repeat_diagnostics_for_an_invalid_top_level_item() {
    let source = "funcao quebrada( { }\nfuncao inicio() { seja idade: inteiro = ; }";
    let diagnostics = lower(source).expect_err("o cabeçalho e o corpo contêm erros");

    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, "pors.expected-identifier");
    assert_eq!(diagnostics[1].code, "pors.expected-expression");
    assert!(diagnostics[0].span.start < diagnostics[1].span.start);
}

#[test]
fn resumes_after_an_unclosed_function_before_later_top_level_declarations() {
    let source = "funcao quebrada() {\n    se (verdadeiro) {\nfuncao segunda() { seja b: inteiro = ; }\nfuncao inicio() {}";
    let diagnostics =
        lower(source).expect_err("a primeira função está aberta e a segunda é inválida");

    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, "pors.unexpected-token");
    assert_eq!(diagnostics[1].code, "pors.expected-expression");
    assert!(diagnostics[0].span.start < diagnostics[1].span.start);
}

#[test]
fn recovers_at_statement_terminators_inside_a_function() {
    let source = "funcao inicio() {\n    seja a: inteiro = ;\n    escreva(;\n}";
    let diagnostics = lower(source).expect_err("as duas instruções têm erros de sintaxe");

    assert_eq!(diagnostics.len(), 2, "{diagnostics:#?}");
    assert!(
        diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code == "pors.expected-expression")
    );
    let semicolons: Vec<_> = source
        .match_indices(';')
        .map(|(offset, _)| offset as u32)
        .collect();
    let starts: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.span.start)
        .collect();
    assert_eq!(starts, semicolons);
}

#[test]
fn suppresses_semantic_cascades_after_a_syntax_error_in_the_same_function() {
    let source = "funcao inicio() { seja idade: inteiro = ; escreva(idade); }";
    let diagnostics = lower(source).expect_err("a declaração tem sintaxe inválida");

    assert_eq!(diagnostics.len(), 1, "{diagnostics:#?}");
    assert_eq!(diagnostics[0].code, "pors.expected-expression");
    assert_eq!(
        &source[diagnostics[0].span.start as usize..diagnostics[0].span.end as usize],
        ";"
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
fn parses_chained_senao_se_as_nested_if_else_branches() {
    let source = "funcao inicio() { seja mutavel valor: inteiro = 2; se (valor == 1) { escreva(1); } senao se (valor == 2) { escreva(2); } senao { escreva(3); } }";
    let program = lower(source).expect("senao se encadeado deve ser válido");
    let StmtKind::If {
        else_branch: Some(first_else),
        ..
    } = &program.body[1].kind
    else {
        panic!("esperava o if inicial com ramo alternativo");
    };
    assert_eq!(first_else.len(), 1);
    let StmtKind::If {
        else_branch: Some(final_else),
        ..
    } = &first_else[0].kind
    else {
        panic!("esperava senao se como If aninhado com ramo final");
    };
    assert_eq!(final_else.len(), 1);
    assert!(matches!(final_else[0].kind, StmtKind::Write(_)));
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

#[test]
fn parses_and_validates_pors_core_libraries_matematica_texto_and_tipos() {
    let source = r#"
funcao somar_texto(a: cadeia, b: cadeia) -> Resultado<inteiro, cadeia> {
    seja x: inteiro = Tipos.para_inteiro(Texto.aparar(a))?;
    seja y: inteiro = Tipos.para_inteiro_base(b, 16)?;
    retorne Resultado::Ok(Matematica.maior_inteiro(x, y) + x);
}

funcao primeiro_char(s: cadeia) -> Opcao<caracter> {
    retorne Texto.obter_caracter(s, 0);
}

funcao inicio() {
    seja pi: real = Matematica.arredondar(Matematica.PI, 2);
    seja raiz: real = Matematica.raiz_quadrada(Matematica.potencia(3.0, 2.0) + Matematica.E);
    seja txt: cadeia = Texto.preencher_a_esquerda('0', 5, Tipos.inteiro_para_cadeia(42));
    seja sub: Resultado<cadeia, cadeia> = Texto.extrair_subtexto(txt, 0, 2);
    seja ok: logico = Tipos.cadeia_e_real("3.14") && Texto.contem(txt, "42");
    escreva(pi, raiz, ok);
    escolha sub {
        caso Resultado::Ok(v) => { escreva(v); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
    escolha primeiro_char("🦀") {
        caso Opcao::Algum(c) => { escreva(c); }
        caso Opcao::Nenhum => { escreva("-"); }
    }
    escolha somar_texto(" 10 ", "0x10") {
        caso Resultado::Ok(n) => { escreva(n); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let program = lower(source).expect("bibliotecas Matematica, Texto e Tipos devem ser aceitas");
    assert_eq!(program.functions.len(), 2);

    let unknown_fn = "funcao inicio() { seja x: real = Matematica.inexistente(1.0); }";
    let errors = lower(unknown_fn).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.unknown-stdlib-function")
    );

    let bad_arity = "funcao inicio() { seja x: real = Matematica.potencia(2.0); }";
    let errors = lower(bad_arity).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.stdlib-call-arity"));

    let bad_type = "funcao inicio() { seja x: real = Matematica.potencia(2, 3.0); }";
    let errors = lower(bad_type).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.stdlib-call-type-mismatch")
    );
}

#[test]
fn parses_and_validates_dynamic_vectors_and_texto_dividir() {
    let source = r#"
funcao primeiro_ou_vazio(itens: vetor<cadeia>) -> Opcao<cadeia> {
    seja item: cadeia = obter(itens, 0)?;
    retorne Opcao::Algum(item);
}

funcao inicio() {
    seja mutavel nums: vetor<inteiro> = [];
    adicionar(nums, 10);
    adicionar(nums, 20);
    remover(nums);
    seja ultimo: Opcao<inteiro> = remover(nums);
    seja partes: vetor<cadeia> = Texto.dividir("a,b,c", ",");
    escolha ultimo {
        caso Opcao::Algum(v) => { escreva(v); }
        caso Opcao::Nenhum => { escreva(0); }
    }
    escolha primeiro_ou_vazio(partes) {
        caso Opcao::Algum(s) => { escreva(s); }
        caso Opcao::Nenhum => { escreva("-"); }
    }
}
"#;
    let program = lower(source).expect("vetores dinâmicos e Texto.dividir devem ser aceitos");
    assert_eq!(program.functions.len(), 1);

    let untyped_empty = "funcao inicio() { escreva([]); }";
    let errors = lower(untyped_empty).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.empty-vector-type-inference")
    );

    let immutable_push = "funcao inicio() { seja v: vetor<inteiro> = []; adicionar(v, 1); }";
    let errors = lower(immutable_push).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.immutable-assignment"));

    let immutable_pop =
        "funcao inicio() { seja v: vetor<inteiro> = [1]; seja x: Opcao<inteiro> = remover(v); }";
    let errors = lower(immutable_pop).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.immutable-assignment"));

    let bad_push_type =
        "funcao inicio() { seja mutavel v: vetor<inteiro> = []; adicionar(v, \"oi\"); }";
    let errors = lower(bad_push_type).unwrap_err();
    assert!(errors.iter().any(|d| d.code == "pors.vector-element-type"));
}

#[test]
fn parses_and_validates_pors_util_and_calendario_libraries() {
    let source = r#"
funcao rolar_dado() -> Resultado<inteiro, cadeia> {
    Util.definir_semente(42);
    Util.aguarde(50);
    seja valor: inteiro = Util.sorteia(1, 6)?;
    retorne Resultado::Ok(valor);
}

funcao inicio() {
    seja t: inteiro = Util.tempo_decorrido();
    seja ano: inteiro = Calendario.ano_atual();
    seja bissexto: logico = Calendario.eh_bissexto(2024);
    seja iso: cadeia = Calendario.data_iso_atual();
    seja seg: inteiro = Calendario.DIA_SEGUNDA_FEIRA;
    seja nov: inteiro = Calendario.MES_NOVEMBRO;
    escreva(t, ano, bissexto, iso, seg, nov);
    escolha rolar_dado() {
        caso Resultado::Ok(n) => { escreva(n); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
    escolha Calendario.nome_mes(nov) {
        caso Resultado::Ok(m) => { escreva(m); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let program = lower(source).expect("Util e Calendario devem ser aceitos em PORS");
    assert_eq!(program.functions.len(), 1);

    let bad_util = "funcao inicio() { seja x: inteiro = Util.inexistente(); }";
    let errors = lower(bad_util).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.unknown-stdlib-function")
    );

    let bad_cal_type = "funcao inicio() { seja h: inteiro = Calendario.hora_atual(12); }";
    let errors = lower(bad_cal_type).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.stdlib-call-type-mismatch")
    );
}

#[test]
fn parses_and_validates_pors_arquivos_library() {
    let source = r#"
funcao gravar_e_ler() -> Resultado<cadeia, cadeia> {
    seja _ok1: logico = Arquivos.escrever_texto("dados/nota.txt", "linha1\n")?;
    seja _ok2: logico = Arquivos.acrescentar_texto("dados/nota.txt", "linha2\n")?;
    seja linhas: vetor<cadeia> = Arquivos.ler_linhas("dados/nota.txt")?;
    seja nomes: vetor<cadeia> = Arquivos.listar("dados")?;
    seja tem: logico = Arquivos.existe("dados/nota.txt")?;
    seja conteudo: cadeia = Arquivos.ler_texto("dados/nota.txt")?;
    seja _rem: logico = Arquivos.remover("dados/nota.txt")?;
    se (tem && tamanho(linhas) == 2 && tamanho(nomes) == 1) {
        retorne Resultado::Ok(conteudo);
    }
    retorne Resultado::Erro("inesperado");
}

funcao inicio() {
    escolha gravar_e_ler() {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let program = lower(source).expect("Arquivos.* deve ser aceito em PORS");
    assert_eq!(program.functions.len(), 1);

    let bad_fn = "funcao inicio() { seja r: Resultado<cadeia, cadeia> = Arquivos.abrir(\"a\"); }";
    let errors = lower(bad_fn).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.unknown-stdlib-function")
    );
}

#[test]
fn parses_and_validates_pors_json_library() {
    let source = r#"
funcao montar() -> Resultado<cadeia, cadeia> {
    seja mutavel doc: cadeia = "{}";
    doc = Json.definir_cadeia(doc, "nome", "Ada")?;
    doc = Json.definir_inteiro(doc, "ano", 1843)?;
    doc = Json.definir_real(doc, "nota", 9.5)?;
    doc = Json.definir_logico(doc, "ativo", verdadeiro)?;
    doc = Json.definir_json(doc, "tags", "[\"pors\",\"rust\"]")?;
    doc = Json.definir_json(doc, "nums", "[1,2,3]")?;
    seja chaves: vetor<cadeia> = Json.obter_chaves(doc)?;
    seja nome: cadeia = Json.obter_cadeia(doc, "nome")?;
    seja ano: inteiro = Json.obter_inteiro(doc, "ano")?;
    seja nota: real = Json.obter_real(doc, "nota")?;
    seja ativo: logico = Json.obter_logico(doc, "ativo")?;
    seja sub: cadeia = Json.obter_json(doc, "tags")?;
    seja tags: vetor<cadeia> = Json.obter_lista_cadeia(doc, "tags")?;
    seja nums: vetor<inteiro> = Json.obter_lista_inteiro(doc, "nums")?;
    seja tem: logico = Json.contem_chave(doc, "nome")?;
    seja sem: cadeia = Json.remover_chave(doc, "ativo")?;
    seja bonito: cadeia = Json.formatar(sem)?;
    seja compacto: cadeia = Json.minificar(bonito)?;
    se (Json.valido(compacto) && tem && ativo && ano == 1843 && nota > 9.0 && tamanho(chaves) == 6 && tamanho(tags) == 2 && tamanho(nums) == 3) {
        retorne Resultado::Ok(nome + "|" + Json.escapar_cadeia(sub));
    }
    retorne Resultado::Erro("inesperado");
}

funcao inicio() {
    escolha montar() {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let program = lower(source).expect("Json.* deve ser aceito em PORS");
    assert_eq!(program.functions.len(), 1);

    let bad_fn = "funcao inicio() { seja r: logico = Json.inexistente(\"{}\"); }";
    let errors = lower(bad_fn).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.unknown-stdlib-function")
    );

    let bad_type = "funcao inicio() { seja r: Resultado<cadeia, cadeia> = Json.definir_inteiro(\"{}\", \"x\", \"nao\"); }";
    let errors = lower(bad_type).unwrap_err();
    assert!(
        errors
            .iter()
            .any(|d| d.code == "pors.stdlib-call-type-mismatch")
    );
}

#[test]
fn parses_bitwise_shift_operators_nested_generics_and_choose_wildcard() {
    let source = r#"
enum Cor {
    Vermelho,
    Verde,
    Azul
}

funcao inicio() {
    seja mutavel bits: inteiro = (1 << 4) | 3;
    bits &= 21;
    bits |= 2;
    bits ^= 1;
    seja inv: inteiro = ~0;
    seja deslocado: inteiro = bits >> 1;
    seja aninhado: Opcao<vetor<inteiro>> = Opcao::Algum([bits, inv, deslocado]);
    seja cor: Cor = Cor::Verde;
    escolha cor {
        caso Cor::Vermelho => { escreva("R"); }
        caso _ => { escreva("OUTRA"); }
    }
    escolha aninhado {
        caso Opcao::Algum(v) => { escreva(tamanho(v)); }
        caso _ => { escreva(0); }
    }
}
"#;
    assert!(lower(source).is_ok());

    let unreachable_after = r#"
enum Cor { A, B }
funcao inicio() {
    seja c: Cor = Cor::A;
    escolha c {
        caso _ => { escreva("x"); }
        caso Cor::A => { escreva("a"); }
    }
}
"#;
    let errs = lower(unreachable_after).unwrap_err();
    assert!(errs.iter().any(|d| d.code == "pors.unreachable-case"));

    let unreachable_all = r#"
enum Cor { A, B }
funcao inicio() {
    seja c: Cor = Cor::A;
    escolha c {
        caso Cor::A => { escreva("a"); }
        caso Cor::B => { escreva("b"); }
        caso _ => { escreva("x"); }
    }
}
"#;
    let errs = lower(unreachable_all).unwrap_err();
    assert!(errs.iter().any(|d| d.code == "pors.unreachable-case"));
}

#[test]
fn parses_and_validates_pors_internet_library() {
    let source = r#"
funcao consultar(url: cadeia) -> Resultado<cadeia, cadeia> {
    Internet.definir_tempo_limite(3000);
    seja ok: logico = Internet.endereco_disponivel(url)?;
    se (!ok) {
        retorne Resultado::Erro("indisponivel");
    }
    seja corpo: cadeia = Internet.obter_texto(url)?;
    seja eco: cadeia = Internet.enviar_json(url, "{\"ok\":true}")?;
    retorne Resultado::Ok(corpo + "|" + eco);
}

funcao inicio() {
    escolha consultar("http://127.0.0.1:8080/api") {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva(e); }
    }
}
"#;
    let program = lower(source).expect("Internet.* deve ser aceito em PORS");
    assert_eq!(program.functions.len(), 1);

    let bad_fn =
        "funcao inicio() { seja r: Resultado<cadeia, cadeia> = Internet.baixar(\"http://x\"); }";
    let errs = lower(bad_fn).unwrap_err();
    assert!(
        errs.iter()
            .any(|d| d.code == "pors.unknown-stdlib-function")
    );
}

#[test]
fn parses_do_while_loop_with_break_and_continue() {
    let source = r#"
funcao inicio() {
    seja mutavel i: inteiro = 0;
    seja mutavel soma: inteiro = 0;
    faca {
        i += 1;
        se (i == 2) {
            continue;
        }
        soma += i;
        se (i == 4) {
            pare;
        }
    } enquanto (i < 10);
    escreva(soma);
}
"#;
    assert!(lower(source).is_ok());
    assert!(portugol_rs_frontend::format(source).is_some());

    let bad_cond = "funcao inicio() { faca { escreva(1); } enquanto (1); }";
    let errs = lower(bad_cond).unwrap_err();
    assert!(errs.iter().any(|d| d.code == "pors.condition-type"));
}

#[test]
fn parses_typed_pors_multimedia_bridge_libraries_and_constants() {
    let source = r#"
funcao inicio() {
    Graficos.iniciar_modo_grafico(verdadeiro);
    Graficos.definir_dimensoes_janela(320, 240);
    Graficos.definir_titulo_janela("Demo PORS");
    seja cor: Resultado<inteiro, cadeia> = Graficos.criar_cor(10, 20, 30);
    escolha cor {
        caso Resultado::Ok(c) => {
            Graficos.definir_cor(c);
            escreva(Graficos.obter_rgb(c, Graficos.CANAL_R));
        }
        caso Resultado::Erro(e) => {
            escreva(e);
        }
    }
    seja xs: vetor<inteiro> = [0, 10, 5];
    seja ys: vetor<inteiro> = [0, 0, 10];
    seja _poly: Resultado<logico, cadeia> = Graficos.desenhar_poligono(xs, ys, verdadeiro);
    Graficos.desenhar_retangulo(1, 2, 30, 40, falso, verdadeiro);
    Graficos.renderizar();
    seja esc: logico = Teclado.tecla_pressionada(Teclado.TECLA_ESC);
    seja letra: caracter = Teclado.caracter_tecla(Teclado.TECLA_A);
    seja clicou: logico = Mouse.botao_pressionado(Mouse.BOTAO_ESQUERDO);
    seja mx: inteiro = Mouse.posicao_x();
    seja som: Resultado<inteiro, cadeia> = Sons.carregar_som("efeito.wav");
    Sons.definir_volume(80);
    escreva(esc, letra, clicou, mx, Sons.obter_volume());
    escolha som {
        caso Resultado::Ok(id) => {
            seja _rep: Resultado<inteiro, cadeia> = Sons.reproduzir_som(id, falso);
            Sons.liberar_som(id);
        }
        caso Resultado::Erro(msg) => {
            escreva(msg);
        }
    }
    Graficos.encerrar_modo_grafico();
}
"#;
    assert!(lower(source).is_ok());

    let bad_arg = "funcao inicio() { Graficos.definir_cor(\"verde\"); }";
    let errs = lower(bad_arg).unwrap_err();
    assert!(
        errs.iter()
            .any(|d| d.code == "pors.stdlib-call-type-mismatch")
    );
}

#[test]
fn accepts_the_canonical_syntax_evolution_program() {
    let source = include_str!("../../../tests/fixtures/pors-syntax-evolution.pors");
    let program = lower(source).expect("programa motivador deve compilar no frontend");
    assert_eq!(program.functions.len(), 1);
}

#[test]
fn escreval_appends_a_newline_expression_to_write() {
    let source = "funcao inicio() { escreval(\"a\"); }";
    let program = lower(source).expect("escreval deve compilar");
    let StmtKind::Write(values) = &program.body[0].kind else {
        panic!("esperava Write");
    };
    assert_eq!(values.len(), 2);
    assert!(matches!(&values[1].kind, ExprKind::String(text) if text == "\n"));
    assert!(values[1].span.is_empty());
}
