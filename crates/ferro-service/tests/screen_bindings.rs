use ferro_service::{
    CodeBehind, MarkupKind, ScreenEvent, ScreenRun, inspect_code_behind, inspect_markup_with_code,
};
use serde_json::json;

const CODE: &str = r#"registro Pessoa {
    nome: cadeia
}

registro Conta {
    nome: cadeia,
    idade: inteiro,
    pode_salvar: logico,
    escolha: inteiro,
    aceito: logico,
    pessoa: Pessoa,
    mensagem: cadeia
}
"#;

const SAVE: &str = r#"
funcao btnSalvar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    t.dados.mensagem = "Salvo: " + t.txtNome.texto;
    t.dados.pode_salvar = falso;
    retorne t;
}
"#;

fn design(body: &str) -> String {
    format!(
        "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"300\" Altura=\"200\" Dados=\"Conta\">\n    <Pilha>\n{body}\n    </Pilha>\n</Janela>\n"
    )
}

fn design_codes(body: &str, code: &str) -> Vec<String> {
    inspect_markup_with_code(&design(body), MarkupKind::Design, CodeBehind::Source(code))
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn vinculos_validos_nao_geram_diagnostico() {
    let body = r#"        <CaixaTexto Nome="txtNome" Texto="{dados.nome}" />
        <Rotulo Nome="lbl" Texto="Olá, {dados.nome}! Idade {dados.idade}, {dados.pessoa.nome}" />
        <Botao Nome="btnSalvar" Texto="Salvar" Habilitado="{dados.pode_salvar}" AoClicar="btnSalvar_clicar" />
        <CaixaSelecao Nome="chk" Texto="Aceito" Marcado="{dados.aceito}" />
        <Combo Nome="cmb" Selecionado="{dados.escolha}"><Item>A</Item></Combo>"#;
    assert_eq!(
        design_codes(body, &format!("{CODE}{SAVE}")),
        Vec::<String>::new()
    );
}

#[test]
fn campo_inexistente_lista_os_campos_do_registro() {
    let inspection = inspect_markup_with_code(
        &design(r#"        <Rotulo Nome="lbl" Texto="{dados.nme}" />"#),
        MarkupKind::Design,
        CodeBehind::Source(CODE),
    );
    let diagnostic = &inspection.diagnostics[0];
    assert_eq!(diagnostic.code, "design.binding-unknown-field");
    assert!(
        diagnostic.message.contains("`dados.nme`"),
        "{}",
        diagnostic.message
    );
    assert!(
        diagnostic.help.as_deref().unwrap().contains("nome, idade"),
        "{:?}",
        diagnostic.help
    );
}

#[test]
fn tipos_errados_nos_vinculos() {
    for body in [
        r#"        <Botao Nome="b" Texto="x" Habilitado="{dados.nome}" />"#,
        r#"        <CaixaTexto Nome="t" Texto="{dados.idade}" />"#,
        r#"        <Rotulo Nome="l" Texto="{dados.pessoa}" />"#,
        r#"        <Combo Nome="c" Selecionado="{dados.nome}" />"#,
    ] {
        assert_eq!(design_codes(body, CODE), ["design.binding-type"], "{body}");
    }
    assert_eq!(
        design_codes(
            r#"        <Rotulo Nome="l" Texto="{dados.nome.x}" />"#,
            CODE
        ),
        ["design.binding-not-record"]
    );
}

#[test]
fn dados_de_exemplo_conferidos_com_o_registro() {
    let source = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"300\" Altura=\"200\" Dados=\"Conta\">\n    <DadosExemplo nome=\"Ana\" idade=\"9\" pode_salvar=\"verdadeiro\" />\n</Janela>\n";
    assert!(
        inspect_markup_with_code(source, MarkupKind::Design, CodeBehind::Source(CODE))
            .diagnostics
            .is_empty()
    );
    let wrong = source.replace("idade=\"9\"", "idade=\"nove\" altura=\"1\"");
    let codes: Vec<_> =
        inspect_markup_with_code(&wrong, MarkupKind::Design, CodeBehind::Source(CODE))
            .diagnostics
            .into_iter()
            .map(|d| d.code)
            .collect();
    assert_eq!(codes, ["design.sample-type", "design.sample-unknown-field"]);
}

#[test]
fn propriedade_vinculada_e_so_de_leitura_no_codigo() {
    let screen = design(r#"        <Rotulo Nome="lbl" Texto="{dados.mensagem}" />"#)
        .replace("Titulo=\"T\"", "Titulo=\"{dados.nome}\"");
    let code = format!(
        "{CODE}\nfuncao f(tela: J) -> J {{\n    seja mutavel t: J = tela;\n    se (verdadeiro) {{\n        t.lbl.texto = \"x\";\n    }}\n    t.titulo = \"y\";\n    retorne t;\n}}\n"
    );
    let inspection = inspect_code_behind(&code, &screen);
    let codes: Vec<_> = inspection
        .diagnostics
        .iter()
        .map(|d| d.code.as_str())
        .collect();
    assert_eq!(
        codes,
        ["design.binding-read-only", "design.binding-read-only"],
        "{:?}",
        inspection.diagnostics
    );
    let first = &inspection.diagnostics[0];
    assert!(
        first
            .message
            .starts_with("`t.lbl.texto` está vinculado a `{dados.mensagem}`"),
        "{}",
        first.message
    );
    assert_eq!(
        first.help.as_deref(),
        Some("Mude `t.dados.mensagem`; a tela acompanha.")
    );
}

fn event(control: &str, event: &str, value: Option<serde_json::Value>) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: event.into(),
        value,
    }
}

#[test]
fn mao_dupla_na_execucao() {
    let screen = design(
        r#"        <CaixaTexto Nome="txtNome" Texto="{dados.nome}" />
        <Rotulo Nome="lbl" Texto="Olá, {dados.nome}!" />
        <Rotulo Nome="lblMensagem" Texto="{dados.mensagem}" />
        <Botao Nome="btnSalvar" Texto="Salvar" Habilitado="{dados.pode_salvar}" AoClicar="btnSalvar_clicar" />
        <Combo Nome="cmb" Selecionado="{dados.escolha}"><Item>A</Item><Item>B</Item></Combo>"#,
    );
    let code = format!("{CODE}{SAVE}").replace(
        "t.dados.pode_salvar = falso;",
        "t.dados.pode_salvar = falso;\n    t.dados.nome = \"\";",
    );
    let (mut run, start) = ScreenRun::start(&screen, &code).unwrap();
    assert_eq!(
        start.snapshot["controles"]["btnSalvar"]["habilitado"], false,
        "dados.pode_salvar começa falso"
    );
    assert_eq!(
        start.snapshot["controles"]["cmb"]["selecionado"], 0,
        "dados.escolha começa 0"
    );

    let typed = run
        .dispatch(&event("txtNome", "AoAlterar", Some(json!("Ana"))))
        .unwrap();
    assert_eq!(
        typed.snapshot["dados"]["nome"], "Ana",
        "digitar atualiza dados"
    );
    assert_eq!(typed.snapshot["controles"]["txtNome"]["texto"], "Ana");
    assert_eq!(
        typed.snapshot["controles"]["lbl"]["texto"], "Olá, Ana!",
        "o rótulo acompanha"
    );

    let chosen = run
        .dispatch(&event("cmb", "AoSelecionar", Some(json!(1))))
        .unwrap();
    assert_eq!(chosen.snapshot["dados"]["escolha"], 1);

    let saved = run.dispatch(&event("btnSalvar", "AoClicar", None)).unwrap();
    assert_eq!(
        saved.snapshot["dados"]["mensagem"], "Salvo: Ana",
        "o código leu t.txtNome.texto vindo de dados"
    );
    assert_eq!(
        saved.snapshot["controles"]["lblMensagem"]["texto"],
        "Salvo: Ana"
    );
    assert_eq!(
        saved.snapshot["controles"]["txtNome"]["texto"], "",
        "mudar dados.nome limpa o campo"
    );
    assert_eq!(saved.snapshot["controles"]["lbl"]["texto"], "Olá, !");
}

#[test]
fn campos_de_dados_para_os_exemplos() {
    let fields = ferro_service::screen_data_fields(&design(""), CODE);
    let summary: Vec<_> = fields
        .iter()
        .map(|f| (f.name.as_str(), f.ty.as_str(), f.editable))
        .collect();
    assert_eq!(
        summary,
        [
            ("nome", "cadeia", true),
            ("idade", "inteiro", true),
            ("pode_salvar", "logico", true),
            ("escolha", "inteiro", true),
            ("aceito", "logico", true),
            ("pessoa", "Pessoa", false),
            ("mensagem", "cadeia", true),
        ]
    );
    assert!(ferro_service::screen_data_fields(&design(""), "funcao quebrada(").is_empty());
}
