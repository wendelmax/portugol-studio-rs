//! Ponte entre o designer visual da IDE e o formato `.design` (ADR 0018,
//! Fase 2): a árvore que o canvas desenha, as edições que ele pede e o esquema
//! que alimenta a caixa de ferramentas e o painel de propriedades.

use serde::{Deserialize, Serialize};

use ferro_tela::edit::{self, Edit};
use ferro_tela::schema::{
    self, AttributeSchema, Binding, Category, ElementSchema, Layout, Placement, ValueType,
};
use ferro_tela::{Element, Node};

use crate::position::LineIndex;

/// Diagnósticos das ligações entre a marcação e o code-behind (ADR 0018,
/// seção 4): `design` vai para o `.design` (handler que não existe) e `code`
/// para o `.fer` (handler com assinatura errada).
#[derive(Debug, Default)]
pub struct ScreenCheck {
    pub design: Vec<ferro_hir::Diagnostic>,
    pub code: Vec<ferro_hir::Diagnostic>,
}

/// Confere se cada evento da marcação (`AoClicar="btnOk_clicar"`) aponta
/// para uma função do code-behind que recebe e devolve a janela. `code` é
/// `None` quando a tela ainda não tem `.fer`. Com erro no `.design` ou no
/// código, não há o que conferir: esses erros já aparecem por conta própria.
pub fn check_screen(design_source: &str, code: Option<&str>) -> ScreenCheck {
    use ferro_hir::Diagnostic;
    let mut check = ScreenCheck::default();
    let Ok(document) = ferro_tela::parse(design_source) else {
        return check;
    };
    let window = document
        .root
        .control_name()
        .map_or("Janela".to_owned(), |name| name.value.clone());
    let mut events = Vec::new();
    collect_events(&document.root, &mut events);
    let program = match code {
        Some(source) => {
            match ferro_frontend::lower_with(source, &code_behind_options(design_source)) {
                Ok(program) => Some((source, program)),
                Err(_) => return check,
            }
        }
        None => None,
    };
    if let Some((_, program)) = &program {
        let (screen, _) = ferro_tela::codebehind::screen_type(&document);
        let data = screen.data.as_ref().and_then(|(name, _)| {
            program
                .record_types
                .iter()
                .find(|record| record.name == *name)
                .map(|record| ferro_hir::Type::Record(record.id))
        });
        check.design.extend(crate::screen_bindings::check_bindings(
            &document, program, data,
        ));
        check.code.extend(crate::screen_bindings::check_read_only(
            &document, &screen, program,
        ));
    }
    let window_type = program.as_ref().and_then(|(_, program)| {
        program
            .record_types
            .iter()
            .find(|record| record.name == window)
            .map(|record| ferro_hir::Type::Record(record.id))
    });
    for (handler, span) in events {
        let function = program.as_ref().and_then(|(source, program)| {
            program
                .functions
                .iter()
                .find(|function| function.source_name == handler)
                .map(|function| (*source, function))
        });
        let Some((source, function)) = function else {
            let message = if program.is_some() {
                format!("A função `{handler}` não existe no código da tela")
            } else {
                format!("A função `{handler}` não existe: esta tela ainda não tem código")
            };
            check.design.push(
                Diagnostic::error("design.handler-missing", message, span)
                    .with_help("Dê duplo clique no controle, no modo Design, para criar a função."),
            );
            continue;
        };
        let parameter_types: Vec<_> = function
            .parameters
            .iter()
            .filter_map(|id| function.bindings.iter().find(|binding| binding.id == *id))
            .map(|binding| binding.ty)
            .collect();
        let ok = window_type
            .is_some_and(|ty| parameter_types == [ty] && function.return_type == Some(ty));
        if !ok {
            let start = crate::ferro_function_name_start(source, function.span.start);
            check.code.push(
                Diagnostic::error(
                    "design.handler-signature",
                    format!("A função `{handler}` trata um evento da tela e precisa receber e devolver a janela"),
                    ferro_hir::Span::new(start, start + handler.len() as u32),
                )
                .with_help(format!("Use `funcao {handler}(tela: {window}) -> {window}`.")),
            );
        }
    }
    check
}

fn collect_events(element: &Element, out: &mut Vec<(String, ferro_hir::Span)>) {
    if let Some(element_schema) = schema::element(&element.name.text) {
        for attribute in &element.attributes {
            let is_event = element_schema
                .attribute(&attribute.name.text)
                .is_some_and(|a| a.value == ValueType::Event);
            if is_event && !attribute.value.trim().is_empty() {
                out.push((attribute.value.trim().to_owned(), attribute.value_span));
            }
        }
    }
    for child in element.elements() {
        collect_events(child, out);
    }
}

/// Campo do registro de `Dados`, para o painel de dados de exemplo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DataField {
    pub name: String,
    /// `inteiro`, `real`, `logico`, `cadeia` ou `caracter`; outro nome para
    /// registros e vetores, que não têm exemplo.
    #[serde(rename = "type")]
    pub ty: String,
    /// Tipo simples, que aceita um valor de exemplo.
    pub editable: bool,
}

