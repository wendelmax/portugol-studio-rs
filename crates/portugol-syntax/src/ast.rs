//! Árvore sintática abstrata do Portugol 2.0.

use crate::span::Span;

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub includes: Vec<Include>,
    pub items: Vec<Item>,
    pub span: Span,
    /// Indica se o arquivo declarou explicitamente um bloco `programa { }`.
    /// Trechos soltos (extensão aceita pelo parser) não precisam de `inicio`.
    pub has_program_block: bool,
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Item {
    Function(Function),
    Declarations(Declarations),
    /// Comando solto no nível do arquivo. Existe para acomodar os trechos de
    /// exemplo do Portugol Studio, que nem sempre estão dentro de `programa`.
    Stmt(Stmt),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Include {
    pub library: Ident,
    pub alias: Option<Ident>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub return_type: Option<Type>,
    pub name: Ident,
    pub params: Vec<Param>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub ty: Type,
    pub by_ref: bool,
    pub name: Ident,
    /// 0 = escalar, 1 = vetor, 2 = matriz.
    pub dims: u8,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Declarations {
    pub is_const: bool,
    pub ty: Type,
    pub declarators: Vec<Declarator>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Declarator {
    Variable {
        name: Ident,
        init: Option<Expr>,
        span: Span,
    },
    Array {
        name: Ident,
        size: Option<Box<Expr>>,
        init: Option<Vec<Expr>>,
        span: Span,
    },
    Matrix {
        name: Ident,
        rows: Option<Box<Expr>>,
        cols: Option<Box<Expr>>,
        init: Option<Vec<Vec<Expr>>>,
        span: Span,
    },
}

impl Declarator {
    pub fn name(&self) -> &Ident {
        match self {
            Declarator::Variable { name, .. }
            | Declarator::Array { name, .. }
            | Declarator::Matrix { name, .. } => name,
        }
    }

    pub fn span(&self) -> Span {
        match self {
            Declarator::Variable { span, .. }
            | Declarator::Array { span, .. }
            | Declarator::Matrix { span, .. } => *span,
        }
    }
}

#[allow(clippy::large_enum_variant)]
#[derive(Debug, Clone, PartialEq)]
pub enum Stmt {
    Declarations(Declarations),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
        span: Span,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
        span: Span,
    },
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
        span: Span,
    },
    For {
        init: Option<ForInit>,
        condition: Option<Expr>,
        step: Option<ForStep>,
        body: Vec<Stmt>,
        span: Span,
    },
    Switch {
        value: Expr,
        cases: Vec<Case>,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    Break {
        span: Span,
    },
    Assign(Assignment),
    Expr {
        expr: Expr,
        span: Span,
    },
}

impl Stmt {
    pub fn span(&self) -> Span {
        match self {
            Stmt::Declarations(d) => d.span,
            Stmt::If { span, .. }
            | Stmt::While { span, .. }
            | Stmt::DoWhile { span, .. }
            | Stmt::For { span, .. }
            | Stmt::Switch { span, .. }
            | Stmt::Return { span, .. }
            | Stmt::Break { span }
            | Stmt::Expr { span, .. } => *span,
            Stmt::Assign(a) => a.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForInit {
    Declarations(Declarations),
    Assignment(Assignment),
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ForStep {
    Assignment(Assignment),
    Expression(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Assignment {
    pub target: Expr,
    pub op: AssignOp,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Xor,
    BitAnd,
    BitOr,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Case {
    /// `None` representa `caso contrario:`.
    pub value: Option<Expr>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ident {
    /// Escopo de biblioteca, como `Graficos` em `Graficos.carregar(...)`.
    pub scope: Option<String>,
    pub name: String,
    pub span: Span,
}

impl Ident {
    pub fn qualified_name(&self) -> String {
        match &self.scope {
            Some(scope) => format!("{}.{}", scope, self.name),
            None => self.name.clone(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Real,
    Inteiro,
    Vazio,
    Logico,
    Cadeia,
    Caracter,
}

impl Type {
    pub fn as_str(self) -> &'static str {
        match self {
            Type::Real => "real",
            Type::Inteiro => "inteiro",
            Type::Vazio => "vazio",
            Type::Logico => "logico",
            Type::Cadeia => "cadeia",
            Type::Caracter => "caracter",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Plus,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Mul,
    Div,
    Mod,
    Add,
    Sub,
    Eq,
    Ne,
    Gt,
    Lt,
    Le,
    Ge,
    And,
    Or,
    Xor,
    Shl,
    Shr,
    BitAnd,
    BitOr,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Ident(Ident),
    Int {
        value: i64,
        span: Span,
    },
    Real {
        value: f64,
        span: Span,
    },
    Bool {
        value: bool,
        span: Span,
    },
    Char {
        value: char,
        span: Span,
    },
    Str {
        value: String,
        span: Span,
    },
    Call {
        scope: Option<String>,
        name: String,
        args: Vec<Expr>,
        span: Span,
    },
    Index {
        scope: Option<String>,
        name: String,
        /// 1 índice = vetor, 2 índices = matriz.
        indices: Vec<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    IncDec {
        target: Box<Expr>,
        /// `1` para `++`, `-1` para `--`.
        delta: i32,
        prefix: bool,
        span: Span,
    },
    Paren {
        expr: Box<Expr>,
        span: Span,
    },
    /// Atribuição usada como expressão, aceita pela gramática dentro de listas
    /// de argumentos (`f(x += 1)`).
    Assign {
        target: Box<Expr>,
        op: AssignOp,
        value: Box<Expr>,
        span: Span,
    },
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Ident(i) => i.span,
            Expr::Int { span, .. }
            | Expr::Real { span, .. }
            | Expr::Bool { span, .. }
            | Expr::Char { span, .. }
            | Expr::Str { span, .. }
            | Expr::Call { span, .. }
            | Expr::Index { span, .. }
            | Expr::Unary { span, .. }
            | Expr::Binary { span, .. }
            | Expr::IncDec { span, .. }
            | Expr::Paren { span, .. }
            | Expr::Assign { span, .. } => *span,
        }
    }
}
