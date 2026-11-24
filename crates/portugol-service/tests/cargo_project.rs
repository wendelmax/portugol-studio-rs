use std::fs;
use std::sync::atomic::AtomicBool;

use portugol_service::{
    CargoRunTarget, cargo_run_targets_with_cancel, check_cargo_project_with_cancel,
    run_cargo_project_target_with_args_and_cancel, run_cargo_project_target_with_cancel,
    run_cargo_project_with_cancel,
};

struct TempDir(std::path::PathBuf);

impl TempDir {
    fn new() -> Self {
        static NEXT: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "portugol-cargo-test-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
        ));
        fs::create_dir_all(path.join("src")).unwrap();
        Self(path)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn builds_and_runs_a_cargo_project_without_shell_interpolation() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"estudo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/main.rs"),
        "fn main() { println!(\"Cargo pronto\"); }",
    )
    .unwrap();

    let result = run_cargo_project_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "executed", "{}", result.stderr);
    assert!(result.stdout.contains("Cargo pronto"));
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn renders_cargo_compiler_errors_instead_of_json_events() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"erro\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(project.0.join("src/main.rs"), "fn main() { let valor = ; }").unwrap();

    let result = run_cargo_project_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "build_error");
    assert!(result.stderr.contains("error"), "{}", result.stderr);
    assert!(
        result.stderr.contains("expected expression"),
        "{}",
        result.stderr
    );
    assert!(
        !result.stderr.contains("compiler-message"),
        "{}",
        result.stderr
    );
}

#[test]
fn reports_a_library_only_project_when_cargo_has_no_binary_target() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"biblioteca\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/lib.rs"),
        "pub fn soma(a: i32, b: i32) -> i32 { a + b }",
    )
    .unwrap();

    let targets = cargo_run_targets_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();
    assert!(targets.targets.is_empty());
    assert!(targets.default_target.is_none());

    let result = run_cargo_project_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "run_error");
    assert!(result.stderr.contains("bin target"), "{}", result.stderr);
}

#[test]
fn sets_cargo_manifest_directory_for_the_launched_binary() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"manifest_env\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/main.rs"),
        "fn main() { println!(\"{}\", std::env::var(\"CARGO_MANIFEST_DIR\").unwrap_or_default()); }",
    )
    .unwrap();

    let result = run_cargo_project_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "executed");
    assert_eq!(
        result.stdout.trim(),
        project.0.canonicalize().unwrap().to_string_lossy()
    );
}

#[test]
fn preserves_successful_build_warnings_in_the_output() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"aviso\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/main.rs"),
        "fn main() { let nao_usada = 1; println!(\"ok\"); }",
    )
    .unwrap();

    let result = run_cargo_project_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "executed");
    assert!(result.stdout.contains("warning"), "{}", result.stdout);
    assert!(result.stdout.contains("ok"), "{}", result.stdout);
}