/// Campos do registro de `Dados` da tela, lidos do code-behind. Vazio sem
/// `Dados`, sem o registro ou com erro no código.
pub fn screen_data_fields(design_source: &str, code_source: &str) -> Vec<DataField> {
    let Ok(document) = ferro_tela::parse(design_source) else {
        return Vec::new();
    };
    let Some(data) = document.root.attribute("Dados").map(|a| a.value.clone()) else {
        return Vec::new();
    };
    let Ok(program) = ferro_frontend::lower_with(code_source, &code_behind_options(design_source))
    else {
        return Vec::new();
    };
    let Some(record) = program.record_types.iter().find(|r| r.name == data) else {
        return Vec::new();
    };
    record
        .fields
        .iter()
        .map(|field| {
            let (ty, editable) = match field.ty {
                ferro_hir::Type::Integer => ("inteiro".to_owned(), true),
                ferro_hir::Type::Real => ("real".to_owned(), true),
                ferro_hir::Type::Boolean => ("logico".to_owned(), true),
                ferro_hir::Type::String => ("cadeia".to_owned(), true),
                ferro_hir::Type::Character => ("caracter".to_owned(), true),
                ferro_hir::Type::Record(id) => (
                    program
                        .record_types
                        .iter()
                        .find(|r| r.id == id)
                        .map_or("registro".to_owned(), |r| r.name.clone()),
                    false,
                ),
                _ => ("vetor".to_owned(), false),
            };
            DataField {
                name: field.name.clone(),
                ty,
                editable,
            }
        })
        .collect()
}

