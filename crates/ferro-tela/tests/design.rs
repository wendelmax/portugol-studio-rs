use ferro_tela::schema::{self, Category, Layout};
use ferro_tela::value::{self, TextPart, Thickness, Track};
use ferro_tela::{Severity, analyze, format_source, parse};

/// Nível 1: posição livre, como no Windows Forms.
const OLA: &str = r#"<Janela Nome="JanelaPrincipal" Titulo="Olá" Largura="300" Altura="160" Formato="1">
    <Rotulo Nome="lblMensagem" X="16" Y="16" Texto="Clique no botão" />
    <Botao Nome="btnOla" X="16" Y="56" Texto="Dizer _olá" AoClicar="btnOla_clicar" />
</Janela>
"#;

/// Níveis 2 e 3: layout e vínculos, como no WPF.
const CALCULADORA: &str = r#"<!-- Janela inicial do projeto. -->
<Janela
    Nome="JanelaPrincipal"
    Titulo="Calculadora"
    Largura="360"
    Altura="240"
    Dados="Conta"
    Formato="1">
    <Encaixe>
        <Rotulo Encaixar="Cima" Texto="Soma de dois números" Margem="8" />

        <Grade Linhas="Auto,Auto,*" Colunas="120,*" Espaco="8" Preenchimento="16">
            <Rotulo Linha="0" Coluna="0" Texto="Primeiro:" AlinhamentoVertical="Centro" />
            <CaixaTexto Nome="txtA" Linha="0" Coluna="1" Texto="{dados.a}" />
            <Rotulo Linha="1" Coluna="0" Texto="Segundo:" />
            <CaixaTexto Nome="txtB" Linha="1" Coluna="1" Texto="{dados.b}" TextoVazio="0" />
            <Pilha Linha="2" Coluna="1" Orientacao="Horizontal" Espaco="8">
                <Combo Nome="cmbOperacao" Selecionado="0">
                    <Item>Somar</Item>
                    <Item>Subtrair &amp; dividir</Item>
                </Combo>
                <Botao
                    Nome="btnSomar"
                    Texto="_Somar"
                    AoClicar="btnSomar_clicar"
                    Habilitado="{dados.pode_somar}" />
                <Rotulo Nome="lblResultado" Texto="Resultado: {dados.resultado}" />
            </Pilha>
        </Grade>
    </Encaixe>
    <Temporizador
        Nome="tmrRelogio"
        Intervalo="1000"
        Ativo="falso"
        AoDisparar="tmrRelogio_disparar" />
    <DadosExemplo resultado="42" pode_somar="verdadeiro" />
</Janela>
"#;

fn codes(source: &str) -> Vec<&'static str> {
    analyze(source).diagnostics.iter().map(|d| d.code).collect()
}

fn window(body: &str) -> String {
    format!(
        "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"100\" Altura=\"100\" Dados=\"D\">\n{body}\n</Janela>\n"
    )
}

#[test]
fn nivel_1_posicao_livre_sem_diagnosticos() {
    let analysis = analyze(OLA);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let document = analysis.document.unwrap();
    let botao = document.find("btnOla").unwrap();
    assert_eq!(botao.attribute("Texto").unwrap().value, "Dizer _olá");
}

#[test]
fn niveis_2_e_3_layout_e_vinculos_sem_diagnosticos() {
    let analysis = analyze(CALCULADORA);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let document = analysis.document.unwrap();
    let names: Vec<_> = document
        .descendants()
        .iter()
        .filter_map(|e| e.control_name().map(|n| n.value.clone()))
        .collect();
    assert_eq!(
        names,
        [
            "txtA",
            "txtB",
            "cmbOperacao",
            "btnSomar",
            "lblResultado",
            "tmrRelogio"
        ]
    );
    let combo = document.find("cmbOperacao").unwrap();
    let items: Vec<_> = combo
        .elements()
        .map(|item| item.texts().next().unwrap().text.clone())
        .collect();
    assert_eq!(items, ["Somar", "Subtrair & dividir"]);
}

#[test]
fn formatacao_canonica_e_estavel() {
    assert_eq!(format_source(OLA).unwrap(), OLA);
    assert_eq!(format_source(CALCULADORA).unwrap(), CALCULADORA);
}