#[test]
fn cargo_check_maps_the_open_file_diagnostic_after_non_bmp_text() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"checagem\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    let source = "fn main() { println!(\"🚀\"); let numero: i32 = \"texto\"; }";
    let source_path = project.0.join("src/main.rs");
    fs::write(&source_path, source).unwrap();

    let result =
        check_cargo_project_with_cancel(&project.0, &source_path, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.status, "check_error");
    let diagnostic = result
        .diagnostics
        .iter()
        .find(|diagnostic| diagnostic.message.contains("mismatched types"))
        .expect("diagnóstico do arquivo ativo");
    assert_eq!(diagnostic.start.line, 1);
    let emoji = source.find("🚀").unwrap();
    let emoji_utf16 = source[..emoji].encode_utf16().count() as u32;
    assert!(diagnostic.start.offset > emoji_utf16);
    assert!(
        diagnostic
            .rendered
            .as_deref()
            .unwrap_or_default()
            .contains("error")
    );
}

#[cfg(unix)]
#[test]
fn cargo_runner_rejects_a_manifest_symlink_before_running_external_project() {
    if !portugol_service::cargo_available() {
        return;
    }
    use std::os::unix::fs::symlink;

    let workspace = TempDir::new();
    let external = TempDir::new();
    fs::write(
        external.0.join("Cargo.toml"),
        "[package]\nname = \"fora_do_workspace\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        external.0.join("src/main.rs"),
        "fn main() { println!(\"executou fora\"); }",
    )
    .unwrap();
    symlink(
        external.0.join("Cargo.toml"),
        workspace.0.join("Cargo.toml"),
    )
    .unwrap();

    let result = run_cargo_project_with_cancel(&workspace.0, &AtomicBool::new(false));

    assert!(
        result.is_err(),
        "Cargo não deve seguir Cargo.toml simbólico"
    );
    assert!(
        result.unwrap_err().contains("link simbólico"),
        "a falha deve explicar a restrição do manifest"
    );
}

#[test]
fn cargo_run_forwards_program_arguments_without_shell_interpretation() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"argumentos\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/main.rs"),
        "fn main() { println!(\"{:?}\", std::env::args().skip(1).collect::<Vec<_>>()); }",
    )
    .unwrap();
    let args = vec![
        String::new(),
        "João Silva".into(),
        "aspas \" duplas".into(),
        "$(nao-executar); *".into(),
    ];
    let target = CargoRunTarget {
        package_name: "argumentos".into(),
        target_name: "argumentos".into(),
    };

    let result = run_cargo_project_target_with_args_and_cancel(
        &project.0,
        Some(&target),
        &args,
        &AtomicBool::new(false),
    )
    .unwrap();

    assert_eq!(result.status, "executed", "{}", result.stderr);
    assert!(
        result.stdout.contains(&format!("{args:?}")),
        "{}",
        result.stdout
    );
}

#[test]
fn cargo_run_rejects_invalid_program_arguments_before_starting_cargo() {
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "this is not valid Cargo metadata",
    )
    .unwrap();
    let nul = vec!["inválido\0".to_string()];

    let result = run_cargo_project_target_with_args_and_cancel(
        &project.0,
        None,
        &nul,
        &AtomicBool::new(false),
    );

    assert!(result.unwrap_err().contains("NUL"));

    let many = vec!["arg".to_string(); 257];
    let result = run_cargo_project_target_with_args_and_cancel(
        &project.0,
        None,
        &many,
        &AtomicBool::new(false),
    );
    assert!(result.unwrap_err().contains("256"));

    let max_bytes = if cfg!(windows) { 8 * 1024 } else { 64 * 1024 };
    let oversized = vec!["x".repeat(max_bytes + 1)];
    let result = run_cargo_project_target_with_args_and_cancel(
        &project.0,
        None,
        &oversized,
        &AtomicBool::new(false),
    );
    assert!(result.unwrap_err().contains("limite total"));
}

#[test]
fn discovers_all_binary_targets_and_the_default_run_across_workspace_packages() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::create_dir_all(project.0.join("app-a/src/bin")).unwrap();
    fs::create_dir_all(project.0.join("app-b/src")).unwrap();
    fs::write(
        project.0.join("Cargo.toml"),
        "[workspace]\nmembers = [\"app-a\", \"app-b\"]\ndefault-members = [\"app-a\"]\nresolver = \"2\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("app-a/Cargo.toml"),
        "[package]\nname = \"app_a\"\nversion = \"0.1.0\"\nedition = \"2021\"\ndefault-run = \"extra\"\n",
    )
    .unwrap();
    fs::write(project.0.join("app-a/src/main.rs"), "fn main() {}").unwrap();
    fs::write(project.0.join("app-a/src/bin/extra.rs"), "fn main() {}").unwrap();
    fs::write(
        project.0.join("app-b/Cargo.toml"),
        "[package]\nname = \"app_b\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(project.0.join("app-b/src/main.rs"), "fn main() {}").unwrap();

    let result = cargo_run_targets_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(result.targets.len(), 3);
    assert!(
        result
            .targets
            .iter()
            .any(|target| target.package_name == "app_b" && target.target_name == "app_b")
    );
    assert_eq!(result.default_target.unwrap().target_name, "extra");
}

