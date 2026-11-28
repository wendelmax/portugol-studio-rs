//! Várias janelas (ADR 0018, Emenda 5): uma tela abre outra com
//! `Telas.abrir`, entrega os dados e recebe a resposta numa função de retorno.

use ferro_service::{ScreenEvent, ScreenRun};

const PRINCIPAL: &str = r#"<Janela Nome="Principal" Titulo="Contas" Largura="300" Altura="200" Dados="Conta" AoAbrir="iniciar">
    <Rotulo Nome="lblNome" Texto="{dados.nome}" X="8" Y="8" />
    <Botao Nome="btnEditar" Texto="Editar" X="8" Y="40" AoClicar="btnEditar_clicar" />
    <Botao Nome="btnSobre" Texto="Sobre" X="8" Y="80" AoClicar="btnSobre_clicar" />
</Janela>
"#;

const PRINCIPAL_CODE: &str = r#"registro Conta {
    nome: cadeia,
    saldo: real
}

funcao iniciar(tela: Principal) -> Principal {
    seja mutavel t: Principal = tela;
    t.dados.nome = "Bia";
    retorne t;
}

funcao btnEditar_clicar(tela: Principal) -> Principal {
    Telas.abrir("EditarConta", tela.dados, "edicao_terminou");
    retorne tela;
}

funcao btnSobre_clicar(tela: Principal) -> Principal {
    Telas.abrir("Sobre");
    retorne tela;
}

funcao edicao_terminou(tela: Principal, resposta: Opcao<Conta>) -> Principal {
    seja mutavel t: Principal = tela;
    escolha resposta {
        caso Opcao::Algum(conta) => { t.dados = conta; }
        caso Opcao::Nenhum => { t.dados.nome = "(cancelado)"; }
    }
    retorne t;
}
"#;

const EDITAR: &str = r#"<Janela Nome="EditarConta" Titulo="Editar" Largura="240" Altura="120" Dados="Conta" AoAbrir="abrir">
    <CaixaTexto Nome="txtNome" Texto="{dados.nome}" X="8" Y="8" />
    <Botao Nome="btnSalvar" Texto="Salvar" X="8" Y="40" AoClicar="btnSalvar_clicar" />
    <Botao Nome="btnCancelar" Texto="Cancelar" X="96" Y="40" AoClicar="btnCancelar_clicar" />
</Janela>
"#;

const EDITAR_CODE: &str = r#"registro Conta {
    nome: cadeia,
    saldo: real
}

funcao abrir(tela: EditarConta) -> EditarConta {
    seja mutavel t: EditarConta = tela;
    t.titulo = "Editar " + t.dados.nome;
    retorne t;
}

funcao btnSalvar_clicar(tela: EditarConta) -> EditarConta {
    Telas.concluir();
    retorne tela;
}

funcao btnCancelar_clicar(tela: EditarConta) -> EditarConta {
    Telas.fechar();
    retorne tela;
}
"#;

fn click(control: &str) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: "AoClicar".into(),
        value: None,
    }
}

fn principal() -> ScreenRun {
    ScreenRun::start(PRINCIPAL, PRINCIPAL_CODE).unwrap().0
}

#[test]
fn abre_a_outra_tela_com_os_dados_e_recebe_a_conta_editada() {
    let mut run = principal();
    let step = run.dispatch(&click("btnEditar")).unwrap();
    let open = step.open.clone().expect("pediu para abrir");
    assert_eq!(open.screen, "EditarConta");
    assert_eq!(open.callback.as_deref(), Some("edicao_terminou"));
    assert_eq!(step.close, None);

    let (mut child, first) = run.open(&open, EDITAR, EDITAR_CODE).unwrap();
    assert_eq!(
        first.snapshot["janela"]["titulo"], "Editar Bia",
        "AoAbrir já vê os dados"
    );
    let typed = ScreenEvent {
        control: "txtNome".into(),
        event: "AoAlterar".into(),
        value: Some("Ana".into()),
    };
    child.dispatch(&typed).unwrap();
    let saved = child.dispatch(&click("btnSalvar")).unwrap();
    assert_eq!(saved.close, Some(true));
    assert_eq!(saved.open, None);

    let back = run.resume(&open, child.data()).unwrap();
    assert_eq!(back.handler.as_deref(), Some("edicao_terminou"));
    assert_eq!(back.snapshot["dados"]["nome"], "Ana");
    assert_eq!(back.snapshot["controles"]["lblNome"]["texto"], "Ana");
}

