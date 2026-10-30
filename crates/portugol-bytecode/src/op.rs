//! Conjunto de instruções da máquina virtual.
//!
//! A VM é de pilha. Cada função tem um quadro com `locals` slots no início
//! (os primeiros são os parâmetros) e a pilha de operandos logo acima.
//! Comentários no formato `[a, b] -> [c]` descrevem o topo da pilha antes e
//! depois da instrução (o último elemento é o topo).

use portugol_syntax::Span;
use portugol_syntax::ast::Type;

/// Tipo primitivo, usado em conversões, valores padrão e `leia`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Inteiro,
    Real,
    Logico,
    Cadeia,
    Caracter,
    Vazio,
}

impl From<Type> for ValueType {
    fn from(ty: Type) -> Self {
        match ty {
            Type::Inteiro => ValueType::Inteiro,
            Type::Real => ValueType::Real,
            Type::Logico => ValueType::Logico,
            Type::Cadeia => ValueType::Cadeia,
            Type::Caracter => ValueType::Caracter,
            Type::Vazio => ValueType::Vazio,
        }
    }
}

impl ValueType {
    pub fn name(self) -> &'static str {
        match self {
            ValueType::Inteiro => "inteiro",
            ValueType::Real => "real",
            ValueType::Logico => "logico",
            ValueType::Cadeia => "cadeia",
            ValueType::Caracter => "caracter",
            ValueType::Vazio => "vazio",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Op {
    // --- constantes --------------------------------------------------------
    PushInt(i32),
    PushReal(f64),
    PushBool(bool),
    PushChar(char),
    /// Índice em [`Module::strings`].
    PushStr(u32),
    PushVoid,
    /// `[]` -> `[valor padrão do tipo]`.
    PushDefault(ValueType),
    /// `[a]` -> `[]`.
    Pop,
    /// `[a]` -> `[a, a]`.
    Dup,

    // --- variáveis ---------------------------------------------------------
    LoadLocal(u32),
    StoreLocal(u32),
    LoadGlobal(u32),
    StoreGlobal(u32),
    /// Lê através da referência guardada no slot local (parâmetro `&`).
    LoadDeref(u32),
    /// `[x]` -> `[]`, escrevendo através da referência do slot local.
    StoreDeref(u32),
    /// `[]` -> `[ref]` para o slot local.
    RefLocal(u32),
    /// `[]` -> `[ref]` para a global.
    RefGlobal(u32),
    /// `[vetor, i]` -> `[ref]`.
    RefElem,
    /// `[matriz, i, j]` -> `[ref]`.
    RefMatElem,

    // --- vetores e matrizes ------------------------------------------------
    /// `[n]` -> `[vetor]` preenchido com o valor padrão do tipo.
    NewArray(ValueType),
    /// `[linhas, colunas]` -> `[matriz]`.
    NewMatrix(ValueType),
    /// `[v0, ..., vn-1]` -> `[vetor]`.
    ArrayLit(u32),
    /// `[v00, v01, ..., v(r-1)(c-1)]` -> `[matriz]`, em ordem de linhas.
    MatrixLit {
        rows: u32,
        cols: u32,
    },
    /// `[vetor, i]` -> `[x]`.
    LoadElem,
    /// `[x, vetor, i]` -> `[]`.
    StoreElem,
    /// `[matriz, i, j]` -> `[x]`.
    LoadMatElem,
    /// `[x, matriz, i, j]` -> `[]`.
    StoreMatElem,

    // --- operadores --------------------------------------------------------
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Neg,
    Not,
    BitNot,
    BitAnd,
    BitOr,
    /// `^`: bit a bit em inteiros, ou-exclusivo em lógicos.
    BitXor,
    Shl,
    Shr,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    /// Conversão implícita para o tipo de destino (atribuição, parâmetro,
    /// retorno). `real -> inteiro` trunca como o cast `(int)` do Java.
    Convert(ValueType),

    // --- fluxo -------------------------------------------------------------
    Jump(u32),
    /// `[c]` -> `[]`.
    JumpIfFalse(u32),
    /// `[c]` -> `[]`.
    JumpIfTrue(u32),
    /// Curto-circuito do `e`: se falso, mantém o valor e salta; senão
    /// descarta e segue.
    JumpIfFalseKeep(u32),
    /// Curto-circuito do `ou`.
    JumpIfTrueKeep(u32),
    /// `[a0, ..., an-1]` -> `[retorno]`.
    Call {
        function: u32,
        argc: u32,
    },
    /// Índice em [`Module::natives`].
    CallNative {
        native: u32,
        argc: u32,
    },
    /// Constante de biblioteca (`Matematica.PI`); índice em
    /// [`Module::native_consts`].
    LoadNativeConst(u32),
    /// `[x]` -> retorna `x` ao chamador.
    Return,
    /// `[]` -> `[vetor de cadeias]` com os parâmetros de execução, passados a
    /// `inicio(cadeia params[])`.
    ProgramArgs,

    // --- entrada e saída ---------------------------------------------------
    /// `[a0, ..., an-1]` -> `[]`.
    Write(u32),
    /// `[]` -> `[valor lido]`.
    Read(ValueType),
    Clear,
    /// `[min, max]` -> `[sorteado]`.
    Random,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalInfo {
    /// Vazio para temporários criados pelo compilador.
    pub name: String,
    pub ty: Type,
    pub dims: u8,
    pub by_ref: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub arity: u32,
    pub return_type: ValueType,
    /// Total de slots locais, parâmetros incluídos.
    pub locals: Vec<LocalInfo>,
    pub code: Vec<Op>,
    /// Mapa de fonte: `spans[i]` é o trecho que gerou `code[i]`.
    pub spans: Vec<Span>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct NativeRef {
    pub library: String,
    pub name: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct GlobalInfo {
    pub name: String,
    pub ty: Type,
    pub dims: u8,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Module {
    pub functions: Vec<Function>,
    /// Função sintética que inicializa as globais, executa comandos soltos
    /// e chama `inicio`.
    pub entry: u32,
    pub globals: Vec<GlobalInfo>,
    pub strings: Vec<String>,
    pub natives: Vec<NativeRef>,
    pub native_consts: Vec<NativeRef>,
}

impl Module {
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
}
