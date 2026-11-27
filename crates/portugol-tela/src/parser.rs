//! Parser do formato `.tela`.
//!
//! ```text
//! arquivo     := comentario* ("versao" INTEIRO)? comentario* janela comentario* EOF
//! janela      := "janela" NOME corpo
//! corpo       := "{" (propriedade | controle | comentario)* "}"
//! propriedade := NOME ":" valor
//! controle    := NOME NOME corpo
//! valor       := INTEIRO | CADEIA | NOME | "[" (valor ("," valor)* ","?)? "]"
//! ```
//!
//! `verdadeiro` e `falso` sem aspas são valores lógicos.

use portugol_hir::{Diagnostic, Span};

use crate::ast::{Body, Control, ListItem, Name, Property, Screen, Value, Window};
use crate::lexer::{Kind, Token, lex};

pub fn parse(source: &str) -> Result<Screen, Diagnostic> {
    let tokens = lex(source)?;
    Parser { tokens, index: 0 }.screen()
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}

impl Parser {
    fn screen(&mut self) -> Result<Screen, Diagnostic> {
        let leading_comments = self.own_line_comments();
        let mut window_comments = Vec::new();

        let mut version = crate::CURRENT_VERSION;
        let mut version_span = None;
        if self.at_ident("versao") {
            let keyword = self.advance();
            let token = self.advance();
            match token.kind {
                Kind::Integer(value) if (0..=i64::from(u32::MAX)).contains(&value) => {
                    version = value as u32;
                    version_span = Some(keyword.span.join(token.span));
                }
                _ => {
                    return Err(Diagnostic::error(
                        "tela.expected-version",
                        "Esperado o número da versão depois de `versao`",
                        token.span,
                    ));
                }
            }
            window_comments.extend(self.same_line_comment());
            window_comments.extend(self.own_line_comments());
        }

        if !self.at_ident("janela") {
            return Err(Diagnostic::error(
                "tela.expected-window",
                "Esperado `janela` seguido do nome da janela",
                self.current().span,
            )
            .with_help("Um arquivo .tela começa com `janela NomeDaJanela { ... }`."));
        }
        let keyword_span = self.advance().span;
        let name = self.name("Esperado o nome da janela")?;
        let (body, _) = self.body()?;

        let trailing_comments = self.all_comments();
        if !matches!(self.current().kind, Kind::Eof) {
            return Err(Diagnostic::error(
                "tela.unexpected-token",
                "Um arquivo .tela descreve uma única janela; nada pode vir depois dela",
                self.current().span,
            ));
        }

        Ok(Screen {
            version,
            version_span,
            leading_comments,
            window: Window {
                keyword_span,
                name,
                comments: window_comments,
                body,
            },
            trailing_comments,
        })
    }

    /// Lê `{ ... }`. Devolve o corpo e o comentário na mesma linha do `}`.
    fn body(&mut self) -> Result<(Body, Option<String>), Diagnostic> {
        let open = self.current().span;
        if !matches!(self.current().kind, Kind::LBrace) {
            return Err(Diagnostic::error(
                "tela.expected-brace",
                "Esperado `{`",
                open,
            ));
        }
        self.advance();
        // Um comentário logo depois de `{` vai para o primeiro item.
        let mut pending = self.all_comments();
        let mut body = Body::default();

        loop {
            match self.current().kind.clone() {
                Kind::RBrace => {
                    let close = self.advance().span;
                    body.closing_comments = pending;
                    body.span = open.join(close);
                    let trailing = self.same_line_comment();
                    return Ok((body, trailing));
                }
                Kind::Eof => {
                    return Err(Diagnostic::error(
                        "tela.unclosed-brace",
                        "Falta `}` para fechar este bloco",
                        open,
                    ));
                }
                Kind::Ident(_) => {
                    let first = self.advance();
                    let Kind::Ident(text) = first.kind else {
                        unreachable!()
                    };
                    let first = Name {
                        text,
                        span: first.span,
                    };
                    match self.current().kind {
                        Kind::Colon => {
                            self.advance();
                            let (value, value_span) = self.value()?;
                            let trailing_comment = self.same_line_comment();
                            body.properties.push(Property {
                                name: first,
                                value,
                                value_span,
                                comments: std::mem::take(&mut pending),
                                trailing_comment,
                            });
                        }
                        Kind::Ident(_) => {
                            let name = self.name("Esperado o nome do controle")?;
                            let comments = std::mem::take(&mut pending);
                            let (control_body, trailing_comment) = self.body()?;
                            body.controls.push(Control {
                                kind: first,
                                name,
                                comments,
                                trailing_comment,
                                body: control_body,
                            });
                        }
                        _ => {
                            return Err(Diagnostic::error(
                                "tela.unexpected-token",
                                format!(
                                    "Esperado `:` depois da propriedade `{}` ou o nome de um controle `{} nome {{ ... }}`",
                                    first.text, first.text
                                ),
                                self.current().span,
                            ));
                        }
                    }
                    pending.extend(self.all_comments());
                }
                _ => {
                    return Err(Diagnostic::error(
                        "tela.unexpected-token",
                        "Esperado uma propriedade (`nome: valor`), um controle ou `}`",
                        self.current().span,
                    ));
                }
            }
        }
    }

