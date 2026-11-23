use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

struct TempDir(PathBuf);

impl TempDir {
    fn new() -> Self {
        let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
        let dir =
            std::env::temp_dir().join(format!("portugol-cli-test-{}-{id}", std::process::id()));
        fs::create_dir_all(&dir).expect("criar pasta temporária");
        Self(dir)
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn cli() -> Command {
    Command::new(env!("CARGO_BIN_EXE_portugol"))
}

#[test]
fn creates_validates_formats_exports_and_executes_pors_files() {
    let dir = TempDir::new();
    let starter = dir.0.join("ola.pors");

    let created = cli().args(["novo"]).arg(&starter).output().unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(starter.exists());

    // Não sobrescreve arquivo existente.
    let duplicate = cli().args(["novo"]).arg(&starter).output().unwrap();
    assert!(!duplicate.status.success());

    // Valida modelo criado.
    let checked = cli().arg(&starter).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");

    // Modelo já está formatado.
    let formatted_check = cli()
        .args(["formatar", "--check"])
        .arg(&starter)
        .output()
        .unwrap();
    assert!(formatted_check.status.success(), "{formatted_check:?}");

    // Executa na VM.
    let executed = cli().args(["executar"]).arg(&starter).output().unwrap();
    assert!(executed.status.success(), "{executed:?}");
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "17");

    // Lista bytecode.
    let listing = cli()
        .args(["executar", "--bytecode"])
        .arg(&starter)
        .output()
        .unwrap();
    assert!(listing.status.success(), "{listing:?}");
    assert!(String::from_utf8_lossy(&listing.stdout).contains("inicio"));

    // Exporta Rust gerado.
    let exported = cli()
        .args(["exportar-rust"])
        .arg(&starter)
        .output()
        .unwrap();
    assert!(exported.status.success(), "{exported:?}");
    assert!(String::from_utf8_lossy(&exported.stdout).contains("fn main()"));
}

#[test]
fn formats_unformatted_pors_and_runs_interactive_input_and_vectors() {
    let dir = TempDir::new();
    let program = dir.0.join("app.pors");
    fs::write(
        &program,
        "funcao somar(itens:vetor<inteiro>)->inteiro{seja mutavel total:inteiro=0;para(seja mutavel i:inteiro=0;i<tamanho(itens);i+=1){total+=itens[i];}retorne total;}funcao inicio(){seja mutavel extra:inteiro=0;leia(extra);seja lista:vetor<inteiro> =[10,20];escreva(somar(lista)+extra);}",
    )
    .unwrap();

    let needs_fmt = cli()
        .args(["formatar", "--check"])
        .arg(&program)
        .output()
        .unwrap();
    assert!(!needs_fmt.status.success());

    let fmt = cli().args(["formatar"]).arg(&program).output().unwrap();
    assert!(fmt.status.success(), "{fmt:?}");

    let clean_check = cli()
        .args(["formatar", "--check"])
        .arg(&program)
        .output()
        .unwrap();
    assert!(clean_check.status.success(), "{clean_check:?}");

    let mut child = cli()
        .args(["executar"])
        .arg(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(b"12\n").unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8_lossy(&output.stdout), "42");
}

#[test]
fn preserves_classic_por_support_and_rejects_unknown_extensions() {
    let dir = TempDir::new();
    let classic = dir.0.join("classico.por");
    fs::write(
        &classic,
        "programa { funcao inicio() { escreva(20 + 22) } }",
    )
    .unwrap();

    let valid = cli().arg(&classic).output().unwrap();
    assert!(valid.status.success(), "{valid:?}");

    let run = cli().args(["executar"]).arg(&classic).output().unwrap();
    assert!(run.status.success(), "{run:?}");
    assert_eq!(String::from_utf8_lossy(&run.stdout), "42");

    let unknown = dir.0.join("notas.txt");
    fs::write(&unknown, "texto").unwrap();
    let rejected = cli().arg(&unknown).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("extensão não suportada"));
}
