//! Tipos que o code-behind de uma tela enxerga (ADR 0018, seção 4).
//!
//! A partir do `.design`, a tela vira um `registro` com o nome da janela: um
//! campo por controle nomeado, as propriedades da própria janela e o campo
//! `dados`, quando a janela declara `Dados`. Cada tipo de controle usado vira
//! um registro com as propriedades que o code-behind pode ler e mudar
//! (`Texto` → `texto`, `TextoVazio` → `texto_vazio`).
//!
//! Este módulo só descreve os tipos; o frontend FERRO os registra antes de
//! analisar o `.fer` da tela. Nada é escrito em disco.

use ferro_hir::{Diagnostic, Span};

use crate::ast::{Document, Element};
use crate::schema::{self, AttributeSchema, ElementSchema, Placement, ValueType};
use crate::value::field_name;

/// Tipo de um campo sintetizado.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    Integer,
    Bool,
    Text,
    /// `vetor<cadeia>`: os itens de `Lista` e `Combo`.
    TextList,
    /// Outro registro: um tipo de controle ou o registro de `Dados`.
    Named(String),
}

/// De onde vem o valor de um campo: o que a execução usa para montar a janela
/// inicial e ler o estado de volta.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldSource {
    /// Um atributo do elemento (`Texto`, `Habilitado`...).
    Attribute(&'static str),
    /// Um controle nomeado, pelo caminho dele na árvore.
    Control(Vec<usize>),
    /// O registro de `Dados`.
    Data,
    /// Os `<Item>` de `Lista` e `Combo`.
    Items,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDef {
    pub name: String,
    pub ty: FieldType,
    pub source: FieldSource,
    /// Onde o campo nasce no `.design` (o `Nome` do controle, o atributo
    /// `Dados`); vazio para as propriedades da janela e dos controles.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeDef {
    pub name: String,
    pub fields: Vec<FieldDef>,
    pub span: Span,
}

/// Os registros de uma tela: os tipos de controle usados, na ordem em que
/// aparecem, e a janela por último (ela usa os outros).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenType {
    pub controls: Vec<TypeDef>,
    pub window: TypeDef,
    /// Nome do registro de `Dados`, que o aluno declara no `.fer`.
    pub data: Option<(String, Span)>,
}

/// Se a propriedade entra no registro do controle. Ficam de fora o nome, os
/// eventos (ligados pela marcação, não por campos), o tipo de `Dados` e os
/// detalhes de layout que só fazem sentido na marcação (espessuras, trilhas,
/// âncoras e o posicionamento dentro de `Grade`, `Encaixe` e layouts).
fn exposed(attribute: &AttributeSchema) -> bool {
    let value = matches!(
        attribute.value,
        ValueType::Integer { .. }
            | ValueType::Text
            | ValueType::Bool
            | ValueType::Choice(_)
            | ValueType::ResourcePath
    );
    let placement = matches!(attribute.placement, Placement::Any | Placement::Free);
    value && placement && attribute.name != "Formato"
}

fn field_type(attribute: &AttributeSchema) -> FieldType {
    match attribute.value {
        ValueType::Integer { .. } => FieldType::Integer,
        ValueType::Bool => FieldType::Bool,
        _ => FieldType::Text,
    }
}

fn properties(element: &ElementSchema) -> Vec<FieldDef> {
    let mut fields: Vec<FieldDef> = element
        .all_attributes()
        .into_iter()
        .filter(|attribute| exposed(attribute))
        .map(|attribute| FieldDef {
            name: field_name(attribute.name),
            ty: field_type(attribute),
            source: FieldSource::Attribute(attribute.name),
            span: Span::default(),
        })
        .collect();
    if element.layout == schema::Layout::Items {
        fields.push(FieldDef {
            name: "itens".to_owned(),
            ty: FieldType::TextList,
            source: FieldSource::Items,
            span: Span::default(),
        });
    }
    fields
}

fn named<'a>(
    element: &'a Element,
    path: &mut Vec<usize>,
    out: &mut Vec<(Vec<usize>, &'a Element)>,
) {
    for (index, child) in element.elements().enumerate() {
        path.push(index);
        if child.control_name().is_some() && schema::element(&child.name.text).is_some() {
            out.push((path.clone(), child));
        }
        named(child, path, out);
        path.pop();
    }
}

