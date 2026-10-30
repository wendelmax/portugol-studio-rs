//! Testes de ponta a ponta: fonte -> análise -> bytecode -> VM.
//!
//! As saídas esperadas seguem o que o Java gerado pelo Portugol Studio
//! imprimiria.

use portugol_syntax::{Severity, parse};
use portugol_vm::{BufferIo, RuntimeError, Vm};

fn executar_com(source: &str, input: &str) -> (String, Result<(), RuntimeError>) {
    let result = parse(source);
    let mut diagnostics = result.diagnostics;
    diagnostics.extend(portugol_semantics::analyze(&result.program));
    let errors: Vec<_> = diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .collect();
    assert!(errors.is_empty(), "erros de análise: {errors:#?}\n{source}");

    let module = portugol_bytecode::compile(&result.program).expect("compila");
    let mut io = BufferIo::new(input);
    let outcome = Vm::new(&module, &mut io).with_seed(7).run();
    (io.output, outcome)
}

fn saida(source: &str) -> String {
    saida_com(source, "")
}

fn saida_com(source: &str, input: &str) -> String {
    let (output, outcome) = executar_com(source, input);
    if let Err(error) = outcome {
        panic!(
            "erro de execução: {}\nsaída até aqui: {output:?}",
            error.message
        );
    }
    output
}

fn erro(source: &str) -> RuntimeError {
    let (output, outcome) = executar_com(source, "");
    outcome.expect_err(&format!("esperava erro de execução; saída: {output:?}"))
}

/// Envolve comandos em `programa { funcao inicio() { ... } }`.
fn inicio(body: &str) -> String {
    format!("programa {{ funcao inicio() {{ {body} }} }}")
}

// --- aritmética e tipos -------------------------------------------------

#[test]
fn aritmetica_inteira_e_real() {
    assert_eq!(
        saida(&inicio(
            r#"escreva(7 / 2, " ", 7 % 3, " ", -7 / 2, " ", -7 % 3, " ", 7.0 / 2, " ", 2 * 3 + 1)"#
        )),
        "3 1 -3 -1 3.5 7"
    );
}

#[test]
fn inteiro_estoura_como_int_do_java() {
    assert_eq!(
        saida(&inicio(
            r#"inteiro x = 2147483647
               x = x + 1
               escreva(x, " ", -2147483647 - 1 - 1)"#
        )),
        "-2147483648 2147483647"
    );
}

#[test]
fn divisao_inteira_por_zero_e_erro() {
    let error = erro(&inicio("inteiro z = 0 escreva(1 / z)"));
    assert_eq!(error.message, "Foi efetuada uma divisão por zero.");
    let error = erro(&inicio("inteiro z = 0 escreva(1 % z)"));
    assert_eq!(error.message, "Foi efetuada uma divisão por zero.");
}