#[test]
fn formatacao_normaliza_e_preserva_comentarios() {
    let messy = "<!--topo-->\n<Janela   Nome='J' Titulo=\"T\"\n Largura=\"1\" Altura=\"1\">\n  <!-- botão -->\n<Botao X=\"1\" Y=\"2\"></Botao>\n\n\n  <Rotulo X=\"1\" Y=\"2\"/>\n</Janela>\n<!-- fim -->";
    let expected = "<!--topo-->\n<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\" Formato=\"1\">\n    <!-- botão -->\n    <Botao X=\"1\" Y=\"2\" />\n\n    <Rotulo X=\"1\" Y=\"2\" />\n</Janela>\n<!-- fim -->\n";
    let formatted = format_source(messy).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn tag_longa_fica_com_um_atributo_por_linha() {
    let long = "x".repeat(80);
    let source = window(&format!("    <Rotulo X=\"1\" Y=\"2\" Texto=\"{long}\" />"));
    let formatted = format_source(&source).unwrap();
    assert!(
        formatted.contains(&format!(
            "    <Rotulo\n        X=\"1\"\n        Y=\"2\"\n        Texto=\"{long}\" />\n"
        )),
        "{formatted}"
    );
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn entidades_sobrevivem_a_formatacao() {
    let source = window(r#"    <Rotulo X="1" Y="2" Texto="a &lt; b &amp; &quot;c&quot; 'd'" />"#);
    let document = parse(&source).unwrap();
    let rotulo = document.descendants()[0];
    assert_eq!(
        rotulo.attribute("Texto").unwrap().value,
        "a < b & \"c\" 'd'"
    );
    let formatted = format_source(&source).unwrap();
    assert!(
        formatted.contains(r#"Texto="a &lt; b &amp; &quot;c&quot; 'd'""#),
        "{formatted}"
    );
}

#[test]
fn erros_de_sintaxe_explicam_o_que_fazer() {
    assert_eq!(
        codes("<?xml version=\"1.0\"?>\n<Janela />"),
        ["design.xml-declaration"]
    );
    assert_eq!(codes("<!DOCTYPE janela>\n<Janela />"), ["design.doctype"]);
    assert_eq!(codes("Janela"), ["design.expected-window"]);
    assert_eq!(codes("<Janela Nome=J />"), ["design.expected-quote"]);
    assert_eq!(codes("<Janela Nome />"), ["design.expected-equals"]);
    assert_eq!(codes("<Janela Nome=\"J />"), ["design.unterminated-value"]);
    assert_eq!(
        codes("<Janela Nome=\"a<b\" />"),
        ["design.unescaped-less-than"]
    );
    assert_eq!(
        codes("<Janela Nome=\"a & b\" />"),
        ["design.bare-ampersand"]
    );
    assert_eq!(
        codes("<Janela Nome=\"&eacute;\" />"),
        ["design.unknown-entity"]
    );
    assert_eq!(codes("<Janela x:Name=\"J\" />"), ["design.namespace"]);
    assert_eq!(
        codes("<Janela><!-- aberto </Janela>"),
        ["design.unclosed-comment"]
    );
    assert_eq!(codes("<Janela><![CDATA[x]]></Janela>"), ["design.cdata"]);
    assert_eq!(
        codes("<Janela /><Janela />"),
        ["design.content-after-window"]
    );

    let analysis = analyze("<Janela>\n    <Pilha>\n</Janela>");
    assert_eq!(analysis.diagnostics[0].code, "design.mismatched-tag");
    assert_eq!(
        analysis.diagnostics[0].message,
        "Esperado `</Pilha>` para fechar `<Pilha>` aberta na linha 2, mas encontrei `</Janela>`"
    );

    let analysis = analyze("<Janela>\n    <Pilha>");
    assert_eq!(analysis.diagnostics[0].code, "design.unclosed-element");
    assert_eq!(
        analysis.diagnostics[0].message,
        "Falta fechar `<Pilha>` aberta na linha 2"
    );
}

#[test]
fn raiz_e_atributos_obrigatorios_da_janela() {
    assert_eq!(codes("<Pilha />"), ["design.expected-window"]);
    assert_eq!(
        codes("<Janela Nome=\"J\" />"),
        [
            "design.missing-attribute",
            "design.missing-attribute",
            "design.missing-attribute"
        ]
    );
    assert_eq!(
        codes("<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\" Formato=\"2\" />"),
        ["design.unsupported-version"]
    );
}

#[test]
fn elementos_e_atributos_desconhecidos_sugerem_correcao() {
    let analysis = analyze(&window("    <Botoa X=\"1\" Y=\"2\" />"));
    assert_eq!(analysis.diagnostics[0].code, "design.unknown-element");
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Você quis dizer `<Botao>`?")
    );

    let analysis = analyze(&window("    <Botao X=\"1\" Y=\"2\" Testo=\"oi\" />"));
    assert_eq!(analysis.diagnostics[0].code, "design.unknown-attribute");
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Você quis dizer `Texto`?")
    );

    let analysis = analyze(&window("    <Botao X=\"1\" Y=\"2\" texto=\"oi\" />"));
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Os atributos começam com maiúscula: `Texto`.")
    );

    assert_eq!(
        codes(&window("    <Botao X=\"1\" Y=\"2\" X=\"3\" />")),
        ["design.duplicate-attribute"]
    );
}

#[test]
fn tipos_e_intervalos_de_valores() {
    for (body, code) in [
        ("<Botao X=\"um\" Y=\"2\" />", "design.invalid-value"),
        (
            "<Botao X=\"1\" Y=\"2\" Largura=\"0\" />",
            "design.out-of-range",
        ),
        (
            "<Botao X=\"1\" Y=\"2\" Visivel=\"sim\" />",
            "design.invalid-value",
        ),
        (
            "<Botao X=\"1\" Y=\"2\" AoClicar=\"clicar()\" />",
            "design.invalid-name",
        ),
        ("<Pilha Orientacao=\"vertical\" />", "design.invalid-value"),
        (
            "<Botao X=\"1\" Y=\"2\" Margem=\"1,2,3\" />",
            "design.invalid-value",
        ),
        ("<Grade Linhas=\"Auto,,*\" />", "design.invalid-value"),
        (
            "<Botao X=\"1\" Y=\"2\" Ancora=\"Cima,Meio\" />",
            "design.invalid-value",
        ),
        (
            "<Botao X=\"1\" Y=\"2\" Ancora=\"Cima,Cima\" />",
            "design.duplicate-anchor",
        ),
        ("<Temporizador Intervalo=\"5\" />", "design.out-of-range"),
        (
            "<DialogoArquivo Modo=\"Excluir\" />",
            "design.invalid-value",
        ),
    ] {
        assert_eq!(codes(&window(&format!("    {body}"))), [code], "{body}");
    }

    let analysis = analyze(&window("    <Pilha Orientacao=\"vertical\" />"));
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Você quis dizer \"Vertical\"?")
    );
}

