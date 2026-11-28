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
        let dir = std::env::temp_dir().join(format!("ferro-cli-test-{}-{id}", std::process::id()));
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
    Command::new(env!("CARGO_BIN_EXE_ferro"))
}

#[test]
fn creates_validates_formats_exports_and_executes_ferro_files() {
    let dir = TempDir::new();
    let starter = dir.0.join("ola.fer");

    let created = cli().args(["novo"]).arg(&starter).output().unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(starter.exists());

    // Não sobrescreve arquivo existente.
    let duplicate = cli().args(["novo"]).arg(&starter).output().unwrap();
    assert!(!duplicate.status.success());

    // Valida modelo criado.
    let checked = cli().args(["valida"]).arg(&starter).output().unwrap();
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
fn direct_file_runs_and_validation_has_an_explicit_command() {
    let dir = TempDir::new();
    let starter = dir.0.join("ola.fer");
    fs::write(&starter, "funcao inicio() { escreva(42); }\n").unwrap();

    let executed = cli().arg(&starter).output().unwrap();
    assert!(executed.status.success(), "{executed:?}");
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "42");

    let checked = cli().args(["valida"]).arg(&starter).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert!(String::from_utf8_lossy(&checked.stdout).contains("1 arquivo(s)"));

    let current_dir_check = cli().current_dir(&dir.0).args(["valida"]).output().unwrap();
    assert!(current_dir_check.status.success(), "{current_dir_check:?}");
    assert!(String::from_utf8_lossy(&current_dir_check.stdout).contains("1 arquivo(s)"));
}

#[test]
fn cli_has_portuguese_help_syntax_and_version_commands() {
    let help = cli().arg("ajuda").output().unwrap();
    assert!(help.status.success(), "{help:?}");
    assert!(String::from_utf8_lossy(&help.stdout).contains("ferro valida"));

    let command_help = cli().args(["ajuda", "executar"]).output().unwrap();
    assert!(command_help.status.success(), "{command_help:?}");
    assert!(String::from_utf8_lossy(&command_help.stdout).contains("ferro executar"));

    let syntax = cli().arg("sintaxe").output().unwrap();
    assert!(syntax.status.success(), "{syntax:?}");
    let syntax = String::from_utf8_lossy(&syntax.stdout);
    assert!(syntax.contains("funcao"));
    assert!(syntax.contains("seja"));

    let version = cli().arg("--version").output().unwrap();
    assert!(version.status.success(), "{version:?}");
    assert!(String::from_utf8_lossy(&version.stdout).starts_with("ferro "));
}

#[test]
fn game_project_creation_reports_that_manifest_type_is_not_supported_yet() {
    let dir = TempDir::new();
    let target = dir.0.join("meu-jogo");
    let result = cli()
        .args(["novo", "--jogo"])
        .arg(&target)
        .output()
        .unwrap();
    assert!(!result.status.success());
    assert!(String::from_utf8_lossy(&result.stderr).contains("só aceita projetos desktop"));
    assert!(!target.exists());
}

#[test]
fn formats_unformatted_ferro_and_runs_interactive_input_and_vectors() {
    let dir = TempDir::new();
    let program = dir.0.join("app.fer");
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
fn rejects_legacy_por_with_guidance_and_unknown_extensions() {
    let dir = TempDir::new();
    let classic = dir.0.join("classico.por");
    fs::write(
        &classic,
        "programa { funcao inicio() { escreva(20 + 22) } }",
    )
    .unwrap();

    let legacy_check = cli().arg(&classic).output().unwrap();
    assert!(!legacy_check.status.success());
    assert!(
        String::from_utf8_lossy(&legacy_check.stderr).contains("não são suportados"),
        "{legacy_check:?}"
    );

    let legacy_run = cli().args(["executar"]).arg(&classic).output().unwrap();
    assert!(!legacy_run.status.success());
    assert!(
        String::from_utf8_lossy(&legacy_run.stderr).contains("não são suportados"),
        "{legacy_run:?}"
    );

    let unknown = dir.0.join("notas.txt");
    fs::write(&unknown, "texto").unwrap();
    let rejected = cli().arg(&unknown).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("extensão não suportada"));
}

