use ferro_service::{Host, ScreenEvent, ScreenRun};

const DESIGN: &str = r#"<Janela Nome="J" Titulo="Notas" Largura="300" Altura="200" Dados="Estado">
    <Botao Nome="btnPerguntar" Texto="?" X="8" Y="8" AoClicar="btnPerguntar_clicar" />
    <Botao Nome="btnCopiar" Texto="Copiar" X="8" Y="48" AoClicar="btnCopiar_clicar" />
    <Botao Nome="btnSalvar" Texto="Salvar" X="8" Y="88" AoClicar="btnSalvar_clicar" />
</Janela>
"#;

const CODE: &str = r#"registro Estado {
    ok: logico,
    salvo: logico,
    erro: cadeia
}

funcao btnPerguntar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    t.dados.ok = Dialogo.confirmar("Continuar?");
    retorne t;
}

funcao btnCopiar_clicar(tela: J) -> J {
    AreaTransferencia.copiar("x");
    retorne tela;
}

funcao btnSalvar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    escolha Arquivos.escrever_texto("notas.txt", "10") {
        caso Resultado::Ok(_) => { t.dados.salvo = verdadeiro; }
        caso Resultado::Erro(e) => { t.dados.erro = e; }
    }
    retorne t;
}
"#;

struct YesHost;

impl Host for YesHost {
    fn confirm(&mut self, _text: &str) -> Result<bool, String> {
        Ok(true)
    }
    fn clipboard_write(&mut self, _text: &str) -> Result<(), String> {
        Ok(())
    }
}

fn click(control: &str) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: "AoClicar".into(),
        value: None,
    }
}

fn run(permissions: &[&str], dir: Option<std::path::PathBuf>) -> ScreenRun {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    run.set_host(Box::new(|| Box::new(YesHost)));
    run.set_project(permissions.iter().map(|p| (*p).to_owned()).collect(), dir);
    run
}

#[test]
fn servico_declarado_roda_e_o_nao_declarado_explica_o_que_acrescentar() {
    let mut run = run(&["Dialogo"], None);
    assert_eq!(
        run.dispatch(&click("btnPerguntar")).unwrap().snapshot["dados"]["ok"],
        true
    );
    let error = run.dispatch(&click("btnCopiar")).unwrap_err();
    assert!(
        error.contains("acrescente `AreaTransferencia` em `Permissoes` no .ferproj"),
        "{error}"
    );
}

#[test]
fn arquivos_so_recebe_a_pasta_do_projeto_com_a_permissao() {
    let dir = std::env::temp_dir().join(format!("ferro-permissoes-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();

    let mut without = run(&["Dialogo"], Some(dir.clone()));
    let step = without.dispatch(&click("btnSalvar")).unwrap();
    let error = step.snapshot["dados"]["erro"].as_str().unwrap().to_owned();
    assert!(
        error.contains("acrescente `Arquivos` em `Permissoes`"),
        "{error}"
    );
    assert!(!dir.join("notas.txt").exists());

    let mut with = run(&["Arquivos"], Some(dir.clone()));
    assert_eq!(
        with.dispatch(&click("btnSalvar")).unwrap().snapshot["dados"]["salvo"],
        true
    );
    assert_eq!(
        std::fs::read_to_string(dir.join("notas.txt")).unwrap(),
        "10"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn tela_fora_de_um_projeto_nao_tem_restricao() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    run.set_host(Box::new(|| Box::new(YesHost)));
    run.dispatch(&click("btnCopiar")).unwrap();
}

#[test]
fn analise_avisa_servico_sem_permissao_no_primeiro_uso() {
    let code = format!("// Notificacao.mostrar em comentário não conta\n{CODE}");
    let manifest = r#"<Projeto Nome="Notas" JanelaInicial="J" Permissoes="Dialogo" />"#;
    let inspection = ferro_service::inspect_code_behind_in_project(&code, DESIGN, Some(manifest));
    let warnings: Vec<_> = inspection
        .diagnostics
        .iter()
        .filter(|d| d.code == "design.missing-permission")
        .collect();
    assert_eq!(warnings.len(), 2, "{warnings:?}");
    assert!(
        warnings[0].message.contains("`AreaTransferencia`"),
        "{warnings:?}"
    );
    assert_eq!(
        warnings[0].help.as_deref(),
        Some(
            "Acrescente `AreaTransferencia` no .ferproj: Permissoes=\"Dialogo,AreaTransferencia\"."
        )
    );
    assert!(warnings[1].message.contains("`Arquivos`"));
    let line = code
        .lines()
        .position(|l| l.contains("AreaTransferencia.copiar"))
        .unwrap() as u32;
    assert_eq!(warnings[0].start.line, line + 1);

    let all = r#"<Projeto Nome="Notas" JanelaInicial="J" Permissoes="Dialogo,Arquivos,AreaTransferencia" />"#;
    let clean = ferro_service::inspect_code_behind_in_project(&code, DESIGN, Some(all));
    assert!(
        !clean
            .diagnostics
            .iter()
            .any(|d| d.code == "design.missing-permission")
    );
    let loose = ferro_service::inspect_code_behind(&code, DESIGN);
    assert!(
        !loose
            .diagnostics
            .iter()
            .any(|d| d.code == "design.missing-permission")
    );
}
