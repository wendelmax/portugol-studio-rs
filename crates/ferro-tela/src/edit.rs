//! Edições estruturais do `.design`, usadas pelo designer visual da IDE.
//!
//! O designer não monta texto: ele descreve a mudança (mudar um atributo,
//! soltar um controle, mover, remover) e este módulo aplica na árvore e
//! reescreve o arquivo com o formatador canônico. Comentários, linhas em branco
//! e a ordem dos atributos continuam como o aluno deixou.
//!
//! Um elemento é localizado pelo seu caminho: as posições entre os elementos
//! filhos, a partir da `<Janela>`. A raiz é o caminho vazio; `[0, 2]` é o
//! terceiro elemento dentro do primeiro filho da janela. Comentários e texto
//! não contam nas posições.

use crate::ast::{Attribute, Document, Element, Name, Node};
use crate::schema::{self, Layout};

/// Uma mudança no `.design`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Edit {
    /// Muda o valor de um atributo, ou o acrescenta no fim da tag.
    SetAttribute {
        path: Vec<usize>,
        name: String,
        value: String,
    },
    /// Tira um atributo; não faz nada se ele não existir.
    RemoveAttribute { path: Vec<usize>, name: String },
    /// Cria um elemento na posição `index` entre os elementos de `parent`.
    /// `index` igual ao número de filhos acrescenta no fim.
    Insert {
        parent: Vec<usize>,
        index: usize,
        tag: String,
        attributes: Vec<(String, String)>,
    },
    /// Apaga o elemento e tudo dentro dele.
    Remove { path: Vec<usize> },
    /// Leva o elemento para a posição `index` de `parent`. `parent` e `index`
    /// são lidos na árvore de antes da mudança; mover para a própria posição
    /// não muda nada. Atributos que só valem no contêiner de origem (`X`/`Y`
    /// numa `Pilha`, `Linha` fora da `Grade`, ...) são retirados.
    Move {
        path: Vec<usize>,
        parent: Vec<usize>,
        index: usize,
    },
}

/// Por que uma edição não pôde ser aplicada. Indica erro do designer, não do
/// aluno, mas a mensagem é legível para aparecer na barra de status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditError(pub String);

impl std::fmt::Display for EditError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for EditError {}

fn error<T>(message: impl Into<String>) -> Result<T, EditError> {
    Err(EditError(message.into()))
}

/// Elemento no caminho, se existir.
pub fn element_at<'a>(document: &'a Document, path: &[usize]) -> Option<&'a Element> {
    let mut element = &document.root;
    for &index in path {
        element = element.elements().nth(index)?;
    }
    Some(element)
}

fn element_at_mut<'a>(
    document: &'a mut Document,
    path: &[usize],
) -> Result<&'a mut Element, EditError> {
    let mut element = &mut document.root;
    for &index in path {
        element = match nth_element_mut(element, index) {
            Some(child) => child,
            None => return error(format!("Não há elemento na posição {path:?}")),
        };
    }
    Ok(element)
}

fn nth_element_mut(element: &mut Element, index: usize) -> Option<&mut Element> {
    element
        .children
        .iter_mut()
        .filter_map(|node| match node {
            Node::Element(child) => Some(child),
            _ => None,
        })
        .nth(index)
}

/// Posição em `children` do elemento de número `index`; o fim da lista se
/// `index` for o número de elementos.
fn node_position(element: &Element, index: usize) -> Option<usize> {
    let mut seen = 0;
    for (position, node) in element.children.iter().enumerate() {
        if let Node::Element(_) = node {
            if seen == index {
                return Some(position);
            }
            seen += 1;
        }
    }
    (seen == index).then_some(element.children.len())
}

fn new_attribute(name: &str, value: &str) -> Attribute {
    Attribute {
        name: Name {
            text: name.to_owned(),
            span: Default::default(),
        },
        value: value.to_owned(),
        raw: value.to_owned(),
        value_span: Default::default(),
    }
}

fn layout_of(element: &Element) -> Layout {
    schema::element(&element.name.text).map_or(Layout::None, |s| s.layout)
}

fn check_child(parent: &Element, tag: &str) -> Result<(), EditError> {
    let parent_name = parent.name.text.as_str();
    let layout = layout_of(parent);
    if tag == schema::ITEM {
        if layout != Layout::Items {
            return error(format!(
                "`<Item>` só vale dentro de `<Lista>` ou `<Combo>`, não em `<{parent_name}>`"
            ));
        }
        return Ok(());
    }
    let Some(child) = schema::element(tag) else {
        return error(format!("`<{tag}>` não é um elemento do designer"));
    };
    if child.name == schema::WINDOW.name {
        return error("Só pode haver uma `<Janela>`, e ela é a raiz");
    }
    if !child.is_visual() {
        if parent_name != schema::WINDOW.name {
            return error(format!(
                "`<{tag}>` não aparece na tela e fica direto na `<Janela>`"
            ));
        }
        return Ok(());
    }
    match layout {
        Layout::Free | Layout::Stack | Layout::Grid | Layout::Dock => Ok(()),
        _ => error(format!(
            "`<{parent_name}>` não aceita outros controles dentro"
        )),
    }
}