#[test]
fn exports_visual_mvu_app_to_rust_and_cargo_project() {
    let dir = TempDir::new();
    let app = dir.0.join("contador.fer");
    fs::write(
        &app,
        r#"registro Estado { contagem: inteiro }
funcao iniciar() -> Estado { retorne Estado { contagem: 0 }; }
funcao atualizar(estado: Estado, alvo: cadeia) -> Estado {
    se (alvo == "inc") { retorne Estado { contagem: estado.contagem + 1 }; }
    retorne estado;
}
funcao visualizar(estado: Estado) -> inteiro {
    retorne UI.coluna([UI.texto("Total: ", estado.contagem), UI.botao("inc", "+1")]);
}
"#,
    )
    .unwrap();

    let rust = cli().args(["exportar-rust"]).arg(&app).output().unwrap();
    assert!(rust.status.success(), "{rust:?}");
    let rust_code = String::from_utf8_lossy(&rust.stdout);
    assert!(rust_code.contains("mod __ferro_ui"));
    assert!(rust_code.contains("fn main()"));

    let cargo_dir = dir.0.join("contador-cargo");
    let exported = cli()
        .args(["exportar-cargo"])
        .arg(&app)
        .arg(&cargo_dir)
        .output()
        .unwrap();
    assert!(exported.status.success(), "{exported:?}");
    assert!(cargo_dir.join("Cargo.toml").exists());
    assert!(cargo_dir.join("src/main.rs").exists());
    assert!(cargo_dir.join("README.md").exists());

    // Recusa sobrescrever projeto Cargo existente.
    let duplicate = cli()
        .args(["exportar-cargo"])
        .arg(&app)
        .arg(&cargo_dir)
        .output()
        .unwrap();
    assert!(!duplicate.status.success());
}

#[test]
fn debugs_ferro_program_interactively_with_breakpoints_steps_and_variables() {
    let dir = TempDir::new();
    let program = dir.0.join("passo.fer");
    fs::write(
        &program,
        "funcao inicio() {\n    seja mutavel x: inteiro = 10;\n    x += 5;\n    escreva(\"x=\", x);\n}\n",
    )
    .unwrap();

    let mut child = cli()
        .args(["depurar", "--ponto", "4"])
        .arg(&program)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    {
        let mut stdin = child.stdin.take().unwrap();
        stdin.write_all(b"c\nv\nc\n").unwrap();
    }
    let output = child.wait_with_output().unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(output.status.success(), "stdout={stdout}\nstderr={stderr}");
    assert!(stdout.contains("[pausa]"), "{stdout}");
    assert!(stdout.contains("x : inteiro = 15"), "{stdout}");
    assert!(stdout.contains("x=15"), "{stdout}");
    assert!(stdout.contains("execução finalizada"), "{stdout}");
}

