//! Árvore do formato `.design`.
//!
//! A árvore é genérica (elementos, atributos, comentários e texto) e guarda a
//! ordem do arquivo, os comentários e as linhas em branco entre irmãos, para
//! que a formatação canônica não perca nada do que o aluno escreveu. O
//! significado de cada elemento vem do [`crate::schema`].

use portugol_hir::Span;

/// Um arquivo `.design`: comentários soltos e a `<Janela>` raiz.
#[derive(Debug, Clone, PartialEq)]
pub struct Document {
    /// Comentários antes de `<Janela>`.
    pub leading: Vec<Comment>,
    pub root: Element,
    /// Comentários depois de `</Janela>`.
    pub trailing: Vec<Comment>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Name {
    pub text: String,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub name: Name,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    /// Do `<` de abertura até o `>` de fechamento.
    pub span: Span,
    /// Havia uma linha em branco antes deste elemento.
    pub blank_line_before: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Attribute {
    pub name: Name,
    /// Valor com as entidades (`&amp;`, ...) já decodificadas.
    pub value: String,
    /// Valor como está no arquivo, entre as aspas.
    pub raw: String,
    /// Span do valor sem as aspas.
    pub value_span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Element(Element),
    Comment(Comment),
    Text(Text),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Comment {
    /// Conteúdo entre `<!--` e `-->`, sem alteração.
    pub text: String,
    pub span: Span,
    pub blank_line_before: bool,
}

/// Texto entre tags. Só `<Item>` aceita texto; nos demais elementos o texto é
/// diagnóstico. Espaços e quebras de linha soltos não viram nós.
#[derive(Debug, Clone, PartialEq)]
pub struct Text {
    /// Texto decodificado e sem espaços nas pontas.
    pub text: String,
    pub span: Span,
    pub blank_line_before: bool,
}

impl Element {
    pub fn attribute(&self, name: &str) -> Option<&Attribute> {
        self.attributes.iter().find(|a| a.name.text == name)
    }

    /// Valor de `Nome`, quando o elemento tem um.
    pub fn control_name(&self) -> Option<&Attribute> {
        self.attribute("Nome")
    }

    pub fn elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|node| match node {
            Node::Element(element) => Some(element),
            _ => None,
        })
    }

    pub fn texts(&self) -> impl Iterator<Item = &Text> {
        self.children.iter().filter_map(|node| match node {
            Node::Text(text) => Some(text),
            _ => None,
        })
    }
}

impl Document {
    /// Todos os elementos abaixo da raiz, em profundidade e na ordem do
    /// arquivo.
    pub fn descendants(&self) -> Vec<&Element> {
        fn walk<'a>(element: &'a Element, out: &mut Vec<&'a Element>) {
            for child in element.elements() {
                out.push(child);
                walk(child, out);
            }
        }
        let mut out = Vec::new();
        walk(&self.root, &mut out);
        out
    }

    /// Elemento cujo `Nome` é `name`.
    pub fn find(&self, name: &str) -> Option<&Element> {
        self.descendants()
            .into_iter()
            .find(|e| e.control_name().is_some_and(|n| n.value == name))
    }
}
