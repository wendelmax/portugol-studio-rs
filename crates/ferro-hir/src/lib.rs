//! HIR tipada, spans e diagnósticos compartilhados entre frontend, VM e geradores.
//!
//! Spans usam offsets UTF-8 em bytes e intervalos semiabertos `[start, end)`.

pub mod diagnostic;

pub use diagnostic::{Diagnostic, Severity};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Span {
    pub start: u32,
    pub end: u32,
}

impl Span {
    pub const fn new(start: u32, end: u32) -> Self {
        Self { start, end }
    }

    pub fn join(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }

    pub fn len(self) -> u32 {
        self.end.saturating_sub(self.start)
    }

    pub fn is_empty(self) -> bool {
        self.start >= self.end
    }
}

/// Converte um offset em bytes para `(linha, coluna)`, ambos baseados em 1.
pub fn line_col(source: &str, offset: u32) -> (usize, usize) {
    let offset = (offset as usize).min(source.len());
    let mut line = 1;
    let mut column = 1;

    for (idx, ch) in source.char_indices() {
        if idx >= offset {
            break;
        }
        if ch == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }

    (line, column)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BuiltinEnum {
    Option(Type),
    Result { ok: Type, err: Type },
}

#[derive(Debug, Clone, PartialEq)]
pub struct EnumDef {
    pub id: TypeId,
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub builtin: Option<BuiltinEnum>,
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

    pub const fn is_implicit_read_target(self) -> bool {
        matches!(
            self,
            Self::Integer | Self::Real | Self::Boolean | Self::Character
        )
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
    /// Expressão final sem `;`, interpretada como retorno da função.
    pub tail_expression: Option<Expr>,
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
    /// `p.x = ...` ou, em registros aninhados, `t.dados.mensagem = ...`:
    /// `fields` é o caminho de índices de campo, nunca vazio.
    FieldAssign {
        binding: BindingId,
        fields: Vec<usize>,
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
    HostCall {
        module: String,
        name: String,
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
    DoWhile {
        body: Vec<Stmt>,
        condition: Expr,
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
    VectorPush {
        binding: BindingId,
        value: Expr,
    },
    VectorPop {
        binding: BindingId,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct ChooseArm {
    /// Índice da variante do enum, ou `None` para o ramo coringa (`caso _ =>`).
    pub variant_index: Option<usize>,
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

/// Uma parte literal ou expressiva de uma cadeia interpolada. O `Span` da
/// parte cobre o texto literal decodificado ou `{...}`, incluindo as chaves.
#[derive(Debug, Clone, PartialEq)]
pub enum StringPart {
    Literal(String, Span),
    Expression(Expr, Span),
}

#[derive(Debug, Clone, PartialEq)]
pub enum ExprKind {
    Binding(BindingId),
    Call {
        function: FunctionId,
        arguments: Vec<Expr>,
    },
    HostCall {
        module: String,
        name: String,
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
    VectorPop {
        binding: BindingId,
    },
    VectorGet {
        binding: BindingId,
        index: Box<Expr>,
    },
    Integer(i32),
    Real(f64),
    Boolean(bool),
    String(String),
    /// Leitura da entrada usada como expressão (`leia()`), convertida ao tipo esperado quando conhecido.
    Read {
        target: Option<Type>,
    },
    InterpolatedString {
        parts: Vec<StringPart>,
    },
    Character(char),
    Parenthesized(Box<Expr>),
    Try {
        operand: Box<Expr>,
    },
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

/// Forma de um tipo, com os nomes e os campos dos registros e enums
/// (`Conta{nome:cadeia,saldo:real}`). Duas telas compiladas à parte trocam
/// dados (ADR 0018, Emenda 5) quando a forma dos tipos é a mesma.
pub fn type_shape(ty: Type, records: &[RecordDef], enums: &[EnumDef]) -> String {
    fn go(ty: Type, records: &[RecordDef], enums: &[EnumDef], depth: usize, out: &mut String) {
        let record = |id: TypeId| records.iter().find(|r| r.id == id);
        let enumeration = |id: TypeId| enums.iter().find(|e| e.id == id);
        match ty {
            Type::Unit => out.push_str("vazio"),
            Type::Integer => out.push_str("inteiro"),
            Type::Real => out.push_str("real"),
            Type::Boolean => out.push_str("logico"),
            Type::String => out.push_str("cadeia"),
            Type::Character => out.push_str("caracter"),
            Type::VectorInteger => out.push_str("vetor<inteiro>"),
            Type::VectorReal => out.push_str("vetor<real>"),
            Type::VectorBoolean => out.push_str("vetor<logico>"),
            Type::VectorString => out.push_str("vetor<cadeia>"),
            Type::VectorCharacter => out.push_str("vetor<caracter>"),
            Type::VectorRecord(id) => {
                out.push_str("vetor<");
                go(Type::Record(id), records, enums, depth, out);
                out.push('>');
            }
            Type::VectorEnum(id) => {
                out.push_str("vetor<");
                go(Type::Enum(id), records, enums, depth, out);
                out.push('>');
            }
            Type::Record(id) => match record(id) {
                Some(definition) => {
                    out.push_str(&definition.name);
                    if depth < 16 {
                        out.push('{');
                        for (index, field) in definition.fields.iter().enumerate() {
                            if index > 0 {
                                out.push(',');
                            }
                            out.push_str(&field.name);
                            out.push(':');
                            go(field.ty, records, enums, depth + 1, out);
                        }
                        out.push('}');
                    }
                }
                None => out.push('?'),
            },
            Type::Enum(id) => match enumeration(id) {
                Some(definition) => {
                    out.push_str(&definition.name);
                    if depth < 16 {
                        out.push('{');
                        for (index, variant) in definition.variants.iter().enumerate() {
                            if index > 0 {
                                out.push(',');
                            }
                            out.push_str(&variant.name);
                            if !variant.payload_types.is_empty() {
                                out.push('(');
                                for (i, payload) in variant.payload_types.iter().enumerate() {
                                    if i > 0 {
                                        out.push(',');
                                    }
                                    go(*payload, records, enums, depth + 1, out);
                                }
                                out.push(')');
                            }
                        }
                        out.push('}');
                    }
                }
                None => out.push('?'),
            },
        }
    }
    let mut out = String::new();
    go(ty, records, enums, 0, &mut out);
    out
}