/// Aplica uma edição na árvore.
pub fn apply(document: &mut Document, edit: &Edit) -> Result<(), EditError> {
    match edit {
        Edit::SetAttribute { path, name, value } => {
            if name.is_empty() {
                return error("Atributo sem nome");
            }
            let element = element_at_mut(document, path)?;
            match element.attributes.iter_mut().find(|a| a.name.text == *name) {
                Some(attribute) => {
                    attribute.value = value.clone();
                    attribute.raw = value.clone();
                }
                None => element.attributes.push(new_attribute(name, value)),
            }
            Ok(())
        }
        Edit::RemoveAttribute { path, name } => {
            let element = element_at_mut(document, path)?;
            element.attributes.retain(|a| a.name.text != *name);
            Ok(())
        }
        Edit::Insert {
            parent,
            index,
            tag,
            attributes,
        } => {
            let parent = element_at_mut(document, parent)?;
            check_child(parent, tag)?;
            let Some(position) = node_position(parent, *index) else {
                return error(format!("Posição {index} fora de `<{}>`", parent.name.text));
            };
            let element = Element {
                name: Name {
                    text: tag.clone(),
                    span: Default::default(),
                },
                attributes: attributes
                    .iter()
                    .map(|(name, value)| new_attribute(name, value))
                    .collect(),
                children: Vec::new(),
                span: Default::default(),
                blank_line_before: false,
            };
            parent.children.insert(position, Node::Element(element));
            Ok(())
        }
        Edit::Remove { path } => {
            let Some((&last, parent_path)) = path.split_last() else {
                return error("A `<Janela>` não pode ser apagada");
            };
            let parent = element_at_mut(document, parent_path)?;
            let Some(position) = node_position(parent, last).filter(|&p| p < parent.children.len())
            else {
                return error(format!("Não há elemento na posição {path:?}"));
            };
            parent.children.remove(position);
            Ok(())
        }
        Edit::Move {
            path,
            parent: target,
            index,
        } => move_element(document, path, target, *index),
    }
}

fn move_element(
    document: &mut Document,
    path: &[usize],
    target: &[usize],
    index: usize,
) -> Result<(), EditError> {
    let Some((&last, source_parent)) = path.split_last() else {
        return error("A `<Janela>` não pode ser movida");
    };
    if target.starts_with(path) {
        return error("Um elemento não pode ir para dentro dele mesmo");
    }
    let Some(element) = element_at(document, path) else {
        return error(format!("Não há elemento na posição {path:?}"));
    };
    let tag = element.name.text.clone();
    let Some(target_element) = element_at(document, target) else {
        return error(format!("Não há elemento na posição {target:?}"));
    };
    check_child(target_element, &tag)?;
    let target_count = target_element.elements().count();
    let same_parent = source_parent == target;
    if index > target_count {
        return error(format!(
            "Posição {index} fora de `<{}>`",
            target_element.name.text
        ));
    }
    if same_parent && (index == last || index == last + 1) {
        return Ok(());
    }

    // O destino é lido antes de tirar o elemento: se ele vem antes do destino
    // no mesmo pai, ou dentro de um ancestral do destino, as posições mudam.
    let mut target = target.to_vec();
    let mut index = index;
    if same_parent {
        if last < index {
            index -= 1;
        }
    } else if target.len() > source_parent.len()
        && target.starts_with(source_parent)
        && target[source_parent.len()] > last
    {
        target[source_parent.len()] -= 1;
    }

    let parent = element_at_mut(document, source_parent)?;
    let position = node_position(parent, last).expect("elemento conferido acima");
    let Node::Element(mut element) = parent.children.remove(position) else {
        unreachable!("node_position aponta para elementos");
    };
    element.blank_line_before = false;

    let destination = element_at_mut(document, &target)?;
    let layout = layout_of(destination);
    element.attributes.retain(|attribute| {
        schema::element(&element.name.text)
            .and_then(|s| s.attribute(&attribute.name.text))
            .is_none_or(|s| s.placement.fits(layout))
    });
    let position = node_position(destination, index).expect("posição conferida acima");
    destination
        .children
        .insert(position, Node::Element(element));
    Ok(())
}

/// Aplica as edições em ordem e devolve o texto formatado. Se uma falhar,
/// nenhuma vale.
pub fn edit_source(source: &str, edits: &[Edit]) -> Result<String, EditError> {
    let mut document = match crate::parse(source) {
        Ok(document) => document,
        Err(diagnostic) => {
            return error(format!(
                "Corrija o erro no texto antes de usar o designer: {}",
                diagnostic.message
            ));
        }
    };
    for edit in edits {
        apply(&mut document, edit)?;
    }
    Ok(crate::format(&document))
}

/// Primeiro nome livre para um controle novo, no estilo do Windows Forms:
/// `botao1`, `botao2`, `caixaTexto1`.
pub fn next_control_name(document: &Document, tag: &str) -> String {
    let mut base = String::with_capacity(tag.len());
    let mut chars = tag.chars();
    if let Some(first) = chars.next() {
        base.extend(first.to_lowercase());
    }
    base.extend(chars);
    let taken = |name: &str| {
        document
            .root
            .control_name()
            .is_some_and(|n| n.value == name)
            || document.find(name).is_some()
    };
    (1..)
        .map(|n| format!("{base}{n}"))
        .find(|name| !taken(name))
        .expect("sempre há um número livre")
}