#[test]
fn creates_validates_and_formats_desktop_projects() {
    let dir = TempDir::new();
    let project = dir.0.join("Meu App");

    let created = cli()
        .args(["novo", "--desktop"])
        .arg(&project)
        .output()
        .unwrap();
    assert!(created.status.success(), "{created:?}");
    let manifest = project.join("meu-app.ferproj");
    let design = project.join("JanelaPrincipal.design");
    assert!(manifest.is_file());
    assert!(design.is_file());
    assert!(project.join("recursos").is_dir());
    assert!(
        fs::read_to_string(&manifest)
            .unwrap()
            .contains("Nome=\"Meu App\"")
    );

    // Não cria por cima de uma pasta com arquivos.
    let again = cli()
        .args(["novo", "--desktop"])
        .arg(&project)
        .output()
        .unwrap();
    assert!(!again.status.success());

    // O nome pode ser diferente do nome da pasta.
    let named = dir.0.join("calc");
    let created = cli()
        .args(["novo", "--desktop"])
        .arg(&named)
        .args(["--nome", "Calculadora de Médias"])
        .output()
        .unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(named.join("calculadora-de-medias.ferproj").is_file());

    // Valida a pasta inteira e cada arquivo.
    let checked = cli().arg(&project).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert!(String::from_utf8_lossy(&checked.stdout).contains("2 arquivo(s), 0 com erro(s)"));

    for file in [&design, &manifest] {
        let formatted = cli()
            .args(["formatar", "--check"])
            .arg(file)
            .output()
            .unwrap();
        assert!(formatted.status.success(), "{formatted:?}");
    }

    // Erro no .design aparece com o código do diagnóstico.
    fs::write(
        &design,
        "<Janela Nome=\"JanelaPrincipal\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <Botoa />\n</Janela>\n",
    )
    .unwrap();
    let broken = cli().arg(&design).output().unwrap();
    assert!(!broken.status.success());
    let stdout = String::from_utf8_lossy(&broken.stdout);
    assert!(stdout.contains("design.unknown-element"), "{stdout}");
    assert!(stdout.contains("<Botao>"), "{stdout}");

    // Formatar reescreve o .design.
    fs::write(
        &design,
        "<Janela Nome='JanelaPrincipal' Titulo=\"T\" Largura=\"1\" Altura=\"1\"></Janela>",
    )
    .unwrap();
    let formatted = cli().args(["formatar"]).arg(&design).output().unwrap();
    assert!(formatted.status.success(), "{formatted:?}");
    assert_eq!(
        fs::read_to_string(&design).unwrap(),
        "<Janela Nome=\"JanelaPrincipal\" Titulo=\"T\" Largura=\"1\" Altura=\"1\" Formato=\"1\" />\n"
    );

    // Sem a janela inicial, o manifesto acusa.
    fs::remove_file(&design).unwrap();
    let missing = cli().arg(&manifest).output().unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stdout).contains("projeto.missing-start-window"));
}

#[test]
fn novo_sem_extensao_cria_arquivo_ferro_e_executa_normalmente() {
    let dir = TempDir::new();
    let base = dir.0.join("saudacao");
    let expected = dir.0.join("saudacao.fer");

    let created = cli().args(["novo"]).arg(&base).output().unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(expected.is_file());

    let executed = cli().args(["executar"]).arg(&expected).output().unwrap();
    assert!(executed.status.success(), "{executed:?}");
    assert_eq!(String::from_utf8_lossy(&executed.stdout), "17");
}

#[test]
fn rejects_unknown_extension() {
    let dir = TempDir::new();
    let old = dir.0.join("antigo.zzz");
    fs::write(&old, "funcao inicio() { escreva(1); }").unwrap();

    let rejected = cli().arg(&old).output().unwrap();
    assert!(!rejected.status.success());
    assert!(String::from_utf8_lossy(&rejected.stderr).contains("extensão não suportada"));
}

#[test]
fn validates_screen_code_behind_with_the_types_from_the_design() {
    let dir = TempDir::new();
    let design = dir.0.join("JanelaPrincipal.design");
    let code = dir.0.join("JanelaPrincipal.fer");
    fs::write(
        &design,
        "<Janela Nome=\"JanelaPrincipal\" Titulo=\"T\" Largura=\"200\" Altura=\"100\">\n    <Botao Nome=\"btnOk\" Texto=\"OK\" X=\"8\" Y=\"8\" AoClicar=\"btnOk_clicar\" />\n</Janela>\n",
    )
    .unwrap();
    fs::write(
        &code,
        "funcao btnOk_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {\n    seja mutavel t: JanelaPrincipal = tela;\n    t.btnOk.texto = \"Clicado\";\n    t.titulo = \"Pronto\";\n    retorne t;\n}\n",
    )
    .unwrap();
    let checked = cli().args(["valida"]).arg(&code).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");

    // Sem o .design ao lado, o mesmo código não conhece JanelaPrincipal.
    fs::remove_file(&design).unwrap();
    let alone = cli().args(["valida"]).arg(&code).output().unwrap();
    assert!(!alone.status.success());
}

