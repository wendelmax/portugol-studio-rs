//! Tradução de um `.design` para um componente [Slint](https://slint.dev)
//! (ADR 0018, seção 6, Fase 6). É o primeiro passo da exportação: o projeto
//! Cargo exportado usa este `.slint` como interface, e o code-behind vira Rust
//! ligado aos *callbacks*.
//!
//! - Posição livre vira `x`/`y`; `Pilha` vira `VerticalLayout` ou
//!   `HorizontalLayout`; `Grade` vira `GridLayout` com `row`/`col`; `Encaixe`
//!   vira uma cadeia desses layouts, como no canvas do designer.
//! - Cada controle nomeado ganha propriedades `in-out` na janela
//!   (`txtA-texto`, `btnOk-habilitado`...), que o adaptador gerado lê antes de
//!   chamar o handler e escreve depois.
//! - `{dados.campo}` vira um *binding* para a propriedade `dados`, uma
//!   `struct` com os campos do registro de `Dados`; nos controles de entrada,
//!   o vínculo é de mão dupla (`<=>`).
//! - Cada evento ligado vira um `callback` com o nome do handler.
//!
//! O texto gerado é determinístico. `<DadosExemplo>` não é exportado: os
//! exemplos são só do designer.

use std::fmt::Write as _;

use crate::ast::{Document, Element};
use crate::codebehind::{FieldSource, FieldType, ScreenType, TypeDef};
use crate::schema;
use crate::value::{self, TextPart};

/// Tipo de um campo da `struct` de dados no Slint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DataType {
    Int,
    Float,
    Bool,
    String,
    /// Outra `struct`, pelo nome, com os campos dela.
    Struct(String, Vec<DataField>),
    /// Vetor (`[T]` no Slint).
    List(Box<DataType>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataField {
    pub name: String,
    pub ty: DataType,
}

/// O registro de `Dados` da tela, com os campos lidos do code-behind.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DataRecord {
    pub name: String,
    pub fields: Vec<DataField>,
}

const INDENT: &str = "    ";

/// Nome Slint de uma propriedade de controle: `txtA` + `texto_vazio` →
/// `txtA-texto-vazio` (o Slint trata `-` e `_` como iguais).
fn control_property(control: &str, field: &str) -> String {
    format!("{control}-{}", field.replace('_', "-"))
}

/// Cadeia Slint. Chaves ficam como estão: no Slint, só `\{` abre uma
/// interpolação, e ela só é escrita pelos vínculos.
fn slint_string(text: &str) -> String {
    let mut out = String::from("\"");
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            other => out.push(other),
        }
    }
    out.push('"');
    out
}

fn data_type(ty: &DataType) -> String {
    match ty {
        DataType::Int => "int".into(),
        DataType::Float => "float".into(),
        DataType::Bool => "bool".into(),
        DataType::String => "string".into(),
        DataType::Struct(name, _) => name.clone(),
        DataType::List(inner) => format!("[{}]", data_type(inner)),
    }
}

fn field_type(ty: &FieldType) -> &'static str {
    match ty {
        FieldType::Integer => "int",
        FieldType::Bool => "bool",
        FieldType::TextList => "[string]",
        _ => "string",
    }
}

/// `struct`s Slint para o registro de `Dados` e os registros aninhados, das
/// mais internas para fora, sem repetição.
fn structs(fields: &[DataField], name: &str, out: &mut Vec<(String, String)>) {
    for field in fields {
        let mut ty = &field.ty;
        while let DataType::List(inner) = ty {
            ty = inner;
        }
        if let DataType::Struct(inner_name, inner_fields) = ty {
            structs(inner_fields, inner_name, out);
        }
    }
    if out.iter().any(|(existing, _)| existing == name) {
        return;
    }
    let mut text = format!("export struct {name} {{\n");
    for field in fields {
        let _ = writeln!(
            text,
            "{INDENT}{}: {},",
            field.name.replace('_', "-"),
            data_type(&field.ty)
        );
    }
    text.push('}');
    out.push((name.to_owned(), text));
}

/// Texto de um atributo como expressão Slint: literal, ou uma cadeia com os
/// vínculos interpolados (`"Olá, \{root.dados.nome}"`).
fn text_expression(raw: &str) -> String {
    let Ok(parts) = value::text_parts(raw, 0) else {
        return slint_string(raw);
    };
    let mut out = String::from("\"");
    for part in parts {
        match part {
            TextPart::Literal(text) => {
                let quoted = slint_string(&text);
                out.push_str(&quoted[1..quoted.len() - 1]);
            }
            TextPart::Binding { path, .. } => {
                let _ = write!(out, "\\{{root.{}}}", path.join(".").replace('_', "-"));
            }
        }
    }
    out.push('"');
    out
}

