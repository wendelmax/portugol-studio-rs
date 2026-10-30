//! Tipos da linguagem e a tabela de compatibilidade portada do Portugol
//! Studio (`TabelaCompatibilidadeTiposPortugol`).
//!
//! `Unknown` representa um tipo que a análise não conseguiu determinar (por
//! exemplo, o retorno de uma função de biblioteca que ainda não é modelada).
//! Ele é compatível com tudo para evitar erros em cascata; a contrapartida é
//! que o analisador fica permissivo nessas expressões.

use portugol_syntax::ast::{BinOp, Type, UnaryOp};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Real,
    Inteiro,
    Vazio,
    Logico,
    Cadeia,
    Caracter,
    /// Tipo não determinado; nunca gera diagnóstico por si só.
    Unknown,
    /// Usado apenas pelo `escreva` (equivalente ao `TODOS` do original).
    Todos,
}

impl Ty {
    pub fn from_ast(ty: Type) -> Self {
        match ty {
            Type::Real => Ty::Real,
            Type::Inteiro => Ty::Inteiro,
            Type::Vazio => Ty::Vazio,
            Type::Logico => Ty::Logico,
            Type::Cadeia => Ty::Cadeia,
            Type::Caracter => Ty::Caracter,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Ty::Real => "real",
            Ty::Inteiro => "inteiro",
            Ty::Vazio => "vazio",
            Ty::Logico => "logico",
            Ty::Cadeia => "cadeia",
            Ty::Caracter => "caracter",
            Ty::Unknown => "desconhecido",
            Ty::Todos => "todos",
        }
    }

    pub fn is_numeric(self) -> bool {
        matches!(self, Ty::Inteiro | Ty::Real)
    }

    pub fn is_known(self) -> bool {
        !matches!(self, Ty::Unknown)
    }
}

/// Resultado de uma conversão implícita entre tipos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Compat {
    /// Tipos iguais ou promoção sem perda representável pela tabela.
    Ok(Ty),
    /// O valor será convertido (gera aviso).
    Convert {
        from: Ty,
        to: Ty,
    },
    Incompatible,
}

/// Compatibilidade de atribuição, passagem de parâmetro e retorno.
///
/// A tabela do original é a mesma para os três casos: apenas `inteiro` e
/// `real` conversam entre si, sempre com aviso de conversão.
pub fn compatible(target: Ty, value: Ty) -> Compat {
    if target == Ty::Unknown {
        return Compat::Ok(value);
    }
    if value == Ty::Unknown {
        return Compat::Ok(target);
    }

    if target == Ty::Todos {
        return if matches!(value, Ty::Vazio | Ty::Todos) {
            Compat::Incompatible
        } else {
            Compat::Ok(value)
        };
    }

    if matches!(target, Ty::Vazio | Ty::Todos) || matches!(value, Ty::Vazio | Ty::Todos) {
        return Compat::Incompatible;
    }

    if target == value {
        return Compat::Ok(target);
    }

    match (target, value) {
        (Ty::Inteiro, Ty::Real) | (Ty::Real, Ty::Inteiro) => Compat::Convert {
            from: value,
            to: target,
        },
        _ => Compat::Incompatible,
    }
}