#[test]
fn posicao_depende_do_conteiner() {
    assert_eq!(
        codes(&window(
            "    <Pilha>\n        <Botao X=\"1\" Y=\"2\" />\n    </Pilha>"
        )),
        ["design.position-in-layout", "design.position-in-layout"]
    );
    assert_eq!(
        codes(&window("    <Botao X=\"1\" Y=\"2\" Linha=\"0\" />")),
        ["design.attached-outside-parent"]
    );
    assert_eq!(
        codes(&window(
            "    <Pilha>\n        <Botao Encaixar=\"Cima\" />\n    </Pilha>"
        )),
        ["design.attached-outside-parent"]
    );

    let analysis = analyze(&window(
        "    <Botao X=\"1\" Y=\"2\" Alinhamento=\"Centro\" />",
    ));
    assert_eq!(analysis.diagnostics[0].code, "design.alignment-ignored");
    assert_eq!(analysis.diagnostics[0].severity, Severity::Warning);

    let analysis = analyze(&window("    <Botao X=\"1\" />"));
    assert_eq!(analysis.diagnostics[0].code, "design.missing-position");
    assert_eq!(analysis.diagnostics[0].severity, Severity::Warning);
    assert!(!analysis.has_errors());

    // Pilha sozinha na janela ocupa todo o espaço e dispensa X e Y.
    assert!(codes(&window("    <Pilha>\n        <Botao />\n    </Pilha>")).is_empty());
    // Com irmãos, precisa de posição.
    assert_eq!(
        codes(&window("    <Pilha />\n    <Botao X=\"1\" Y=\"1\" />")),
        ["design.missing-position"]
    );
}

#[test]
fn celulas_da_grade_precisam_existir() {
    assert!(codes(&window(
        "    <Grade Linhas=\"Auto,*\" Colunas=\"*,*,*\">\n        <Botao Linha=\"1\" Coluna=\"2\" />\n    </Grade>"
    ))
    .is_empty());
    assert_eq!(
        codes(&window(
            "    <Grade Linhas=\"Auto,*\">\n        <Botao Linha=\"2\" Coluna=\"1\" />\n    </Grade>"
        )),
        ["design.out-of-range", "design.out-of-range"]
    );
}

