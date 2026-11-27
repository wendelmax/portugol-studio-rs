//! Ponte entre o designer visual da IDE e o formato `.design` (ADR 0018,
//! Fase 2): a árvore que o canvas desenha, as edições que ele pede e o esquema
//! que alimenta a caixa de ferramentas e o painel de propriedades.

use serde::{Deserialize, Serialize};

use portugol_tela::edit::{self, Edit};
use portugol_tela::schema::{
    self, AttributeSchema, Binding, Category, ElementSchema, Layout, Placement, ValueType,
};
use portugol_tela::{Element, Node};

use crate::position::LineIndex;

/// Um elemento do `.design` como o canvas o recebe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignNode {
    pub tag: String,
    /// Caminho até o elemento, usado nas edições.
    pub path: Vec<usize>,
    pub attributes: Vec<DesignAttribute>,
    /// Texto de um `<Item>`.
    pub text: Option<String>,
    /// Início e fim da tag no texto, em UTF-16, para sincronizar a seleção do
    /// canvas com a do editor.
    pub from: u32,
    pub to: u32,
    pub children: Vec<DesignNode>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DesignAttribute {
    pub name: String,
    /// Valor com as entidades já decodificadas.
    pub value: String,
}

/// Árvore do `.design`, ou `None` se o texto tiver erro de sintaxe. Erros de
/// validação não impedem o canvas de desenhar.
pub fn design_tree(source: &str) -> Option<DesignNode> {
    let document = portugol_tela::parse(source).ok()?;
    let index = LineIndex::new(source);
    Some(node(&index, &document.root, Vec::new()))
}

fn node(index: &LineIndex, element: &Element, path: Vec<usize>) -> DesignNode {
    let children = element
        .elements()
        .enumerate()
        .map(|(i, child)| {
            let mut child_path = path.clone();
            child_path.push(i);
            node(index, child, child_path)
        })
        .collect();
    let text = element.children.iter().find_map(|n| match n {
        Node::Text(t) => Some(t.text.clone()),
        _ => None,
    });
    DesignNode {
        tag: element.name.text.clone(),
        path,
        attributes: element
            .attributes
            .iter()
            .map(|a| DesignAttribute {
                name: a.name.text.clone(),
                value: a.value.clone(),
            })
            .collect(),
        text,
        from: index.position(element.span.start).offset,
        to: index.position(element.span.end).offset,
        children,
    }
}

/// Edição pedida pelo designer, no formato JSON da IDE.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum DesignEdit {
    SetAttribute {
        path: Vec<usize>,
        name: String,
        value: String,
    },
    RemoveAttribute {
        path: Vec<usize>,
        name: String,
    },
    /// Sem `Nome` nos atributos, controles, componentes e `Painel` recebem o
    /// próximo nome livre (`botao1`, `botao2`...). `Pilha`, `Grade`,
    /// `Encaixe` e `Item` ficam sem nome, como nos exemplos.
    Insert {
        parent: Vec<usize>,
        index: usize,
        tag: String,
        #[serde(default)]
        attributes: Vec<(String, String)>,
    },
    Remove {
        path: Vec<usize>,
    },
    Move {
        path: Vec<usize>,
        parent: Vec<usize>,
        index: usize,
    },
}

fn names_automatically(tag: &str) -> bool {
    schema::element(tag).is_some_and(|s| {
        s.name != schema::WINDOW.name && (s.category != Category::Container || s.name == "Painel")
    })
}

