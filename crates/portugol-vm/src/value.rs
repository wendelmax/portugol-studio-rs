//! Valores em tempo de execução.

use std::cell::RefCell;
use std::fmt::Write as _;
use std::rc::Rc;

use portugol_bytecode::ValueType;

pub type ArrayRef = Rc<RefCell<Vec<Value>>>;
pub type MatrixRef = Rc<RefCell<Matrix>>;

#[derive(Debug, Clone)]
pub struct Matrix {
    pub rows: usize,
    pub cols: usize,
    /// Elementos em ordem de linhas.
    pub data: Vec<Value>,
}

/// Alvo de um parâmetro por referência (`&`).
#[derive(Debug, Clone)]
pub enum Reference {
    /// Slot absoluto na pilha da VM (local de um quadro abaixo do atual).
    Stack(usize),
    Global(usize),
    Elem(ArrayRef, usize),
    /// Índice já linearizado (`linha * colunas + coluna`).
    MatElem(MatrixRef, usize),
}

/// `inteiro` e `real` seguem os tipos `int` e `double` do Java gerado pelo
/// Portugol Studio: inteiros têm 32 bits e estouram com *wrap-around*.
/// Vetores e matrizes são compartilhados por referência, como os arrays Java.
#[derive(Debug, Clone)]
pub enum Value {
    Void,
    Int(i32),
    Real(f64),
    Bool(bool),
    Char(char),
    Str(Rc<str>),
    Array(ArrayRef),
    Matrix(MatrixRef),
    Ref(Rc<Reference>),
}

impl Value {
    pub fn default_for(ty: ValueType) -> Value {
        match ty {
            ValueType::Inteiro => Value::Int(0),
            ValueType::Real => Value::Real(0.0),
            ValueType::Logico => Value::Bool(false),
            ValueType::Caracter => Value::Char('\0'),
            // No Java um `String` não inicializado é `null`; o original trata
            // esses casos como cadeia vazia.
            ValueType::Cadeia => Value::Str(Rc::from("")),
            ValueType::Vazio => Value::Void,
        }
    }

    pub fn str(text: impl Into<Rc<str>>) -> Value {
        Value::Str(text.into())
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Void => "vazio",
            Value::Int(_) => "inteiro",
            Value::Real(_) => "real",
            Value::Bool(_) => "logico",
            Value::Char(_) => "caracter",
            Value::Str(_) => "cadeia",
            Value::Array(_) => "vetor",
            Value::Matrix(_) => "matriz",
            Value::Ref(_) => "referência",
        }
    }

    /// Texto exibido pelo `escreva`.
    pub fn display(&self) -> String {
        match self {
            Value::Bool(true) => "verdadeiro".to_string(),
            Value::Bool(false) => "falso".to_string(),
            other => other.concat_text(),
        }
    }

    /// Texto usado na concatenação com `+`. O original gera
    /// `StringBuilder.append(boolean)`, que produz `true`/`false` e não
    /// `verdadeiro`/`falso`; mantemos o mesmo comportamento.
    pub fn concat_text(&self) -> String {
        match self {
            Value::Void => String::new(),
            Value::Int(value) => value.to_string(),
            Value::Real(value) => java_double(*value),
            Value::Bool(value) => value.to_string(),
            Value::Char(value) => value.to_string(),
            Value::Str(value) => value.to_string(),
            Value::Array(_) => "[vetor]".to_string(),
            Value::Matrix(_) => "[matriz]".to_string(),
            Value::Ref(_) => "[referência]".to_string(),
        }
    }
}

/// Formata um `double` como `Double.toString` do Java: notação decimal entre
/// 10^-3 e 10^7, notação científica (`1.0E7`) fora disso, e sempre ao menos
/// uma casa decimal.
pub fn java_double(value: f64) -> String {
    if value.is_nan() {
        return "NaN".to_string();
    }
    if value.is_infinite() {
        return if value > 0.0 { "Infinity" } else { "-Infinity" }.to_string();
    }
    if value == 0.0 {
        return if value.is_sign_negative() {
            "-0.0"
        } else {
            "0.0"
        }
        .to_string();
    }

    // `{:e}` do Rust já dá os dígitos mais curtos que fazem ida e volta.
    let sci = format!("{:e}", value.abs());
    let (mantissa, exponent) = sci.split_once('e').expect("formato {:e}");
    let exponent: i32 = exponent.parse().expect("expoente");
    let digits: String = mantissa.chars().filter(|c| *c != '.').collect();

    let mut out = String::new();
    if value < 0.0 {
        out.push('-');
    }

    let magnitude = value.abs();
    if (1e-3..1e7).contains(&magnitude) {
        if exponent >= 0 {
            let int_len = exponent as usize + 1;
            if digits.len() <= int_len {
                out.push_str(&digits);
                out.extend(std::iter::repeat_n('0', int_len - digits.len()));
                out.push_str(".0");
            } else {
                out.push_str(&digits[..int_len]);
                out.push('.');
                out.push_str(&digits[int_len..]);
            }
        } else {
            out.push_str("0.");
            out.extend(std::iter::repeat_n('0', (-exponent - 1) as usize));
            out.push_str(&digits);
        }
    } else {
        out.push_str(&digits[..1]);
        out.push('.');
        if digits.len() > 1 {
            out.push_str(&digits[1..]);
        } else {
            out.push('0');
        }
        let _ = write!(out, "E{exponent}");
    }
    out
}

#[cfg(test)]
mod tests {
    use super::java_double;

    #[test]
    fn formata_como_java() {
        let casos = [
            (1.0, "1.0"),
            (-2.5, "-2.5"),
            (0.1, "0.1"),
            (0.001, "0.001"),
            (0.0001, "1.0E-4"),
            (1234567.0, "1234567.0"),
            (10_000_000.0, "1.0E7"),
            (12_345_678.9, "1.23456789E7"),
            (std::f64::consts::PI, "3.141592653589793"),
            (1.0 / 3.0, "0.3333333333333333"),
            (100.0, "100.0"),
            (1e21, "1.0E21"),
            (-0.0, "-0.0"),
            (f64::NAN, "NaN"),
            (f64::INFINITY, "Infinity"),
        ];
        for (valor, esperado) in casos {
            assert_eq!(java_double(valor), esperado, "{valor}");
        }
    }
}
