//! Leitura dos valores de atributos: vínculos `{dados.campo}`, espessuras
//! (`Margem`), trilhas da `Grade` e listas separadas por vírgula.
//!
//! As funções são públicas porque o frontend PORS e o renderizador também
//! precisam delas para resolver vínculos e montar o layout.

use ferro_hir::{Diagnostic, Span};

/// Parte de um valor de texto: literal ou vínculo.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TextPart {
    Literal(String),
    /// Caminho do vínculo sem as chaves, por exemplo `["dados", "nome"]`.
    Binding {
        path: Vec<String>,
        span: Span,
    },
}

/// Divide o valor bruto de um atributo em literais e vínculos.
///
/// `\{` e `\}` escrevem chaves literais; um `{` sem `}` correspondente é
/// diagnóstico. `raw` é o valor como está no arquivo e `offset` é a posição
/// dele, para os spans. Entidades nos literais são decodificadas.
pub fn text_parts(raw: &str, offset: u32) -> Result<Vec<TextPart>, Diagnostic> {
    let mut parts = Vec::new();
    let mut literal = String::new();
    let mut literal_start = 0usize;
    let mut chars = raw.char_indices().peekable();

    let flush =
        |literal: &mut String, start: usize, parts: &mut Vec<TextPart>| -> Result<(), Diagnostic> {
            if !literal.is_empty() {
                let decoded = crate::parser::decode(literal, offset as usize + start)?;
                parts.push(TextPart::Literal(decoded));
                literal.clear();
            }
            Ok(())
        };

    while let Some((index, c)) = chars.next() {
        match c {
            '\\' if matches!(chars.peek(), Some((_, '{' | '}'))) => {
                if literal.is_empty() {
                    literal_start = index;
                }
                let (_, brace) = chars.next().expect("peek garantiu a chave");
                literal.push(brace);
            }
            '{' => {
                let Some(close) = raw[index..].find('}') else {
                    return Err(Diagnostic::error(
                        "design.unclosed-binding",
                        "Falta `}` para fechar o vínculo",
                        Span::new(offset + index as u32, offset + raw.len() as u32),
                    )
                    .with_help("Para escrever uma chave literal, use `\\{`."));
                };
                flush(&mut literal, literal_start, &mut parts)?;
                let inner = &raw[index + 1..index + close];
                let inner_start = offset + index as u32 + 1;
                let span = Span::new(inner_start, inner_start + inner.len() as u32);
                parts.push(TextPart::Binding {
                    path: binding_path(inner, span)?,
                    span,
                });
                for _ in 0..inner.chars().count() + 1 {
                    chars.next();
                }
            }
            '}' => {
                return Err(Diagnostic::error(
                    "design.unopened-binding",
                    "`}` sem `{` correspondente",
                    Span::new(offset + index as u32, offset + index as u32 + 1),
                )
                .with_help("Para escrever uma chave literal, use `\\}`."));
            }
            other => {
                if literal.is_empty() {
                    literal_start = index;
                }
                literal.push(other);
            }
        }
    }
    flush(&mut literal, literal_start, &mut parts)?;
    Ok(parts)
}

/// Valida o caminho de um vínculo: `dados` seguido de um ou mais campos.
fn binding_path(inner: &str, span: Span) -> Result<Vec<String>, Diagnostic> {
    let trimmed = inner.trim();
    let invalid = || {
        Diagnostic::error(
            "design.invalid-binding",
            format!("Vínculo inválido: `{{{inner}}}`"),
            span,
        )
        .with_help(
            "Um vínculo mostra um campo do estado, como `{dados.nome}`; não use chamadas nem operações.",
        )
    };
    let path: Vec<&str> = trimmed.split('.').collect();
    if path.len() < 2 || !path.iter().all(|p| is_identifier(p)) {
        return Err(invalid());
    }
    if path[0] != "dados" {
        return Err(Diagnostic::error(
            "design.binding-outside-data",
            format!("O vínculo `{{{trimmed}}}` deve começar por `dados.`"),
            span,
        )
        .with_help(format!(
            "A tela mostra o estado guardado em `dados`; por exemplo, `{{dados.{}}}`.",
            path.last().copied().unwrap_or("campo")
        )));
    }
    Ok(path.into_iter().map(str::to_owned).collect())
}

/// O valor é exatamente um vínculo, sem texto em volta.
pub fn single_binding(parts: &[TextPart]) -> Option<(&[String], Span)> {
    match parts {
        [TextPart::Binding { path, span }] => Some((path, *span)),
        _ => None,
    }
}

pub fn has_binding(parts: &[TextPart]) -> bool {
    parts.iter().any(|p| matches!(p, TextPart::Binding { .. }))
}

/// Espessura de `Margem` e `Preenchimento`: esquerda, cima, direita, baixo.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Thickness {
    pub left: i64,
    pub top: i64,
    pub right: i64,
    pub bottom: i64,
}

/// Aceita `"8"`, `"8,4"` (horizontal, vertical) ou `"1,2,3,4"` (esquerda,
/// cima, direita, baixo), como no WPF.
pub fn thickness(value: &str) -> Option<Thickness> {
    let numbers: Option<Vec<i64>> = value
        .split(',')
        .map(|part| {
            part.trim()
                .parse::<i64>()
                .ok()
                .filter(|n| (0..=crate::schema::MAX_COORDINATE).contains(n))
        })
        .collect();
    match numbers?.as_slice() {
        [all] => Some(Thickness {
            left: *all,
            top: *all,
            right: *all,
            bottom: *all,
        }),
        [horizontal, vertical] => Some(Thickness {
            left: *horizontal,
            top: *vertical,
            right: *horizontal,
            bottom: *vertical,
        }),
        [left, top, right, bottom] => Some(Thickness {
            left: *left,
            top: *top,
            right: *right,
            bottom: *bottom,
        }),
        _ => None,
    }
}

/// Tamanho de uma linha ou coluna da `Grade`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Track {
    /// Do tamanho do conteúdo.
    Auto,
    /// Parte proporcional do espaço que sobra (`*` vale 1, `2*` vale 2).
    Star(i64),
    /// Tamanho fixo em pixels lógicos.
    Pixels(i64),
}

/// Lê `"Auto,*,2*,120"`.
pub fn tracks(value: &str) -> Option<Vec<Track>> {
    value
        .split(',')
        .map(|part| {
            let part = part.trim();
            if part == "Auto" {
                Some(Track::Auto)
            } else if let Some(weight) = part.strip_suffix('*') {
                if weight.is_empty() {
                    Some(Track::Star(1))
                } else {
                    weight
                        .parse::<i64>()
                        .ok()
                        .filter(|w| (1..=100).contains(w))
                        .map(Track::Star)
                }
            } else {
                part.parse::<i64>()
                    .ok()
                    .filter(|n| (1..=crate::schema::MAX_COORDINATE).contains(n))
                    .map(Track::Pixels)
            }
        })
        .collect()
}

/// Lista separada por vírgulas, sem itens vazios.
pub fn list(value: &str) -> Option<Vec<&str>> {
    let items: Vec<&str> = value.split(',').map(str::trim).collect();
    items.iter().all(|i| !i.is_empty()).then_some(items)
}

/// Letras ASCII, dígitos e `_`, começando por letra ou `_`.
pub fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Nome do campo no code-behind: `TextoVazio` → `texto_vazio`.
pub fn field_name(attribute: &str) -> String {
    let mut out = String::with_capacity(attribute.len() + 4);
    for (index, c) in attribute.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if index > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}
