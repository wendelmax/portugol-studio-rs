use portugol_service::{RustSourceRunResult, run_rust_source};

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
            reason: portugol_service::RustSourceLimitReason::Time,
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
        reason: portugol_service::RustSourceLimitReason::Output,
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
