//! Tabela de símbolos com escopos léxicos.
//!
//! O original usa uma `Memoria` com escopo global + pilha de escopos locais.
//! Aqui a pilha é explícita: `locals` da base para o topo e `global` sempre
//! consultado por último.

use std::collections::HashMap;

use portugol_syntax::Span;

use crate::types::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Variable,
    Vector,
    Matrix,
    Function,
}

impl SymbolKind {
    pub fn dims(self) -> u8 {
        match self {
            SymbolKind::Variable | SymbolKind::Function => 0,
            SymbolKind::Vector => 1,
            SymbolKind::Matrix => 2,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ParamInfo {
    pub name: String,
    pub ty: Ty,
    pub by_ref: bool,
    pub dims: u8,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FunctionInfo {
    pub name: String,
    pub return_ty: Ty,
    pub params: Vec<ParamInfo>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: Ty,
    pub kind: SymbolKind,
    pub is_const: bool,
    pub initialized: bool,
    /// Valor inteiro de uma constante literal, usado para resolver tamanhos de
    /// vetores/matrizes em tempo de compilação.
    pub const_int: Option<i64>,
    pub span: Span,
    pub function: Option<FunctionInfo>,
}

impl Symbol {
    pub fn variable(name: impl Into<String>, ty: Ty, span: Span) -> Self {
        Self {
            name: name.into(),
            ty,
            kind: SymbolKind::Variable,
            is_const: false,
            initialized: false,
            const_int: None,
            span,
            function: None,
        }
    }

    pub fn aggregate(name: impl Into<String>, ty: Ty, kind: SymbolKind, span: Span) -> Self {
        Self {
            name: name.into(),
            ty,
            kind,
            is_const: false,
            initialized: false,
            const_int: None,
            span,
            function: None,
        }
    }

    pub fn parameter(name: impl Into<String>, ty: Ty, dims: u8, span: Span) -> Self {
        let kind = match dims {
            0 => SymbolKind::Variable,
            1 => SymbolKind::Vector,
            _ => SymbolKind::Matrix,
        };

        Self {
            name: name.into(),
            ty,
            kind,
            is_const: false,
            initialized: true,
            const_int: None,
            span,
            function: None,
        }
    }

    pub fn function(info: FunctionInfo) -> Self {
        let span = info.span;
        let ty = info.return_ty;
        let name = info.name.clone();
        Self {
            name,
            ty,
            kind: SymbolKind::Function,
            is_const: false,
            initialized: true,
            const_int: None,
            span,
            function: Some(info),
        }
    }
}

/// Índice especial para o escopo global em [`ScopeTable::find_scope`].
const GLOBAL_SCOPE: usize = usize::MAX;

#[derive(Debug, Default)]
pub struct ScopeTable {
    global: HashMap<String, Symbol>,
    locals: Vec<HashMap<String, Symbol>>,
}

impl ScopeTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push_scope(&mut self) {
        self.locals.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.locals.pop();
    }

    pub fn is_global(&self) -> bool {
        self.locals.is_empty()
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        for scope in self.locals.iter().rev() {
            if let Some(symbol) = scope.get(name) {
                return Some(symbol);
            }
        }
        self.global.get(name)
    }

    pub fn get_mut(&mut self, name: &str) -> Option<&mut Symbol> {
        for scope in self.locals.iter_mut().rev() {
            if let Some(symbol) = scope.get_mut(name) {
                return Some(symbol);
            }
        }
        self.global.get_mut(name)
    }

    pub fn set_initialized(&mut self, name: &str, initialized: bool) {
        if let Some(symbol) = self.get_mut(name) {
            symbol.initialized = initialized;
        }
    }

    fn find_scope(&self, name: &str) -> Option<usize> {
        for (index, scope) in self.locals.iter().enumerate().rev() {
            if scope.contains_key(name) {
                return Some(index);
            }
        }
        if self.global.contains_key(name) {
            return Some(GLOBAL_SCOPE);
        }
        None
    }

    fn current_scope(&self) -> usize {
        self.locals.len().checked_sub(1).unwrap_or(GLOBAL_SCOPE)
    }

    /// Insere o símbolo no escopo atual.
    ///
    /// Retorna `Ok(true)` se foi inserido, `Ok(false)` quando oculta um símbolo
    /// externo (o chamador emite aviso) ou `Err(existing)` em redeclaração no
    /// mesmo escopo.
    pub fn insert(&mut self, symbol: Symbol) -> Result<bool, Symbol> {
        let existing_scope = self.find_scope(&symbol.name);

        if let Some(scope) = existing_scope
            && scope == self.current_scope()
        {
            return Err(symbol);
        }

        let hidden = existing_scope.is_some();

        if let Some(scope) = self.locals.last_mut() {
            scope.insert(symbol.name.clone(), symbol);
        } else {
            self.global.insert(symbol.name.clone(), symbol);
        }

        Ok(hidden)
    }

    /// Insere sem verificar colisões. Usado apenas para o pré-registro de
    /// funções, que precisa preservar a primeira declaração.
    pub fn insert_function(&mut self, symbol: Symbol) -> Option<Symbol> {
        self.global.insert(symbol.name.clone(), symbol)
    }
}
