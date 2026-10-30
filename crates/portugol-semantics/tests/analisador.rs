//! Casos espelhando `AnalisadorSemanticoTest.java` do Portugol Studio original,
//! mais alguns extras para regras sem cobertura lá.

use portugol_semantics::analyze;
use portugol_syntax::{Diagnostic, Severity, parse};

fn analisar(source: &str) -> Vec<Diagnostic> {
    let result = parse(source);
    let syntax_errors: Vec<_> = result
        .diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(
        syntax_errors.is_empty(),
        "erros de sintaxe inesperados: {:#?}\nfonte:\n{}",
        syntax_errors,
        source
    );
    analyze(&result.program)
}

fn erros(source: &str) -> Vec<Diagnostic> {
    analisar(source)
        .into_iter()
        .filter(|d| d.severity == Severity::Error)
        .collect()
}

fn codigos(diagnostics: &[Diagnostic]) -> Vec<&'static str> {
    diagnostics.iter().map(|d| d.code).collect()
}

/// Sem erros nem avisos.
fn assert_limpo(source: &str) {
    let diagnostics = analisar(source);
    assert!(
        diagnostics.is_empty(),
        "diagnósticos inesperados: {:#?}\nfonte:\n{}",
        diagnostics,
        source
    );
}

/// Exatamente os erros listados, na ordem.
fn assert_erros(source: &str, esperados: &[&str]) {
    let erros = erros(source);
    assert_eq!(
        codigos(&erros),
        esperados,
        "erros: {:#?}\nfonte:\n{}",
        erros,
        source
    );
}

/// Ao menos um erro, e o primeiro tem o código dado.
fn assert_primeiro_erro(source: &str, esperado: &str) {
    let erros = erros(source);
    assert!(
        !erros.is_empty(),
        "esperava erro {esperado}\nfonte:\n{source}"
    );
    assert_eq!(erros[0].code, esperado, "erros: {:#?}", erros);
}

fn assert_tem_erro(source: &str) {
    assert!(
        !erros(source).is_empty(),
        "esperava ao menos um erro\nfonte:\n{source}"
    );
}

// --- AnalisadorSemanticoTest.java -------------------------------------------

#[test]
fn variavel_declarada_no_final_do_codigo() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                escreva(x)
            }
            inteiro x = 1
        }
        "#,
        &[],
    );
}

#[test]
fn funcao_inicio_inexistente() {
    assert_erros(
        r#"
        programa {
            funcao teste() {
                escreva("Olá Mundo")
            }
        }
        "#,
        &["sem.missing-entry-point"],
    );
}

#[test]
fn vetor_com_tamanho_expressao() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                const inteiro x = 2
                inteiro y = 5
                inteiro vet[1 + x + y]
            }
        }
        "#,
        &["sem.array-size-not-constant"],
    );
}

#[test]
fn escreva_com_funcao_sem_retorno() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                escreva("Olá Mundo", nada())
            }
            funcao nada() {
            }
        }
        "#,
        &["sem.argument-type"],
    );
}

#[test]
fn variavel_nao_declarada_no_leia() {
    assert_erros(
        "programa { funcao inicio() { leia(cs_as) } }",
        &["sem.symbol-not-declared"],
    );
}

#[test]
fn variavel_nao_declarada_no_escreva() {
    assert_erros(
        "programa { funcao inicio() { escreva(cs_as) } }",
        &["sem.symbol-not-declared"],
    );
}

#[test]
fn variavel_nao_declarada() {
    assert_erros(
        "programa { funcao inicio() { x = 1 } }",
        &["sem.symbol-not-declared"],
    );
}

#[test]
fn inclusao_biblioteca_duplicada() {
    assert_primeiro_erro(
        r#"
        programa {
            inclua biblioteca Graficos
            inclua biblioteca Graficos
            funcao inicio() {}
        }
        "#,
        "sem.duplicate-library",
    );
}

