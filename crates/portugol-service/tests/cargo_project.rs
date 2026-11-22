use std::fs;
use std::sync::atomic::AtomicBool;

use portugol_service::{check_cargo_project_with_cancel, run_cargo_project_with_cancel};

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
