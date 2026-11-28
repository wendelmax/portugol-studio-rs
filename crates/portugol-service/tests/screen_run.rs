use portugol_service::{ScreenEvent, ScreenRun, ScreenStartError};
use serde_json::json;

const DESIGN: &str = r#"<Janela Nome="J" Titulo="Calculadora" Largura="320" Altura="240" Dados="Conta" AoAbrir="abrir">
    <Pilha>
        <CaixaTexto Nome="txtA" TextoVazio="Número" />
        <Botao Nome="btnSomar" Texto="Somar" AoClicar="btnSomar_clicar" />
        <Rotulo Nome="lblResultado" Texto="?" />
        <CaixaSelecao Nome="chkDobro" Texto="Dobrar" />
        <BotaoOpcao Nome="opA" Texto="A" Grupo="g" Marcado="verdadeiro" />
        <BotaoOpcao Nome="opB" Texto="B" Grupo="g" />
        <Combo Nome="cmb" AoSelecionar="cmb_selecionar">
            <Item>Um</Item>
            <Item>Dois</Item>
        </Combo>
    </Pilha>
</Janela>
"#;

const CODE: &str = r#"registro Conta {
    cliques: inteiro,
    escolha: cadeia
}

funcao abrir(tela: J) -> J {
    seja mutavel t: J = tela;
    t.titulo = "Calculadora pronta";
    retorne t;
}

funcao btnSomar_clicar(tela: J) -> J {
    seja mutavel t: J = tela;
    escolha Tipos.para_inteiro(t.txtA.texto) {
        caso Resultado::Ok(a) => {
            seja mutavel valor: inteiro = a + 1;
            se (t.chkDobro.marcado) {
                valor = valor * 2;
            }
            t.lblResultado.texto = Tipos.inteiro_para_cadeia(valor);
        }
        caso Resultado::Erro(_) => {
            t.lblResultado.texto = "Não é um número";
        }
    }
    t.dados.cliques += 1;
    escreva("clicou");
    retorne t;
}

funcao cmb_selecionar(tela: J) -> J {
    seja mutavel t: J = tela;
    seja itens: vetor<cadeia> = t.cmb.itens;
    t.dados.escolha = itens[t.cmb.selecionado];
    retorne t;
}
"#;

fn event(control: &str, event: &str, value: Option<serde_json::Value>) -> ScreenEvent {
    ScreenEvent {
        control: control.into(),
        event: event.into(),
        value,
    }
}

#[test]
fn inicia_com_os_valores_do_design_e_roda_ao_abrir() {
    let (run, step) = ScreenRun::start(DESIGN, CODE).unwrap();
    let snapshot = &step.snapshot;
    assert_eq!(step.handler.as_deref(), Some("abrir"));
    assert_eq!(snapshot["janela"]["titulo"], "Calculadora pronta");
    assert_eq!(snapshot["janela"]["largura"], 320);
    let txt = &snapshot["controles"]["txtA"];
    assert_eq!(txt["texto"], "");
    assert_eq!(txt["texto_vazio"], "Número");
    assert_eq!(txt["habilitado"], true);
    assert_eq!(txt["visivel"], true);
    assert_eq!(txt["largura"], 160, "tamanho padrão do esquema");
    assert_eq!(snapshot["controles"]["opA"]["marcado"], true);
    assert_eq!(snapshot["controles"]["cmb"]["itens"], json!(["Um", "Dois"]));
    assert_eq!(snapshot["controles"]["cmb"]["selecionado"], -1);
    assert_eq!(snapshot["dados"], json!({ "cliques": 0, "escolha": "" }));
    assert_eq!(run.history_len(), 2, "estado inicial e depois de AoAbrir");
}

