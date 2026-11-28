//! Parser do pseudo-XAML dos arquivos `.design`.
//!
//! É XML simples: elementos, atributos entre aspas, comentários `<!-- -->`,
//! texto e as cinco entidades predefinidas. Não há declaração `<?xml?>`,
//! DOCTYPE, CDATA, namespaces nem referências numéricas; cada um desses casos
//! gera um diagnóstico que explica o que fazer.

use ferro_hir::{Diagnostic, Span, line_col};

use crate::ast::{Attribute, Comment, Document, Element, Name, Node, Text};

pub fn parse(source: &str) -> Result<Document, Diagnostic> {
    let mut parser = Parser { source, pos: 0 };
    parser.document()
}

struct Parser<'a> {
    source: &'a str,
    pos: usize,
}

impl Parser<'_> {
    fn document(&mut self) -> Result<Document, Diagnostic> {
        let leading = self.loose_comments()?;
        self.reject_prolog()?;
        if !self.at("<") || self.at("</") {
            return Err(self.error_here(
                "design.expected-window",
                "Um arquivo .design começa com `<Janela ...>`",
            ));
        }
        let root = self.element()?;
        let trailing = self.loose_comments()?;
        if self.pos < self.source.len() {
            let start = self.pos;
            let end = self.source[start..]
                .find(['\n', '\r'])
                .map_or(self.source.len(), |offset| start + offset);
            return Err(Diagnostic::error(
                "design.content-after-window",
                "Um arquivo .design descreve uma única janela; nada pode vir depois de `</Janela>`",
                span(start, end),
            ));
        }
        Ok(Document {
            leading,
            root,
            trailing,
        })
    }

    /// Comentários e espaços fora da raiz.
    fn loose_comments(&mut self) -> Result<Vec<Comment>, Diagnostic> {
        let mut comments = Vec::new();
        loop {
            let blank = self.skip_whitespace();
            if self.at("<!--") {
                comments.push(self.comment(blank)?);
            } else {
                return Ok(comments);
            }
        }
    }

    fn reject_prolog(&mut self) -> Result<(), Diagnostic> {
        if self.at("<?") {
            let end = self.find_from("?>").map_or(self.source.len(), |e| e + 2);
            return Err(Diagnostic::error(
                "design.xml-declaration",
                "A declaração `<?xml ...?>` não é usada em arquivos .design",
                span(self.pos, end),
            )
            .with_help("Remova a linha; o arquivo começa direto em `<Janela>`."));
        }
        if self.at("<!") && !self.at("<!--") {
            let end = self.find_from(">").map_or(self.source.len(), |e| e + 1);
            return Err(Diagnostic::error(
                "design.doctype",
                "Declarações `<!DOCTYPE ...>` não são usadas em arquivos .design",
                span(self.pos, end),
            )
            .with_help("Remova a declaração."));
        }
        Ok(())
    }

    fn element(&mut self) -> Result<Element, Diagnostic> {
        let start = self.pos;
        self.pos += 1; // `<`
        let name = self.name("Esperado o nome do elemento depois de `<`")?;
        let mut attributes: Vec<Attribute> = Vec::new();

        loop {
            let had_space = self.skip_whitespace_any();
            if self.at("/>") {
                self.pos += 2;
                return Ok(Element {
                    name,
                    attributes,
                    children: Vec::new(),
                    span: span(start, self.pos),
                    blank_line_before: false,
                });
            }
            if self.at(">") {
                self.pos += 1;
                break;
            }
            if self.pos >= self.source.len() {
                return Err(Diagnostic::error(
                    "design.unclosed-tag",
                    format!("Falta `>` para terminar a tag `<{}`", name.text),
                    name.span,
                ));
            }
            if !had_space && !attributes.is_empty() {
                return Err(
                    self.error_here("design.expected-space", "Separe os atributos com um espaço")
                );
            }
            attributes.push(self.attribute(&name)?);
        }

        let children = self.content(&name)?;
        Ok(Element {
            name,
            attributes,
            children,
            span: span(start, self.pos),
            blank_line_before: false,
        })
    }

    fn attribute(&mut self, element: &Name) -> Result<Attribute, Diagnostic> {
        let name = self.name(&format!(
            "Esperado o nome de um atributo, `>` ou `/>` na tag `<{}>`",
            element.text
        ))?;
        self.skip_whitespace_any();
        if !self.at("=") {
            return Err(self
                .error_here(
                    "design.expected-equals",
                    format!("Esperado `=` depois do atributo `{}`", name.text),
                )
                .with_help(format!("Escreva `{}=\"valor\"`.", name.text)));
        }
        self.pos += 1;
        self.skip_whitespace_any();
        let quote = match self.peek() {
            Some(q @ ('"' | '\'')) => q,
            _ => {
                return Err(self
                    .error_here(
                        "design.expected-quote",
                        format!("O valor de `{}` deve ficar entre aspas", name.text),
                    )
                    .with_help(format!("Escreva `{}=\"valor\"`.", name.text)));
            }
        };
        self.pos += 1;
        let value_start = self.pos;
        let Some(offset) = self.source[value_start..].find(quote) else {
            return Err(Diagnostic::error(
                "design.unterminated-value",
                format!("O valor de `{}` não fecha as aspas", name.text),
                span(value_start - 1, value_start),
            ));
        };
        let value_end = value_start + offset;
        let raw = &self.source[value_start..value_end];
        if let Some(lt) = raw.find('<') {
            return Err(Diagnostic::error(
                "design.unescaped-less-than",
                "`<` não pode aparecer dentro de um valor",
                span(value_start + lt, value_start + lt + 1),
            )
            .with_help("Escreva `&lt;` no lugar de `<`."));
        }
        let value = decode(raw, value_start)?;
        self.pos = value_end + 1;
        Ok(Attribute {
            name,
            value,
            raw: raw.to_owned(),
            value_span: span(value_start, value_end),
        })
    }

    fn content(&mut self, parent: &Name) -> Result<Vec<Node>, Diagnostic> {
        let mut children = Vec::new();
        loop {
            let blank = self.skip_whitespace();
            if self.pos >= self.source.len() {
                let (line, _) = line_col(self.source, parent.span.start);
                return Err(Diagnostic::error(
                    "design.unclosed-element",
                    format!("Falta fechar `<{}>` aberta na linha {line}", parent.text),
                    parent.span,
                )
                .with_help(format!("Escreva `</{}>` no fim do conteúdo.", parent.text)));
            }
            if self.at("</") {
                let start = self.pos;
                self.pos += 2;
                let closing = self.name("Esperado o nome do elemento depois de `</`")?;
                self.skip_whitespace_any();
                if !self.at(">") {
                    return Err(self.error_here(
                        "design.unclosed-tag",
                        format!("Falta `>` para terminar `</{}`", closing.text),
                    ));
                }
                self.pos += 1;
                if closing.text != parent.text {
                    let (line, _) = line_col(self.source, parent.span.start);
                    return Err(Diagnostic::error(
                        "design.mismatched-tag",
                        format!(
                            "Esperado `</{}>` para fechar `<{}>` aberta na linha {line}, mas encontrei `</{}>`",
                            parent.text, parent.text, closing.text
                        ),
                        span(start, self.pos),
                    ));
                }
                return Ok(children);
            }
            if self.at("<!--") {
                children.push(Node::Comment(self.comment(blank)?));
                continue;
            }
            if self.at("<![CDATA[") {
                return Err(self
                    .error_here(
                        "design.cdata",
                        "Blocos CDATA não são usados em arquivos .design",
                    )
                    .with_help(
                        "Escreva o texto direto, trocando `<` por `&lt;` e `&` por `&amp;`.",
                    ));
            }
            if self.at("<?") || self.at("<!") {
                return Err(self.error_here(
                    "design.unexpected-markup",
                    "Instruções `<?...?>` e `<!...>` não são usadas em arquivos .design",
                ));
            }
            if self.at("<") {
                let mut element = self.element()?;
                element.blank_line_before = blank;
                children.push(Node::Element(element));
                continue;
            }
            let start = self.pos;
            let end = self.source[start..]
                .find('<')
                .map_or(self.source.len(), |offset| start + offset);
            let raw = &self.source[start..end];
            let trimmed_start = start + (raw.len() - raw.trim_start().len());
            let trimmed = raw.trim();
            let text = decode(trimmed, trimmed_start)?;
            children.push(Node::Text(Text {
                text,
                span: span(trimmed_start, trimmed_start + trimmed.len()),
                blank_line_before: blank,
            }));
            self.pos = end;
        }
    }

    fn comment(&mut self, blank_line_before: bool) -> Result<Comment, Diagnostic> {
        let start = self.pos;
        let body_start = start + 4;
        let Some(offset) = self.source[body_start..].find("-->") else {
            return Err(Diagnostic::error(
                "design.unclosed-comment",
                "Comentário sem `-->` de fechamento",
                span(start, body_start),
            ));
        };
        let body_end = body_start + offset;
        self.pos = body_end + 3;
        Ok(Comment {
            text: self.source[body_start..body_end].to_owned(),
            span: span(start, self.pos),
            blank_line_before,
        })
    }

    fn name(&mut self, message: &str) -> Result<Name, Diagnostic> {
        let start = self.pos;
        let mut end = start;
        for (index, c) in self.source[start..].char_indices() {
            let valid = if index == 0 {
                c.is_alphabetic() || c == '_'
            } else {
                c.is_alphanumeric() || c == '_' || c == '-' || c == '.' || c == ':'
            };
            if !valid {
                break;
            }
            end = start + index + c.len_utf8();
        }
        if end == start {
            return Err(self.error_here("design.expected-name", message.to_owned()));
        }
        self.pos = end;
        let text = &self.source[start..end];
        if text.contains(':') {
            return Err(Diagnostic::error(
                "design.namespace",
                format!("Prefixos de namespace como `{text}` não são usados em arquivos .design"),
                span(start, end),
            )
            .with_help(
                "Use só o nome, sem `x:` ou `d:`; por exemplo, `Nome` no lugar de `x:Name`.",
            ));
        }
        Ok(Name {
            text: text.to_owned(),
            span: span(start, end),
        })
    }

    /// Pula espaços e devolve se havia uma linha em branco (duas quebras).
    fn skip_whitespace(&mut self) -> bool {
        let mut newlines = 0;
        while let Some(c) = self.peek() {
            if !c.is_whitespace() {
                break;
            }
            if c == '\n' {
                newlines += 1;
            }
            self.pos += c.len_utf8();
        }
        newlines >= 2
    }

    fn skip_whitespace_any(&mut self) -> bool {
        let before = self.pos;
        self.skip_whitespace();
        self.pos > before
    }

    fn peek(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn at(&self, text: &str) -> bool {
        self.source[self.pos..].starts_with(text)
    }

    fn find_from(&self, text: &str) -> Option<usize> {
        self.source[self.pos..].find(text).map(|o| self.pos + o)
    }

    fn error_here(&self, code: &'static str, message: impl Into<String>) -> Diagnostic {
        let end = self.peek().map_or(self.pos, |c| self.pos + c.len_utf8());
        Diagnostic::error(code, message, span(self.pos, end))
    }
}

