use ferro_rust::export_screen_cargo_project;

fn counter() -> (String, String) {
    let template = ferro_tela::project::example_template("contador", "Contador").unwrap();
    let file = |ext: &str| {
        template
            .files
            .iter()
            .find(|f| f.path.ends_with(ext))
            .unwrap()
            .content
            .clone()
    };
    (file(".design"), file(".fer"))
}

pub const CADASTRO_DESIGN: &str = r#"<Janela Nome="Cadastro" Titulo="Cadastro" Largura="360" Altura="240" Dados="Pessoa" AoAbrir="Cadastro_abrir">
    <CaixaTexto Nome="txtNome" X="8" Y="8" Texto="{dados.nome}" TextoVazio="Seu nome" />
    <Combo Nome="cmb" X="8" Y="44" Selecionado="{dados.opcao}"><Item>A</Item><Item>B</Item></Combo>
    <CaixaSelecao Nome="chk" X="8" Y="80" Texto="Aceito" Marcado="{dados.aceito}" />
    <Lista Nome="lst" X="180" Y="8"><Item>um</Item></Lista>
    <Rotulo Nome="lblResumo" X="8" Y="120" Largura="200" Texto="{dados.resumo}" />
    <Botao Nome="btnOk" X="8" Y="160" Texto="Somar" Habilitado="{dados.aceito}" AoClicar="btnOk_clicar" />
    <Temporizador Nome="relogio" Intervalo="1000" AoDisparar="relogio_disparar" />
</Janela>
"#;

pub const CADASTRO_CODE: &str = r#"registro Item {
    nome: cadeia,
    preco: real
}

registro Pessoa {
    nome: cadeia,
    opcao: inteiro,
    aceito: logico,
    resumo: cadeia,
    itens: vetor<Item>,
    ticks: inteiro
}

funcao Cadastro_abrir(tela: Cadastro) -> Cadastro {
    seja mutavel t: Cadastro = tela;
    t.titulo = "Cadastro aberto";
    retorne t;
}

funcao btnOk_clicar(tela: Cadastro) -> Cadastro {
    seja mutavel t: Cadastro = tela;
    t.dados.resumo = t.dados.nome + " escolheu " + Tipos.inteiro_para_cadeia(t.dados.opcao);
    t.lst.itens = ["um", "dois", t.dados.nome];
    t.btnOk.texto = "De novo";
    retorne t;
}

funcao relogio_disparar(tela: Cadastro) -> Cadastro {
    seja mutavel t: Cadastro = tela;
    t.dados.ticks = t.dados.ticks + 1;
    retorne t;
}
"#;

fn write(project: &ferro_rust::ScreenCargoProject, dir: &std::path::Path) {
    for (path, content) in &project.files {
        let target = dir.join(path);
        std::fs::create_dir_all(target.parent().unwrap()).unwrap();
        std::fs::write(target, content).unwrap();
    }
}

#[test]
fn contador_vira_projeto_cargo_com_slint() {
    let (design, code) = counter();
    let project = export_screen_cargo_project(&design, &code, "Meu Contador").unwrap();
    assert_eq!(project.package_name, "meu-contador");
    let paths: Vec<&str> = project.files.iter().map(|(p, _)| p.as_str()).collect();
    assert_eq!(
        paths,
        [
            "Cargo.toml",
            "build.rs",
            "ui/JanelaPrincipal.slint",
            "src/main.rs",
            "README.md"
        ]
    );
    let file = |name: &str| &project.files.iter().find(|(p, _)| p == name).unwrap().1;
    assert!(
        file("Cargo.toml").contains("slint = \"1\"")
            && file("Cargo.toml").contains("slint-build = \"1\"")
    );
    let main = file("src/main.rs");
    assert!(
        main.contains("fn pt_btnInc_clicar(pt_tela: pt_JanelaPrincipal) -> pt_JanelaPrincipal {"),
        "{main}"
    );
    assert!(main.contains("ui.on_btnInc_clicar(move || {"), "{main}");
    assert!(
        main.contains("let tela = pt_btnInc_clicar(__ler_tela(&ui));"),
        "{main}"
    );
    assert!(
        main.contains("ui.set_dados(__para_slint_Estado(&tela.pt_dados));"),
        "{main}"
    );
    assert_eq!(main.matches("fn main()").count(), 1, "{main}");
    assert!(file("README.md").contains("Made with Slint"));
    if let Ok(dir) = std::env::var("FERRO_EXPORTAR_EM") {
        write(&project, &std::path::Path::new(&dir).join("contador"));
        let cadastro =
            export_screen_cargo_project(CADASTRO_DESIGN, CADASTRO_CODE, "cadastro").unwrap();
        write(&cadastro, &std::path::Path::new(&dir).join("cadastro"));
    }
}

