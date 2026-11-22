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