/// Decodifica as entidades predefinidas. `offset` é a posição de `raw` no
/// arquivo, para o span dos diagnósticos.
pub(crate) fn decode(raw: &str, offset: usize) -> Result<String, Diagnostic> {
    if !raw.contains('&') {
        return Ok(raw.to_owned());
    }
    let mut out = String::with_capacity(raw.len());
    let mut rest = raw;
    let mut position = offset;
    while let Some(amp) = rest.find('&') {
        out.push_str(&rest[..amp]);
        position += amp;
        rest = &rest[amp..];
        let Some(semicolon) = rest.find(';').filter(|&s| s <= 10) else {
            return Err(Diagnostic::error(
                "design.bare-ampersand",
                "`&` sozinho não é permitido",
                span(position, position + 1),
            )
            .with_help("Escreva `&amp;` no lugar de `&`."));
        };
        let entity = &rest[1..semicolon];
        let decoded = match entity {
            "lt" => '<',
            "gt" => '>',
            "amp" => '&',
            "quot" => '"',
            "apos" => '\'',
            _ => {
                return Err(Diagnostic::error(
                    "design.unknown-entity",
                    format!("Entidade desconhecida: `&{entity};`"),
                    span(position, position + semicolon + 1),
                )
                .with_help(
                    "Use só &lt;, &gt;, &amp;, &quot; e &apos;; os demais caracteres podem ser digitados direto.",
                ));
            }
        };
        out.push(decoded);
        position += semicolon + 1;
        rest = &rest[semicolon + 1..];
    }
    out.push_str(rest);
    Ok(out)
}

fn span(start: usize, end: usize) -> Span {
    Span::new(start as u32, end as u32)
}
