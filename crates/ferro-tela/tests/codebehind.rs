use ferro_tela::codebehind::{FieldType, screen_type};
use ferro_tela::parse;

const CALCULADORA: &str = r#"<Janela Nome="JanelaPrincipal" Titulo="Calculadora" Largura="360" Altura="240" Dados="Conta">
    <Pilha Margem="16" Espaco="8">
        <CaixaTexto Nome="txtA" TextoVazio="Primeiro número" />
        <CaixaTexto Nome="txtB" />
        <Botao Nome="btnSomar" Texto="Somar" AoClicar="btnSomar_clicar" />
        <Combo Nome="cmbOperacao"><Item>Somar</Item></Combo>
        <Rotulo Texto="sem nome" />
    </Pilha>
    <Temporizador Nome="relogio" Intervalo="1000" />
</Janela>
"#;

fn field<'a>(fields: &'a [ferro_tela::codebehind::FieldDef], name: &str) -> &'a FieldType {
    &fields
        .iter()
        .find(|f| f.name == name)
        .unwrap_or_else(|| panic!("sem campo {name}"))
        .ty
}

#[test]
fn janela_tem_um_campo_por_controle_nomeado_e_dados() {
    let (screen, diagnostics) = screen_type(&parse(CALCULADORA).unwrap());
    assert!(diagnostics.is_empty(), "{diagnostics:?}");
    assert_eq!(screen.window.name, "JanelaPrincipal");
    let names: Vec<_> = screen
        .window
        .fields
        .iter()
        .map(|f| f.name.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "titulo",
            "largura",
            "altura",
            "redimensionavel",
            "txtA",
            "txtB",
            "btnSomar",
            "cmbOperacao",
            "relogio",
            "dados"
        ]
    );
    assert_eq!(
        field(&screen.window.fields, "txtA"),
        &FieldType::Named("CaixaTexto".into())
    );
    assert_eq!(
        field(&screen.window.fields, "dados"),
        &FieldType::Named("Conta".into())
    );
    assert_eq!(screen.data.as_ref().map(|(n, _)| n.as_str()), Some("Conta"));
    // A Pilha sem nome e o Rótulo sem nome não viram campos.
    let controls: Vec<_> = screen.controls.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(controls, ["CaixaTexto", "Botao", "Combo", "Temporizador"]);
}

#[test]
fn controles_expoem_propriedades_em_minusculas() {
    let (screen, _) = screen_type(&parse(CALCULADORA).unwrap());
    let caixa = &screen.controls[0];
    assert_eq!(field(&caixa.fields, "texto"), &FieldType::Text);
    assert_eq!(field(&caixa.fields, "texto_vazio"), &FieldType::Text);
    assert_eq!(field(&caixa.fields, "habilitado"), &FieldType::Bool);
    assert_eq!(field(&caixa.fields, "x"), &FieldType::Integer);
    let names: Vec<_> = caixa.fields.iter().map(|f| f.name.as_str()).collect();
    for hidden in [
        "nome",
        "margem",
        "ancora",
        "linha",
        "alinhamento",
        "ao_alterar",
    ] {
        assert!(
            !names.contains(&hidden),
            "{hidden} não deveria aparecer: {names:?}"
        );
    }
    let combo = screen.controls.iter().find(|c| c.name == "Combo").unwrap();
    assert_eq!(field(&combo.fields, "itens"), &FieldType::TextList);
    assert_eq!(field(&combo.fields, "selecionado"), &FieldType::Integer);
    let relogio = screen
        .controls
        .iter()
        .find(|c| c.name == "Temporizador")
        .unwrap();
    assert_eq!(field(&relogio.fields, "intervalo"), &FieldType::Integer);
    assert_eq!(field(&relogio.fields, "ativo"), &FieldType::Bool);
}

#[test]
fn sem_dados_nao_ha_campo_dados_e_conflito_com_propriedade_da_janela_e_erro() {
    let source = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <Rotulo Nome=\"titulo\" X=\"0\" Y=\"0\" />\n</Janela>\n";
    let (screen, diagnostics) = screen_type(&parse(source).unwrap());
    assert!(screen.data.is_none());
    assert!(screen.window.fields.iter().all(|f| f.name != "dados"));
    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "design.name-conflict");
    assert_eq!(
        &source[diagnostics[0].span.start as usize..diagnostics[0].span.end as usize],
        "titulo"
    );
}
