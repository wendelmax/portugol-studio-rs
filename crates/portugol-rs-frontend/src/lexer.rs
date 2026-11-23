use portugol_syntax::{Diagnostic, Span};

#[derive(Debug, Clone, PartialEq)]
pub(super) enum Kind {
    Ident(String),
    Integer(i64),
    Real(f64),
    String(String),
    Character(char),
    LParen,
    RParen,
    LBracket,
    RBracket,
    LBrace,
    RBrace,
    Colon,
    ColonColon,
    Dot,
    Semicolon,
    Comma,
    Arrow,
    FatArrow,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    PlusEq,
    MinusEq,
    StarEq,
    SlashEq,
    PercentEq,
    Assign,
    Equal,
    NotEqual,
    Bang,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
    And,
    Or,
    Eof,
}

#[derive(Debug, Clone)]
pub(super) struct Token {
    pub kind: Kind,
    pub span: Span,
}

pub(super) fn lex(source: &str) -> (Vec<Token>, Vec<Diagnostic>) {
    let mut lexer = Lexer {
        source,
        offset: 0,
        tokens: vec![],
        diagnostics: vec![],
    };
    lexer.scan();
    (lexer.tokens, lexer.diagnostics)
}

struct Lexer<'a> {
    source: &'a str,
    offset: usize,
    tokens: Vec<Token>,
    diagnostics: Vec<Diagnostic>,
}

