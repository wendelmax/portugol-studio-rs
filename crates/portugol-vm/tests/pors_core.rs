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

#[test]
fn executes_zero_dep_pors_json_and_enforces_limits_in_minimal_vm() {
    let source = r#"
funcao executar() -> Resultado<cadeia, cadeia> {
    seja mutavel doc: cadeia = "{}";
    doc = Json.definir_cadeia(doc, "nome", "Ada \"Lovelace\"\n")?;
    doc = Json.definir_inteiro(doc, "ano", 1843)?;
    doc = Json.definir_real(doc, "nota", 9.5)?;
    doc = Json.definir_logico(doc, "ativo", verdadeiro)?;
    doc = Json.definir_json(doc, "tags", "[\"pors\",\"rust\"]")?;
    doc = Json.definir_json(doc, "nums", "[10, 20, 30]")?;
    doc = Json.definir_json(doc, "meta", "{\"emoji\":\"\\uD83D\\uDE00\"}")?;

    seja nome: cadeia = Json.obter_cadeia(doc, "nome")?;
    seja ano: inteiro = Json.obter_inteiro(doc, "ano")?;
    seja nota: real = Json.obter_real(doc, "nota")?;
    seja ativo: logico = Json.obter_logico(doc, "ativo")?;
    seja tags: vetor<cadeia> = Json.obter_lista_cadeia(doc, "tags")?;
    seja nums: vetor<inteiro> = Json.obter_lista_inteiro(doc, "nums")?;
    seja meta: cadeia = Json.obter_json(doc, "meta")?;
    seja emoji: cadeia = Json.obter_cadeia(meta, "emoji")?;
    seja chaves: vetor<cadeia> = Json.obter_chaves(doc)?;

    seja sem_ativo: cadeia = Json.remover_chave(doc, "ativo")?;
    seja tem_ativo: logico = Json.contem_chave(sem_ativo, "ativo")?;
    seja bonito: cadeia = Json.formatar("{\"a\":1,\"b\":[true,null]}")?;
    seja compacto: cadeia = Json.minificar(bonito)?;

    retorne Resultado::Ok(
        Texto.aparar(nome) + "|" +
        Tipos.inteiro_para_cadeia(ano) + "|" +
        Tipos.real_para_cadeia(nota) + "|" +
        Tipos.logico_para_cadeia(ativo) + "|" +
        tags[1] + "|" +
        Tipos.inteiro_para_cadeia(nums[0] + nums[2]) + "|" +
        emoji + "|" +
        Tipos.inteiro_para_cadeia(tamanho(chaves)) + "|" +
        Tipos.logico_para_cadeia(tem_ativo) + "|" +
        compacto + "|" +
        Json.escapar_cadeia("a\"b")
    );
}

funcao inicio() {
    escolha executar() {
        caso Resultado::Ok(s) => { escreva(s, "|"); }
        caso Resultado::Erro(e) => { escreva("ERR:", e, "|"); }
    }
    escreva(Json.valido("{\"ok\":1}"), "|", Json.valido("{invalido}"), "|");
    escolha Json.obter_inteiro("{\"x\":3.5}", "x") {
        caso Resultado::Ok(_) => { escreva("BAD_INT|"); }
        caso Resultado::Erro(_) => { escreva("INT_ERR|"); }
    }
    seja mutavel fundo: cadeia = "0";
    para (seja mutavel i: inteiro = 0; i < 70; i += 1) {
        fundo = "[" + fundo + "]";
    }
    escreva(Json.valido(fundo));
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução de Json na VM mínima");
    assert_eq!(
        io.output,
        "Ada \"Lovelace\"|1843|9.5|verdadeiro|rust|40|\u{1F600}|7|falso|{\"a\":1,\"b\":[true,null]}|\"a\\\"b\"|verdadeiro|falso|INT_ERR|falso"
    );
}

#[test]
fn executes_bitwise_shifts_and_choose_wildcard_in_minimal_vm() {
    let source = r#"
enum Cor {
    Vermelho,
    Verde,
    Azul
}

funcao rotulo(c: Cor) -> cadeia {
    escolha c {
        caso Cor::Vermelho => { retorne "R"; }
        caso _ => { retorne "OUTRA"; }
    }
}

funcao inicio() {
    seja mutavel bits: inteiro = (1 << 4) | 5;
    bits &= 21;
    bits |= 2;
    bits ^= 1;
    seja shifted: inteiro = bits >> 1;
    seja not_zero: inteiro = ~0;
    escreva(
        bits, "|",
        shifted, "|",
        not_zero, "|",
        rotulo(Cor::Vermelho), "|",
        rotulo(Cor::Azul)
    );
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("execução bitwise e wildcard na VM mínima");
    // bits: (16 | 5) = 21; 21 & 21 = 21; 21 | 2 = 23; 23 ^ 1 = 22; 22 >> 1 = 11; ~0 = -1
    assert_eq!(io.output, "22|11|-1|R|OUTRA");
}

#[test]
fn executes_consented_pors_internet_and_cancels_stalled_requests_in_minimal_vm() {
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;
    use std::time::{Duration, Instant};

    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let server = thread::spawn(move || {
        for _ in 0..4 {
            let Ok((mut stream, _)) = listener.accept() else {
                break;
            };
            let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
            let mut raw = Vec::new();
            let mut chunk = [0u8; 1024];
            let header_end = loop {
                if let Some(pos) = raw.windows(4).position(|w| w == b"\r\n\r\n") {
                    break pos;
                }
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break raw.len(),
                    Ok(n) => raw.extend_from_slice(&chunk[..n]),
                }
            };
            let head = String::from_utf8_lossy(&raw[..header_end]).into_owned();
            let content_len: usize = head
                .lines()
                .find_map(|line| {
                    let (k, v) = line.split_once(':')?;
                    k.eq_ignore_ascii_case("content-length")
                        .then(|| v.trim().parse().ok())
                        .flatten()
                })
                .unwrap_or(0);
            let body_start = (header_end + 4).min(raw.len());
            while raw.len().saturating_sub(body_start) < content_len {
                match stream.read(&mut chunk) {
                    Ok(0) | Err(_) => break,
                    Ok(n) => raw.extend_from_slice(&chunk[..n]),
                }
            }
            let first = head.lines().next().unwrap_or("");
            if first.starts_with("GET /ola ") {
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\nOla PORS!",
                );
            } else if first.starts_with("POST /eco ") {
                let body = &raw[body_start..raw.len().min(body_start + content_len)];
                let resp = format!(
                    "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    body.len(),
                    String::from_utf8_lossy(body)
                );
                let _ = stream.write_all(resp.as_bytes());
            } else if first.starts_with("HEAD /ola ") {
                let _ = stream.write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Length: 9\r\nConnection: close\r\n\r\n",
                );
            } else {
                let _ = stream.write_all(
                    b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                );
            }
        }
    });

    let base = format!("http://{addr}");
    let source = format!(
        r#"
funcao executar() -> Resultado<cadeia, cadeia> {{
    Internet.definir_tempo_limite(2000);
    seja disp_ok: logico = Internet.endereco_disponivel("{base}/ola")?;
    seja disp_404: logico = Internet.endereco_disponivel("{base}/ausente")?;
    seja texto: cadeia = Internet.obter_texto("{base}/ola")?;
    seja eco: cadeia = Internet.enviar_json("{base}/eco", "{{\"ok\":true}}")?;
    retorne Resultado::Ok(
        Tipos.logico_para_cadeia(disp_ok) + "|" +
        Tipos.logico_para_cadeia(disp_404) + "|" +
        texto + "|" +
        eco
    );
}}

funcao inicio() {{
    escolha executar() {{
        caso Resultado::Ok(s) => {{ escreva(s, "|"); }}
        caso Resultado::Erro(e) => {{ escreva("ERR:", e, "|"); }}
    }}
    escolha Internet.obter_texto("file:///etc/passwd") {{
        caso Resultado::Ok(_) => {{ escreva("LEAK|"); }}
        caso Resultado::Erro(_) => {{ escreva("BLOCKED_SCHEME|"); }}
    }}
    escolha Internet.enviar_json("{base}/eco", "{{invalido}}") {{
        caso Resultado::Ok(_) => {{ escreva("BAD_JSON"); }}
        caso Resultado::Erro(_) => {{ escreva("INVALID_JSON"); }}
    }}
}}
"#
    );
    let module = compile_pors_source(&source);

    // Sem consentimento explícito, deve recusar acesso à rede com Resultado::Erro.
    let mut denied_io = BufferIo::default();
    run(&module, &mut denied_io).expect("sem consentimento deve retornar Resultado::Erro");
    assert!(denied_io.output.starts_with("ERR:Acesso à rede bloqueado"));

    // Com consentimento explícito, executa GET, POST JSON, HEAD e bloqueia esquema/JSON inválido.
    let mut io = BufferIo::default();
    Vm::new(&module, &mut io)
        .with_pors_network_enabled(true)
        .run()
        .expect("execução autorizada de Internet na VM mínima");
    assert_eq!(
        io.output,
        "verdadeiro|falso|Ola PORS!|{\"ok\":true}|BLOCKED_SCHEME|INVALID_JSON"
    );
    let _ = server.join();

    // Cancelamento cooperativo durante requisição travada (nunca bloqueia debugger).
    let stall_listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let stall_addr = stall_listener.local_addr().unwrap();
    let _stall = thread::spawn(move || {
        if let Ok((_stream, _)) = stall_listener.accept() {
            thread::sleep(Duration::from_secs(5));
        }
    });
    let stall_src = format!(
        "funcao inicio() {{ Internet.definir_tempo_limite(10000); escolha Internet.obter_texto(\"http://{stall_addr}/slow\") {{ caso _ => {{}} }} }}"
    );
    let stall_module = compile_pors_source(&stall_src);
    let interrupt = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&interrupt);
    thread::spawn(move || {
        thread::sleep(Duration::from_millis(80));
        flag.store(true, Ordering::Relaxed);
    });
    let started = Instant::now();
    let mut stall_io = BufferIo::default();
    let err = Vm::new(&stall_module, &mut stall_io)
        .with_pors_network_enabled(true)
        .with_interrupt(interrupt)
        .run()
        .expect_err("requisição travada deve ser cancelada quando interrupt=true");
    assert!(err.message.contains("interrompido"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn executes_do_while_with_break_and_continue_in_minimal_vm() {
    let source = r#"
funcao inicio() {
    seja mutavel vezes: inteiro = 0;
    faca {
        vezes += 1;
    } enquanto (falso);

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

    escreva(vezes, "|", i, "|", soma);
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("faca-enquanto deve executar na VM mínima");
    // vezes=1 (executa ao menos uma vez com cond falsa); i=4; soma=1+3+4=8
    assert_eq!(io.output, "1|4|8");
}

#[cfg(feature = "stdlib")]
#[test]
fn executes_pors_multimedia_bridge_with_mock_host_and_workspace_confinement() {
    use std::path::Path;
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    use portugol_vm::graphics::{InputState, WindowCommand};
    use portugol_vm::{Host, Sound};

    #[derive(Default)]
    struct MockLog {
        opened: Vec<(i32, i32)>,
        commands: Vec<WindowCommand>,
        frames: Vec<Vec<u8>>,
        closed: usize,
        sounds_loaded: Vec<String>,
        sounds_played: usize,
    }

    struct MockSound {
        log: Arc<Mutex<MockLog>>,
        pos_ms: u64,
    }

    impl Sound for MockSound {
        fn play(&mut self, _looping: bool) -> Result<(), String> {
            self.log.lock().unwrap().sounds_played += 1;
            Ok(())
        }
        fn pause(&mut self) {}
        fn stop(&mut self) {}
        fn duration(&self) -> Duration {
            Duration::from_millis(1200)
        }
        fn position(&self) -> Duration {
            Duration::from_millis(self.pos_ms)
        }
        fn seek(&mut self, position: Duration) -> Result<(), String> {
            self.pos_ms = position.as_millis() as u64;
            Ok(())
        }
        fn set_gain(&mut self, _gain: f32) {}
    }

    struct MockHost {
        log: Arc<Mutex<MockLog>>,
        input: Arc<InputState>,
    }

    impl Host for MockHost {
        fn load_sound(&mut self, path: &Path) -> Result<Box<dyn Sound>, String> {
            self.log
                .lock()
                .unwrap()
                .sounds_loaded
                .push(path.to_string_lossy().into_owned());
            Ok(Box::new(MockSound {
                log: Arc::clone(&self.log),
                pos_ms: 0,
            }))
        }
        fn graphics_open(&mut self, width: i32, height: i32) -> Result<(), String> {
            self.log.lock().unwrap().opened.push((width, height));
            Ok(())
        }
        fn graphics_close(&mut self) {
            self.log.lock().unwrap().closed += 1;
        }
        fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
            self.log.lock().unwrap().frames.push(frame);
            Ok(())
        }
        fn graphics_window(&mut self, command: WindowCommand) -> Result<(), String> {
            self.log.lock().unwrap().commands.push(command);
            Ok(())
        }
        fn screen_size(&mut self) -> Result<(i32, i32), String> {
            Ok((1920, 1080))
        }
        fn input(&mut self) -> Option<Arc<InputState>> {
            Some(Arc::clone(&self.input))
        }
    }

    let tmp_dir = std::env::temp_dir().join(format!(
        "pors_multimedia_bridge_test_{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&tmp_dir);
    std::fs::create_dir_all(&tmp_dir).unwrap();
    std::fs::write(tmp_dir.join("beep.wav"), b"RIFFdemo").unwrap();

    let log = Arc::new(Mutex::new(MockLog::default()));
    let input = Arc::new(InputState::default());
    input.key_pressed(65); // TECLA_A
    input.mouse_moved(123, 45);
    input.button_pressed(0); // BOTAO_ESQUERDO

    let source = r#"
funcao inicio() {
    Graficos.iniciar_modo_grafico(verdadeiro);
    Graficos.definir_dimensoes_janela(400, 300);
    Graficos.definir_titulo_janela("PORS Host");
    seja cor_res: Resultado<inteiro, cadeia> = Graficos.criar_cor(12, 34, 56);
    escolha cor_res {
        caso Resultado::Ok(c) => {
            Graficos.definir_cor(c);
            escreva("rgb:", Graficos.obter_rgb(c, Graficos.CANAL_R), ",", Graficos.obter_rgb(c, Graficos.CANAL_G), ",", Graficos.obter_rgb(c, Graficos.CANAL_B), "|");
        }
        caso Resultado::Erro(e) => { escreva("err:", e, "|"); }
    }
    seja xs: vetor<inteiro> = [0, 20, 10];
    seja ys: vetor<inteiro> = [0, 0, 20];
    escolha Graficos.desenhar_poligono(xs, ys, verdadeiro) {
        caso Resultado::Ok(_) => { escreva("poly:ok|"); }
        caso Resultado::Erro(e) => { escreva("poly:", e, "|"); }
    }
    Graficos.desenhar_retangulo(10, 20, 30, 40, falso, verdadeiro);
    Graficos.renderizar();
    escreva("win:", Graficos.largura_janela(), "x", Graficos.altura_janela(), "|");
    escreva("key:", Teclado.tecla_pressionada(Teclado.TECLA_A), ",", Teclado.caracter_tecla(Teclado.TECLA_A), "|");
    escreva("mouse:", Mouse.botao_pressionado(Mouse.BOTAO_ESQUERDO), ",", Mouse.posicao_x(), ",", Mouse.posicao_y(), "|");

    escolha Sons.carregar_som("beep.wav") {
        caso Resultado::Ok(id) => {
            escolha Sons.reproduzir_som(id, falso) {
                caso Resultado::Ok(_) => { escreva("som:ok|"); }
                caso Resultado::Erro(e) => { escreva("som:", e, "|"); }
            }
            Sons.definir_volume(75);
            escreva("vol:", Sons.obter_volume(), "|");
            Sons.liberar_som(id);
        }
        caso Resultado::Erro(e) => { escreva("som:", e, "|"); }
    }

    escolha Sons.carregar_som("../fora.wav") {
        caso Resultado::Ok(_) => { escreva("esc:ok"); }
        caso Resultado::Erro(_) => { escreva("esc:bloqueado"); }
    }
    Graficos.encerrar_modo_grafico();
}
"#;

    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    Vm::new(&module, &mut io)
        .with_working_dir(&tmp_dir)
        .with_host(Box::new(MockHost {
            log: Arc::clone(&log),
            input: Arc::clone(&input),
        }))
        .run()
        .expect("ponte multimídia PORS deve executar com sucesso no host");

    assert_eq!(
        io.output,
        "rgb:12,34,56|poly:ok|win:400x300|key:verdadeiro,A|mouse:verdadeiro,123,45|som:ok|vol:75|esc:bloqueado"
    );
    let recorded = log.lock().unwrap();
    assert_eq!(recorded.opened, vec![(640, 480)]);
    assert_eq!(recorded.frames.len(), 1);
    assert_eq!(recorded.closed, 1);
    assert_eq!(recorded.sounds_loaded.len(), 1);
    assert_eq!(recorded.sounds_played, 1);

    let _ = std::fs::remove_dir_all(&tmp_dir);
}

#[cfg(not(feature = "stdlib"))]
#[test]
fn reports_clean_typed_errors_for_pors_multimedia_without_stdlib() {
    let source = r#"
funcao inicio() {
    escolha Graficos.criar_cor(10, 20, 30) {
        caso Resultado::Ok(_) => { escreva("ok"); }
        caso Resultado::Erro(e) => { escreva("res:", e); }
    }
}
"#;
    let module = compile_pors_source(source);
    let mut io = BufferIo::default();
    run(&module, &mut io).expect("função multimídia com Resultado deve retornar Erro sem stdlib");
    assert!(io.output.contains("stdlib"));

    let void_src = "funcao inicio() { Graficos.iniciar_modo_grafico(verdadeiro); }";
    let void_module = compile_pors_source(void_src);
    let mut void_io = BufferIo::default();
    let err = run(&void_module, &mut void_io)
        .expect_err("função multimídia direta sem stdlib deve falhar com mensagem clara");
    assert!(err.message.contains("stdlib"));
}
