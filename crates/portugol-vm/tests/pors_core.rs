use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use portugol_bytecode::Module;
use portugol_vm::{BufferIo, RuntimeLimitReason, Status, Vm, run};

fn compile_pors_source(source: &str) -> Module {
    let program = portugol_rs_frontend::lower(source).expect("fonte PORS válida");
    portugol_bytecode::compile_pors(&program).expect("bytecode PORS válido")
}

#[test]
fn executes_pors_functions_recursion_loops_and_shadowing_in_minimal_vm() {
    let source = r#"
funcao fatorial(n: inteiro) -> inteiro {
    se (n <= 1) {
        retorne 1;
    } senao {
        retorne n * fatorial(n - 1);
    }
}

funcao inicio() {
    seja valor: inteiro = 5;
    seja mutavel soma: inteiro = 0;
    para (seja mutavel i: inteiro = 1; i <= 3; i += 1) {
        seja valor: inteiro = i * 2;
        soma += valor;
    }
    escreva(fatorial(valor), "|", soma);
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução PORS na VM mínima");
    assert_eq!(io.output, "120|12");
}

#[test]
fn reads_typed_input_through_buffer_io_in_minimal_vm() {
    let source = r#"
funcao inicio() {
    seja mutavel nome: cadeia = "";
    seja mutavel idade: inteiro = 0;
    leia(nome);
    leia(idade);
    escreva(nome, ":", idade + 1);
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::new("Ada\n36\n");
    run(&module, &mut io).expect("entrada tipada em BufferIo");
    assert_eq!(io.output, "Ada:37");
}

#[test]
fn enforces_string_memory_limit_in_minimal_vm() {
    let source = r#"
funcao inicio() {
    seja mutavel texto: cadeia = "x";
    enquanto (verdadeiro) {
        texto += texto;
    }
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    let error = Vm::new(&module, &mut io)
        .with_string_memory_limit(1024)
        .run()
        .expect_err("limite de memória de cadeias deve encerrar a execução");
    assert_eq!(error.limit, Some(RuntimeLimitReason::StringMemory));
}

#[test]
fn supports_cooperative_step_slices_and_interrupt_in_minimal_vm() {
    let source = r#"
funcao inicio() {
    seja mutavel n: inteiro = 0;
    enquanto (verdadeiro) {
        n += 1;
    }
}
"#;
    let module = compile_pors_source(source);
    let interrupt = Arc::new(AtomicBool::new(false));
    let mut io = BufferIo::default();
    let mut vm = Vm::new(&module, &mut io).with_interrupt(Arc::clone(&interrupt));

    assert_eq!(vm.step(200).expect("primeira fatia"), Status::Running);
    interrupt.store(true, Ordering::Relaxed);
    let error = vm
        .step(200)
        .expect_err("sinal de interrupção deve parar a execução");
    assert!(error.message.contains("interrompido"));
}

#[cfg(not(feature = "stdlib"))]
#[test]
fn reports_unavailable_native_libraries_when_stdlib_feature_is_disabled() {
    use portugol_bytecode::{Function, NativeRef, Op, ValueType};
    use portugol_syntax::Span;

    let span = Span::new(0, 1);
    let module = Module {
        functions: vec![Function {
            name: "inicio".into(),
            arity: 0,
            return_type: ValueType::Vazio,
            locals: vec![],
            code: vec![
                Op::CallNative { native: 0, argc: 0 },
                Op::Pop,
                Op::PushVoid,
                Op::Return,
            ],
            spans: vec![span; 4],
            span,
        }],
        entry: 0,
        globals: vec![],
        strings: vec![],
        natives: vec![NativeRef {
            library: "Util".into(),
            name: "tempo_decorrido".into(),
        }],
        native_consts: vec![],
    };
    let mut io = BufferIo::default();
    let error = run(&module, &mut io).expect_err("chamada nativa sem stdlib deve falhar");
    assert!(error.message.contains("Util.tempo_decorrido"));
}

#[test]
fn executes_builtin_option_result_and_try_propagation_in_minimal_vm() {
    let source = r#"
registro Item { valor: inteiro }

funcao buscar(n: inteiro) -> Opcao<Item> {
    se (n > 0) {
        retorne Opcao::Algum(Item { valor: n * 10 });
    }
    retorne Opcao::Nenhum;
}

funcao somar_com_dez(n: inteiro) -> Opcao<inteiro> {
    retorne Opcao::Algum(10 + buscar(n)?.valor);
}

funcao dividir(a: inteiro, b: inteiro) -> Resultado<inteiro, cadeia> {
    se (b == 0) {
        retorne Resultado::Erro("zero");
    }
    retorne Resultado::Ok(a / b);
}

funcao rotular_divisao(a: inteiro, b: inteiro) -> Resultado<cadeia, cadeia> {
    seja total: inteiro = 1 + dividir(a, b)?;
    se (total >= 5) {
        retorne Resultado::Ok("alto");
    }
    retorne Resultado::Ok("baixo");
}

funcao inicio() {
    escolha somar_com_dez(3) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha somar_com_dez(0) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha rotular_divisao(20, 4) {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("E:", e, "|"); }
    }
    escolha rotular_divisao(20, 0) {
        caso Resultado::Ok(s) => { escreva(s); }
        caso Resultado::Erro(e) => { escreva("E:", e); }
    }
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de Opcao, Resultado e ? na VM mínima");
    assert_eq!(io.output, "40|N|alto|E:zero");
}

#[test]
fn executes_pors_matematica_texto_and_tipos_in_minimal_vm() {
    let source = r#"
funcao somar_entradas(a: cadeia, b: cadeia) -> Resultado<inteiro, cadeia> {
    seja x: inteiro = Tipos.para_inteiro(Texto.aparar(a))?;
    seja y: inteiro = Tipos.para_inteiro_base(b, 16)?;
    retorne Resultado::Ok(x + y);
}

funcao inicio() {
    seja pi2: real = Matematica.arredondar(Matematica.PI, 2);
    seja pot: real = Matematica.potencia(2.0, 3.0);
    seja raiz: real = Matematica.raiz_quadrada(81.0);
    seja abs_i: inteiro = Matematica.valor_absoluto_inteiro(-7);
    seja max_i: inteiro = Matematica.maior_inteiro(3, 9);
    escreva(pi2, "|", pot, "|", raiz, "|", abs_i, "|", max_i, "|");

    seja frase: cadeia = "  olá 🦀  ";
    seja limpa: cadeia = Texto.aparar(frase);
    seja alta: cadeia = Texto.caixa_alta(limpa);
    seja trocada: cadeia = Texto.substituir(limpa, "olá", "oi");
    seja pad: cadeia = Texto.preencher_a_esquerda('0', 5, "42");
    escreva(Texto.numero_caracteres(limpa), "|", alta, "|", trocada, "|", pad, "|");

    escolha Texto.obter_caracter(limpa, 4) {
        caso Opcao::Algum(c) => { escreva(c, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha Texto.posicao_texto("🦀", limpa, 0) {
        caso Opcao::Algum(pos) => { escreva(pos, "|"); }
        caso Opcao::Nenhum => { escreva("-1|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 0, 3) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Texto.extrair_subtexto(limpa, 2, 99) {
        caso Resultado::Ok(sub) => { escreva(sub, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }

    escolha somar_entradas(" 26 ", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha somar_entradas("abc", "0x10") {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.para_real("2.5") {
        caso Resultado::Ok(r) => { escreva(Tipos.real_para_cadeia(r), "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Tipos.inteiro_para_cadeia_base(255, 16) {
        caso Resultado::Ok(hex) => { escreva(hex, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Tipos.cadeia_e_inteiro("123"), "|", Tipos.cadeia_e_logico("verdadeiro"));
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de Matematica, Texto e Tipos na VM mínima");
    assert_eq!(
        io.output,
        "3.14|8.0|9.0|7|9|5|OLÁ 🦀|oi 🦀|00042|🦀|4|olá|ERR|42|ERR|2.5|FF|verdadeiro|verdadeiro"
    );
}

#[test]
fn enforces_string_memory_budget_for_pors_texto_allocations() {
    let pad_source = r#"
funcao inicio() {
    seja grande: cadeia = Texto.preencher_a_esquerda('x', 2048, "a");
    escreva(grande);
}
"#;
    let module = compile_pors_source(pad_source);
    let mut io = BufferIo::default();
    let error = Vm::new(&module, &mut io)
        .with_string_memory_limit(512)
        .run()
        .expect_err("preencher_a_esquerda acima do limite deve falhar");
    assert_eq!(error.limit, Some(RuntimeLimitReason::StringMemory));

    let replace_source = r#"
funcao inicio() {
    seja grande: cadeia = Texto.substituir("aaaaaaaaaa", "a", "012345678901234567890123456789012345678901234567890123456789");
    escreva(grande);
}
"#;
    let module = compile_pors_source(replace_source);
    let mut io = BufferIo::default();
    let error = Vm::new(&module, &mut io)
        .with_string_memory_limit(256)
        .run()
        .expect_err("substituir acima do limite deve falhar");
    assert_eq!(error.limit, Some(RuntimeLimitReason::StringMemory));
}