#[test]
fn divisao_real_por_zero_da_infinito() {
    assert_eq!(
        saida(&inicio(r#"real z = 0.0 escreva(1.0 / z, " ", -1 / z)"#)),
        "Infinity -Infinity"
    );
}

#[test]
fn formato_de_real_igual_ao_java() {
    assert_eq!(
        saida(&inicio(
            r#"escreva(1.0, " ", 0.1 + 0.2, " ", 10000000.0, " ", 0.0001, " ", 100.0 / 3)"#
        )),
        "1.0 0.30000000000000004 1.0E7 1.0E-4 33.333333333333336"
    );
}

#[test]
fn conversao_na_atribuicao_trunca() {
    assert_eq!(
        saida(&inicio(
            r#"inteiro i = 3.99
               real r = 7 / 2
               inteiro j = -2.5
               i += 1.5
               escreva(i, " ", r, " ", j)"#
        )),
        "4 3.0 -2"
    );
}

#[test]
fn logico_escreve_em_portugues_mas_concatena_como_java() {
    assert_eq!(
        saida(&inicio(
            r#"logico b = verdadeiro
               escreva(b, " ", "valor: " + b, " ", nao b)"#
        )),
        "verdadeiro valor: true falso"
    );
}

#[test]
fn concatenacao_de_tipos() {
    assert_eq!(
        saida(&inicio(
            r#"caracter c = 'x'
               escreva("n" + 1 + 2, " ", 1 + 2 + "n", " ", "c=" + c, " ", "r=" + 2.5)"#
        )),
        "n12 3n c=x r=2.5"
    );
}

#[test]
fn comparacao_de_cadeias_por_valor() {
    assert_eq!(
        saida(&inicio(
            r#"cadeia a = "ab"
               cadeia b = "a" + "b"
               escreva(a == b, " ", a != b)"#
        )),
        "verdadeiro falso"
    );
}

#[test]
fn curto_circuito() {
    assert_eq!(
        saida(
            r#"
            programa {
                funcao logico marca(cadeia s, logico v) { escreva(s) retorne v }
                funcao inicio() {
                    se (marca("a", falso) e marca("b", verdadeiro)) { }
                    se (marca("c", verdadeiro) ou marca("d", verdadeiro)) { }
                    se (marca("e", verdadeiro) e marca("f", falso)) { }
                }
            }
            "#
        ),
        "acef"
    );
}

#[test]
fn operadores_bit_a_bit() {
    assert_eq!(
        saida(&inicio(
            r#"escreva(6 & 3, " ", 6 | 3, " ", 6 ^ 3, " ", ~0, " ", 1 << 4, " ", -16 >> 2)"#
        )),
        "2 7 5 -1 16 -4"
    );
}

// --- controle de fluxo --------------------------------------------------

#[test]
fn lacos_e_pare() {
    assert_eq!(
        saida(&inicio(
            r#"inteiro i = 0
               enquanto (verdadeiro) { i++ se (i == 3) { pare } }
               faca { i-- } enquanto (i > 0)
               para (inteiro j = 0; j < 10; j += 3) { escreva(j) }
               escreva(" ", i)"#
        )),
        "0369 0"
    );
}

#[test]
fn escolha_cai_no_proximo_sem_pare() {
    let programa = |x: i32| {
        inicio(&format!(
            r#"inteiro x = {x}
               escolha (x) {{
                   caso 1: escreva("um ")
                   caso 2: escreva("dois ") pare
                   caso contrario: escreva("outro ")
               }}
               escreva("fim")"#
        ))
    };
    assert_eq!(saida(&programa(1)), "um dois fim");
    assert_eq!(saida(&programa(2)), "dois fim");
    assert_eq!(saida(&programa(9)), "outro fim");
}

#[test]
fn escolha_com_caracter() {
    assert_eq!(
        saida(&inicio(
            r#"caracter c = 'b'
               escolha (c) { caso 'a': escreva("A") pare caso 'b': escreva("B") pare }"#
        )),
        "B"
    );
}

// --- funções e referências ---------------------------------------------

#[test]
fn recursao() {
    assert_eq!(
        saida(
            r#"
            programa {
                funcao inteiro fib(inteiro n) {
                    se (n < 2) { retorne n }
                    retorne fib(n - 1) + fib(n - 2)
                }
                funcao inicio() { escreva(fib(20)) }
            }
            "#
        ),
        "6765"
    );
}

#[test]
fn recursao_infinita_estoura_a_pilha() {
    let error = erro(
        r#"
        programa {
            funcao f() { f() }
            funcao inicio() { f() }
        }
        "#,
    );
    assert!(
        error.message.contains("estouro de pilha"),
        "{}",
        error.message
    );
}

/// Como no original, só variáveis simples podem ser passadas por referência
/// (elementos de vetor são rejeitados pela análise semântica).
#[test]
fn passagem_por_referencia() {
    assert_eq!(
        saida(
            r#"
            programa {
                inteiro g = 1
                funcao troca(inteiro &a, inteiro &b) {
                    inteiro t = a
                    a = b
                    b = t
                }
                funcao incrementa(inteiro &n) { n++ repassa(n) }
                funcao repassa(inteiro &n) { n += 10 }
                funcao inicio() {
                    inteiro x = 1, y = 2
                    troca(x, y)
                    incrementa(g)
                    escreva(x, y, " ", g)
                }
            }
            "#
        ),
        "21 12"
    );
}

#[test]
fn vetores_sao_compartilhados_com_a_funcao() {
    assert_eq!(
        saida(
            r#"
            programa {
                funcao zera(inteiro v[], inteiro m[][]) {
                    v[0] = 0
                    m[1][1] = 0
                }
                funcao inicio() {
                    inteiro v[] = {1, 2}
                    inteiro m[2][2] = {{1, 2}, {3, 4}}
                    zera(v, m)
                    escreva(v[0], v[1], m[0][0], m[1][1])
                }
            }
            "#
        ),
        "0210"
    );
}

#[test]
fn indice_avaliado_uma_vez_em_atribuicao_composta() {
    assert_eq!(
        saida(
            r#"
            programa {
                inteiro chamadas = 0
                funcao inteiro i() { chamadas++ retorne 0 }
                funcao inicio() {
                    inteiro v[1] = {10}
                    v[i()] += 5
                    v[i()]++
                    escreva(v[0], " ", chamadas)
                }
            }
            "#
        ),
        "16 2"
    );
}

#[test]
fn incremento_prefixo_e_sufixo() {
    assert_eq!(
        saida(&inicio(
            r#"inteiro a = 5
               inteiro b = a++
               inteiro c = ++a
               escreva(a, b, c)"#
        )),
        "757"
    );
}

#[test]
fn global_declarada_depois_de_inicio() {
    assert_eq!(
        saida(
            r#"
            programa {
                funcao inicio() { escreva(x) }
                inteiro x = 42
            }
            "#
        ),
        "42"
    );
}

#[test]
fn parametros_de_execucao() {
    let source = r#"
        programa {
            inclua biblioteca Util --> u
            funcao inicio(cadeia params[]) {
                escreva(u.numero_elementos(params), " ", params[1])
            }
        }
    "#;
    let ast = parse(source).program;
    let module = portugol_bytecode::compile(&ast).unwrap();
    let mut io = BufferIo::default();
    Vm::new(&module, &mut io)
        .with_args(vec!["a".into(), "b".into()])
        .run()
        .unwrap();
    assert_eq!(io.output, "2 b");
}

// --- vetores e matrizes -------------------------------------------------

#[test]
fn vetor_e_matriz_com_valores_padrao() {
    assert_eq!(
        saida(&inicio(
            r#"inteiro v[3]
               real m[2][2]
               logico l[1]
               cadeia s[1]
               v[1] = 7
               escreva(v[0], v[1], " ", m[1][1], " ", l[0], " [", s[0], "]")"#
        )),
        "07 0.0 falso []"
    );
}

#[test]
fn indice_invalido_e_erro() {
    let error = erro(&inicio("inteiro v[3] escreva(v[3])"));
    assert!(error.message.contains("índice"), "{}", error.message);
    let error = erro(&inicio("inteiro m[2][2] inteiro i = -1 m[0][i] = 1"));
    assert!(error.message.contains("índice"), "{}", error.message);
}

// --- entrada -----------------------------------------------------------

#[test]
fn leia_de_cada_tipo() {
    assert_eq!(
        saida_com(
            &inicio(
                r#"inteiro i
                   real r
                   cadeia s
                   caracter c
                   logico l
                   inteiro v[2]
                   leia(i, r, s, c, l, v[1])
                   escreva(i + 1, " ", r * 2, " ", s, " ", c, " ", l, " ", v[1])"#
            ),
            "41\n1.5\nolá mundo\nxyz\nverdadeiro\n9\n"
        ),
        "42 3.0 olá mundo x verdadeiro 9"
    );
}

#[test]
fn leia_inteiro_converte_para_real() {
    assert_eq!(
        saida_com(&inicio("real r leia(r) escreva(r)"), "3\n"),
        "3.0"
    );
}

#[test]
fn leia_com_valor_invalido_e_erro() {
    let (_, outcome) = executar_com(&inicio("inteiro i leia(i)"), "abc\n");
    let error = outcome.unwrap_err();
    assert!(
        error
            .message
            .contains("esperava um valor do tipo 'inteiro'"),
        "{}",
        error.message
    );
}

// --- bibliotecas -------------------------------------------------------

#[test]
fn biblioteca_matematica() {
    assert_eq!(
        saida(
            r#"
            programa {
                inclua biblioteca Matematica --> m
                funcao inicio() {
                    escreva(m.potencia(2.0, 10.0), " ", m.raiz(27.0, 3.0), " ",
                            m.arredondar(3.14159, 2), " ", m.valor_absoluto(-4), " ",
                            m.maior_numero(1, 2.5), " ", m.PI)
                }
            }
            "#
        ),
        "1024.0 3.0 3.14 4.0 2.5 3.141592653589793"
    );
}

#[test]
fn biblioteca_texto() {
    assert_eq!(
        saida(
            r#"
            programa {
                inclua biblioteca Texto --> t
                funcao inicio() {
                    cadeia s = "salgado"
                    escreva(t.numero_caracteres(s), " ", t.caixa_alta(s), " ",
                            t.extrair_subtexto(s, 3, 7), " ", t.posicao_texto("ga", s, 0), " ",
                            t.obter_caracter(s, 1), " ", t.preencher_a_esquerda('0', 5, "42"), " ",
                            t.substituir(s, "a", "4"))
                }
            }
            "#
        ),
        "7 SALGADO gado 3 a 00042 s4lg4do"
    );
}

#[test]
fn biblioteca_tipos() {
    assert_eq!(
        saida(
            r#"
            programa {
                inclua biblioteca Tipos --> tp
                funcao inicio() {
                    escreva(tp.cadeia_para_inteiro("ff", 16), " ",
                            tp.inteiro_para_cadeia(255, 16), " ",
                            tp.inteiro_para_cadeia(5, 2), " ",
                            tp.cadeia_e_inteiro("-12", 10), " ",
                            tp.cadeia_e_real("1.5"), " ",
                            tp.cadeia_para_real("2.5") * 2, " ",
                            tp.real_para_inteiro(9.9), " ",
                            tp.logico_para_caracter(verdadeiro))
                }
            }
            "#
        ),
        "255 000000FF 00000000000000000000000000000101 verdadeiro verdadeiro 5.0 9 S"
    );
}

#[test]
fn erro_de_biblioteca_usa_mensagem_do_original() {
    let error = erro(
        r#"
        programa {
            inclua biblioteca Tipos --> tp
            funcao inicio() { escreva(tp.cadeia_para_inteiro("12a", 10)) }
        }
        "#,
    );
    assert_eq!(
        error.message,
        "o valor '12a' não é um número inteiro válido"
    );
}

#[test]
fn sorteia_respeita_o_intervalo() {
    let output = saida(&inicio(
        r#"para (inteiro i = 0; i < 200; i++) {
               inteiro s = sorteia(1, 3)
               se (s < 1 ou s > 3) { escreva("fora") }
           }
           escreva("ok")"#,
    ));
    assert_eq!(output, "ok");
}

#[test]
fn biblioteca_grafica_ainda_nao_disponivel() {
    let error = erro(
        r#"
        programa {
            inclua biblioteca Graficos --> g
            funcao inicio() { g.iniciar_modo_grafico(verdadeiro) }
        }
        "#,
    );
    assert!(
        error
            .message
            .contains("\"Graficos.iniciar_modo_grafico\" ainda não está disponível"),
        "{}",
        error.message
    );
}

// --- execução em fatias ------------------------------------------------

#[test]
fn execucao_em_fatias_e_interrupcao() {
    let source = inicio("inteiro i = 0 enquanto (verdadeiro) { i++ }");
    let ast = parse(&source).program;
    let module = portugol_bytecode::compile(&ast).unwrap();
    let mut io = BufferIo::default();
    let mut vm = Vm::new(&module, &mut io);
    assert_eq!(vm.step(1000).unwrap(), portugol_vm::Status::Running);
    vm.interrupt_handle()
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let error = vm.step(1000).unwrap_err();
    assert_eq!(error.message, "O programa foi interrompido.");
}

#[test]
fn erro_aponta_a_linha_e_a_pilha_de_chamadas() {
    let source = "programa {\n funcao f(inteiro d) {\n  escreva(10 / d)\n }\n funcao inicio() {\n  f(0)\n }\n}";
    let error = erro(source);
    let (line, _) = portugol_syntax::span::line_col(source, error.span.start);
    assert_eq!(line, 3);
    let names: Vec<_> = error.trace.iter().map(|(name, _)| name.as_str()).collect();
    assert_eq!(names, ["f", "inicio", "<programa>"]);
}
