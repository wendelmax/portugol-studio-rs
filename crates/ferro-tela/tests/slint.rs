use ferro_tela::codebehind::screen_type;
use ferro_tela::parse;
use ferro_tela::slint::{DataField, DataRecord, DataType, export};

fn slint(design: &str, data: Option<&DataRecord>) -> String {
    let document = parse(design).unwrap();
    let (screen, _) = screen_type(&document);
    export(&document, &screen, data)
}

#[test]
fn contador_vira_componente_com_dados_e_callback() {
    let template = ferro_tela::project::example_template("contador", "Contador").unwrap();
    let design = &template
        .files
        .iter()
        .find(|f| f.path.ends_with(".design"))
        .unwrap()
        .content;
    let data = DataRecord {
        name: "Estado".into(),
        fields: vec![DataField {
            name: "contagem".into(),
            ty: DataType::Int,
        }],
    };
    let out = slint(design, Some(&data));
    println!("{out}");
    assert!(
        out.contains("export struct Estado {\n    contagem: int,\n}"),
        "{out}"
    );
    assert!(
        out.contains("export component JanelaPrincipal inherits Window {"),
        "{out}"
    );
    assert!(
        out.contains("    title: root.JanelaPrincipal-titulo;"),
        "{out}"
    );
    assert!(
        out.contains("    in-out property <string> JanelaPrincipal-titulo: \"Contador\";"),
        "{out}"
    );
    assert!(
        out.contains("    width: root.JanelaPrincipal-largura * 1px;"),
        "{out}"
    );
    assert!(
        out.contains("    in-out property <int> JanelaPrincipal-largura: 260;"),
        "{out}"
    );
    assert!(out.contains("    in-out property <Estado> dados;"), "{out}");
    assert!(out.contains("    callback btnInc_clicar();"), "{out}");
    assert!(out.contains("VerticalLayout {"), "{out}");
    assert!(out.contains("lblContagem := Text {"), "{out}");
    assert!(
        out.contains("text: \"Contagem: \\{root.dados.contagem}\";"),
        "{out}"
    );
    assert!(out.contains("btnInc := Button {"), "{out}");
    assert!(
        out.contains("clicked => { root.btnInc_clicar(); }"),
        "{out}"
    );
    assert!(
        !out.contains("DadosExemplo") && !out.contains("contagem: 3"),
        "exemplos do designer não são exportados"
    );
    // O texto vinculado do rótulo é só lido pelo adaptador.
    assert!(
        out.contains(
            "out property <string> lblContagem-texto: \"Contagem: \\{root.dados.contagem}\";"
        ),
        "{out}"
    );
    assert!(
        out.contains("in-out property <bool> btnInc-habilitado: true;"),
        "{out}"
    );
}

#[test]
fn posicao_livre_controles_de_entrada_e_mao_dupla() {
    let design = r#"<Janela Nome="J" Titulo="Cadastro" Largura="300" Altura="200" Dados="Pessoa">
    <CaixaTexto Nome="txtNome" X="8" Y="8" Texto="{dados.nome}" TextoVazio="Seu nome" AoAlterar="txtNome_alterar" />
    <CaixaTexto Nome="txtSenha" X="8" Y="44" Senha="verdadeiro" />
    <Combo Nome="cmb" X="8" Y="80" Selecionado="{dados.opcao}"><Item>A "x"</Item><Item>B</Item></Combo>
    <Lista Nome="lst" X="160" Y="8" />
    <Temporizador Nome="relogio" Intervalo="500" Ativo="verdadeiro" AoDisparar="relogio_disparar" />
</Janela>
"#;
    let data = DataRecord {
        name: "Pessoa".into(),
        fields: vec![
            DataField {
                name: "nome".into(),
                ty: DataType::String,
            },
            DataField {
                name: "opcao".into(),
                ty: DataType::Int,
            },
        ],
    };
    let out = slint(design, Some(&data));
    assert!(out.contains("txtNome := LineEdit {"), "{out}");
    assert!(out.contains("x: root.txtNome-x * 1px;"), "{out}");
    assert!(out.contains("text <=> root.dados.nome;"), "{out}");
    assert!(
        out.contains("placeholder-text: root.txtNome-texto-vazio;"),
        "{out}"
    );
    assert!(
        out.contains("in-out property <string> txtNome-texto-vazio: \"Seu nome\";"),
        "{out}"
    );
    assert!(
        out.contains("edited(text) => { root.txtNome_alterar(); }"),
        "{out}"
    );
    assert!(out.contains("text <=> root.txtSenha-texto;"), "{out}");
    assert!(out.contains("input-type: password;"), "{out}");
    assert!(out.contains("current-index <=> root.dados.opcao;"), "{out}");
    assert!(
        out.contains("in-out property <[string]> cmb-itens: [\"A \\\"x\\\"\", \"B\"];"),
        "{out}"
    );
    assert!(out.contains("model: root.lst-itens-lista;"), "{out}");
    assert!(out.contains("relogio := Timer {"), "{out}");
    assert!(
        out.contains("interval: root.relogio-intervalo * 1ms;"),
        "{out}"
    );
    assert!(
        out.contains("in-out property <bool> relogio-ativo: true;"),
        "{out}"
    );
    assert!(
        out.contains("triggered => { root.relogio_disparar(); }"),
        "{out}"
    );
}

#[test]
fn grade_e_encaixe_viram_layouts() {
    let design = r#"<Janela Nome="J" Titulo="T" Largura="300" Altura="200">
    <Encaixe>
        <Rotulo Texto="Topo" Encaixar="Cima" />
        <Grade Linhas="Auto,*" Colunas="100,*" Espaco="4">
            <Rotulo Texto="Nome" Linha="0" Coluna="0" />
            <Rotulo Texto="Valor \{literal\}" Linha="1" Coluna="1" />
        </Grade>
    </Encaixe>
</Janela>
"#;
    let out = slint(design, None);
    assert!(
        !out.contains("property <") || !out.contains("dados"),
        "sem Dados não há propriedade dados"
    );
    let topo = out.find("text: \"Topo\";").unwrap();
    let grade = out.find("GridLayout {").unwrap();
    assert!(topo < grade, "o filho de cima vem antes do resto: {out}");
    assert!(out.contains("spacing: 4px;"), "{out}");
    assert!(
        out.contains("row: 1;\n") && out.contains("col: 1;\n"),
        "{out}"
    );
    assert!(
        out.contains("text: \"Valor {literal}\";"),
        "chave literal vira chave comum no Slint: {out}"
    );
}