#[test]
fn funcao_com_nome_de_variavel_global() {
    assert_primeiro_erro(
        r#"
        programa {
            inteiro teste
            funcao teste() {}
            funcao inicio() {}
        }
        "#,
        "sem.redeclared",
    );
}

#[test]
fn chamada_funcao_inexistente() {
    assert_primeiro_erro(
        "programa { funcao inicio() { teste() } }",
        "sem.symbol-not-declared",
    );
}

#[test]
fn para_multideclarado_fora_de_escopo() {
    assert_tem_erro(
        r#"
        programa {
            funcao inicio() {
                para (inteiro i = 0, j = 0, k = 5, l; i < 5; i++) {
                    l = 1
                    escreva(i + j + k + l)
                }
                escreva(j)
            }
        }
        "#,
    );
}

#[test]
fn para_multideclarado_loop_variaveis() {
    assert_tem_erro(
        r#"
        programa {
            funcao inicio() {
                para (inteiro i = k, j = i, k = j; i < 5; i++) {
                    escreva(i + " " + j + " " + k + "\n")
                }
            }
        }
        "#,
    );
}

#[test]
fn para_multideclarado_funcional() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                para (inteiro i = 0, j = 0, k = 5, l; i < 5; i++) {
                    para (j = 1; j < 5; i++) {
                        j = 5
                    }
                    para (k = 5; k < 5; k++) {
                    }
                    para (l = 0; l < 5; l++) {
                    }
                    escreva("i=" + i + ";j=" + j + ";k=" + k + ";l=" + l + "\n")
                }
                para (inteiro i = 2, j = i, k = j; i < 5; i++) {
                    escreva(i + " " + j + " " + k + "\n")
                }
            }
        }
        "#,
    );
}

#[test]
fn leia_com_funcao() {
    assert_tem_erro(
        r#"
        programa {
            funcao inicio() {
                leia(teste())
            }
            funcao teste() {}
        }
        "#,
    );
}

#[test]
fn leia_com_constante_de_biblioteca() {
    assert_tem_erro(
        r#"
        programa {
            inclua biblioteca Graficos
            funcao inicio() {
                leia(Graficos.COR_AMARELO)
            }
        }
        "#,
    );
}

#[test]
fn leia_com_variavel_constante() {
    assert_tem_erro(
        r#"
        programa {
            funcao inicio() {
                const inteiro x = 1
                leia(x)
            }
        }
        "#,
    );
}

#[test]
fn leia_com_constante() {
    assert_tem_erro("programa { funcao inicio() { leia(10) } }");
}

#[test]
fn leia_sem_parametros() {
    assert_tem_erro("programa { funcao inicio() { leia() } }");
}

#[test]
fn leia_com_matriz() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro a[3][2]
                leia(a[0][0])
            }
        }
        "#,
    );
}

#[test]
fn leia_com_vetor() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro a[3]
                leia(a[0])
            }
        }
        "#,
    );
}

#[test]
fn leia_com_variavel() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro a
                leia(a)
            }
        }
        "#,
    );
}

#[test]
fn leia_com_varias_variaveis() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro a, b, c
                leia(a, b, c)
            }
        }
        "#,
    );
}

#[test]
fn referencia_matriz_com_indice_real() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a[2][2]
                real x = 1.0
                escreva(a[x][0])
            }
        }
        "#,
        &["sem.invalid-index"],
    );
}

#[test]
fn referencia_vetor_com_indice_cadeia() {
    let erros = erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a[] = {1, 2, 3}
                escreva(a["1"])
            }
        }
        "#,
    );
    assert_eq!(codigos(&erros), ["sem.invalid-index"]);
    assert!(erros[0].message.contains("cadeia"), "{}", erros[0].message);
}

#[test]
fn menos_unario_em_cadeia() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                cadeia a = "teste"
                escreva(-a)
            }
        }
        "#,
        &["sem.incompatible-types"],
    );
}