/// Registros da tela e os conflitos de nome entre controles e propriedades
/// da janela (por exemplo, um controle chamado `titulo`).
pub fn screen_type(document: &Document) -> (ScreenType, Vec<Diagnostic>) {
    let root = &document.root;
    let mut diagnostics = Vec::new();
    let window_name = root
        .control_name()
        .map(|n| n.value.clone())
        .unwrap_or_else(|| schema::WINDOW.name.to_owned());

    let mut fields = properties(&schema::WINDOW);
    let window_properties: Vec<String> = fields.iter().map(|field| field.name.clone()).collect();
    let mut controls: Vec<TypeDef> = Vec::new();
    let mut elements = Vec::new();
    named(root, &mut Vec::new(), &mut elements);
    for (path, element) in elements {
        let tag = element.name.text.as_str();
        let Some(control_schema) = schema::element(tag) else {
            continue;
        };
        let name = element.control_name().expect("filtrado acima");
        if !window_properties.contains(&name.value) && fields.iter().any(|f| f.name == name.value) {
            // Nome repetido: o validador já aponta `design.duplicate-name`.
            continue;
        }
        if window_properties.contains(&name.value) {
            diagnostics.push(
                Diagnostic::error(
                    "design.name-conflict",
                    format!(
                        "`{}` já é uma propriedade da janela no código; escolha outro nome para o controle",
                        name.value
                    ),
                    name.value_span,
                )
                .with_help(format!(
                    "No código, `tela.{}` é a propriedade da janela.",
                    name.value
                )),
            );
            continue;
        }
        if !controls.iter().any(|control| control.name == tag) {
            controls.push(TypeDef {
                name: tag.to_owned(),
                fields: properties(control_schema),
                span: element.name.span,
            });
        }
        fields.push(FieldDef {
            name: name.value.clone(),
            ty: FieldType::Named(tag.to_owned()),
            source: FieldSource::Control(path),
            span: name.value_span,
        });
    }

    let data = root
        .attribute("Dados")
        .filter(|attribute| !attribute.value.is_empty())
        .map(|attribute| (attribute.value.clone(), attribute.value_span));
    if let Some((data_type, span)) = &data {
        fields.push(FieldDef {
            name: "dados".to_owned(),
            ty: FieldType::Named(data_type.clone()),
            source: FieldSource::Data,
            span: *span,
        });
    }

    let window = TypeDef {
        name: window_name,
        fields,
        span: root.control_name().map_or(root.name.span, |n| n.value_span),
    };
    (
        ScreenType {
            controls,
            window,
            data,
        },
        diagnostics,
    )
}

/// Opções do frontend FERRO para analisar o code-behind de uma tela: os
/// registros da tela (controles antes da janela) e sem exigir `inicio`. Com
/// erro de sintaxe no `.design`, os tipos da tela ficam de fora.
#[cfg(feature = "frontend")]
pub fn lower_options(design_source: &str) -> ferro_frontend::LowerOptions {
    use ferro_frontend::{ExternalField, ExternalRecord, ExternalType, LowerOptions};
    let convert = |definition: &TypeDef| ExternalRecord {
        name: definition.name.clone(),
        fields: definition
            .fields
            .iter()
            .map(|field| ExternalField {
                name: field.name.clone(),
                ty: match &field.ty {
                    FieldType::Integer => ExternalType::Integer,
                    FieldType::Bool => ExternalType::Boolean,
                    FieldType::Text => ExternalType::String,
                    FieldType::TextList => ExternalType::VectorString,
                    FieldType::Named(name) => ExternalType::Named(name.clone()),
                },
            })
            .collect(),
    };
    let records = crate::parse(design_source)
        .map(|document| {
            let (screen, _) = screen_type(&document);
            screen
                .controls
                .iter()
                .chain(std::iter::once(&screen.window))
                .map(convert)
                .collect()
        })
        .unwrap_or_default();
    LowerOptions {
        records,
        require_entry: false,
    }
}