#[test]
fn ui_obsoleto_aparece_como_aviso_sem_falhar() {
    let dir = TempDir::new();
    let file = dir.0.join("contador.fer");
    fs::write(
        &file,
        "funcao iniciar() -> inteiro {\n    retorne 0;\n}\n\nfuncao atualizar(estado: inteiro, tipo: cadeia, alvo: cadeia) -> inteiro {\n    retorne estado;\n}\n\nfuncao visualizar(estado: inteiro) -> inteiro {\n    retorne UI.texto(\"oi\");\n}\n",
    )
    .unwrap();
    let checked = cli().args(["valida"]).arg(&file).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");
    let stdout = String::from_utf8_lossy(&checked.stdout);
    assert!(stdout.contains("1 aviso(s)"), "{stdout}");
}

#[test]
fn creates_the_counter_example_and_it_validates() {
    let dir = TempDir::new();
    let project = dir.0.join("Contador");
    let created = cli()
        .args(["novo", "--desktop"])
        .arg(&project)
        .args(["--exemplo", "contador"])
        .output()
        .unwrap();
    assert!(created.status.success(), "{created:?}");
    assert!(project.join("JanelaPrincipal.fer").is_file());
    let checked = cli().arg(&project).output().unwrap();
    assert!(checked.status.success(), "{checked:?}");
    assert!(String::from_utf8_lossy(&checked.stdout).contains("3 arquivo(s), 0 com erro(s)"));
}

#[test]
fn exports_a_screen_as_a_slint_cargo_project() {
    let dir = TempDir::new();
    let project = dir.0.join("Contador");
    let created = cli()
        .args(["novo", "--desktop"])
        .arg(&project)
        .args(["--exemplo", "contador"])
        .output()
        .unwrap();
    assert!(created.status.success(), "{created:?}");
    let target = dir.0.join("contador-slint");
    let exported = cli()
        .args(["exportar-cargo"])
        .arg(project.join("JanelaPrincipal.design"))
        .arg(&target)
        .output()
        .unwrap();
    assert!(exported.status.success(), "{exported:?}");
    assert!(
        String::from_utf8_lossy(&exported.stdout)
            .contains("\"contador\" gerado com a tela em Slint")
    );
    for file in [
        "Cargo.toml",
        "build.rs",
        "ui/JanelaPrincipal.slint",
        "src/main.rs",
        "README.md",
    ] {
        assert!(target.join(file).is_file(), "{file}");
    }
    let main = fs::read_to_string(target.join("src/main.rs")).unwrap();
    assert!(main.contains("ui.on_btnInc_clicar(move || {"));

    fs::remove_file(project.join("JanelaPrincipal.fer")).unwrap();
    let missing = cli()
        .args(["exportar-cargo"])
        .arg(project.join("JanelaPrincipal.design"))
        .arg(dir.0.join("outro"))
        .output()
        .unwrap();
    assert!(!missing.status.success());
    assert!(String::from_utf8_lossy(&missing.stderr).contains("código da tela"));
}

#[test]
fn formats_screen_code_behind_using_the_design() {
    let dir = TempDir::new();
    fs::write(
        dir.0.join("JanelaPrincipal.design"),
        "<Janela Nome=\"JanelaPrincipal\" Titulo=\"T\" Largura=\"200\" Altura=\"100\">\n    <Botao Nome=\"btnOk\" Texto=\"OK\" X=\"8\" Y=\"8\" AoClicar=\"btnOk_clicar\" />\n</Janela>\n",
    )
    .unwrap();
    let code = dir.0.join("JanelaPrincipal.fer");
    fs::write(
        &code,
        "funcao btnOk_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {\nseja mutavel t: JanelaPrincipal = tela;\n    retorne t;\n}\n",
    )
    .unwrap();
    let formatted = cli().arg("formatar").arg(&code).output().unwrap();
    assert!(formatted.status.success(), "{formatted:?}");
    assert!(
        fs::read_to_string(&code)
            .unwrap()
            .contains("\n    seja mutavel t")
    );
    let checked = cli()
        .args(["formatar", "--check"])
        .arg(&code)
        .output()
        .unwrap();
    assert!(checked.status.success(), "{checked:?}");
}