#[test]
fn fechar_entrega_nenhum() {
    let mut run = principal();
    let open = run.dispatch(&click("btnEditar")).unwrap().open.unwrap();
    let (mut child, _) = run.open(&open, EDITAR, EDITAR_CODE).unwrap();
    assert_eq!(
        child.dispatch(&click("btnCancelar")).unwrap().close,
        Some(false)
    );
    let back = run.resume(&open, None).unwrap();
    assert_eq!(back.snapshot["dados"]["nome"], "(cancelado)");
}

#[test]
fn tela_sem_dados_e_sem_resposta() {
    let mut run = principal();
    let open = run.dispatch(&click("btnSobre")).unwrap().open.unwrap();
    assert_eq!(open.callback, None);
    let sobre = r#"<Janela Nome="Sobre" Titulo="Sobre" Largura="200" Altura="100" />"#;
    run.open(&open, sobre, "").unwrap();
    let before = run.current_step();
    assert_eq!(run.resume(&open, None).unwrap(), before);
}

#[test]
fn registro_diferente_na_outra_tela_e_explicado() {
    let mut run = principal();
    let open = run.dispatch(&click("btnEditar")).unwrap().open.unwrap();
    let other = EDITAR_CODE.replace("saldo: real", "saldo: inteiro");
    let error = run.open(&open, EDITAR, &other).err().unwrap().to_string();
    assert!(
        error.contains("Conta{nome:cadeia,saldo:inteiro}"),
        "{error}"
    );
    assert!(error.contains("Conta{nome:cadeia,saldo:real}"), "{error}");

    let wrong_name = EDITAR.replace("Nome=\"EditarConta\"", "Nome=\"Outra\"");
    let other = EDITAR_CODE.replace("EditarConta", "Outra");
    let error = run
        .open(&open, &wrong_name, &other)
        .err()
        .unwrap()
        .to_string();
    assert!(error.contains("precisa se chamar `EditarConta`"), "{error}");
}

#[test]
fn pedidos_em_conflito_no_mesmo_evento() {
    let code = PRINCIPAL_CODE.replace(
        "    Telas.abrir(\"Sobre\");\n",
        "    Telas.abrir(\"Sobre\");\n    Telas.fechar();\n",
    );
    let (mut run, _) = ScreenRun::start(PRINCIPAL, &code).unwrap();
    let error = run.dispatch(&click("btnSobre")).unwrap_err();
    assert!(error.contains("abrir outra e para fechar"), "{error}");
}

#[test]
fn a_analise_confere_a_funcao_de_retorno() {
    let start = |code: &str| {
        ScreenRun::start(PRINCIPAL, code)
            .err()
            .map(|e| e.to_string())
    };
    let missing = PRINCIPAL_CODE.replace("\"edicao_terminou\");", "\"nao_existe\");");
    assert!(start(&missing).unwrap().contains("`nao_existe` não existe"));
    let wrong = PRINCIPAL_CODE.replace("resposta: Opcao<Conta>", "resposta: Conta");
    assert!(start(&wrong).unwrap().contains("Opcao<Conta>"));
    let not_literal = PRINCIPAL_CODE.replace(
        "Telas.abrir(\"Sobre\");",
        "seja nome: cadeia = \"Sobre\";\n    Telas.abrir(nome);",
    );
    assert!(start(&not_literal).unwrap().contains("texto entre aspas"));
}
