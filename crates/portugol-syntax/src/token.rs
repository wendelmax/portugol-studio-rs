//! Tokens da linguagem Portugol 2.0.
//!
//! A gramática de referência é `core/src/main/antlr/PortugolLexico.g4` do
//! Portugol Studio original (Java/ANTLR). Mantemos os mesmos nomes de palavras
//! reservadas e operadores para facilitar a conferência durante o port.

use logos::Logos;

#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip(r"//[^\n]*", allow_greedy = true))]
#[logos(skip r"/\*([^*]|\*+[^*/])*\*+/")]
pub enum TokenKind {
    // Delimitadores
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    // Tipos
    #[token("real")]
    TypeReal,
    #[token("inteiro")]
    TypeInteiro,
    #[token("vazio")]
    TypeVazio,
    #[token("logico")]
    TypeLogico,
    #[token("cadeia")]
    TypeCadeia,
    #[token("caracter")]
    TypeCaracter,

    // Palavras reservadas
    #[token("faca")]
    Faca,
    #[token("enquanto")]
    Enquanto,
    #[token("para")]
    Para,
    #[token("se")]
    Se,
    #[token("senao")]
    Senao,
    #[token("const")]
    Const,
    #[token("funcao")]
    Funcao,
    #[token("programa")]
    Programa,
    #[token("escolha")]
    Escolha,
    #[token("caso")]
    Caso,
    #[token("contrario")]
    Contrario,
    #[token("pare")]
    Pare,
    #[token("retorne")]
    Retorne,
    #[token("inclua")]
    Inclua,
    #[token("biblioteca")]
    Biblioteca,

    // Operadores lógicos e literais
    #[token("nao")]
    Nao,
    #[token("e")]
    ELogico,
    #[token("ou")]
    OuLogico,
    #[token("verdadeiro")]
    Verdadeiro,
    #[token("falso")]
    Falso,

    // Operadores
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("=")]
    Assign,
    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token(">")]
    Gt,
    #[token("<")]
    Lt,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,
    #[token("++")]
    Inc,
    #[token("--")]
    Dec,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,
    #[token("^")]
    Caret,
    #[token("|")]
    Pipe,
    #[token("~")]
    Tilde,
    #[token("-->")]
    Arrow,
    #[token("&")]
    Amp,
    #[token("+=")]
    PlusEq,
    #[token("-=")]
    MinusEq,
    #[token("*=")]
    StarEq,
    #[token("/=")]
    SlashEq,
    #[token("%=")]
    PercentEq,
    #[token("^=")]
    CaretEq,
    #[token("&=")]
    AmpEq,
    #[token("|=")]
    PipeEq,

    // Separadores
    #[token(".")]
    Dot,
    #[token(",")]
    Comma,
    #[token(";")]
    Semi,
    #[token(":")]
    Colon,

    // Literais e identificadores
    #[regex(r"'((\\(u[0-9A-Fa-f]{4}|[0-7]{1,3}|.))|[^'\\])'")]
    Char,
    #[regex(r#""(\\.|[^"\\])*""#)]
    Str,
    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", priority = 1)]
    Ident,
    #[regex(r"[0-9]+\.[0-9]*|\.[0-9]+")]
    Real,
    #[regex(r"0[xX][0-9A-Fa-f]{1,6}")]
    Hex,
    #[regex(r"[0-9]+")]
    Int,

    /// Fim do arquivo. Produzido pelo `logos` uma única vez.
    #[end]
    Eof,
}

impl TokenKind {
    pub fn describe(self) -> &'static str {
        match self {
            TokenKind::LParen => "'('",
            TokenKind::RParen => "')'",
            TokenKind::LBracket => "'['",
            TokenKind::RBracket => "']'",
            TokenKind::LBrace => "'{'",
            TokenKind::RBrace => "'}'",
            TokenKind::TypeReal => "'real'",
            TokenKind::TypeInteiro => "'inteiro'",
            TokenKind::TypeVazio => "'vazio'",
            TokenKind::TypeLogico => "'logico'",
            TokenKind::TypeCadeia => "'cadeia'",
            TokenKind::TypeCaracter => "'caracter'",
            TokenKind::Faca => "'faca'",
            TokenKind::Enquanto => "'enquanto'",
            TokenKind::Para => "'para'",
            TokenKind::Se => "'se'",
            TokenKind::Senao => "'senao'",
            TokenKind::Const => "'const'",
            TokenKind::Funcao => "'funcao'",
            TokenKind::Programa => "'programa'",
            TokenKind::Escolha => "'escolha'",
            TokenKind::Caso => "'caso'",
            TokenKind::Contrario => "'contrario'",
            TokenKind::Pare => "'pare'",
            TokenKind::Retorne => "'retorne'",
            TokenKind::Inclua => "'inclua'",
            TokenKind::Biblioteca => "'biblioteca'",
            TokenKind::Nao => "'nao'",
            TokenKind::ELogico => "'e'",
            TokenKind::OuLogico => "'ou'",
            TokenKind::Verdadeiro => "'verdadeiro'",
            TokenKind::Falso => "'falso'",
            TokenKind::Minus => "'-'",
            TokenKind::Plus => "'+'",
            TokenKind::Star => "'*'",
            TokenKind::Slash => "'/'",
            TokenKind::Percent => "'%'",
            TokenKind::Assign => "'='",
            TokenKind::Eq => "'=='",
            TokenKind::Ne => "'!='",
            TokenKind::Gt => "'>'",
            TokenKind::Lt => "'<'",
            TokenKind::Le => "'<='",
            TokenKind::Ge => "'>='",
            TokenKind::Inc => "'++'",
            TokenKind::Dec => "'--'",
            TokenKind::Shl => "'<<'",
            TokenKind::Shr => "'>>'",
            TokenKind::Caret => "'^'",
            TokenKind::Pipe => "'|'",
            TokenKind::Tilde => "'~'",
            TokenKind::Arrow => "'-->'",
            TokenKind::Amp => "'&'",
            TokenKind::PlusEq => "'+='",
            TokenKind::MinusEq => "'-='",
            TokenKind::StarEq => "'*='",
            TokenKind::SlashEq => "'/='",
            TokenKind::PercentEq => "'%='",
            TokenKind::CaretEq => "'^='",
            TokenKind::AmpEq => "'&='",
            TokenKind::PipeEq => "'|='",
            TokenKind::Dot => "'.'",
            TokenKind::Comma => "','",
            TokenKind::Semi => "';'",
            TokenKind::Colon => "':'",
            TokenKind::Char => "literal de caracter",
            TokenKind::Str => "cadeia de caracteres",
            TokenKind::Ident => "identificador",
            TokenKind::Real => "número real",
            TokenKind::Hex => "número hexadecimal",
            TokenKind::Int => "número inteiro",
            TokenKind::Eof => "fim do arquivo",
        }
    }

    pub fn is_type(self) -> bool {
        matches!(
            self,
            TokenKind::TypeReal
                | TokenKind::TypeInteiro
                | TokenKind::TypeVazio
                | TokenKind::TypeLogico
                | TokenKind::TypeCadeia
                | TokenKind::TypeCaracter
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: crate::span::Span,
}