#[test]
fn leaves_the_default_empty_when_a_package_has_multiple_bins_without_default_run() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::create_dir_all(project.0.join("src/bin")).unwrap();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"escolha\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(project.0.join("src/main.rs"), "fn main() {}").unwrap();
    fs::write(project.0.join("src/bin/um.rs"), "fn main() {}").unwrap();
    fs::write(project.0.join("src/bin/dois.rs"), "fn main() {}").unwrap();

    let targets = cargo_run_targets_with_cancel(&project.0, &AtomicBool::new(false)).unwrap();

    assert_eq!(targets.targets.len(), 3);
    assert!(targets.default_target.is_none());
}

#[test]
fn runs_only_the_selected_binary_target() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::create_dir_all(project.0.join("src/bin")).unwrap();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"alvos\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/main.rs"),
        "fn main() { println!(\"principal\"); }",
    )
    .unwrap();
    fs::write(
        project.0.join("src/bin/secundario.rs"),
        "fn main() { println!(\"secundario\"); }",
    )
    .unwrap();

    let target = CargoRunTarget {
        package_name: "alvos".into(),
        target_name: "secundario".into(),
    };
    let result =
        run_cargo_project_target_with_cancel(&project.0, Some(&target), &AtomicBool::new(false))
            .unwrap();

    assert_eq!(result.status, "executed", "{}", result.stderr);
    assert!(result.stdout.contains("secundario"));
    assert!(!result.stdout.contains("principal"));
}

#[test]
fn rejects_a_binary_target_that_is_not_in_the_project_metadata() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"validacao\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(project.0.join("src/main.rs"), "fn main() {}").unwrap();
    let target = CargoRunTarget {
        package_name: "validacao".into(),
        target_name: "../../arbitrary".into(),
    };

    let error =
        run_cargo_project_target_with_cancel(&project.0, Some(&target), &AtomicBool::new(false))
            .unwrap_err();

    assert!(error.contains("alvo binário"));
}

#[test]
fn runs_cargo_tests_and_reports_passing_failing_and_build_error_states() {
    if !portugol_service::cargo_available() {
        return;
    }
    let project = TempDir::new();
    fs::write(
        project.0.join("Cargo.toml"),
        "[package]\nname = \"suite_demo\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )
    .unwrap();
    fs::write(
        project.0.join("src/lib.rs"),
        "#[test] fn soma_ok() { assert_eq!(2 + 2, 4); }\n#[test] #[ignore] fn pulado() {}\n",
    )
    .unwrap();
    let passed =
        portugol_service::test_cargo_project_with_cancel(&project.0, &AtomicBool::new(false))
            .unwrap();
    assert_eq!(passed.status, "passed", "stderr={}", passed.stderr);
    assert_eq!(passed.passed, 1);
    assert_eq!(passed.failed, 0);
    assert_eq!(passed.ignored, 1);
    assert!(passed.failures.is_empty());

    fs::write(
        project.0.join("src/lib.rs"),
        "#[test] fn ok() { assert_eq!(1, 1); }\n#[test] fn falha_esperada() { assert_eq!(1, 2); }\n",
    )
    .unwrap();
    let failed =
        portugol_service::test_cargo_project_with_cancel(&project.0, &AtomicBool::new(false))
            .unwrap();
    assert_eq!(failed.status, "failed", "stdout={}", failed.stdout);
    assert_eq!(failed.passed, 1);
    assert_eq!(failed.failed, 1);
    assert!(failed.failures.iter().any(|f| f.contains("falha_esperada")));

    fs::write(project.0.join("src/lib.rs"), "fn quebrado( { }").unwrap();
    let build_err =
        portugol_service::test_cargo_project_with_cancel(&project.0, &AtomicBool::new(false))
            .unwrap();
    assert_eq!(build_err.status, "build_error");
    assert_eq!(build_err.passed, 0);
    assert_eq!(build_err.failed, 0);
}
