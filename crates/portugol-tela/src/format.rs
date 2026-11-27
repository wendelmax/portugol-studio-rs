//! Formatação canônica de arquivos `.tela`.
//!
//! O designer da IDE reescreve o arquivo com esta função, então a saída é
//! determinística: mesma árvore, mesmo texto. Comentários, a ordem das
//! propriedades e a ordem dos controles são preservados.

use std::fmt::Write as _;

use crate::ast::{Body, Control, Property, Screen, Value};

const INDENT: &str = "    ";
/// Controles sem filhos nem comentários ficam numa linha quando cabem nesta
/// largura.
const MAX_INLINE_WIDTH: usize = 100;

pub fn format(screen: &Screen) -> String {
    let mut out = String::new();
    for comment in &screen.leading_comments {
        write_comment(&mut out, 0, comment);
    }
    let _ = writeln!(out, "versao {}", screen.version);
    out.push('\n');

    for comment in &screen.window.comments {
        write_comment(&mut out, 0, comment);
    }
    let _ = write!(out, "janela {} ", screen.window.name.text);
    write_body(&mut out, 0, &screen.window.body);
    out.push('\n');

    for comment in &screen.trailing_comments {
        write_comment(&mut out, 0, comment);
    }
    out
}

fn write_body(out: &mut String, level: usize, body: &Body) {
    out.push_str("{\n");
    let inner = level + 1;

    for property in &body.properties {
        for comment in &property.comments {
            write_comment(out, inner, comment);
        }
        indent(out, inner);
        write_property(out, property);
        write_trailing(out, property.trailing_comment.as_deref());
        out.push('\n');
    }

    let mut previous_multiline = false;
    for (index, control) in body.controls.iter().enumerate() {
        let inline = inline_control(inner, control);
        let separate = index == 0 && !body.properties.is_empty()
            || index > 0
                && (previous_multiline || inline.is_none() || !control.comments.is_empty());
        if separate {
            out.push('\n');
        }
        for comment in &control.comments {
            write_comment(out, inner, comment);
        }
        indent(out, inner);
        match &inline {
            Some(line) => out.push_str(line),
            None => {
                let _ = write!(out, "{} {} ", control.kind.text, control.name.text);
                write_body(out, inner, &control.body);
            }
        }
        write_trailing(out, control.trailing_comment.as_deref());
        out.push('\n');
        previous_multiline = inline.is_none();
    }

    if !body.closing_comments.is_empty() {
        if !body.properties.is_empty() || !body.controls.is_empty() {
            out.push('\n');
        }
        for comment in &body.closing_comments {
            write_comment(out, inner, comment);
        }
    }

    indent(out, level);
    out.push('}');
}

/// Forma de uma linha do controle, quando ela é possível.
fn inline_control(level: usize, control: &Control) -> Option<String> {
    let body = &control.body;
    let has_comments = !body.closing_comments.is_empty()
        || body
            .properties
            .iter()
            .any(|p| !p.comments.is_empty() || p.trailing_comment.is_some());
    if !body.controls.is_empty() || has_comments {
        return None;
    }

    let mut line = format!("{} {} {{", control.kind.text, control.name.text);
    if body.properties.is_empty() {
        line.push('}');
    } else {
        for (index, property) in body.properties.iter().enumerate() {
            line.push_str(if index == 0 { " " } else { "  " });
            write_property(&mut line, property);
        }
        line.push_str(" }");
    }

    let width = level * INDENT.len() + line.chars().count();
    (width <= MAX_INLINE_WIDTH).then_some(line)
}

fn write_property(out: &mut String, property: &Property) {
    out.push_str(&property.name.text);
    out.push_str(": ");
    write_value(out, &property.value);
}

fn write_value(out: &mut String, value: &Value) {
    match value {
        Value::Integer(value) => {
            let _ = write!(out, "{value}");
        }
        Value::Text(text) => {
            out.push('"');
            for c in text.chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    '\t' => out.push_str("\\t"),
                    other => out.push(other),
                }
            }
            out.push('"');
        }
        Value::Bool(true) => out.push_str("verdadeiro"),
        Value::Bool(false) => out.push_str("falso"),
        Value::Name(name) => out.push_str(name),
        Value::List(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                write_value(out, &item.value);
            }
            out.push(']');
        }
    }
}

fn write_comment(out: &mut String, level: usize, comment: &str) {
    indent(out, level);
    if comment.is_empty() {
        out.push_str("//\n");
    } else {
        let _ = writeln!(out, "// {comment}");
    }
}

fn write_trailing(out: &mut String, comment: Option<&str>) {
    if let Some(comment) = comment {
        if comment.is_empty() {
            out.push_str(" //");
        } else {
            let _ = write!(out, " // {comment}");
        }
    }
}

fn indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str(INDENT);
    }
}
