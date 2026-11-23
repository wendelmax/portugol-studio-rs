//! HIR tipada e independente de sintaxe e backend.
//!
//! Spans usam offsets UTF-8 em bytes e intervalos semiabertos, iguais ao contrato
//! do parser existente, sem depender de `portugol-syntax`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Type {
    Unit,
    Integer,
    Real,
    Boolean,
    String,
    Character,
    VectorInteger,
    VectorReal,
    VectorBoolean,
    VectorString,
    VectorCharacter,
    Record(TypeId),
    VectorRecord(TypeId),
    Enum(TypeId),
    VectorEnum(TypeId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TypeId(pub u32);

#[derive(Debug, Clone, PartialEq)]
pub struct RecordDef {
    pub id: TypeId,
    pub name: String,
    pub fields: Vec<RecordField>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RecordField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub id: TypeId,
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumVariant {
    pub name: String,
    pub payload_types: Vec<Type>,
    pub span: Span,
}

impl Type {
    pub const fn vector_element(self) -> Option<Self> {
        match self {
            Self::VectorInteger => Some(Self::Integer),
            Self::VectorReal => Some(Self::Real),
            Self::VectorBoolean => Some(Self::Boolean),
            Self::VectorString => Some(Self::String),
            Self::VectorCharacter => Some(Self::Character),
            Self::VectorRecord(id) => Some(Self::Record(id)),
            Self::VectorEnum(id) => Some(Self::Enum(id)),
            _ => None,
        }
    }

    pub const fn vector_of(element: Self) -> Option<Self> {
        match element {
            Self::Integer => Some(Self::VectorInteger),
            Self::Real => Some(Self::VectorReal),
            Self::Boolean => Some(Self::VectorBoolean),
            Self::String => Some(Self::VectorString),
            Self::Character => Some(Self::VectorCharacter),
            Self::Record(id) => Some(Self::VectorRecord(id)),
            Self::Enum(id) => Some(Self::VectorEnum(id)),
            _ => None,
        }
    }

    pub const fn is_vector(self) -> bool {
        self.vector_element().is_some()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BindingId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FunctionId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mutability {
    Immutable,
    Mutable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Negate,
    Positive,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    And,
    Or,
    Xor,
    Equal,
    NotEqual,
    Greater,
    Less,
    LessEqual,
    GreaterEqual,
    ShiftLeft,
    ShiftRight,
    BitAnd,
    BitOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Xor,
    BitAnd,
    BitOr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopControl {
    Break,
    Continue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Program {
    pub record_types: Vec<RecordDef>,
    pub enum_types: Vec<EnumDef>,
    pub bindings: Vec<Binding>,
    pub body: Vec<Stmt>,
    /// Funções auxiliares. `body` representa o corpo da única função `inicio`.
    pub functions: Vec<Function>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub id: FunctionId,
    pub source_name: String,
    pub parameters: Vec<BindingId>,
    /// Inclui parâmetros nos primeiros slots e declarações locais depois deles.
    pub bindings: Vec<Binding>,
    pub return_type: Option<Type>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    pub id: BindingId,
    pub source_name: String,
    pub ty: Type,
    pub mutability: Mutability,
    pub is_const: bool,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Stmt {
    pub kind: StmtKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StmtKind {
    Declare {
        binding: BindingId,
        initializer: Option<Expr>,
    },
    Assign {
        binding: BindingId,
        op: AssignOp,
        value: Expr,
    },
    FieldAssign {
        binding: BindingId,
        field_index: usize,
        op: AssignOp,
        value: Expr,
    },
    Choose {
        value: Expr,
        arms: Vec<ChooseArm>,
    },
    Read {
        binding: BindingId,
        name_span: Span,
    },
    Call {
        function: FunctionId,
        arguments: Vec<Expr>,
    },
    Write(Vec<Expr>),
    If {
        condition: Expr,
        then_branch: Vec<Stmt>,
        else_branch: Option<Vec<Stmt>>,
    },
    While {
        condition: Expr,
        body: Vec<Stmt>,
    },
    For {
        initializer: Box<Stmt>,
        condition: Expr,
        step: Box<Stmt>,
        body: Vec<Stmt>,
    },
    LoopControl {
        control: LoopControl,
    },
    Return {
        value: Option<Expr>,
    },
    IndexAssign {
        binding: BindingId,
        index: Expr,
        value: Expr,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChooseArm {
    pub variant_index: usize,
    pub payload_bindings: Vec<BindingId>,
    pub body: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expr {
    pub kind: ExprKind,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Binding(BindingId),
    Call {
        function: FunctionId,
        arguments: Vec<Expr>,
    },
    RecordLiteral {
        type_id: TypeId,
        fields: Vec<Expr>,
    },
    Field {
        value: Box<Expr>,
        field_index: usize,
    },
    EnumConstructor {
        type_id: TypeId,
        variant_index: usize,
        payload: Vec<Expr>,
    },
    VectorLiteral(Vec<Expr>),
    Index {
        binding: BindingId,
        index: Box<Expr>,
    },
    Length {
        binding: BindingId,
    },
    Integer(i32),
    Real(f64),
    Boolean(bool),
    String(String),
    Character(char),
    Parenthesized(Box<Expr>),
    Unary {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    Binary {
        op: BinaryOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

#[cfg(test)]
mod tests;
