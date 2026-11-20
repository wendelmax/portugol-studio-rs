use portugol_syntax::{
    Program,
    ast::{Declarations, Declarator, ForInit, Ident, Item, Stmt, Type},
};
use serde::Serialize;

use crate::{Position, position::LineIndex};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Library,
    Function,
    Constant,
    Variable,
    Parameter,
    Array,
    Matrix,
}

#[derive(Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: &'static str,
    pub position: Position,
    pub children: Vec<SymbolInfo>,
}

fn symbol(index: &LineIndex, name: &Ident, kind: SymbolKind, ty: &'static str) -> SymbolInfo {
    SymbolInfo {
        name: name.name.clone(),
        kind,
        ty,
        position: index.position(name.span.start),
        children: Vec::new(),
    }
}

fn declarations(index: &LineIndex, values: &Declarations, symbols: &mut Vec<SymbolInfo>) {
    for value in &values.declarators {
        let kind = match value {
            Declarator::Array { .. } => SymbolKind::Array,
            Declarator::Matrix { .. } => SymbolKind::Matrix,
            Declarator::Variable { .. } if values.is_const => SymbolKind::Constant,
            Declarator::Variable { .. } => SymbolKind::Variable,
        };
        symbols.push(symbol(index, value.name(), kind, values.ty.as_str()));
    }
}

fn collect_statements(index: &LineIndex, body: &[Stmt], symbols: &mut Vec<SymbolInfo>) {
    for statement in body {
        match statement {
            Stmt::Declarations(values) => declarations(index, values, symbols),
            Stmt::If {
                then_branch,
                else_branch,
                ..
            } => {
                collect_statements(index, then_branch, symbols);
                if let Some(branch) = else_branch {
                    collect_statements(index, branch, symbols);
                }
            }
            Stmt::While { body, .. } | Stmt::DoWhile { body, .. } => {
                collect_statements(index, body, symbols);
            }
            Stmt::For { init, body, .. } => {
                if let Some(ForInit::Declarations(values)) = init {
                    declarations(index, values, symbols);
                }
                collect_statements(index, body, symbols);
            }
            Stmt::Switch { cases, .. } => {
                for case in cases {
                    collect_statements(index, &case.body, symbols);
                }
            }
            Stmt::Return { .. } | Stmt::Break { .. } | Stmt::Assign(_) | Stmt::Expr { .. } => {}
        }
    }
}

pub(crate) fn outline(program: &Program, index: &LineIndex) -> Vec<SymbolInfo> {
    let mut symbols: Vec<_> = program
        .includes
        .iter()
        .map(|include| {
            let mut info = symbol(index, &include.library, SymbolKind::Library, "");
            if let Some(alias) = &include.alias {
                info.name.push_str(" → ");
                info.name.push_str(&alias.name);
            }
            info
        })
        .collect();

    for item in &program.items {
        match item {
            Item::Function(function) => {
                let mut info = symbol(
                    index,
                    &function.name,
                    SymbolKind::Function,
                    function.return_type.unwrap_or(Type::Vazio).as_str(),
                );
                info.children.extend(function.params.iter().map(|param| {
                    let kind = match param.dims {
                        1 => SymbolKind::Array,
                        2 => SymbolKind::Matrix,
                        _ => SymbolKind::Parameter,
                    };
                    symbol(index, &param.name, kind, param.ty.as_str())
                }));
                collect_statements(index, &function.body, &mut info.children);
                symbols.push(info);
            }
            Item::Declarations(values) => declarations(index, values, &mut symbols),
            Item::Stmt(statement) => {
                collect_statements(index, std::slice::from_ref(statement), &mut symbols);
            }
        }
    }
    symbols.sort_by_key(|symbol| symbol.position.offset);
    symbols
}