#[test]
fn nao_em_cadeia() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                cadeia a = "teste"
                se (nao a) {
                    escreva("nao a")
                }
            }
        }
        "#,
        &["sem.incompatible-types"],
    );
}

#[test]
fn variavel_nao_inicializada() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a, b
                escreva(a)
                b = a + 2
                a = a + b
                escreva(b)
                a = 1
                escreva(a)
            }
        }
        "#,
        &[
            "sem.symbol-not-initialized",
            "sem.symbol-not-initialized",
            "sem.symbol-not-initialized",
        ],
    );
}

#[test]
fn soma_inteiro_com_logico() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a = 1
                logico b = verdadeiro
                b = a + b
            }
        }
        "#,
        &["sem.incompatible-types"],
    );
}

#[test]
fn variavel_nao_declarada_em_soma() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a = 0, b = 1
                a = a + c
            }
        }
        "#,
        &["sem.symbol-not-declared"],
    );
}

#[test]
fn escolha_codigo_correto() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                inteiro a = 1
                escolha (a) {
                    caso 2:
                        pare
                    caso 1:
                        pare
                    caso contrario:
                        pare
                }
            }
        }
        "#,
        &[],
    );
}

#[test]
fn escolha_logico_com_caso_real() {
    let erros = erros(
        r#"
        programa {
            funcao inicio() {
                logico a = verdadeiro
                escolha (a) {
                    caso 2.0:
                        pare
                    caso 'a':
                        pare
                }
            }
        }
        "#,
    );
    assert_eq!(erros.len(), 2, "{:#?}", erros);
    assert_eq!(erros[0].code, "sem.switch-type");
    assert!(erros[0].message.contains("logico"), "{}", erros[0].message);
    assert_eq!(erros[1].code, "sem.case-type");
    assert!(erros[1].message.contains("real"), "{}", erros[1].message);
}

#[test]
fn caso_deve_ter_tipo_do_escolha() {
    let erros = erros(
        r#"
        programa {
            funcao inicio() {
                caracter a = 'a'
                escolha (a) {
                    caso 1:
                        pare
                    caso 'a':
                        pare
                }
            }
        }
        "#,
    );
    assert_eq!(codigos(&erros), ["sem.case-type"], "{:#?}", erros);
    assert!(
        erros[0].message.contains("caracter"),
        "{}",
        erros[0].message
    );
}

#[test]
fn escolha_so_aceita_inteiro_ou_caracter() {
    assert_erros(
        r#"
        programa {
            funcao inicio() {
                cadeia a = "a"
                escolha (a) {
                    caso 1:
                        pare
                    caso 'a':
                        pare
                }
            }
        }
        "#,
        &["sem.switch-type"],
    );
}

#[test]
fn escolha_aninhado_com_tipos_diferentes() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro a = 1
                caracter b = 'a'
                escolha (a) {
                    caso 1:
                        escolha (b) {
                            caso 'a':
                                escreva("\nletra a")
                                pare
                            caso 'b':
                                escreva("\nletra B")
                                pare
                        }
                        escreva("1")
                        pare
                    caso 2:
                        escolha (b) {
                            caso 'a':
                                escolha (a) {
                                    caso 1:
                                        escreva("teste")
                                        pare
                                }
                                pare
                            caso 'b':
                                escreva("Erro")
                                pare
                        }
                        escreva("2")
                        pare
                }
            }
        }
        "#,
    );
}

// --- Extras ------------------------------------------------------------------

#[test]
fn constante_exige_inicializacao() {
    assert_primeiro_erro(
        "programa { funcao inicio() { const inteiro x } }",
        "sem.const-without-init",
    );
}

#[test]
fn atribuir_a_constante() {
    assert_erros(
        "programa { funcao inicio() { const inteiro x = 1 x = 2 } }",
        &["sem.assign-to-const"],
    );
}

