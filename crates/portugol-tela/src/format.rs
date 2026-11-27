//! Formatação canônica de arquivos `.design`.
//!
//! O designer da IDE reescreve o arquivo com esta função, então a saída é
//! determinística: mesma árvore, mesmo texto. Comentários, a ordem dos
//! elementos e dos atributos e as linhas em branco entre irmãos são
//! preservados.

use crate::ast::{Attribute, Comment, Document, Element, Node};

const INDENT: &str = "    ";
/// Tags que passam desta largura ficam com um atributo por linha.
const MAX_WIDTH: usize = 100;

pub fn format(document: &Document) -> String {
    let mut out = String::new();
    for comment in &document.leading {
        write_comment(&mut out, 0, comment);
    }
    let mut root = document.root.clone();
    if root.name.text == crate::schema::WINDOW.name && root.attribute("Formato").is_none() {
        root.attributes.push(Attribute {
            name: crate::ast::Name {
                text: "Formato".to_owned(),
                span: Default::default(),
            },
            value: crate::CURRENT_VERSION.to_string(),
            raw: crate::CURRENT_VERSION.to_string(),
            value_span: Default::default(),
        });
    }
    write_element(&mut out, 0, &root);
    for comment in &document.trailing {
        write_comment(&mut out, 0, comment);
    }
    out
}

fn write_element(out: &mut String, level: usize, element: &Element) {
    let name = &element.name.text;
    let attributes: Vec<String> = element.attributes.iter().map(attribute).collect();

    if let [Node::Text(text)] = element.children.as_slice() {
        indent(out, level);
        write_open(out, level, name, &attributes, ">");
        out.push_str(&escape_text(&text.text));
        out.push_str("</");
        out.push_str(name);
        out.push_str(">\n");
        return;
    }

    indent(out, level);
    if element.children.is_empty() {
        write_open(out, level, name, &attributes, " />");
        out.push('\n');
        return;
    }

    write_open(out, level, name, &attributes, ">");
    out.push('\n');
    for (index, child) in element.children.iter().enumerate() {
        let blank = match child {
            Node::Element(e) => e.blank_line_before,
            Node::Comment(c) => c.blank_line_before,
            Node::Text(t) => t.blank_line_before,
        };
        if blank && index > 0 {
            out.push('\n');
        }
        match child {
            Node::Element(e) => write_element(out, level + 1, e),
            Node::Comment(c) => write_comment(out, level + 1, c),
            Node::Text(t) => {
                indent(out, level + 1);
                out.push_str(&escape_text(&t.text));
                out.push('\n');
            }
        }
    }
    indent(out, level);
    out.push_str("</");
    out.push_str(name);
    out.push_str(">\n");
}

/// Escreve `<Nome atributos` seguido de `end`, numa linha quando cabe e com um
/// atributo por linha quando não cabe.
fn write_open(out: &mut String, level: usize, name: &str, attributes: &[String], end: &str) {
    let inline: usize = attributes.iter().map(|a| a.chars().count() + 1).sum();
    let width = level * INDENT.len() + 1 + name.chars().count() + inline + end.len();
    out.push('<');
    out.push_str(name);
    if width <= MAX_WIDTH || attributes.len() <= 1 {
        for attribute in attributes {
            out.push(' ');
            out.push_str(attribute);
        }
    } else {
        for attribute in attributes {
            out.push('\n');
            indent(out, level + 1);
            out.push_str(attribute);
        }
    }
    out.push_str(end);
}

fn attribute(attribute: &Attribute) -> String {
    format!(
        "{}=\"{}\"",
        attribute.name.text,
        escape_attribute(&attribute.value)
    )
}

fn write_comment(out: &mut String, level: usize, comment: &Comment) {
    indent(out, level);
    out.push_str("<!--");
    out.push_str(&comment.text);
    out.push_str("-->\n");
}

fn escape_attribute(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '"' => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

fn escape_text(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            other => out.push(other),
        }
    }
    out
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str(INDENT);
    }
}