/// Aplica as edições e devolve o novo texto do `.design`, já formatado. Se
/// uma edição falhar, nenhuma vale e o erro explica o motivo.
pub fn edit_design(source: &str, edits: Vec<DesignEdit>) -> Result<String, String> {
    let mut document = portugol_tela::parse(source).map_err(|diagnostic| {
        format!(
            "Corrija o erro no texto antes de usar o designer: {}",
            diagnostic.message
        )
    })?;
    for design_edit in edits {
        let edit = match design_edit {
            DesignEdit::SetAttribute { path, name, value } => {
                Edit::SetAttribute { path, name, value }
            }
            DesignEdit::RemoveAttribute { path, name } => Edit::RemoveAttribute { path, name },
            DesignEdit::Insert {
                parent,
                index,
                tag,
                mut attributes,
            } => {
                if names_automatically(&tag) && attributes.iter().all(|(name, _)| name != "Nome") {
                    let name = edit::next_control_name(&document, &tag);
                    attributes.insert(0, ("Nome".to_owned(), name));
                }
                Edit::Insert {
                    parent,
                    index,
                    tag,
                    attributes,
                }
            }
            DesignEdit::Remove { path } => Edit::Remove { path },
            DesignEdit::Move {
                path,
                parent,
                index,
            } => Edit::Move {
                path,
                parent,
                index,
            },
        };
        edit::apply(&mut document, &edit).map_err(|error| error.to_string())?;
    }
    Ok(portugol_tela::format(&document))
}

/// Esquema do formato para a caixa de ferramentas e o painel de propriedades.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DesignSchema {
    pub window: ElementInfo,
    pub elements: Vec<ElementInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ElementInfo {
    pub name: &'static str,
    pub label: &'static str,
    pub description: &'static str,
    /// `control`, `container` ou `component`.
    pub category: &'static str,
    /// `none`, `free`, `stack`, `grid`, `dock` ou `items`.
    pub layout: &'static str,
    pub default_event: Option<&'static str>,
    pub default_width: i64,
    pub default_height: i64,
    /// Todos os atributos aceitos, os comuns primeiro.
    pub attributes: Vec<AttributeInfo>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttributeInfo {
    pub name: &'static str,
    pub description: &'static str,
    /// `integer`, `text`, `bool`, `event`, `type`, `name`, `choice`,
    /// `anchors`, `thickness`, `tracks` ou `path`.
    #[serde(rename = "type")]
    pub value_type: &'static str,
    pub min: Option<i64>,
    pub max: Option<i64>,
    /// Opções de `choice`; bordas de `anchors`.
    pub choices: &'static [&'static str],
    pub required: bool,
    /// `none`, `oneWay` ou `twoWay`.
    pub binding: &'static str,
    /// `any`, `free`, `grid`, `dock` ou `layout`.
    pub placement: &'static str,
}

pub fn design_schema() -> DesignSchema {
    DesignSchema {
        window: element_info(&schema::WINDOW),
        elements: schema::ELEMENTS.iter().map(element_info).collect(),
    }
}

fn element_info(element: &ElementSchema) -> ElementInfo {
    ElementInfo {
        name: element.name,
        label: element.label,
        description: element.description,
        category: match element.category {
            Category::Control => "control",
            Category::Container => "container",
            Category::Component => "component",
        },
        layout: match element.layout {
            Layout::None => "none",
            Layout::Free => "free",
            Layout::Stack => "stack",
            Layout::Grid => "grid",
            Layout::Dock => "dock",
            Layout::Items => "items",
        },
        default_event: element.default_event,
        default_width: element.default_size.0,
        default_height: element.default_size.1,
        attributes: element
            .all_attributes()
            .into_iter()
            .map(attribute_info)
            .collect(),
    }
}