/// Caminho `root.dados.campo` de um atributo que é exatamente um vínculo.
fn single_binding(raw: &str) -> Option<String> {
    let parts = value::text_parts(raw, 0).ok()?;
    value::single_binding(&parts)
        .map(|(path, _)| format!("root.{}", path.join(".").replace('_', "-")))
}

struct Writer<'a> {
    out: String,
    screen: &'a ScreenType,
    callbacks: Vec<String>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Parent {
    /// `Janela` ou `Painel`: posição por `x`/`y`.
    Free,
    /// `Pilha`, `Grade` ou um elo do `Encaixe`.
    Layout,
    Grid,
}

impl Writer<'_> {
    fn line(&mut self, depth: usize, text: &str) {
        for _ in 0..depth {
            self.out.push_str(INDENT);
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    fn control_type(&self, tag: &str) -> Option<&TypeDef> {
        self.screen
            .controls
            .iter()
            .find(|control| control.name == tag)
    }

    /// Valor de uma propriedade do elemento: vínculo, ou a propriedade
    /// `in-out` do controle nomeado, ou o literal do atributo.
    fn property(&self, element: &Element, attribute: &str) -> Option<String> {
        let raw = element.attribute(attribute).map(|a| a.raw.as_str());
        if let Some(raw) = raw
            && value::text_parts(raw, 0).is_ok_and(|parts| value::has_binding(&parts))
        {
            return Some(text_expression(raw));
        }
        let name = element.control_name()?;
        let field = value::field_name(attribute);
        let exposed = self
            .control_type(&element.name.text)
            .is_some_and(|definition| definition.fields.iter().any(|f| f.name == field));
        exposed.then(|| format!("root.{}", control_property(&name.value, &field)))
    }

    fn callback(&mut self, element: &Element, attribute: &str, slint: &str, depth: usize) {
        if let Some(handler) = element
            .attribute(attribute)
            .map(|a| a.value.trim().to_owned())
            .filter(|h| !h.is_empty())
        {
            if !self.callbacks.contains(&handler) {
                self.callbacks.push(handler.clone());
            }
            self.push_line(depth, format!("{slint} => {{ root.{handler}(); }}"));
        }
    }

    fn push_line(&mut self, depth: usize, text: String) {
        self.line(depth, &text);
    }

    fn geometry(&mut self, element: &Element, parent: Parent, depth: usize) {
        let integer = |name: &str| {
            element
                .attribute(name)
                .and_then(|a| a.value.trim().parse::<i64>().ok())
        };
        let margin = element
            .attribute("Margem")
            .and_then(|a| value::thickness(&a.value))
            .map_or((0, 0), |t| (t.left, t.top));
        match parent {
            Parent::Free => {
                let named = element.control_name().is_some()
                    && schema::element(&element.name.text).is_some_and(|s| s.is_visual());
                for (attribute, slint, offset) in [("X", "x", margin.0), ("Y", "y", margin.1)] {
                    let field = slint;
                    if named && self.property(element, attribute).is_some() {
                        let name = &element.control_name().expect("nomeado").value;
                        let base = format!("root.{}", control_property(name, field));
                        let text = if offset == 0 {
                            format!("{slint}: {base} * 1px;")
                        } else {
                            format!("{slint}: ({base} + {offset}) * 1px;")
                        };
                        self.push_line(depth, text);
                    } else {
                        self.push_line(
                            depth,
                            format!("{slint}: {}px;", integer(attribute).unwrap_or(0) + offset),
                        );
                    }
                }
            }
            Parent::Grid => {
                if let Some(row) = integer("Linha") {
                    self.push_line(depth, format!("row: {row};"));
                }
                if let Some(col) = integer("Coluna") {
                    self.push_line(depth, format!("col: {col};"));
                }
            }
            Parent::Layout => {}
        }
        let sizes = schema::element(&element.name.text).map(|s| s.default_size);
        for (attribute, slint, default) in [
            ("Largura", "width", sizes.map(|s| s.0)),
            ("Altura", "height", sizes.map(|s| s.1)),
        ] {
            let field = if attribute == "Largura" {
                "largura"
            } else {
                "altura"
            };
            let has = element.attribute(attribute).is_some();
            if let Some(name) = element
                .control_name()
                .filter(|_| self.property(element, attribute).is_some())
            {
                if has || parent == Parent::Free {
                    self.push_line(
                        depth,
                        format!(
                            "{slint}: root.{} * 1px;",
                            control_property(&name.value, field)
                        ),
                    );
                }
            } else if let Some(value) = integer(attribute) {
                self.push_line(depth, format!("{slint}: {value}px;"));
            } else if parent == Parent::Free
                && let Some(default) = default.filter(|d| *d > 0)
            {
                self.push_line(depth, format!("{slint}: {default}px;"));
            }
        }
    }

    fn common(&mut self, element: &Element, depth: usize) {
        for (attribute, slint) in [("Visivel", "visible"), ("Habilitado", "enabled")] {
            if let Some(raw) = element.attribute(attribute).map(|a| a.raw.clone())
                && let Some(path) = single_binding(&raw)
            {
                self.push_line(depth, format!("{slint}: {path};"));
            } else if let Some(value) = self.property(element, attribute) {
                self.push_line(depth, format!("{slint}: {value};"));
            }
        }
    }

    /// Valor de texto para o `Texto` (ou outro atributo de texto).
    fn text(&mut self, element: &Element, attribute: &str, slint: &str, depth: usize) {
        if let Some(value) = self.property(element, attribute) {
            self.push_line(depth, format!("{slint}: {value};"));
        } else if let Some(attribute) = element.attribute(attribute) {
            self.push_line(
                depth,
                format!("{slint}: {};", text_expression(&attribute.raw)),
            );
        }
    }

    /// Mão dupla: `{dados.campo}` vira `<=> root.dados.campo`; senão, a
    /// propriedade `in-out` do controle.
    fn two_way(&mut self, element: &Element, attribute: &str, slint: &str, depth: usize) {
        if let Some(raw) = element.attribute(attribute).map(|a| a.raw.clone())
            && let Some(path) = single_binding(&raw)
        {
            self.push_line(depth, format!("{slint} <=> {path};"));
        } else if let Some(name) = element.control_name() {
            let field = value::field_name(attribute);
            self.push_line(
                depth,
                format!(
                    "{slint} <=> root.{};",
                    control_property(&name.value, &field)
                ),
            );
        }
    }

    fn items(element: &Element) -> Vec<String> {
        element
            .elements()
            .filter(|child| child.name.text == schema::ITEM)
            .map(|item| {
                item.texts()
                    .map(|t| t.text.clone())
                    .collect::<Vec<_>>()
                    .join(" ")
            })
            .collect()
    }

    fn element(&mut self, element: &Element, parent: Parent, depth: usize) {
        let tag = element.name.text.as_str();
        let id = element
            .control_name()
            .map(|n| format!("{} := ", n.value))
            .unwrap_or_default();
        match tag {
            "Rotulo" => {
                self.push_line(depth, format!("{id}Text {{"));
                self.geometry(element, parent, depth + 1);
                self.text(element, "Texto", "text", depth + 1);
                self.push_line(depth + 1, "vertical-alignment: center;".into());
                self.common_visible_only(element, depth + 1);
            }
            "Botao" => {
                self.push_line(depth, format!("{id}Button {{"));
                self.geometry(element, parent, depth + 1);
                self.text(element, "Texto", "text", depth + 1);
                self.common(element, depth + 1);
                self.callback(element, "AoClicar", "clicked", depth + 1);
            }
            "CaixaTexto" => {
                self.push_line(depth, format!("{id}LineEdit {{"));
                self.geometry(element, parent, depth + 1);
                self.two_way(element, "Texto", "text", depth + 1);
                self.text(element, "TextoVazio", "placeholder-text", depth + 1);
                if element
                    .attribute("Senha")
                    .is_some_and(|a| a.value.trim() == "verdadeiro")
                {
                    self.push_line(depth + 1, "input-type: password;".into());
                }
                self.common(element, depth + 1);
                self.callback(element, "AoAlterar", "edited(text)", depth + 1);
            }
            "CaixaSelecao" | "BotaoOpcao" => {
                // O Slint não tem botão de opção nos widgets padrão: os dois
                // viram CheckBox; o grupo é mantido pelo adaptador.
                self.push_line(depth, format!("{id}CheckBox {{"));
                self.geometry(element, parent, depth + 1);
                self.text(element, "Texto", "text", depth + 1);
                self.two_way(element, "Marcado", "checked", depth + 1);
                self.common(element, depth + 1);
                self.callback(element, "AoAlterar", "toggled", depth + 1);
            }
            "Combo" => {
                self.push_line(depth, format!("{id}ComboBox {{"));
                self.geometry(element, parent, depth + 1);
                self.list_model(element, "model", false, depth + 1);
                self.two_way(element, "Selecionado", "current-index", depth + 1);
                self.common(element, depth + 1);
                self.callback(element, "AoSelecionar", "selected(value)", depth + 1);
            }
            "Lista" => {
                self.push_line(depth, format!("{id}StandardListView {{"));
                self.geometry(element, parent, depth + 1);
                self.list_model(element, "model", true, depth + 1);
                self.two_way(element, "Selecionado", "current-item", depth + 1);
                self.common(element, depth + 1);
                self.callback(
                    element,
                    "AoSelecionar",
                    "current-item-changed(index)",
                    depth + 1,
                );
            }
            "Imagem" => {
                self.push_line(depth, format!("{id}Image {{"));
                self.geometry(element, parent, depth + 1);
                if let Some(path) = element.attribute("Arquivo") {
                    self.push_line(
                        depth + 1,
                        format!("source: @image-url({});", slint_string(&path.value)),
                    );
                }
                self.common_visible_only(element, depth + 1);
            }
            "Painel" => {
                self.push_line(depth, format!("{id}Rectangle {{"));
                self.geometry(element, parent, depth + 1);
                if element
                    .attribute("Borda")
                    .is_some_and(|a| a.value.trim() == "verdadeiro")
                    || element.attribute("Titulo").is_some()
                {
                    self.push_line(depth + 1, "border-width: 1px;".into());
                    self.push_line(depth + 1, "border-color: #9ca3af;".into());
                    self.push_line(depth + 1, "border-radius: 4px;".into());
                }
                self.common_visible_only(element, depth + 1);
                self.children_free(element, depth + 1);
            }
            "Pilha" => {
                let horizontal = element
                    .attribute("Orientacao")
                    .is_some_and(|a| a.value.trim() == "Horizontal");
                self.push_line(
                    depth,
                    format!(
                        "{id}{} {{",
                        if horizontal {
                            "HorizontalLayout"
                        } else {
                            "VerticalLayout"
                        }
                    ),
                );
                self.geometry(element, parent, depth + 1);
                self.layout_spacing(element, depth + 1);
                self.common_visible_only(element, depth + 1);
                for child in visual_children(element) {
                    self.element(child, Parent::Layout, depth + 1);
                }
            }
            "Grade" => {
                self.push_line(depth, format!("{id}GridLayout {{"));
                self.geometry(element, parent, depth + 1);
                self.layout_spacing(element, depth + 1);
                self.common_visible_only(element, depth + 1);
                for child in visual_children(element) {
                    self.element(child, Parent::Grid, depth + 1);
                }
            }
            "Encaixe" => {
                self.push_line(depth, format!("{id}VerticalLayout {{"));
                self.geometry(element, parent, depth + 1);
                self.layout_spacing(element, depth + 1);
                self.common_visible_only(element, depth + 1);
                let children: Vec<&Element> = visual_children(element).collect();
                self.dock(&children, depth + 1);
            }
            _ => return,
        }
        self.push_line(depth, "}".into());
    }

    fn common_visible_only(&mut self, element: &Element, depth: usize) {
        if let Some(raw) = element.attribute("Visivel").map(|a| a.raw.clone())
            && let Some(path) = single_binding(&raw)
        {
            self.push_line(depth, format!("visible: {path};"));
        } else if let Some(value) = self.property(element, "Visivel") {
            self.push_line(depth, format!("visible: {value};"));
        }
    }

    /// Itens de `Combo` (`[string]`) e `Lista` (`[StandardListViewItem]`,
    /// que o adaptador preenche a partir dos itens do controle).
    fn list_model(&mut self, element: &Element, slint: &str, list_view: bool, depth: usize) {
        if let Some(name) = element.control_name() {
            let field = if list_view { "itens-lista" } else { "itens" };
            self.push_line(
                depth,
                format!("{slint}: root.{};", control_property(&name.value, field)),
            );
            return;
        }
        let items: Vec<String> = Self::items(element)
            .iter()
            .map(|item| slint_string(item))
            .collect();
        let model = if list_view {
            items
                .iter()
                .map(|i| format!("{{ text: {i} }}"))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            items.join(", ")
        };
        self.push_line(depth, format!("{slint}: [{model}];"));
    }

    fn layout_spacing(&mut self, element: &Element, depth: usize) {
        if let Some(spacing) = element
            .attribute("Espaco")
            .and_then(|a| a.value.trim().parse::<i64>().ok())
        {
            self.push_line(depth, format!("spacing: {spacing}px;"));
        }
        if let Some(padding) = element
            .attribute("Preenchimento")
            .and_then(|a| value::thickness(&a.value))
        {
            for (side, amount) in [
                ("left", padding.left),
                ("top", padding.top),
                ("right", padding.right),
                ("bottom", padding.bottom),
            ] {
                self.push_line(depth, format!("padding-{side}: {amount}px;"));
            }
        }
        if let Some(margin) = element
            .attribute("Margem")
            .and_then(|a| value::thickness(&a.value))
        {
            // Num layout, a margem do próprio layout vira preenchimento.
            if element.attribute("Preenchimento").is_none() {
                for (side, amount) in [
                    ("left", margin.left),
                    ("top", margin.top),
                    ("right", margin.right),
                    ("bottom", margin.bottom),
                ] {
                    self.push_line(depth, format!("padding-{side}: {amount}px;"));
                }
            }
        }
    }

    fn children_free(&mut self, element: &Element, depth: usize) {
        let children: Vec<&Element> = visual_children(element).collect();
        let fills = children.len() == 1
            && matches!(
                children[0].name.text.as_str(),
                "Pilha" | "Grade" | "Encaixe"
            );
        for child in children {
            self.element(
                child,
                if fills { Parent::Layout } else { Parent::Free },
                depth,
            );
        }
    }

    /// O `Encaixe` como cadeia de layouts: cada filho ocupa a borda dele e o
    /// último sem `Encaixar` preenche o resto.
    fn dock(&mut self, children: &[&Element], depth: usize) {
        let Some((first, rest)) = children.split_first() else {
            return;
        };
        let side = first
            .attribute("Encaixar")
            .map(|a| a.value.trim().to_owned());
        if side.is_none() && rest.is_empty() {
            self.element(first, Parent::Layout, depth);
            return;
        }
        let side = side.unwrap_or_else(|| "Esquerda".into());
        let (layout, child_first) = match side.as_str() {
            "Cima" => ("VerticalLayout", true),
            "Baixo" => ("VerticalLayout", false),
            "Direita" => ("HorizontalLayout", false),
            _ => ("HorizontalLayout", true),
        };
        self.push_line(depth, format!("{layout} {{"));
        if child_first {
            self.element(first, Parent::Layout, depth + 1);
            self.dock(rest, depth + 1);
        } else {
            self.dock(rest, depth + 1);
            self.element(first, Parent::Layout, depth + 1);
        }
        self.push_line(depth, "}".into());
    }

    fn timers(&mut self, root: &Element, depth: usize) {
        for timer in root.elements().filter(|e| e.name.text == "Temporizador") {
            let Some(name) = timer.control_name() else {
                continue;
            };
            let name = name.value.clone();
            self.push_line(depth, format!("{name} := Timer {{"));
            self.push_line(
                depth + 1,
                format!(
                    "interval: root.{} * 1ms;",
                    control_property(&name, "intervalo")
                ),
            );
            self.push_line(
                depth + 1,
                format!("running: root.{};", control_property(&name, "ativo")),
            );
            self.callback(timer, "AoDisparar", "triggered", depth + 1);
            self.push_line(depth, "}".into());
        }
    }
}

fn visual_children(element: &Element) -> impl Iterator<Item = &Element> {
    element
        .elements()
        .filter(|child| schema::element(&child.name.text).is_some_and(|s| s.is_visual()))
}

/// Valor inicial de uma propriedade de controle, a partir do atributo.
fn initial(element: &Element, field: &crate::codebehind::FieldDef) -> Option<String> {
    let FieldSource::Attribute(attribute) = field.source else {
        if field.source == FieldSource::Items {
            let items: Vec<String> = Writer::items(element)
                .iter()
                .map(|item| slint_string(item))
                .collect();
            return Some(format!("[{}]", items.join(", ")));
        }
        return None;
    };
    let raw = element
        .attribute(attribute)
        .map(|a| a.value.trim().to_owned());
    match (&field.ty, raw) {
        (_, Some(raw)) if raw.starts_with('{') => None,
        (FieldType::Integer, Some(raw)) => raw.parse::<i64>().ok().map(|n| n.to_string()),
        (FieldType::Integer, None) => match attribute {
            "Selecionado" => Some("-1".into()),
            "Largura" | "Altura" => schema::element(&element.name.text).map(|s| {
                (if attribute == "Largura" {
                    s.default_size.0
                } else {
                    s.default_size.1
                })
                .to_string()
            }),
            _ => None,
        },
        (FieldType::Bool, Some(raw)) => Some((raw == "verdadeiro").to_string()),
        (FieldType::Bool, None) => {
            matches!(attribute, "Visivel" | "Habilitado").then(|| "true".into())
        }
        (_, Some(_)) => Some(text_expression(
            &element.attribute(attribute).expect("atributo").raw,
        )),
        _ => None,
    }
}

/// Componente `.slint` da tela. `data` traz o registro de `Dados` com os tipos
/// lidos do code-behind; sem ele, a janela não tem a propriedade `dados`.
pub fn export(document: &Document, screen: &ScreenType, data: Option<&DataRecord>) -> String {
    let root = &document.root;
    let window = &screen.window.name;
    let mut header = format!(
        "// Gerado pelo Ferro Studio a partir de {window}.design. Para mudar a tela,\n// edite o .design no Ferro Studio e exporte de novo.\n\nimport {{ Button, CheckBox, ComboBox, LineEdit, StandardListView }} from \"std-widgets.slint\";\n"
    );
    let mut definitions = Vec::new();
    if let Some(data) = data {
        structs(&data.fields, &data.name, &mut definitions);
    }
    for (_, text) in &definitions {
        header.push('\n');
        header.push_str(text);
        header.push('\n');
    }

    let mut writer = Writer {
        out: String::new(),
        screen,
        callbacks: Vec::new(),
    };
    writer.push_line(0, format!("export component {window} inherits Window {{"));
    if let Some(title) = root.attribute("Titulo") {
        let title = text_expression(&title.raw);
        writer.push_line(1, format!("title: {title};"));
    }
    for (attribute, slint) in [("Largura", "width"), ("Altura", "height")] {
        if let Some(value) = root
            .attribute(attribute)
            .and_then(|a| a.value.trim().parse::<i64>().ok())
        {
            writer.push_line(1, format!("{slint}: {value}px;"));
        }
    }
    if let Some(data) = data {
        writer.push_line(1, format!("in-out property <{}> dados;", data.name));
    }

    // Propriedades dos controles nomeados, lidas e escritas pelo adaptador.
    let mut properties = Vec::new();
    for field in &screen.window.fields {
        let (FieldSource::Control(path), FieldType::Named(tag)) = (&field.source, &field.ty) else {
            continue;
        };
        let Some(element) = element_at(root, path) else {
            continue;
        };
        let Some(definition) = screen.controls.iter().find(|c| c.name == *tag) else {
            continue;
        };
        for control_field in &definition.fields {
            if let FieldSource::Attribute(attribute) = control_field.source
                && element.attribute(attribute).is_some_and(|a| {
                    value::text_parts(&a.raw, 0).is_ok_and(|p| value::has_binding(&p))
                })
            {
                continue;
            }
            let ty = field_type(&control_field.ty);
            let name = control_property(&field.name, &control_field.name);
            let text = match initial(element, control_field) {
                Some(value) => format!("in-out property <{ty}> {name}: {value};"),
                None => format!("in-out property <{ty}> {name};"),
            };
            properties.push(text);
        }
        if tag == "Lista" {
            properties.push(format!(
                "in-out property <[StandardListViewItem]> {};",
                control_property(&field.name, "itens-lista")
            ));
        }
    }
    for property in properties {
        writer.push_line(1, property);
    }

    let body_start = writer.out.len();
    writer.children_free(root, 1);
    writer.timers(root, 1);
    let body = writer.out.split_off(body_start);
    for callback in writer.callbacks.clone() {
        writer.push_line(1, format!("callback {callback}();"));
    }
    writer.out.push('\n');
    writer.out.push_str(&body);
    writer.push_line(0, "}".into());

    header.push('\n');
    header.push_str(&writer.out);
    header
}

fn element_at<'a>(root: &'a Element, path: &[usize]) -> Option<&'a Element> {
    let mut element = root;
    for &index in path {
        element = element.elements().nth(index)?;
    }
    Some(element)
}