#[test]
fn listas_itens_e_componentes() {
    assert_eq!(
        codes(&window(
            "    <Lista X=\"1\" Y=\"1\" Selecionado=\"1\">\n        <Item>Um</Item>\n    </Lista>"
        )),
        ["design.out-of-range"]
    );
    assert_eq!(
        codes(&window(
            "    <Lista X=\"1\" Y=\"1\">\n        <Botao />\n    </Lista>"
        )),
        ["design.list-children"]
    );
    assert_eq!(
        codes(&window("    <Item>solto</Item>")),
        ["design.item-outside-list"]
    );
    assert_eq!(
        codes(&window(
            "    <Lista X=\"1\" Y=\"1\">\n        <Item Texto=\"x\">Um</Item>\n    </Lista>"
        )),
        ["design.item-attributes"]
    );
    assert_eq!(
        codes(&window("    <Rotulo X=\"1\" Y=\"1\">texto</Rotulo>")),
        ["design.unexpected-text"]
    );
    assert_eq!(
        codes(&window(
            "    <Botao X=\"1\" Y=\"1\">\n        <Rotulo />\n    </Botao>"
        )),
        ["design.children-not-allowed"]
    );
    assert_eq!(
        codes(&window(
            "    <Pilha>\n        <Temporizador Intervalo=\"100\" />\n    </Pilha>"
        )),
        ["design.component-in-container"]
    );
    assert_eq!(codes(&window("    <Janela />")), ["design.nested-window"]);
}

#[test]
fn vinculos() {
    assert!(
        codes(&window(
            "    <Rotulo X=\"1\" Y=\"1\" Texto=\"Olá, {dados.nome}!\" />"
        ))
        .is_empty()
    );
    assert!(
        codes(&window(
            "    <Rotulo X=\"1\" Y=\"1\" Texto=\"chaves \\{literais\\}\" />"
        ))
        .is_empty()
    );

    for (body, code) in [
        (
            "<Rotulo X=\"1\" Y=\"1\" Texto=\"{nome}\" />",
            "design.invalid-binding",
        ),
        (
            "<Rotulo X=\"1\" Y=\"1\" Texto=\"{tela.nome}\" />",
            "design.binding-outside-data",
        ),
        (
            "<Rotulo X=\"1\" Y=\"1\" Texto=\"{dados.total + 1}\" />",
            "design.invalid-binding",
        ),
        (
            "<Rotulo X=\"1\" Y=\"1\" Texto=\"{dados.nome\" />",
            "design.unclosed-binding",
        ),
        (
            "<Rotulo X=\"1\" Y=\"1\" Texto=\"fim}\" />",
            "design.unopened-binding",
        ),
        (
            "<Botao X=\"1\" Y=\"1\" AoClicar=\"{dados.f}\" />",
            "design.binding-not-allowed",
        ),
        (
            "<Botao X=\"1\" Y=\"1\" Habilitado=\"sim {dados.ok}\" />",
            "design.binding-must-be-whole",
        ),
        (
            "<CaixaTexto X=\"1\" Y=\"1\" Texto=\"Nome: {dados.nome}\" />",
            "design.binding-must-be-whole",
        ),
    ] {
        assert_eq!(codes(&window(&format!("    {body}"))), [code], "{body}");
    }

    assert_eq!(
        codes("<Janela Nome=\"J\" Titulo=\"{dados.titulo}\" Largura=\"1\" Altura=\"1\" />"),
        ["design.binding-without-data"]
    );
    assert_eq!(
        codes(
            "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <DadosExemplo nome=\"Ana\" />\n</Janela>"
        ),
        ["design.sample-without-data"]
    );
}

#[test]
fn partes_de_texto_com_spans() {
    let raw = "Olá, {dados.nome}!";
    let parts = value::text_parts(raw, 10).unwrap();
    assert_eq!(parts.len(), 3);
    assert_eq!(parts[0], TextPart::Literal("Olá, ".into()));
    let TextPart::Binding { path, span } = &parts[1] else {
        panic!("{parts:?}");
    };
    assert_eq!(path, &["dados", "nome"]);
    let start = raw.find("dados").unwrap() as u32 + 10;
    assert_eq!((span.start, span.end), (start, start + 10));
    assert_eq!(parts[2], TextPart::Literal("!".into()));
}

#[test]
fn nomes_de_controles() {
    assert_eq!(
        codes(&window(
            "    <Botao Nome=\"b\" X=\"1\" Y=\"1\" />\n    <Rotulo Nome=\"b\" X=\"1\" Y=\"1\" />"
        )),
        ["design.duplicate-name"]
    );
    assert_eq!(
        codes(&window("    <Botao Nome=\"J\" X=\"1\" Y=\"1\" />")),
        ["design.duplicate-name"]
    );
    assert_eq!(
        codes(&window("    <Botao Nome=\"botão\" X=\"1\" Y=\"1\" />")),
        ["design.invalid-name"]
    );
    assert_eq!(
        codes(&window("    <Botao Nome=\"dados\" X=\"1\" Y=\"1\" />")),
        ["design.reserved-name"]
    );
}