/// Opções para analisar o code-behind de uma tela (ver
/// [`ferro_tela::codebehind::lower_options`]).
pub fn code_behind_options(design_source: &str) -> ferro_frontend::LowerOptions {
    ferro_tela::codebehind::lower_options(design_source)
}

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
    let document = ferro_tela::parse(source).ok()?;
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
    let mut document = ferro_tela::parse(source).map_err(|diagnostic| {
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
                    attributes.insert(0, ("Nome".to_owned(), name.clone()));
                    // Como no Windows Forms, o texto visível começa igual ao
                    // nome. Não vale para texto digitado (`CaixaTexto`).
                    let shows_text = schema::element(&tag)
                        .and_then(|s| s.attribute("Texto"))
                        .is_some_and(|a| a.binding == Binding::OneWay);
                    if shows_text && attributes.iter().all(|(n, _)| n != "Texto") {
                        attributes.insert(1, ("Texto".to_owned(), name));
                    }
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
    Ok(ferro_tela::format(&document))
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
            out.contains("<Botao Nome=\"botao1\" Texto=\"botao1\" X=\"10\" Y=\"40\" />"),
            "{out}"
        );
        assert!(
            out.contains("<Botao Nome=\"botao2\" Texto=\"botao2\" />"),
            "{out}"
        );
        assert!(out.contains("<Pilha />"), "{out}");
        assert!(
            out.contains("<Combo Nome=\"c\" X=\"0\" Y=\"0\" Largura=\"120\">"),
            "{out}"
        );
    }

    #[test]
    fn caixa_de_texto_nao_ganha_texto_automatico() {
        let edits = vec![DesignEdit::Insert {
            parent: vec![],
            index: 0,
            tag: "CaixaTexto".into(),
            attributes: vec![],
        }];
        let out = edit_design(SOURCE, edits).unwrap();
        assert!(out.contains("<CaixaTexto Nome=\"caixaTexto1\" />"), "{out}");
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

#[cfg(test)]
mod code_behind_tests {
    const DESIGN: &str = "<Janela Nome=\"JanelaPrincipal\" Titulo=\"Calculadora\" Largura=\"360\" Altura=\"240\" Dados=\"Conta\">\n    <Pilha>\n        <CaixaTexto Nome=\"txtA\" />\n        <Botao Nome=\"btnSomar\" Texto=\"Somar\" AoClicar=\"btnSomar_clicar\" />\n    </Pilha>\n</Janela>\n";
    const CODE: &str = "registro Conta {\n    mensagem: cadeia\n}\n\nfuncao btnSomar_clicar(tela: JanelaPrincipal) -> JanelaPrincipal {\n    seja mutavel t: JanelaPrincipal = tela;\n    t.dados.mensagem = t.txtA.texto;\n    t.btnSomar.habilitado = falso;\n    retorne t;\n}\n";

    #[test]
    fn code_behind_sem_erros_e_com_o_handler_no_painel_de_estrutura() {
        let inspection = crate::inspect_code_behind(CODE, DESIGN);
        assert!(
            inspection.diagnostics.is_empty(),
            "{:?}",
            inspection.diagnostics
        );
        assert!(
            inspection
                .symbols
                .iter()
                .any(|s| s.name == "btnSomar_clicar")
        );
    }

    #[test]
    fn campo_de_controle_inexistente_aponta_a_linha_no_fer() {
        let code = CODE.replace("t.btnSomar.habilitado", "t.btnSubtrair.habilitado");
        let inspection = crate::inspect_code_behind(&code, DESIGN);
        let diagnostic = &inspection.diagnostics[0];
        assert_eq!(diagnostic.code, "ferro.unknown-field");
        assert_eq!(diagnostic.start.line, 8);
    }

    #[test]
    fn sem_o_design_o_tipo_da_janela_e_desconhecido() {
        let inspection = crate::inspect_ferro_rs(CODE);
        assert!(!inspection.diagnostics.is_empty());
        let broken = crate::inspect_code_behind(CODE, "<Janela");
        assert!(
            broken
                .diagnostics
                .iter()
                .any(|d| d.code == "ferro.unknown-type"),
            "{:?}",
            broken.diagnostics
        );
    }
}

#[cfg(test)]
mod binding_tests {
    use crate::{CodeBehind, MarkupKind, inspect_code_behind, inspect_markup_with_code};

    const DESIGN: &str = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"200\" Altura=\"100\">\n    <Botao Nome=\"btnOk\" Texto=\"OK\" X=\"8\" Y=\"8\" AoClicar=\"btnOk_clicar\" />\n</Janela>\n";

    #[test]
    fn handler_existente_nao_gera_diagnostico() {
        let code = "funcao btnOk_clicar(tela: J) -> J {\n    retorne tela;\n}\n";
        assert!(
            inspect_markup_with_code(DESIGN, MarkupKind::Design, CodeBehind::Source(code))
                .diagnostics
                .is_empty()
        );
        assert!(inspect_code_behind(code, DESIGN).diagnostics.is_empty());
    }

    #[test]
    fn handler_ausente_aponta_o_atributo_no_design() {
        let code = "funcao outro(tela: J) -> J {\n    retorne tela;\n}\n";
        let inspection =
            inspect_markup_with_code(DESIGN, MarkupKind::Design, CodeBehind::Source(code));
        let diagnostic = &inspection.diagnostics[0];
        assert_eq!(diagnostic.code, "design.handler-missing");
        assert_eq!(diagnostic.start.line, 2);
        assert!(diagnostic.message.contains("`btnOk_clicar`"));
        let missing = inspect_markup_with_code(DESIGN, MarkupKind::Design, CodeBehind::Missing);
        assert!(
            missing.diagnostics[0]
                .message
                .contains("ainda não tem código")
        );
        assert!(
            inspect_markup_with_code(DESIGN, MarkupKind::Design, CodeBehind::Unknown)
                .diagnostics
                .is_empty()
        );
    }

    #[test]
    fn assinatura_errada_aponta_o_nome_da_funcao_no_fer() {
        let code = "funcao btnOk_clicar(x: inteiro) -> inteiro {\n    retorne x;\n}\n";
        let inspection = inspect_code_behind(code, DESIGN);
        let diagnostic = &inspection.diagnostics[0];
        assert_eq!(diagnostic.code, "design.handler-signature");
        assert_eq!((diagnostic.start.line, diagnostic.start.column), (1, 8));
        assert_eq!(
            diagnostic.end.column - diagnostic.start.column,
            "btnOk_clicar".len() as u32
        );
        assert_eq!(
            diagnostic.help.as_deref(),
            Some("Use `funcao btnOk_clicar(tela: J) -> J`.")
        );
    }

    #[test]
    fn com_erro_no_codigo_as_ligacoes_nao_sao_conferidas() {
        let code = "funcao btnOk_clicar(tela: J) -> J {\n    retorne 1;\n}\n";
        let inspection =
            inspect_markup_with_code(DESIGN, MarkupKind::Design, CodeBehind::Source(code));
        assert!(
            inspection.diagnostics.is_empty(),
            "{:?}",
            inspection.diagnostics
        );
    }
}

#[cfg(test)]
mod ui_removed_tests {
    #[test]
    fn inspecao_mostra_o_erro_do_ui() {
        let source = "funcao inicio() {\n    seja no: inteiro = UI.texto(\"oi\");\n}\n";
        let inspection = crate::inspect_ferro_rs(source);
        let error = inspection
            .diagnostics
            .iter()
            .find(|d| d.code == "ferro.ui-removido")
            .expect("erro de UI removido");
        assert_eq!(error.severity, crate::Level::Error);
        assert!(error.help.as_deref().unwrap_or("").contains("<Pilha>"));
    }
}

#[cfg(test)]
mod format_code_behind_tests {
    const DESIGN: &str = "<Janela Nome=\"J\" Titulo=\"T\" Largura=\"200\" Altura=\"100\">\n    <Botao Nome=\"btnOk\" Texto=\"OK\" X=\"8\" Y=\"8\" AoClicar=\"btnOk_clicar\" />\n</Janela>\n";
    const MESSY: &str = "funcao btnOk_clicar(tela: J) -> J {\nseja mutavel t: J = tela;\n        t.titulo=\"x\";\n    retorne t;\n}\n";

    #[test]
    fn codigo_da_tela_formata_com_os_tipos_do_design() {
        assert!(
            crate::format_ferro_rs(MESSY).is_none(),
            "sem o .design o código não analisa"
        );
        let edits = crate::format_code_behind(MESSY, DESIGN).expect("formata com os tipos da tela");
        assert!(!edits.is_empty());
    }
}