fn attribute_info(attribute: &AttributeSchema) -> AttributeInfo {
    let (value_type, min, max, choices): (_, _, _, &'static [&'static str]) = match attribute.value
    {
        ValueType::Integer { min, max } => ("integer", Some(min), Some(max), &[]),
        ValueType::Text => ("text", None, None, &[]),
        ValueType::Bool => ("bool", None, None, &[]),
        ValueType::Event => ("event", None, None, &[]),
        ValueType::TypeName => ("type", None, None, &[]),
        ValueType::ControlName => ("name", None, None, &[]),
        ValueType::Choice(options) => ("choice", None, None, options),
        ValueType::Anchors => ("anchors", None, None, schema::ANCHORS),
        ValueType::Thickness => ("thickness", None, None, &[]),
        ValueType::Tracks => ("tracks", None, None, &[]),
        ValueType::ResourcePath => ("path", None, None, &[]),
    };
    AttributeInfo {
        name: attribute.name,
        description: attribute.description,
        value_type,
        min,
        max,
        choices,
        required: attribute.required,
        binding: match attribute.binding {
            Binding::None => "none",
            Binding::OneWay => "oneWay",
            Binding::TwoWay => "twoWay",
        },
        placement: match attribute.placement {
            Placement::Any => "any",
            Placement::Free => "free",
            Placement::Grid => "grid",
            Placement::Dock => "dock",
            Placement::Layout => "layout",
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SOURCE: &str = "<Janela Nome=\"J\" Titulo=\"Olá\" Largura=\"300\" Altura=\"200\" Formato=\"1\">\n    <Combo Nome=\"c\" X=\"0\" Y=\"0\">\n        <Item>Um &amp; dois</Item>\n    </Combo>\n</Janela>\n";

    #[test]
    fn arvore_com_caminhos_texto_e_offsets_utf16() {
        let root = design_tree(SOURCE).unwrap();
        assert_eq!(root.tag, "Janela");
        assert!(root.path.is_empty());
        assert_eq!(root.from, 0);
        assert_eq!(root.to as usize, SOURCE.trim_end().encode_utf16().count());
        let combo = &root.children[0];
        assert_eq!(combo.path, [0]);
        let item = &combo.children[0];
        assert_eq!(item.path, [0, 0]);
        assert_eq!(item.text.as_deref(), Some("Um & dois"));
        let utf16: Vec<u16> = SOURCE.encode_utf16().collect();
        let tag = String::from_utf16(&utf16[combo.from as usize..combo.from as usize + 6]).unwrap();
        assert_eq!(tag, "<Combo");
        assert!(design_tree("<Janela").is_none());
    }

    #[test]
    fn edicoes_chegam_em_json_e_dao_nome_automatico() {
        let edits: Vec<DesignEdit> = serde_json::from_str(
            r#"[
                {"kind":"insert","parent":[],"index":1,"tag":"Botao","attributes":[["X","10"],["Y","40"]]},
                {"kind":"insert","parent":[],"index":2,"tag":"Botao"},
                {"kind":"insert","parent":[],"index":3,"tag":"Pilha"},
                {"kind":"setAttribute","path":[0],"name":"Largura","value":"120"}
            ]"#,
        )
        .unwrap();
        let out = edit_design(SOURCE, edits).unwrap();
        assert!(
            out.contains("<Botao Nome=\"botao1\" X=\"10\" Y=\"40\" />"),
            "{out}"
        );
        assert!(out.contains("<Botao Nome=\"botao2\" />"), "{out}");
        assert!(out.contains("<Pilha />"), "{out}");
        assert!(
            out.contains("<Combo Nome=\"c\" X=\"0\" Y=\"0\" Largura=\"120\">"),
            "{out}"
        );
    }

    #[test]
    fn erro_de_edicao_vira_mensagem() {
        let edits = vec![DesignEdit::Remove { path: vec![] }];
        assert!(edit_design(SOURCE, edits).unwrap_err().contains("Janela"));
        assert!(
            edit_design("<Janela", Vec::new())
                .unwrap_err()
                .starts_with("Corrija")
        );
    }

    #[test]
    fn esquema_para_a_caixa_de_ferramentas() {
        let schema = design_schema();
        assert_eq!(schema.window.name, "Janela");
        let botao = schema.elements.iter().find(|e| e.name == "Botao").unwrap();
        assert_eq!(botao.category, "control");
        assert_eq!(botao.default_event, Some("AoClicar"));
        let x = botao.attributes.iter().find(|a| a.name == "X").unwrap();
        assert_eq!((x.value_type, x.placement), ("integer", "free"));
        let json = serde_json::to_value(&schema).unwrap();
        assert!(json["elements"][0]["attributes"][0]["type"].is_string());
        assert!(json["elements"][0]["defaultWidth"].is_number());
    }
}
