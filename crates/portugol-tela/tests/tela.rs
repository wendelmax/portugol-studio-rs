use portugol_tela::{Value, analyze, format_source, parse, schema};

const CALCULADORA: &str = r#"versao 1

// Janela inicial do projeto.
janela JanelaPrincipal {
    titulo: "Calculadora"
    largura: 360
    altura: 240
    dados: Dados

    caixa_texto txtA { x: 16  y: 16  largura: 120 }
    caixa_texto txtB { x: 16  y: 52  largura: 120 }
    botao btnSomar { x: 16  y: 92  texto: "Somar"  ao_clicar: btnSomar_clicar }
    rotulo lblResultado { x: 16  y: 132  texto: "Resultado: -" }

    painel pnlOpcoes {
        x: 180
        y: 16
        largura: 160
        altura: 120
        titulo: "Opções"

        caixa_selecao chkArredondar { x: 8  y: 24  texto: "Arredondar"  marcado: verdadeiro }
        combo cmbOperacao { x: 8  y: 56  itens: ["Somar", "Subtrair"]  selecionado: 0 }
    }

    temporizador tmrRelogio { intervalo: 1000  ativo: falso  ao_disparar: tmrRelogio_disparar }
}
"#;

fn codes(source: &str) -> Vec<&'static str> {
    analyze(source).diagnostics.iter().map(|d| d.code).collect()
}

fn window(body: &str) -> String {
    format!("janela J {{\n    titulo: \"T\"\n    largura: 100\n    altura: 100\n{body}\n}}\n")
}

#[test]
fn analisa_janela_completa_sem_diagnosticos() {
    let analysis = analyze(CALCULADORA);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let screen = analysis.screen.unwrap();
    assert_eq!(screen.window.name.text, "JanelaPrincipal");
    let names: Vec<_> = screen
        .all_controls()
        .iter()
        .map(|c| c.name.text.as_str())
        .collect();
    assert_eq!(
        names,
        [
            "txtA",
            "txtB",
            "btnSomar",
            "lblResultado",
            "pnlOpcoes",
            "chkArredondar",
            "cmbOperacao",
            "tmrRelogio"
        ]
    );
    let combo = screen.all_controls()[6];
    assert!(matches!(
        &combo.body.property("itens").unwrap().value,
        Value::List(items) if items.len() == 2
    ));
}

#[test]
fn formatacao_canonica_e_estavel() {
    assert_eq!(format_source(CALCULADORA).unwrap(), CALCULADORA);
}

