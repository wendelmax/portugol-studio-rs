//! Análise e formatação dos arquivos de marcação de projetos desktop PORS
//! (ADR 0018): o desenho da tela (`.design`) e o manifesto (`.porsproj`).

use portugol_tela::{Element, project, schema};

use crate::outline::{SymbolInfo, SymbolKind};
use crate::position::LineIndex;
use crate::{Inspection, TextEdit, describe};

/// Tipo de arquivo de marcação, decidido pela extensão.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkupKind {
    Design,
    Manifest,
}

impl MarkupKind {
    /// Tipo pelo caminho ou nome do arquivo; `None` se não for marcação.
    pub fn from_path(path: &str) -> Option<Self> {
        let extension = path.rsplit_once('.')?.1.to_ascii_lowercase();
        if extension == portugol_tela::FILE_EXTENSION {
            Some(Self::Design)
        } else if extension == project::MANIFEST_EXTENSION {
            Some(Self::Manifest)
        } else {
            None
        }
    }
}

/// Diagnósticos e, no `.design`, a árvore de elementos para o painel de
/// estrutura. A existência da janela inicial do manifesto não é conferida
/// aqui, porque depende dos arquivos da pasta.
pub fn inspect_markup(source: &str, kind: MarkupKind) -> Inspection {
    let index = LineIndex::new(source);
    match kind {
        MarkupKind::Design => {
            let analysis = portugol_tela::analyze(source);
            let symbols = analysis
                .document
                .as_ref()
                .map(|document| vec![element_symbol(&index, &document.root)])
                .unwrap_or_default();
            Inspection {
                diagnostics: analysis
                    .diagnostics
                    .iter()
                    .map(|d| describe(&index, d))
                    .collect(),
                symbols,
            }
        }
        MarkupKind::Manifest => Inspection {
            diagnostics: project::analyze_manifest(source)
                .diagnostics
                .iter()
                .map(|d| describe(&index, d))
                .collect(),
            symbols: Vec::new(),
        },
    }
}

/// Um elemento do `.design` no painel de estrutura: o `Nome`, quando há, e o
/// tipo do elemento.
fn element_symbol(index: &LineIndex, element: &Element) -> SymbolInfo {
    let tag = element.name.text.as_str();
    let ty = schema::element(tag)
        .map(|s| s.name)
        .or_else(|| {
            [schema::ITEM, schema::SAMPLE_DATA]
                .into_iter()
                .find(|t| *t == tag)
        })
        .unwrap_or("elemento");
    let name = element
        .control_name()
        .map(|n| n.value.clone())
        .or_else(|| {
            (tag == schema::ITEM)
                .then(|| element.texts().next().map(|t| t.text.clone()))
                .flatten()
        })
        .unwrap_or_else(|| format!("<{tag}>"));
    SymbolInfo {
        name,
        kind: SymbolKind::Variable,
        ty,
        position: index.position(element.name.span.start),
        children: element
            .elements()
            .map(|child| element_symbol(index, child))
            .collect(),
    }
}

/// Formata um `.design` ou `.porsproj`. Devolve uma única edição que troca o
/// documento inteiro (offsets em UTF-16, como os do editor), nenhuma edição
/// se já estiver formatado e `None` se houver erro de sintaxe.
pub fn format_markup(source: &str) -> Option<Vec<TextEdit>> {
    let formatted = portugol_tela::format_source(source).ok()?;
    if formatted == source {
        return Some(Vec::new());
    }
    Some(vec![TextEdit {
        from: 0,
        to: source.encode_utf16().count(),
        insert: formatted,
    }])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Level;

    #[test]
    fn decide_o_tipo_pela_extensao() {
        assert_eq!(
            MarkupKind::from_path("App/JanelaPrincipal.design"),
            Some(MarkupKind::Design)
        );
        assert_eq!(
            MarkupKind::from_path("meu-app.PORSPROJ"),
            Some(MarkupKind::Manifest)
        );
        assert_eq!(MarkupKind::from_path("programa.pors"), None);
        assert_eq!(MarkupKind::from_path("sem-extensao"), None);
    }

    #[test]
    fn diagnosticos_do_design_tem_posicao_e_codigo() {
        let source = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <Botoa />\n</Janela>\n";
        let inspection = inspect_markup(source, MarkupKind::Design);
        assert_eq!(inspection.diagnostics.len(), 1);
        let diagnostic = &inspection.diagnostics[0];
        assert_eq!(diagnostic.code, "design.unknown-element");
        assert_eq!(diagnostic.severity, Level::Error);
        assert_eq!(diagnostic.start.line, 2);
    }

    #[test]
    fn estrutura_do_design_mostra_a_arvore_de_elementos() {
        let source = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"1\" Altura=\"1\">\n    <Pilha>\n        <Botao Nome=\"btnOk\" />\n        <Combo><Item>Um</Item></Combo>\n    </Pilha>\n</Janela>\n";
        let inspection = inspect_markup(source, MarkupKind::Design);
        let root = &inspection.symbols[0];
        assert_eq!((root.name.as_str(), root.ty), ("J", "Janela"));
        let pilha = &root.children[0];
        assert_eq!((pilha.name.as_str(), pilha.ty), ("<Pilha>", "Pilha"));
        assert_eq!(
            (pilha.children[0].name.as_str(), pilha.children[0].ty),
            ("btnOk", "Botao")
        );
        let item = &pilha.children[1].children[0];
        assert_eq!((item.name.as_str(), item.ty), ("Um", "Item"));
    }

    #[test]
    fn diagnosticos_do_manifesto() {
        let inspection = inspect_markup("<Projeto Nome=\"A\" />", MarkupKind::Manifest);
        assert_eq!(inspection.diagnostics[0].code, "projeto.missing-attribute");
        assert!(inspection.symbols.is_empty());
    }

    #[test]
    fn formata_trocando_o_documento_inteiro_em_utf16() {
        let source = "<Janela Nome='J' Titulo=\"Olá 😀\" Largura=\"1\" Altura=\"1\"></Janela>";
        let edits = format_markup(source).unwrap();
        assert_eq!(edits.len(), 1);
        assert_eq!(edits[0].to, source.encode_utf16().count());
        assert!(format_markup(&edits[0].insert).unwrap().is_empty());
        assert!(format_markup("<Janela").is_none());
    }
}