#[test]
fn funcao_com_retorno_precisa_retornar() {
    assert_erros(
        r#"
        programa {
            funcao inteiro f(inteiro x) {
                se (x > 0) {
                    retorne 1
                }
            }
            funcao inicio() { escreva(f(1)) }
        }
        "#,
        &["sem.missing-return"],
    );
}

#[test]
fn retorno_em_todos_os_ramos() {
    assert_limpo(
        r#"
        programa {
            funcao inteiro f(inteiro x) {
                se (x > 0) {
                    retorne 1
                } senao {
                    retorne 0
                }
            }
            funcao inicio() { escreva(f(1)) }
        }
        "#,
    );
}

#[test]
fn tipo_de_retorno_incompativel() {
    assert_primeiro_erro(
        r#"
        programa {
            funcao inteiro f() { retorne "texto" }
            funcao inicio() { escreva(f()) }
        }
        "#,
        "sem.incompatible-types",
    );
}

#[test]
fn quantidade_de_argumentos() {
    assert_primeiro_erro(
        r#"
        programa {
            funcao f(inteiro a, inteiro b) {}
            funcao inicio() { f(1) }
        }
        "#,
        "sem.wrong-argument-count",
    );
}

#[test]
fn conversao_real_para_inteiro_gera_aviso() {
    let diagnostics = analisar(
        r#"
        programa {
            funcao inicio() {
                inteiro x = 1.5
                escreva(x)
            }
        }
        "#,
    );
    assert!(
        diagnostics.iter().all(|d| d.severity == Severity::Warning),
        "{:#?}",
        diagnostics
    );
    assert!(!diagnostics.is_empty(), "esperava aviso de conversão");
}

#[test]
fn conversao_por_referencia_e_erro() {
    assert_tem_erro(
        r#"
        programa {
            funcao f(inteiro &x) { x = 1 }
            funcao inicio() {
                real r = 1.0
                f(r)
            }
        }
        "#,
    );
}

#[test]
fn redeclaracao_no_mesmo_escopo() {
    assert_erros(
        "programa { funcao inicio() { inteiro x = 1 inteiro x = 2 } }",
        &["sem.redeclared"],
    );
}

#[test]
fn ocultar_global_gera_aviso() {
    let diagnostics = analisar(
        r#"
        programa {
            inteiro x = 1
            funcao inicio() {
                inteiro x = 2
                escreva(x)
            }
        }
        "#,
    );
    assert_eq!(codigos(&diagnostics), ["sem.shadowed"]);
    assert_eq!(diagnostics[0].severity, Severity::Warning);
}

#[test]
fn biblioteca_desconhecida() {
    assert_primeiro_erro(
        "programa { inclua biblioteca Inexistente funcao inicio() {} }",
        "sem.unknown-library",
    );
}

#[test]
fn biblioteca_com_alias() {
    assert_limpo(
        r#"
        programa {
            inclua biblioteca Matematica --> mat
            funcao inicio() {
                escreva(mat.raiz(4.0, 2.0))
            }
        }
        "#,
    );
}

#[test]
fn literais_de_vetor_e_matriz() {
    assert_limpo(
        r#"
        programa {
            funcao inicio() {
                inteiro v[3] = {1, 2, 3}
                inteiro m[2][2] = {{1, 2}, {3, 4}}
                escreva(v[0] + m[1][1])
            }
        }
        "#,
    );
}

#[test]
fn literal_de_vetor_com_tamanho_errado() {
    assert_primeiro_erro(
        "programa { funcao inicio() { inteiro v[2] = {1, 2, 3} escreva(v[0]) } }",
        "sem.array-size-mismatch",
    );
}

#[test]
fn trecho_sem_programa_nao_exige_inicio() {
    assert_limpo(
        r#"
        funcao real dobro(real valor) {
            retorne valor * 2
        }
        "#,
    );
}
