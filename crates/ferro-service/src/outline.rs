use serde::Serialize;

use crate::Position;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SymbolKind {
    Function,
    Variable,
    Parameter,
}

#[derive(Serialize)]
pub struct SymbolInfo {
    pub name: String,
    pub kind: SymbolKind,
    pub ty: &'static str,
    pub position: Position,
    pub children: Vec<SymbolInfo>,
}