#[test]
fn caminhos_de_recurso_ficam_no_projeto() {
    assert!(
        codes(&window(
            "    <Imagem X=\"1\" Y=\"1\" Arquivo=\"recursos/logo.png\" />"
        ))
        .is_empty()
    );
    for path in [
        "../segredo.png",
        "/etc/passwd",
        "C:\\x.png",
        "recursos/../../x",
    ] {
        assert_eq!(
            codes(&window(&format!(
                "    <Imagem X=\"1\" Y=\"1\" Arquivo=\"{path}\" />"
            ))),
            ["design.invalid-path"],
            "{path}"
        );
    }
}

#[test]
fn limites() {
    let elements: String = (0..=ferro_tela::MAX_ELEMENTS)
        .map(|i| format!("    <Rotulo Nome=\"r{i}\" X=\"0\" Y=\"0\" />\n"))
        .collect();
    assert_eq!(codes(&window(&elements)), ["design.too-many-elements"]);

    let depth = ferro_tela::MAX_DEPTH;
    let nested = format!(
        "{}{}",
        "<Pilha>".repeat(depth + 1),
        "</Pilha>".repeat(depth + 1)
    );
    assert_eq!(codes(&window(&nested)), ["design.too-deep"]);
}

#[test]
fn valores_compostos() {
    assert_eq!(
        value::thickness("8,4"),
        Some(Thickness {
            left: 8,
            top: 4,
            right: 8,
            bottom: 4
        })
    );
    assert_eq!(value::thickness("1,2,3"), None);
    assert_eq!(
        value::tracks("Auto, *, 2*, 120"),
        Some(vec![
            Track::Auto,
            Track::Star(1),
            Track::Star(2),
            Track::Pixels(120)
        ])
    );
    assert_eq!(value::tracks("auto"), None);
    assert_eq!(value::field_name("TextoVazio"), "texto_vazio");
    assert_eq!(value::field_name("X"), "x");
}

#[test]
fn esquema_e_consistente() {
    for element in std::iter::once(&schema::WINDOW).chain(schema::ELEMENTS) {
        if let Some(event) = element.default_event {
            let attribute = element
                .attribute(event)
                .unwrap_or_else(|| panic!("{}: evento padrão {event} ausente", element.name));
            assert_eq!(
                attribute.value,
                schema::ValueType::Event,
                "{}",
                element.name
            );
        }
        let names: Vec<_> = element.all_attributes().iter().map(|a| a.name).collect();
        let mut unique = names.clone();
        unique.sort_unstable();
        unique.dedup();
        assert_eq!(
            unique.len(),
            names.len(),
            "{}: atributo repetido",
            element.name
        );
        for name in names {
            assert!(
                name.chars().next().unwrap().is_ascii_uppercase() && value::is_identifier(name),
                "{}: {name} não está em PascalCase",
                element.name
            );
        }
        assert_eq!(
            element.category == Category::Component,
            element.default_size == (0, 0),
            "{}",
            element.name
        );
        if element.category == Category::Container {
            assert_ne!(element.layout, Layout::None, "{}", element.name);
        }
    }
}

#[test]
fn spans_apontam_para_o_texto() {
    let source = window("    <Botao X=\"1\" Y=\"dois\" />");
    let analysis = analyze(&source);
    let span = analysis.diagnostics[0].span;
    assert_eq!(&source[span.start as usize..span.end as usize], "dois");
}

#[test]
fn formatar_duas_vezes_nao_muda_nada() {
    let sources = [
        OLA.to_owned(),
        CALCULADORA.to_owned(),
        window("    <!--a-->\n\n\n    <Pilha><Botao/><!--b--></Pilha>"),
        window("    <Lista X=\"1\" Y=\"1\">\n<Item> espaços </Item><Item>&lt;b&gt;</Item></Lista>"),
        window("    <Rotulo X=\"1\" Y=\"1\" Texto=\"Olá, {dados.nome} \\{x\\}\" />"),
        "<!--\n  comentário\n  em várias linhas\n-->\n<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\"/>".to_owned(),
    ];
    for source in sources {
        let once = format_source(&source).unwrap();
        assert_eq!(format_source(&once).unwrap(), once, "{source}");
        assert_eq!(
            codes(&once),
            codes(&source),
            "formatar mudou os diagnósticos: {source}"
        );
    }
}
