use ferro_service::{
    RustSourceRunPhase, RustSourceRunResult, run_rust_source, run_rust_source_with_args_and_cancel,
    run_rust_source_with_cancel,
};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

#[test]
fn compiles_and_executes_a_standalone_rust_source() {
    let source = r#"fn main() {
    println!("saída real");
    eprintln!("erro real");
    std::process::exit(7);
}"#;

    let result = run_rust_source(source).expect("rustc deve estar instalado nos testes");
    assert!(matches!(
        result,
        RustSourceRunResult::Executed {
            stdout,
            stderr,
            exit_code: Some(7),
        } if stdout == "saída real\n" && stderr == "erro real\n"
    ));
}

#[test]
fn reports_rustc_error_at_the_utf16_source_position() {
    let source = "fn main() {\n    let sorriso = \"😃\";\n    let valor: i32 = \"texto\";\n}";
    let byte_offset = source.find("\"texto\"").unwrap();
    let expected = source[..byte_offset].encode_utf16().count();

    let result = run_rust_source(source).expect("rustc deve estar instalado nos testes");
    let RustSourceRunResult::CompileError { diagnostics, .. } = result else {
        panic!("o tipo incompatível precisa gerar diagnóstico de compilação");
    };
    let diagnostic = diagnostics
        .iter()
        .find(|diagnostic| diagnostic.severity == "error")
        .expect("diagnóstico de erro do rustc");

    assert_eq!(diagnostic.start.offset as usize, expected);
    assert!(diagnostic.message.contains("mismatched types"));
}

#[test]
fn stops_an_infinite_rust_program_at_the_runtime_limit() {
    let result =
        run_rust_source("fn main() { loop {} }").expect("rustc deve estar instalado nos testes");

    assert!(matches!(
        result,
        RustSourceRunResult::RuntimeLimit {
            reason: ferro_service::RustSourceLimitReason::Time,
            ..
        }
    ));
}

#[test]
fn cancels_a_running_rust_program_and_reports_runtime_phase() {
    let marker = std::env::temp_dir().join(format!("ferro-rust-cancel-{}", std::process::id()));
    let _ = std::fs::remove_file(&marker);
    let source =
        format!("fn main() {{ std::fs::write({marker:?}, \"started\").unwrap(); loop {{}} }}");
    let cancel = Arc::new(AtomicBool::new(false));
    let worker_cancel = Arc::clone(&cancel);
    let worker = std::thread::spawn(move || run_rust_source_with_cancel(&source, &worker_cancel));

    let started = std::time::Instant::now();
    while !marker.exists() && started.elapsed() < std::time::Duration::from_secs(10) {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        marker.exists(),
        "programa deve iniciar antes do cancelamento"
    );
    cancel.store(true, Ordering::Release);

    let result = worker
        .join()
        .expect("worker deve concluir")
        .expect("execução deve retornar resultado");
    let _ = std::fs::remove_file(marker);
    assert!(matches!(
        result,
        RustSourceRunResult::Cancelled {
            phase: RustSourceRunPhase::Runtime,
            ..
        }
    ));
    assert!(started.elapsed() < std::time::Duration::from_secs(2));
}

#[test]
fn cancellation_requested_before_rustc_finishes_reports_compilation_phase() {
    let cancel = AtomicBool::new(true);
    let result = run_rust_source_with_cancel("fn main() { println!(\"ready\"); }", &cancel)
        .expect("cancelamento deve ser um resultado tipado");

    assert!(matches!(
        result,
        RustSourceRunResult::Cancelled {
            phase: RustSourceRunPhase::Compilation,
            ..
        }
    ));
}

