use portugol_syntax::ast::{Declarator, Expr, Item, Stmt, Type};
use portugol_syntax::{Severity, parse};

fn assert_ok(source: &str) -> portugol_syntax::ParseResult {
    let result = parse(source);
    let errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();

    assert!(
        errors.is_empty(),
        "erros inesperados: {:#?}\nfonte:\n{}",
        errors,
        source
    );

    result
}

#[test]
fn programa_minimo() {
    let result = assert_ok(
        r#"
        programa {
            funcao inicio() {
                inteiro x = 1
                escreva(x)
            }
        }
        "#,
    );

    assert_eq!(result.program.items.len(), 1);
    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava uma função");
    };
    assert_eq!(function.name.name, "inicio");
    assert_eq!(function.body.len(), 2);
}

#[test]
fn trecho_sem_bloco_programa() {
    let result = assert_ok(
        r#"
        inclua biblioteca Matematica --> m
        const real PI2 = 6.28

        funcao real dobro(real valor) {
            retorne valor * 2
        }
        "#,
    );

    assert_eq!(result.program.includes.len(), 1);
    assert_eq!(result.program.includes[0].alias.as_ref().unwrap().name, "m");
    assert_eq!(result.program.items.len(), 2);

    let Item::Declarations(decl) = &result.program.items[0] else {
        panic!("esperava declarações");
    };
    assert!(decl.is_const);
    assert_eq!(decl.ty, Type::Real);
}

#[test]
fn vetores_e_matrizes() {
    let result = assert_ok(
        r#"
        programa {
            funcao inicio() {
                inteiro v[3] = {1, 2, 3}
                real m[2][3] = {{1.0, 2.0, 3.0}, {4.0, 5.0, 6.0}}
                caracter letras[] = {'a', 'b'}
                inteiro numeros[10]
            }
        }
        "#,
    );

    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };

    let Stmt::Declarations(decl) = &function.body[0] else {
        panic!("esperava declaração");
    };
    assert!(matches!(decl.declarators[0], Declarator::Array { .. }));

    let Stmt::Declarations(decl) = &function.body[1] else {
        panic!("esperava declaração");
    };
    let Declarator::Matrix { init, .. } = &decl.declarators[0] else {
        panic!("esperava matriz");
    };
    assert_eq!(init.as_ref().unwrap().len(), 2);
    assert_eq!(init.as_ref().unwrap()[0].len(), 3);

    let Stmt::Declarations(decl) = &function.body[2] else {
        panic!("esperava declaração");
    };
    let Declarator::Array { size, .. } = &decl.declarators[0] else {
        panic!("esperava vetor");
    };
    assert!(size.is_none());

    let Stmt::Declarations(decl) = &function.body[3] else {
        panic!("esperava declaração");
    };
    let Declarator::Array { size, .. } = &decl.declarators[0] else {
        panic!("esperava vetor");
    };
    assert!(size.is_some());
}

#[test]
fn estruturas_de_controle() {
    let result = assert_ok(
        r#"
        programa {
            funcao inicio() {
                inteiro i = 0
                enquanto (i < 10) {
                    se (i % 2 == 0) {
                        escreva(i)
                    } senao {
                        escreva("ímpar")
                    }
                    i++
                }

                faca {
                    i--
                } enquanto (i > 0)

                para (inteiro j = 0; j < 10; j++) {
                    escolha (j) {
                        caso 1:
                            escreva("um")
                            pare
                        caso 2:
                            escreva("dois")
                        caso contrario:
                            escreva("outro")
                    }
                }
            }
        }
        "#,
    );

    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };

    assert!(matches!(function.body[0], Stmt::Declarations(_)));
    assert!(matches!(function.body[1], Stmt::While { .. }));
    assert!(matches!(function.body[2], Stmt::DoWhile { .. }));
    assert!(matches!(function.body[3], Stmt::For { .. }));
}

#[test]
fn chamadas_e_escopo_de_biblioteca() {
    let result = assert_ok(
        r#"
        programa {
            inclua biblioteca Graficos --> g
            funcao inicio() {
                g.iniciar_modo_grafico(verdadeiro)
                real x = Matematica.arredondar(Matematica.PI, 2)
                escreva(x)
            }
        }
        "#,
    );

    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };

    let Stmt::Expr { expr, .. } = &function.body[0] else {
        panic!("esperava chamada");
    };
    let Expr::Call { scope, name, .. } = expr else {
        panic!("esperava chamada de função");
    };
    assert_eq!(scope.as_deref(), Some("g"));
    assert_eq!(name, "iniciar_modo_grafico");
}

#[test]
fn precedencia_de_operadores() {
    let result =
        assert_ok("programa { funcao inicio() { logico x = 1 + 2 * 3 == 7 e nao falso } }");
    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };
    let Stmt::Declarations(decl) = &function.body[0] else {
        panic!("esperava declaração");
    };
    let Declarator::Variable {
        init: Some(expr), ..
    } = &decl.declarators[0]
    else {
        panic!("esperava inicialização");
    };
    let Expr::Binary { op, .. } = expr else {
        panic!("esperava operação binária");
    };
    assert!(matches!(op, portugol_syntax::ast::BinOp::And));
}

#[test]
fn comentarios_e_escapes() {
    let result = assert_ok(
        r#"
        // comentário de linha
        /* comentário
           de bloco */
        programa {
            funcao inicio() {
                cadeia texto = "linha\ncom \"aspas\" e \\ barra"
                caracter letra = '\n'
                escreva(texto, letra)
            }
        }
        "#,
    );

    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };
    let Stmt::Declarations(decl) = &function.body[0] else {
        panic!("esperava declaração");
    };
    let Declarator::Variable {
        init: Some(Expr::Str { value, .. }),
        ..
    } = &decl.declarators[0]
    else {
        panic!("esperava string");
    };
    assert_eq!(value, "linha\ncom \"aspas\" e \\ barra");
}

#[test]
fn trechos_com_comandos_soltos() {
    let result = assert_ok(
        r#"
        logico condicao = verdadeiro
        se (condicao) {
            escreva("sim")
        }
        inteiro x = 0
        x += 1
        "#,
    );

    assert_eq!(result.program.items.len(), 4);
    assert!(matches!(
        result.program.items[1],
        Item::Stmt(Stmt::If { .. })
    ));
    assert!(matches!(
        result.program.items[3],
        Item::Stmt(Stmt::Assign(_))
    ));
}

#[test]
fn atribuicao_dentro_de_argumentos() {
    let result = assert_ok(
        r#"
        programa {
            funcao inicio() {
                escreva(Matematica.menor(x += 1, 0))
            }
        }
        "#,
    );

    let Item::Function(function) = &result.program.items[0] else {
        panic!("esperava função");
    };
    let Stmt::Expr {
        expr: Expr::Call { args, .. },
        ..
    } = &function.body[0]
    else {
        panic!("esperava chamada");
    };
    let Expr::Call { args: inner, .. } = &args[0] else {
        panic!("esperava chamada interna");
    };
    assert!(matches!(inner[0], Expr::Assign { .. }));
}

#[test]
fn erro_recupera_e_continua() {
    let result = parse(
        r#"
        programa {
            funcao inicio() {
                inteiro x = )
            }

            funcao depois() {
                escreva("o parser continuou")
            }
        }
        "#,
    );

    assert!(
        result
            .diagnostics
            .iter()
            .any(|d| d.severity == Severity::Error)
    );
    assert_eq!(
        result.program.items.len(),
        2,
        "as duas funções deveriam existir"
    );
}