#[test]
fn digitar_marcar_e_clicar_chamam_a_funcao_com_a_janela_atual() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let typed = run
        .dispatch(&event("txtA", "AoAlterar", Some(json!("20"))))
        .unwrap();
    assert_eq!(typed.handler, None, "AoAlterar não está ligado");
    assert_eq!(typed.snapshot["controles"]["txtA"]["texto"], "20");
    run.dispatch(&event("chkDobro", "AoAlterar", Some(json!(true))))
        .unwrap();
    let clicked = run.dispatch(&event("btnSomar", "AoClicar", None)).unwrap();
    assert_eq!(clicked.handler.as_deref(), Some("btnSomar_clicar"));
    assert_eq!(clicked.output, "clicou");
    assert_eq!(clicked.snapshot["controles"]["lblResultado"]["texto"], "42");
    assert_eq!(clicked.snapshot["dados"]["cliques"], 1);
    let again = run.dispatch(&event("btnSomar", "AoClicar", None)).unwrap();
    assert_eq!(again.snapshot["dados"]["cliques"], 2);
    assert_eq!(
        run.snapshot_at(1).unwrap()["dados"]["cliques"],
        0,
        "o histórico guarda os estados anteriores"
    );
}

#[test]
fn botao_de_opcao_desmarca_os_outros_do_grupo_e_combo_escolhe_item() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    let step = run
        .dispatch(&event("opB", "AoAlterar", Some(json!(true))))
        .unwrap();
    assert_eq!(step.snapshot["controles"]["opA"]["marcado"], false);
    assert_eq!(step.snapshot["controles"]["opB"]["marcado"], true);
    let chosen = run
        .dispatch(&event("cmb", "AoSelecionar", Some(json!(1))))
        .unwrap();
    assert_eq!(chosen.snapshot["controles"]["cmb"]["selecionado"], 1);
    assert_eq!(chosen.snapshot["dados"]["escolha"], "Dois");
}

#[test]
fn erro_no_handler_mantem_a_janela_como_estava() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    // Sem item escolhido (selecionado = -1), o índice está fora do vetor.
    let before = run.history_len();
    let error = run
        .dispatch(&event("cmb", "AoSelecionar", Some(json!(-1))))
        .unwrap_err();
    assert!(error.starts_with("Erro em `cmb_selecionar`"), "{error}");
    assert_eq!(run.history_len(), before);
    let ok = run
        .dispatch(&event("txtA", "AoAlterar", Some(json!("x"))))
        .unwrap();
    assert_eq!(
        ok.snapshot["controles"]["cmb"]["selecionado"], -1,
        "o valor aplicado antes do erro foi desfeito"
    );
}

#[test]
fn laco_infinito_e_interrompido_pelo_orcamento() {
    let code = CODE.replace("escreva(\"clicou\");", "enquanto (verdadeiro) {\n    }");
    let (run, _) = ScreenRun::start(DESIGN, &code).unwrap();
    let mut run = run.with_step_budget(10_000);
    let error = run
        .dispatch(&event("btnSomar", "AoClicar", None))
        .unwrap_err();
    assert!(error.contains("limite"), "{error}");
}

#[test]
fn nao_inicia_com_erros_e_explica_o_motivo() {
    let missing = CODE.replace("funcao abrir(", "funcao abrir_outra(");
    match ScreenRun::start(DESIGN, &missing) {
        Err(ScreenStartError::Invalid(message)) => {
            assert!(message.contains("`abrir`"), "{message}")
        }
        other => panic!("{:?}", other.map(|_| ())),
    }
    let broken = CODE.replace(
        "retorne t;\n}\n\nfuncao btnSomar",
        "retorne 1;\n}\n\nfuncao btnSomar",
    );
    assert!(
        matches!(ScreenRun::start(DESIGN, &broken), Err(ScreenStartError::Invalid(m)) if m.starts_with("Código da tela"))
    );
    assert!(
        matches!(ScreenRun::start("<Janela", CODE), Err(ScreenStartError::Invalid(m)) if m.starts_with("Desenho da tela"))
    );
}

#[test]
fn controle_desconhecido_e_erro() {
    let (mut run, _) = ScreenRun::start(DESIGN, CODE).unwrap();
    assert!(
        run.dispatch(&event("btnFantasma", "AoClicar", None))
            .is_err()
    );
}

#[test]
fn a_sessao_pode_ir_para_outra_thread() {
    fn assert_send<T: Send>() {}
    assert_send::<ScreenRun>();
}