#[test]
fn caps_combined_stdout_and_stderr_from_rust_programs() {
    let source = r#"fn main() {
    loop { print!("0123456789abcdef"); }
}"#;
    let result = run_rust_source(source).expect("rustc deve estar instalado nos testes");

    let RustSourceRunResult::RuntimeLimit {
        stdout,
        stderr,
        reason: ferro_service::RustSourceLimitReason::Output,
    } = result
    else {
        panic!("saída ilimitada deve atingir o limite combinado");
    };
    assert!(stdout.len() + stderr.len() <= 1_048_576);
}

#[cfg(unix)]
#[test]
fn closes_inherited_pipes_when_the_rust_program_leaves_a_child_running() {
    let source = r#"fn main() {
    std::process::Command::new("sh")
        .args(["-c", "sleep 5"])
        .spawn()
        .expect("spawn child");
}"#;
    let started = std::time::Instant::now();

    let result = run_rust_source(source).expect("rustc deve estar instalado nos testes");

    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(matches!(
        result,
        RustSourceRunResult::Executed {
            exit_code: Some(0),
            ..
        }
    ));
}

#[cfg(windows)]
#[test]
fn closes_inherited_pipes_when_a_windows_child_outlives_its_rust_parent() {
    let source = r#"fn main() {
    std::thread::sleep(std::time::Duration::from_millis(300));
    std::process::Command::new("cmd")
        .args(["/C", "ping -n 6 127.0.0.1 > NUL"])
        .spawn()
        .expect("spawn child");
}"#;
    let started = std::time::Instant::now();

    let result = run_rust_source(source).expect("rustc deve estar instalado nos testes");

    assert!(started.elapsed() < std::time::Duration::from_secs(2));
    assert!(matches!(
        result,
        RustSourceRunResult::Executed {
            exit_code: Some(0),
            ..
        }
    ));
}

#[test]
fn rejects_source_larger_than_one_mib_before_compilation() {
    let source = " ".repeat(1_048_577);
    let error = run_rust_source(&source).expect_err("fonte acima do limite deve ser rejeitada");

    assert!(error.contains("1 MiB"));
}

#[test]
fn passes_each_rust_argument_verbatim_to_std_env_args() {
    let source = r#"fn main() {
    println!("{}", std::env::args().skip(1).collect::<Vec<_>>().join("|"));
}"#;
    let args = vec![
        "João Silva".to_owned(),
        "19".to_owned(),
        "aspas \" e barra \\".to_owned(),
        String::new(),
    ];
    let cancel = AtomicBool::new(false);

    let result = run_rust_source_with_args_and_cancel(source, &args, &cancel)
        .expect("argumentos válidos devem compilar e executar");

    assert!(matches!(
        result,
        RustSourceRunResult::Executed { stdout, exit_code: Some(0), .. }
            if stdout == "João Silva|19|aspas \" e barra \\|\n"
    ));
}

#[test]
fn rejects_too_many_rust_arguments_before_compilation() {
    let args = vec![String::new(); 257];
    let cancel = AtomicBool::new(false);
    let error = run_rust_source_with_args_and_cancel("fn main() {}", &args, &cancel)
        .expect_err("mais de 256 argumentos devem ser recusados");

    assert!(error.contains("256"));
}

#[test]
fn rejects_rust_arguments_over_the_platform_budget_before_compilation() {
    let args = vec!["x".repeat(65_537)];
    let cancel = AtomicBool::new(false);
    let error = run_rust_source_with_args_and_cancel("fn main() {}", &args, &cancel)
        .expect_err("mais de 64 KiB de argumentos devem ser recusados");

    let expected_limit = if cfg!(windows) { "8 KiB" } else { "64 KiB" };
    assert!(error.contains(expected_limit));
}

#[test]
fn rejects_nul_in_rust_arguments_before_compilation() {
    let args = vec!["bad\0arg".to_owned()];
    let cancel = AtomicBool::new(false);
    let error = run_rust_source_with_args_and_cancel("fn main() {}", &args, &cancel)
        .expect_err("NUL não pode ser passado como argumento de processo");

    assert!(error.contains("NUL"));
}