#[test]
fn formatacao_normaliza_espacos_e_preserva_comentarios() {
    let messy = "// topo\njanela   J{titulo:\"T\" largura:100\naltura : 80 // fim da linha\n// antes do botão\nbotao  b{x:1 y:2}\n// solto\n}\n// rodapé";
    let expected = "// topo\nversao 1\n\njanela J {\n    titulo: \"T\"\n    largura: 100\n    altura: 80 // fim da linha\n\n    // antes do botão\n    botao b { x: 1  y: 2 }\n\n    // solto\n}\n// rodapé\n";
    let formatted = format_source(messy).unwrap();
    assert_eq!(formatted, expected);
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn comentario_na_linha_da_versao_nao_se_perde() {
    let source = "versao 1 // formato atual\njanela J { titulo: \"T\"  largura: 1  altura: 1 }";
    let formatted = format_source(source).unwrap();
    assert!(
        formatted.starts_with("versao 1\n\n// formato atual\njanela J {"),
        "{formatted}"
    );
    assert_eq!(format_source(&formatted).unwrap(), formatted);
}

#[test]
fn controle_com_comentario_interno_fica_em_varias_linhas() {
    let source =
        window("    botao b {\n        // posição\n        x: 1\n        y: 2 // topo\n    }");
    let formatted = format_source(&source).unwrap();
    assert!(
        formatted.contains(
            "    botao b {\n        // posição\n        x: 1\n        y: 2 // topo\n    }\n"
        ),
        "{formatted}"
    );
}

#[test]
fn controle_longo_quebra_linha() {
    let long = "x".repeat(90);
    let source = window(&format!("    rotulo r {{ x: 1  y: 2  texto: \"{long}\" }}"));
    let formatted = format_source(&source).unwrap();
    assert!(
        formatted.contains("    rotulo r {\n        x: 1\n"),
        "{formatted}"
    );
}

#[test]
fn escapes_de_cadeia_sobrevivem_a_formatacao() {
    let source = window(r#"    rotulo r { x: 1  y: 2  texto: "a \"b\" \\ c\nd" }"#);
    let formatted = format_source(&source).unwrap();
    assert!(
        formatted.contains(r#"texto: "a \"b\" \\ c\nd""#),
        "{formatted}"
    );
    let screen = parse(&formatted).unwrap();
    assert_eq!(
        screen.all_controls()[0]
            .body
            .property("texto")
            .unwrap()
            .value,
        Value::Text("a \"b\" \\ c\nd".into())
    );
}

#[test]
fn erros_de_sintaxe() {
    assert_eq!(codes("botao b {}"), ["tela.expected-window"]);
    assert_eq!(codes("janela J {"), ["tela.unclosed-brace"]);
    assert_eq!(
        codes("janela J { titulo \"x\" }"),
        ["tela.unexpected-token"]
    );
    assert_eq!(
        codes("janela J { titulo: \"x }"),
        ["tela.unterminated-string"]
    );
    assert_eq!(
        codes("janela J { itens: [\"a\" \"b\"] }"),
        ["tela.expected-comma"]
    );
    assert_eq!(
        codes("janela J { itens: [\n// c\n] }"),
        ["tela.comment-in-list"]
    );
    assert_eq!(codes("janela J {} janela K {}"), ["tela.unexpected-token"]);
    assert_eq!(codes("janela J { x: @ }"), ["tela.unexpected-character"]);
}

#[test]
fn propriedades_obrigatorias_e_desconhecidas() {
    assert_eq!(
        codes("janela J { titulo: \"T\" }"),
        ["tela.missing-property", "tela.missing-property"]
    );

    let analysis = analyze(&window("    botao b { x: 1  y: 2  testo: \"oi\" }"));
    assert_eq!(analysis.diagnostics.len(), 1);
    let diagnostic = &analysis.diagnostics[0];
    assert_eq!(diagnostic.code, "tela.unknown-property");
    assert_eq!(diagnostic.help.as_deref(), Some("Você quis dizer `texto`?"));

    assert_eq!(
        codes(&window("    botao b { x: 1 }")),
        ["tela.missing-property"]
    );
    assert_eq!(
        codes(&window("    botao b { x: 1  y: 2  x: 3 }")),
        ["tela.duplicate-property"]
    );
}

#[test]
fn tipos_e_intervalos_de_valores() {
    assert_eq!(
        codes(&window("    botao b { x: \"1\"  y: 2 }")),
        ["tela.invalid-value"]
    );
    assert_eq!(
        codes(&window("    botao b { x: 1  y: 2  texto: Somar }")),
        ["tela.invalid-value"]
    );
    assert_eq!(
        codes(&window("    botao b { x: 1  y: 2  ao_clicar: \"f\" }")),
        ["tela.invalid-value"]
    );
    assert_eq!(
        codes(&window("    botao b { x: 1  y: 2  largura: 0 }")),
        ["tela.out-of-range"]
    );
    assert_eq!(
        codes(&window("    temporizador t { intervalo: 5 }")),
        ["tela.out-of-range"]
    );
    assert_eq!(
        codes(&window(
            "    lista l { x: 1  y: 2  itens: [\"a\"]  selecionado: 1 }"
        )),
        ["tela.out-of-range"]
    );
    assert_eq!(
        codes(&window("    lista l { x: 1  y: 2  itens: [\"a\", 2] }")),
        ["tela.invalid-value"]
    );
    assert_eq!(
        codes(&window(
            "    botao b { x: 1  y: 2  ancora: [cima, meio, cima] }"
        )),
        ["tela.invalid-value", "tela.duplicate-anchor"]
    );
    assert_eq!(
        codes(&window("    dialogo_arquivo d { modo: excluir }")),
        ["tela.invalid-value"]
    );
    assert!(codes(&window("    lista l { x: 1  y: 2  selecionado: -1 }")).is_empty());
}

#[test]
fn caminhos_de_recurso_ficam_no_projeto() {
    assert!(
        codes(&window(
            "    imagem i { x: 1  y: 2  arquivo: \"recursos/logo.png\" }"
        ))
        .is_empty()
    );
    for path in [
        "../segredo.png",
        "/etc/passwd",
        "C:\\\\x.png",
        "recursos/../../x",
    ] {
        assert_eq!(
            codes(&window(&format!(
                "    imagem i {{ x: 1  y: 2  arquivo: \"{path}\" }}"
            ))),
            ["tela.invalid-path"],
            "{path}"
        );
    }
}

#[test]
fn nomes_de_controles() {
    assert_eq!(
        codes(&window(
            "    botao b { x: 1  y: 2 }\n    rotulo b { x: 1  y: 2 }"
        )),
        ["tela.duplicate-name"]
    );
    assert_eq!(
        codes(&window("    botao J { x: 1  y: 2 }")),
        ["tela.duplicate-name"]
    );
    assert_eq!(
        codes(&window("    botao botão { x: 1  y: 2 }")),
        ["tela.invalid-name"]
    );
    assert_eq!(
        codes(&window("    botao se { x: 1  y: 2 }")),
        ["tela.reserved-name"]
    );
    assert_eq!(
        codes(&window("    botao b { x: 1  y: 2  ao_clicar: ação }")),
        ["tela.invalid-name"]
    );
}

#[test]
fn controles_desconhecidos_e_aninhamento() {
    let analysis = analyze(&window("    botoa b { x: 1  y: 2 }"));
    assert_eq!(analysis.diagnostics[0].code, "tela.unknown-control");
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Você quis dizer `botao`?")
    );

    assert_eq!(
        codes(&window(
            "    botao b { x: 1  y: 2\n        rotulo r { x: 1  y: 2 } }"
        )),
        ["tela.children-not-allowed"]
    );
    assert_eq!(
        codes(&window(
            "    painel p { x: 1  y: 2\n        temporizador t { intervalo: 100 } }"
        )),
        ["tela.nonvisual-in-container"]
    );

    let mut nested = String::new();
    for level in 0..=portugol_tela::MAX_DEPTH {
        nested.push_str(&format!("painel p{level} {{ x: 0  y: 0\n"));
    }
    nested.push_str(&"}".repeat(portugol_tela::MAX_DEPTH + 1));
    assert_eq!(codes(&window(&nested)), ["tela.too-deep"]);
}

#[test]
fn limite_de_controles() {
    let controls: String = (0..=portugol_tela::MAX_CONTROLS)
        .map(|i| format!("    rotulo r{i} {{ x: 0  y: 0 }}\n"))
        .collect();
    assert_eq!(codes(&window(&controls)), ["tela.too-many-controls"]);
}

#[test]
fn versao_nao_suportada() {
    assert_eq!(
        codes("versao 2\njanela J { titulo: \"T\"  largura: 1  altura: 1 }"),
        ["tela.unsupported-version"]
    );
}

#[test]
fn esquema_e_consistente() {
    for control in schema::CONTROLS {
        if let Some(event) = control.default_event {
            let property = control
                .property(event)
                .unwrap_or_else(|| panic!("{}: evento padrão {event} ausente", control.name));
            assert_eq!(property.value, schema::ValueType::Event, "{}", control.name);
        }
        let mut names: Vec<_> = control.all_properties().map(|p| p.name).collect();
        let total = names.len();
        names.sort_unstable();
        names.dedup();
        assert_eq!(names.len(), total, "{}: propriedade repetida", control.name);
        assert_eq!(
            control.visual,
            control.default_size != (0, 0),
            "{}",
            control.name
        );
    }
}

#[test]
fn spans_apontam_para_o_texto() {
    let source = window("    botao b { x: 1  y: \"dois\" }");
    let analysis = analyze(&source);
    let span = analysis.diagnostics[0].span;
    assert_eq!(&source[span.start as usize..span.end as usize], "\"dois\"");
}
