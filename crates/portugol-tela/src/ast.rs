//! Árvore sintática do formato `.tela`.
//!
//! A árvore guarda os comentários junto do item que eles acompanham, para que
//! a formatação canônica não perca nada do que o aluno escreveu.

use portugol_hir::Span;

/// Um arquivo `.tela`: o cabeçalho de versão e uma janela.
#[derive(Debug, Clone, PartialEq)]
pub struct Screen {
    /// Versão do esquema declarada em `versao N`. Sem a linha, vale
    /// [`crate::CURRENT_VERSION`].
    pub version: u32,
    pub version_span: Option<Span>,
    /// Comentários antes de `versao` (ou da janela, sem a linha de versão).
    pub leading_comments: Vec<String>,
    pub window: Window,
    /// Comentários depois da janela.
    pub trailing_comments: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Window {
    pub keyword_span: Span,
    pub name: Name,
    /// Comentários entre `versao` e `janela`.
    pub comments: Vec<String>,
    pub body: Body,
}

/// Conteúdo entre chaves de uma janela ou de um controle.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Body {
    pub properties: Vec<Property>,
    pub controls: Vec<Control>,
    /// Comentários soltos antes do `}`.
    pub closing_comments: Vec<String>,
    /// Span das chaves, incluindo `{` e `}`.
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Control {
    /// Tipo do controle (`botao`, `rotulo`, ...), validado contra o esquema.
    pub kind: Name,
    pub name: Name,
    /// Comentários nas linhas acima do controle.
    pub comments: Vec<String>,
    /// Comentário na mesma linha do `}` que fecha o controle.
    pub trailing_comment: Option<String>,
    pub body: Body,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub name: Name,
    pub value: Value,
    pub value_span: Span,
    /// Comentários nas linhas acima da propriedade.
    pub comments: Vec<String>,
    /// Comentário na mesma linha, depois do valor.
    pub trailing_comment: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Integer(i64),
    Text(String),
    Bool(bool),
    /// Um nome sem aspas: handler de evento, nome de registro ou opção.
    Name(String),
    List(Vec<ListItem>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ListItem {
    pub value: Value,
    pub span: Span,
}

impl Value {
    /// Descrição do tipo, usada em mensagens de erro.
    pub fn kind_label(&self) -> &'static str {
        match self {
            Value::Integer(_) => "inteiro",
            Value::Text(_) => "cadeia",
            Value::Bool(_) => "lógico",
            Value::Name(_) => "nome",
            Value::List(_) => "lista",
        }
    }
}

impl Body {
    pub fn property(&self, name: &str) -> Option<&Property> {
        self.properties.iter().find(|p| p.name.text == name)
    }
}

impl Screen {
    /// Todos os controles da janela, em profundidade e na ordem do arquivo.
    pub fn all_controls(&self) -> Vec<&Control> {
        fn walk<'a>(controls: &'a [Control], out: &mut Vec<&'a Control>) {
            for control in controls {
                out.push(control);
                walk(&control.body.controls, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.window.body.controls, &mut out);
        out
    }
}