impl Lexer<'_> {
    fn scan(&mut self) {
        while self.offset < self.source.len() {
            let start = self.offset;
            let ch = self.peek().unwrap();
            if ch.is_whitespace() {
                self.bump();
                continue;
            }
            if self.starts_with("//") {
                self.line_comment();
                continue;
            }
            if self.starts_with("/*") {
                self.block_comment();
                continue;
            }
            if ch.is_alphabetic() || ch == '_' {
                self.identifier(start);
                continue;
            }
            if ch.is_ascii_digit() {
                self.number(start);
                continue;
            }
            match ch {
                '"' => self.quoted(start, '"'),
                '\'' => self.quoted(start, '\''),
                '(' => self.single(start, Kind::LParen),
                ')' => self.single(start, Kind::RParen),
                '[' => self.single(start, Kind::LBracket),
                ']' => self.single(start, Kind::RBracket),
                '{' => self.single(start, Kind::LBrace),
                '}' => self.single(start, Kind::RBrace),
                ':' if self.starts_with("::") => {
                    self.bump();
                    self.bump();
                    self.push(start, Kind::ColonColon);
                }
                ':' => self.single(start, Kind::Colon),
                '.' => self.single(start, Kind::Dot),
                ';' => self.single(start, Kind::Semicolon),
                ',' => self.single(start, Kind::Comma),
                '+' => self.pair(start, '=', Kind::PlusEq, Kind::Plus),
                '-' if self.starts_with("->") => {
                    self.bump();
                    self.bump();
                    self.push(start, Kind::Arrow);
                }
                '-' => self.pair(start, '=', Kind::MinusEq, Kind::Minus),
                '*' => self.pair(start, '=', Kind::StarEq, Kind::Star),
                '/' => self.pair(start, '=', Kind::SlashEq, Kind::Slash),
                '%' => self.pair(start, '=', Kind::PercentEq, Kind::Percent),
                '=' if self.starts_with("=>") => {
                    self.bump();
                    self.bump();
                    self.push(start, Kind::FatArrow);
                }
                '=' => self.pair(start, '=', Kind::Equal, Kind::Assign),
                '!' => self.pair(start, '=', Kind::NotEqual, Kind::Bang),
                '>' => self.pair(start, '=', Kind::GreaterEqual, Kind::Greater),
                '<' => self.pair(start, '=', Kind::LessEqual, Kind::Less),
                '&' if self.starts_with("&&") => {
                    self.bump();
                    self.bump();
                    self.push(start, Kind::And);
                }
                '|' if self.starts_with("||") => {
                    self.bump();
                    self.bump();
                    self.push(start, Kind::Or);
                }
                _ => {
                    self.bump();
                    self.error(
                        "pors.invalid-character",
                        format!("Caractere inesperado: {ch}"),
                        start,
                        self.offset,
                    );
                }
            }
        }
        let end = self.source.len() as u32;
        self.tokens.push(Token {
            kind: Kind::Eof,
            span: Span::new(end, end),
        });
    }

    fn identifier(&mut self, start: usize) {
        self.bump();
        while self
            .peek()
            .is_some_and(|ch| ch.is_alphanumeric() || ch == '_')
        {
            self.bump();
        }
        self.push(
            start,
            Kind::Ident(self.source[start..self.offset].to_owned()),
        );
    }

    fn number(&mut self, start: usize) {
        while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
            self.bump();
        }
        let is_real = self.peek() == Some('.')
            && self.peek_after_char().is_some_and(|ch| ch.is_ascii_digit());
        if is_real {
            self.bump();
            while self.peek().is_some_and(|ch| ch.is_ascii_digit()) {
                self.bump();
            }
        }
        let text = &self.source[start..self.offset];
        if is_real {
            match text.parse::<f64>() {
                Ok(value) => self.push(start, Kind::Real(value)),
                Err(_) => self.error(
                    "pors.invalid-number",
                    "Número real inválido".into(),
                    start,
                    self.offset,
                ),
            }
        } else {
            match text.parse::<i64>() {
                Ok(value) => self.push(start, Kind::Integer(value)),
                Err(_) => self.error(
                    "pors.invalid-number",
                    "Número inteiro fora do intervalo suportado".into(),
                    start,
                    self.offset,
                ),
            }
        }
    }

    fn quoted(&mut self, start: usize, quote: char) {
        self.bump();
        let mut value = String::new();
        let mut closed = false;
        while let Some(ch) = self.peek() {
            self.bump();
            if ch == quote {
                closed = true;
                break;
            }
            if ch == '\\' {
                let Some(escaped) = self.peek() else {
                    break;
                };
                self.bump();
                value.push(match escaped {
                    'n' => '\n',
                    'r' => '\r',
                    't' => '\t',
                    '0' => '\0',
                    '\\' => '\\',
                    '"' => '"',
                    '\'' => '\'',
                    other => other,
                });
            } else {
                value.push(ch);
            }
        }
        if !closed {
            self.error(
                "pors.unterminated-literal",
                "Literal não foi fechado".into(),
                start,
                self.offset,
            );
            return;
        }
        if quote == '\'' {
            let mut chars = value.chars();
            if let (Some(ch), None) = (chars.next(), chars.next()) {
                self.push(start, Kind::Character(ch));
            } else {
                self.error(
                    "pors.invalid-character-literal",
                    "Literal caracter deve conter um caractere".into(),
                    start,
                    self.offset,
                );
            }
        } else {
            self.push(start, Kind::String(value));
        }
    }

    fn line_comment(&mut self) {
        while self.peek().is_some_and(|ch| ch != '\n') {
            self.bump();
        }
    }
    fn block_comment(&mut self) {
        let start = self.offset;
        self.bump();
        self.bump();
        while self.offset < self.source.len() && !self.starts_with("*/") {
            self.bump();
        }
        if self.starts_with("*/") {
            self.bump();
            self.bump();
        } else {
            self.error(
                "pors.unterminated-comment",
                "Comentário de bloco não foi fechado".into(),
                start,
                self.offset,
            );
        }
    }
    fn pair(&mut self, start: usize, second: char, pair: Kind, single: Kind) {
        self.bump();
        if self.peek() == Some(second) {
            self.bump();
            self.push(start, pair);
        } else {
            self.push(start, single);
        }
    }
    fn single(&mut self, start: usize, kind: Kind) {
        self.bump();
        self.push(start, kind);
    }
    fn push(&mut self, start: usize, kind: Kind) {
        self.tokens.push(Token {
            kind,
            span: Span::new(start as u32, self.offset as u32),
        });
    }
    fn error(&mut self, code: &'static str, message: String, start: usize, end: usize) {
        self.diagnostics.push(Diagnostic::error(
            code,
            message,
            Span::new(start as u32, end as u32),
        ));
    }
    fn starts_with(&self, text: &str) -> bool {
        self.source[self.offset..].starts_with(text)
    }
    fn peek(&self) -> Option<char> {
        self.source[self.offset..].chars().next()
    }
    fn peek_after_char(&self) -> Option<char> {
        self.source[self.offset..].chars().nth(1)
    }
    fn bump(&mut self) {
        if let Some(ch) = self.peek() {
            self.offset += ch.len_utf8();
        }
    }
}