/// Tipo resultante de uma operação binária.
pub fn binary_result(op: BinOp, left: Ty, right: Ty) -> Option<Ty> {
    if left == Ty::Unknown || right == Ty::Unknown {
        return Some(Ty::Unknown);
    }

    if matches!(left, Ty::Todos | Ty::Vazio) || matches!(right, Ty::Todos | Ty::Vazio) {
        return None;
    }

    match op {
        BinOp::Add => add_result(left, right),
        BinOp::Sub | BinOp::Mul | BinOp::Div => numeric_result(left, right),
        BinOp::Mod => {
            if left == Ty::Inteiro && right == Ty::Inteiro {
                Some(Ty::Inteiro)
            } else {
                None
            }
        }
        BinOp::Eq | BinOp::Ne => {
            if equals_compatible(left, right) {
                Some(Ty::Logico)
            } else {
                None
            }
        }
        BinOp::Gt | BinOp::Lt | BinOp::Le | BinOp::Ge => {
            if relational_compatible(left, right) {
                Some(Ty::Logico)
            } else {
                None
            }
        }
        BinOp::And | BinOp::Or => {
            if left == Ty::Logico && right == Ty::Logico {
                Some(Ty::Logico)
            } else {
                None
            }
        }
        // `^`, deslocamentos e `&`/`|` são operações bit a bit no original.
        BinOp::Xor | BinOp::Shl | BinOp::Shr | BinOp::BitAnd | BinOp::BitOr => {
            if left == Ty::Inteiro && right == Ty::Inteiro {
                Some(Ty::Inteiro)
            } else {
                None
            }
        }
    }
}

fn add_result(left: Ty, right: Ty) -> Option<Ty> {
    use Ty::*;

    if left == Cadeia {
        return if matches!(right, Cadeia | Caracter | Inteiro | Logico | Real) {
            Some(Cadeia)
        } else {
            None
        };
    }

    if right == Cadeia {
        return if matches!(left, Cadeia | Caracter | Inteiro | Logico | Real) {
            Some(Cadeia)
        } else {
            None
        };
    }

    match (left, right) {
        (Caracter, Caracter) => Some(Cadeia),
        (Inteiro, Inteiro) => Some(Inteiro),
        (Inteiro, Real) | (Real, Inteiro) | (Real, Real) => Some(Real),
        (Logico, Logico) => Some(Logico),
        _ => None,
    }
}

fn numeric_result(left: Ty, right: Ty) -> Option<Ty> {
    match (left, right) {
        (Ty::Inteiro, Ty::Inteiro) => Some(Ty::Inteiro),
        (Ty::Inteiro, Ty::Real) | (Ty::Real, Ty::Inteiro) | (Ty::Real, Ty::Real) => Some(Ty::Real),
        _ => None,
    }
}

fn equals_compatible(left: Ty, right: Ty) -> bool {
    use Ty::*;
    matches!(
        (left, right),
        (Cadeia, Cadeia) | (Caracter, Caracter) | (Logico, Logico)
    ) || (left.is_numeric() && right.is_numeric())
}

fn relational_compatible(left: Ty, right: Ty) -> bool {
    use Ty::*;
    matches!((left, right), (Cadeia, Cadeia) | (Caracter, Caracter))
        || (left.is_numeric() && right.is_numeric())
}

/// Tipo resultante de uma operação unária.
pub fn unary_result(op: UnaryOp, operand: Ty) -> Option<Ty> {
    if operand == Ty::Unknown {
        return Some(Ty::Unknown);
    }

    match op {
        UnaryOp::Neg | UnaryOp::Plus => {
            if operand.is_numeric() {
                Some(operand)
            } else {
                None
            }
        }
        UnaryOp::Not => {
            if operand == Ty::Logico {
                Some(Ty::Logico)
            } else {
                None
            }
        }
        UnaryOp::BitNot => {
            if operand == Ty::Inteiro {
                Some(Ty::Inteiro)
            } else {
                None
            }
        }
    }
}

/// Operação binária correspondente a uma atribuição composta.
pub fn compound_op(op: portugol_syntax::ast::AssignOp) -> Option<BinOp> {
    use portugol_syntax::ast::AssignOp;
    match op {
        AssignOp::Assign => None,
        AssignOp::Add => Some(BinOp::Add),
        AssignOp::Sub => Some(BinOp::Sub),
        AssignOp::Mul => Some(BinOp::Mul),
        AssignOp::Div => Some(BinOp::Div),
        AssignOp::Mod => Some(BinOp::Mod),
        AssignOp::Xor => Some(BinOp::Xor),
        AssignOp::BitAnd => Some(BinOp::BitAnd),
        AssignOp::BitOr => Some(BinOp::BitOr),
    }
}
