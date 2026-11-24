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

#[test]
fn executes_dynamic_vectors_safe_get_pop_and_texto_dividir_in_minimal_vm() {
    let source = r#"
registro Ponto { x: inteiro }

funcao somar_topo_e_primeiro(mutavel itens: vetor<inteiro>) -> Opcao<inteiro> {
    seja topo: inteiro = remover(itens)?;
    seja primeiro: inteiro = obter(itens, 0)?;
    retorne Opcao::Algum(topo + primeiro);
}

funcao inicio() {
    seja mutavel nums: vetor<inteiro> = [];
    escreva(tamanho(nums), "|");
    adicionar(nums, 5);
    adicionar(nums, 15);
    adicionar(nums, 25);
    escreva(tamanho(nums), "|");
    escolha somar_topo_e_primeiro(nums) {
        caso Opcao::Algum(s) => { escreva(s, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    remover(nums);
    escreva(tamanho(nums), "|");
    escolha obter(nums, -1) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }
    escolha obter(nums, 99) {
        caso Opcao::Algum(v) => { escreva(v, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja mutavel pontos: vetor<Ponto> = [];
    adicionar(pontos, Ponto { x: 7 });
    escolha obter(pontos, 0) {
        caso Opcao::Algum(p) => { escreva(p.x, "|"); }
        caso Opcao::Nenhum => { escreva("N|"); }
    }

    seja partes: vetor<cadeia> = Texto.dividir("um,dois,três", ",");
    escreva(tamanho(partes), "|", partes[1], "|");
    seja letras: vetor<cadeia> = Texto.dividir("a🦀", "");
    escreva(tamanho(letras), "|", letras[1]);
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de vetores dinâmicos e Texto.dividir na VM mínima");
    assert_eq!(io.output, "0|3|30|2|N|N|7|3|dois|2|🦀");
}

#[test]
fn executes_pors_util_and_calendario_deterministically_in_minimal_vm() {
    // 2026-11-23T15:30:45.123Z = 1_795_447_845_123 ms? Let's use 1_700_000_000_500 ms:
    // 1700000000 s = 19675 days + 79600 s => 2023-11-14T22:13:20.500Z (Tuesday = 3)
    const EPOCH_MS: u64 = 1_700_000_000_500;
    let source = r#"
funcao inicio() {
    Util.definir_semente(42);
    escolha Util.sorteia(1, 100) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Util.sorteia(10, 5) {
        caso Resultado::Ok(n) => { escreva(n, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escreva(Util.tempo_decorrido(), "|");
    Util.aguarde(1500);
    escreva(Util.tempo_decorrido(), "|");

    escreva(
        Calendario.data_iso_atual(), "|",
        Calendario.ano_atual(), "-",
        Calendario.mes_atual(), "-",
        Calendario.dia_mes_atual(), "|",
        Calendario.dia_semana_atual(), "|",
        Calendario.hora_atual(falso), ":",
        Calendario.hora_atual(verdadeiro), ":",
        Calendario.minuto_atual(), ":",
        Calendario.segundo_atual(), ".",
        Calendario.milisegundo_atual(), "|",
        Calendario.eh_bissexto(2024), "|",
        Calendario.DIA_TERCA_FEIRA, "|",
        Calendario.MES_NOVEMBRO, "|"
    );
    escolha Calendario.dia_semana_completo(Calendario.dia_semana_atual(), falso, falso) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.nome_mes(Calendario.mes_atual()) {
        caso Resultado::Ok(m) => { escreva(m, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, Calendario.MES_FEVEREIRO) {
        caso Resultado::Ok(d) => { escreva(d, "|"); }
        caso Resultado::Erro(_) => { escreva("ERR|"); }
    }
    escolha Calendario.dias_no_mes(2024, 13) {
        caso Resultado::Ok(d) => { escreva(d); }
        caso Resultado::Erro(_) => { escreva("ERR"); }
    }
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    Vm::new(&module, &mut io)
        .with_pors_simulated_time(EPOCH_MS)
        .run()
        .expect("execução determinística de Util e Calendario na VM mínima");

    let mut io_repeat = BufferIo::default();
    Vm::new(&module, &mut io_repeat)
        .with_pors_simulated_time(EPOCH_MS)
        .run()
        .expect("repetição determinística");
    assert_eq!(io.output, io_repeat.output);
    assert!(
        io.output.ends_with(
            "|ERR|0|1500|2023-11-14|2023-11-14|3|22:10:13:22.0|verdadeiro|3|11|Terça-Feira|Novembro|29|ERR"
        ),
        "saída inesperada: {}",
        io.output
    );
}

#[test]
fn interrupts_pors_util_aguarde_promptly_when_cancelled() {
    let source = "funcao inicio() { Util.aguarde(5000); }";
    let module = compile_pors_source(source);
    let interrupt = Arc::new(AtomicBool::new(true));
    let mut io = BufferIo::default();
    let error = Vm::new(&module, &mut io)
        .with_interrupt(interrupt)
        .run()
        .expect_err("Util.aguarde deve ser cancelado imediatamente quando interrupt=true");
    assert!(error.message.contains("interrompido"));
}

#[test]
fn executes_workspace_confined_pors_arquivos_and_blocks_escapes_in_minimal_vm() {
    let workspace =
        std::env::temp_dir().join(format!("portugol-pors-arquivos-vm-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).unwrap();

    let source = r#"
funcao ciclo() -> Resultado<cadeia, cadeia> {
    seja _w: logico = Arquivos.escrever_texto("sub/notas.txt", "alpha\n")?;
    seja _a: logico = Arquivos.acrescentar_texto("sub/notas.txt", "beta\n")?;
    seja tem: logico = Arquivos.existe("sub/notas.txt")?;
    seja linhas: vetor<cadeia> = Arquivos.ler_linhas("sub/notas.txt")?;
    seja itens: vetor<cadeia> = Arquivos.listar("sub")?;
    seja texto: cadeia = Arquivos.ler_texto("sub/notas.txt")?;
    seja _r: logico = Arquivos.remover("sub/notas.txt")?;
    seja depois: logico = Arquivos.existe("sub/notas.txt")?;
    retorne Resultado::Ok(
        Tipos.logico_para_cadeia(tem) + "|" +
        Tipos.inteiro_para_cadeia(tamanho(linhas)) + "|" +
        linhas[1] + "|" +
        itens[0] + "|" +
        Tipos.logico_para_cadeia(depois) + "|" +
        Texto.aparar(texto)
    );
}

funcao inicio() {
    escolha ciclo() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escolha Arquivos.ler_texto("../fora.txt") {
        caso Resultado::Ok(_) => { escreva("LEAK|"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_PARENT|"); }
    }
    escolha Arquivos.ler_texto("/etc/passwd") {
        caso Resultado::Ok(_) => { escreva("LEAK"); }
        caso Resultado::Erro(_) => { escreva("BLOCKED_ABS"); }
    }
}
"#;
    let module = compile_pors_source(source);

    // Sem workspace configurado, deve recusar acesso com erro tipado (consentimento explícito).
    let mut no_ws_io = BufferIo::default();
    run(&module, &mut no_ws_io).expect("sem workspace deve retornar Resultado::Erro");
    assert!(no_ws_io.output.starts_with("ERR:Acesso a arquivos requer"));

    // Com workspace configurado, executa ciclo completo e bloqueia escapes.
    let mut io = BufferIo::default();
    Vm::new(&module, &mut io)
        .with_pors_workspace_dir(&workspace)
        .run()
        .expect("execução confinada de Arquivos na VM mínima");
    assert_eq!(
        io.output,
        "verdadeiro|2|beta|notas.txt|falso|alpha\nbeta|BLOCKED_PARENT|BLOCKED_ABS"
    );

    let _ = std::fs::remove_dir_all(&workspace);
}