#[test]
fn erros_e_recursos_fora_da_exportacao_explicam() {
    let (design, _) = counter();
    let error = export_screen_cargo_project(&design, "registro Estado { contagem: inteiro }", "x")
        .unwrap_err();
    assert!(error.contains("btnInc_clicar"), "{error}");
    let error = export_screen_cargo_project(&design, "registro Estado { contagem: inteiro ", "x")
        .unwrap_err();
    assert!(error.contains("JanelaPrincipal.fer:"), "{error}");
}

const SERVICOS_DESIGN: &str = r#"<Janela Nome="J" Titulo="Serviços" Largura="300" Altura="200" Dados="Estado">
    <Botao Nome="btnOk" X="8" Y="8" Texto="Ok" AoClicar="btnOk_clicar" />
    <DialogoArquivo Nome="dlgAbrir" Titulo="Abrir" Filtro="*.txt" />
</Janela>
"#;

const SERVICOS_CODE: &str = r#"registro Estado {
    ok: logico,
    caminho: cadeia,
    colado: cadeia
}

funcao btnOk_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    Dialogo.mensagem("Olá");
    t.dados.ok = Dialogo.confirmar("Continuar?");
    escolha Dialogo.abrir_arquivo(t.dlgAbrir) {
        caso Opcao::Algum(caminho) => { t.dados.caminho = caminho; }
        caso Opcao::Nenhum => { }
    }
    AreaTransferencia.copiar("x");
    escolha AreaTransferencia.colar() {
        caso Opcao::Algum(texto) => { t.dados.colado = texto; }
        caso Opcao::Nenhum => { }
    }
    Notificacao.mostrar("Pronto", "Feito");
    retorne t;
}
"#;

#[test]
fn servicos_do_sistema_trazem_o_codigo_e_a_dependencia() {
    let project = export_screen_cargo_project(SERVICOS_DESIGN, SERVICOS_CODE, "servicos").unwrap();
    let file = |name: &str| {
        project
            .files
            .iter()
            .find(|(p, _)| p == name)
            .unwrap()
            .1
            .clone()
    };
    let cargo = file("Cargo.toml");
    for dependency in ["rfd = \"0.15\"", "arboard = \"3\"", "notify-rust = \"4\""] {
        assert!(cargo.contains(dependency), "{cargo}");
    }
    let main = file("src/main.rs");
    assert!(main.contains("__ferro_dialogo::mensagem(&("), "{main}");
    assert!(main.contains("__ferro_dialogo::abrir_arquivo(&("), "{main}");
    assert!(main.contains("mod __ferro_notificacao {"), "{main}");
    if let Ok(dir) = std::env::var("FERRO_EXPORTAR_EM") {
        write(&project, &std::path::Path::new(&dir).join("servicos"));
    }

    let (design, code) = counter();
    let plain = export_screen_cargo_project(&design, &code, "x").unwrap();
    let cargo = &plain.files[0].1;
    assert!(
        !cargo.contains("rfd") && !cargo.contains("arboard"),
        "só o que a tela usa: {cargo}"
    );
    let comment = format!("// Dialogo.mensagem só no comentário\n{code}");
    let commented = export_screen_cargo_project(&design, &comment, "x").unwrap();
    assert!(!commented.files[0].1.contains("rfd"));
}