    fn value(&mut self) -> Result<(Value, Span), Diagnostic> {
        let token = self.advance();
        let value = match token.kind {
            Kind::Integer(value) => Value::Integer(value),
            Kind::String(value) => Value::Text(value),
            Kind::Ident(name) if name == "verdadeiro" => Value::Bool(true),
            Kind::Ident(name) if name == "falso" => Value::Bool(false),
            Kind::Ident(name) => Value::Name(name),
            Kind::LBracket => return self.list(token.span),
            _ => {
                return Err(Diagnostic::error(
                    "tela.expected-value",
                    "Esperado um valor: número, \"cadeia\", verdadeiro/falso, nome ou [lista]",
                    token.span,
                ));
            }
        };
        Ok((value, token.span))
    }

    fn list(&mut self, open: Span) -> Result<(Value, Span), Diagnostic> {
        let mut items = Vec::new();
        loop {
            self.reject_comment_in_list()?;
            if matches!(self.current().kind, Kind::RBracket) {
                let close = self.advance().span;
                return Ok((Value::List(items), open.join(close)));
            }
            if matches!(self.current().kind, Kind::Eof) {
                return Err(Diagnostic::error(
                    "tela.unclosed-bracket",
                    "Falta `]` para fechar esta lista",
                    open,
                ));
            }
            let (value, span) = self.value()?;
            if matches!(value, Value::List(_)) {
                return Err(Diagnostic::error(
                    "tela.nested-list",
                    "Listas não podem conter outras listas",
                    span,
                ));
            }
            items.push(ListItem { value, span });
            self.reject_comment_in_list()?;
            match self.current().kind {
                Kind::Comma => {
                    self.advance();
                }
                Kind::RBracket => {}
                _ => {
                    return Err(Diagnostic::error(
                        "tela.expected-comma",
                        "Esperado `,` ou `]` na lista",
                        self.current().span,
                    ));
                }
            }
        }
    }

    fn name(&mut self, message: &str) -> Result<Name, Diagnostic> {
        let token = self.advance();
        match token.kind {
            Kind::Ident(text) => Ok(Name {
                text,
                span: token.span,
            }),
            _ => Err(Diagnostic::error(
                "tela.expected-name",
                message.to_owned(),
                token.span,
            )),
        }
    }

    /// Comentário na mesma linha do token anterior, se houver.
    fn same_line_comment(&mut self) -> Option<String> {
        if let Kind::Comment {
            text,
            own_line: false,
        } = &self.current().kind
        {
            let text = text.clone();
            self.advance();
            return Some(text);
        }
        None
    }

    fn own_line_comments(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Kind::Comment {
            text,
            own_line: true,
        } = &self.current().kind
        {
            out.push(text.clone());
            self.advance();
        }
        out
    }

    fn all_comments(&mut self) -> Vec<String> {
        let mut out = Vec::new();
        while let Kind::Comment { text, .. } = &self.current().kind {
            out.push(text.clone());
            self.advance();
        }
        out
    }

    /// Comentários dentro de listas seriam perdidos na formatação; por isso
    /// são recusados em vez de descartados.
    fn reject_comment_in_list(&self) -> Result<(), Diagnostic> {
        if matches!(self.current().kind, Kind::Comment { .. }) {
            return Err(Diagnostic::error(
                "tela.comment-in-list",
                "Comentários não são permitidos dentro de listas",
                self.current().span,
            )
            .with_help("Escreva o comentário na linha acima da propriedade."));
        }
        Ok(())
    }

    fn at_ident(&self, word: &str) -> bool {
        matches!(&self.current().kind, Kind::Ident(name) if name == word)
    }

    fn current(&self) -> &Token {
        &self.tokens[self.index]
    }

    fn advance(&mut self) -> Token {
        let token = self.tokens[self.index].clone();
        if self.index + 1 < self.tokens.len() {
            self.index += 1;
        }
        token
    }
}
