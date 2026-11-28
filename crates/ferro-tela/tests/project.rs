use ferro_tela::project::{
    self, DEFAULT_WINDOW, MANIFEST_EXTENSION, analyze_manifest, check_start_window,
    desktop_template, file_stem,
};
use ferro_tela::{analyze, format_source};

fn codes(source: &str) -> Vec<&'static str> {
    analyze_manifest(source)
        .diagnostics
        .iter()
        .map(|d| d.code)
        .collect()
}

#[test]
fn le_manifesto_valido() {
    let analysis = analyze_manifest(
        "<Projeto Nome=\"Meu App\" Versao=\"1.2.3\" JanelaInicial=\"JanelaPrincipal\" Icone=\"recursos/icone.png\" Formato=\"1\" />",
    );
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let manifest = analysis.manifest.unwrap();
    assert_eq!(manifest.name, "Meu App");
    assert_eq!(manifest.version, "1.2.3");
    assert_eq!(manifest.start_window, "JanelaPrincipal");
    assert_eq!(manifest.icon.as_deref(), Some("recursos/icone.png"));
}

#[test]
fn versao_padrao() {
    let manifest = analyze_manifest("<Projeto Nome=\"A\" JanelaInicial=\"J\" />")
        .manifest
        .unwrap();
    assert_eq!(manifest.version, "0.1.0");
}

#[test]
fn erros_do_manifesto() {
    assert_eq!(codes("<Janela />"), ["projeto.expected-project"]);
    assert_eq!(
        codes("<Projeto />"),
        ["projeto.missing-attribute", "projeto.missing-attribute"]
    );
    for (source, code) in [
        (
            "<Projeto Nome=\"\" JanelaInicial=\"J\" />",
            "projeto.invalid-name",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"Janela Principal\" />",
            "projeto.invalid-window-name",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\" Versao=\"1.0\" />",
            "projeto.invalid-version",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\" Icone=\"../x.png\" />",
            "projeto.invalid-path",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\" Formato=\"2\" />",
            "projeto.unsupported-version",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\" Autor=\"x\" />",
            "projeto.unknown-attribute",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\"><Janela /></Projeto>",
            "projeto.children-not-allowed",
        ),
        (
            "<Projeto Nome=\"A\" JanelaInicial=\"J\">texto</Projeto>",
            "projeto.unexpected-text",
        ),
    ] {
        assert_eq!(codes(source), [code], "{source}");
    }
    let analysis = analyze_manifest("<Projeto nome=\"A\" JanelaInicial=\"J\" />");
    assert_eq!(
        analysis.diagnostics[0].help.as_deref(),
        Some("Os atributos começam com maiúscula: `Nome`.")
    );
    assert!(analysis.manifest.is_none());
}

#[test]
fn janela_inicial_precisa_existir() {
    let manifest = analyze_manifest("<Projeto Nome=\"A\" JanelaInicial=\"Inicio\" />")
        .manifest
        .unwrap();
    assert!(check_start_window(&manifest, &["Inicio.design", "Outra.design"]).is_none());
    let diagnostic = check_start_window(&manifest, &["Outra.design"]).unwrap();
    assert_eq!(diagnostic.code, "projeto.missing-start-window");
}

#[test]
fn nome_de_arquivo_derivado_do_nome() {
    assert_eq!(file_stem("Meu App 2"), "meu-app-2");
    assert_eq!(
        file_stem("  Calculadora de Médias!  "),
        "calculadora-de-medias"
    );
    assert_eq!(file_stem("Ação & Reação"), "acao-reacao");
    assert_eq!(file_stem("!!!"), "");
}

#[test]
fn modelo_desktop_e_valido_e_canonico() {
    let template = desktop_template("Calculadora de Médias").unwrap();
    let paths: Vec<_> = template.files.iter().map(|f| f.path.as_str()).collect();
    assert_eq!(
        paths,
        [
            format!("calculadora-de-medias.{MANIFEST_EXTENSION}").as_str(),
            format!("{DEFAULT_WINDOW}.design").as_str()
        ]
    );
    assert_eq!(template.directories, [project::RESOURCES_DIR]);

    let manifest = &template.files[0].content;
    let analysis = analyze_manifest(manifest);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    let manifest_data = analysis.manifest.unwrap();
    assert_eq!(manifest_data.name, "Calculadora de Médias");
    assert!(check_start_window(&manifest_data, &[&template.files[1].path]).is_none());
    assert_eq!(&format_source(manifest).unwrap(), manifest);

    let design = &template.files[1].content;
    let analysis = analyze(design);
    assert!(
        analysis.diagnostics.is_empty(),
        "{:?}",
        analysis.diagnostics
    );
    assert_eq!(&format_source(design).unwrap(), design);
    let root = analysis.document.unwrap().root;
    assert_eq!(
        root.attribute("Titulo").unwrap().value,
        "Calculadora de Médias"
    );
}

#[test]
fn nome_com_caracteres_especiais_nao_quebra_o_modelo() {
    let template = desktop_template("Notas & \"Médias\" <2026> {beta}").unwrap();
    let name = "Notas & \"Médias\" <2026> {beta}";
    let design = analyze(&template.files[1].content);
    assert!(design.diagnostics.is_empty(), "{:?}", design.diagnostics);
    // No .design as chaves ficam escapadas para não virar vínculo.
    let titulo = design
        .document
        .unwrap()
        .root
        .attribute("Titulo")
        .unwrap()
        .clone();
    let parts = ferro_tela::value::text_parts(&titulo.raw, 0).unwrap();
    assert_eq!(parts, [ferro_tela::value::TextPart::Literal(name.into())]);
    // No manifesto, o nome é guardado exatamente como digitado.
    let manifest = analyze_manifest(&template.files[0].content);
    assert_eq!(manifest.manifest.unwrap().name, name);
}

#[test]
fn nomes_de_projeto_invalidos() {
    assert!(desktop_template("   ").is_err());
    assert!(desktop_template("!!!").is_err());
    assert!(desktop_template("linha\nquebrada").is_err());
    assert!(desktop_template(&"a".repeat(project::MAX_NAME_CHARS + 1)).is_err());
}
