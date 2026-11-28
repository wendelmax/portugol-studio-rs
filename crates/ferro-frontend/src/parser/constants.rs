use ferro_hir as hir;
use ferro_hir::{Diagnostic, Span};

use crate::lexer::Kind;

pub(super) fn is_top_level_start(kind: &Kind) -> bool {
    matches!(kind, Kind::Ident(keyword) if matches!(keyword.as_str(), "funcao" | "registro" | "enum"))
}

pub(super) fn can_start_expression(kind: &Kind) -> bool {
    matches!(
        kind,
        Kind::Ident(_)
            | Kind::Integer(_)
            | Kind::Real(_)
            | Kind::String { .. }
            | Kind::Character(_)
            | Kind::LParen
            | Kind::LBracket
            | Kind::Bang
            | Kind::Tilde
            | Kind::Minus
            | Kind::Plus
    )
}

pub(super) fn is_recoverable_syntax(code: &str) -> bool {
    matches!(
        code,
        "ferro.expected-expression"
            | "ferro.expected-identifier"
            | "ferro.expected-keyword"
            | "ferro.expected-statement"
            | "ferro.unexpected-token"
    )
}

pub(super) fn append_unique(target: &mut Vec<Diagnostic>, incoming: Vec<Diagnostic>) {
    for diagnostic in incoming {
        if !target.contains(&diagnostic) {
            target.push(diagnostic);
        }
    }
}

pub(super) fn calendario_constant(name: &str) -> Option<i32> {
    const DIAS: [&str; 7] = [
        "DIA_DOMINGO",
        "DIA_SEGUNDA_FEIRA",
        "DIA_TERCA_FEIRA",
        "DIA_QUARTA_FEIRA",
        "DIA_QUINTA_FEIRA",
        "DIA_SEXTA_FEIRA",
        "DIA_SABADO",
    ];
    const MESES: [&str; 12] = [
        "MES_JANEIRO",
        "MES_FEVEREIRO",
        "MES_MARCO",
        "MES_ABRIL",
        "MES_MAIO",
        "MES_JUNHO",
        "MES_JULHO",
        "MES_AGOSTO",
        "MES_SETEMBRO",
        "MES_OUTUBRO",
        "MES_NOVEMBRO",
        "MES_DEZEMBRO",
    ];
    DIAS.iter()
        .position(|day| *day == name)
        .or_else(|| MESES.iter().position(|month| *month == name))
        .map(|index| index as i32 + 1)
}

pub(super) fn graficos_constant(name: &str) -> Option<i32> {
    Some(match name {
        "COR_PRETO" => 0xFF00_0000_u32 as i32,
        "COR_BRANCO" => 0xFFFF_FFFF_u32 as i32,
        "COR_AZUL" => 0xFF00_00FF_u32 as i32,
        "COR_VERMELHO" => 0xFFFF_0000_u32 as i32,
        "COR_VERDE" => 0xFF00_FF00_u32 as i32,
        "COR_AMARELO" => 0xFFFF_FF00_u32 as i32,
        "GRADIENTE_DIREITA" => 0,
        "GRADIENTE_ESQUERDA" => 1,
        "GRADIENTE_ACIMA" => 2,
        "GRADIENTE_ABAIXO" => 3,
        "GRADIENTE_INFERIOR_DIREITO" => 4,
        "GRADIENTE_INFERIOR_ESQUERDO" => 5,
        "GRADIENTE_SUPERIOR_DIREITO" => 6,
        "GRADIENTE_SUPERIOR_ESQUERDO" => 7,
        "CANAL_R" => 0,
        "CANAL_G" => 1,
        "CANAL_B" => 2,
        _ => return None,
    })
}

pub(super) fn teclado_constant(name: &str) -> Option<i32> {
    if let Some(letter) = name
        .strip_prefix("TECLA_")
        .filter(|rest| rest.len() == 1 && rest.as_bytes()[0].is_ascii_uppercase())
    {
        return Some(letter.as_bytes()[0] as i32);
    }
    let code = match name {
        "TECLA_ENTER" => 10,
        "TECLA_BACKSPACE" => 8,
        "TECLA_TAB" => 9,
        "TECLA_CANCELAR" => 3,
        "TECLA_LIMPAR" => 12,
        "TECLA_SHIFT" => 16,
        "TECLA_CONTROL" => 17,
        "TECLA_ALT" => 18,
        "TECLA_PAUSE" => 19,
        "TECLA_ESC" => 27,
        "TECLA_ESPACO" => 32,
        "TECLA_PAGE_UP" => 33,
        "TECLA_PAGE_DOWN" => 34,
        "TECLA_END" => 35,
        "TECLA_HOME" => 36,
        "TECLA_SETA_ESQUERDA" => 37,
        "TECLA_SETA_ACIMA" => 38,
        "TECLA_SETA_DIREITA" => 39,
        "TECLA_SETA_ABAIXO" => 40,
        "TECLA_VIRGULA" => 44,
        "TECLA_MENOS" => 45,
        "TECLA_PONTO_FINAL" => 46,
        "TECLA_BARRA" => 47,
        "TECLA_PONTO_VIRGULA" => 59,
        "TECLA_IGUAL" => 61,
        "TECLA_ABRE_COLCHETES" => 91,
        "TECLA_BARRA_INVERTIDA" => 92,
        "TECLA_FECHA_COLCHETES" => 93,
        "TECLA_MULTIPLICACAO" => 106,
        "TECLA_ADICAO" => 107,
        "TECLA_NUM_SEPARADOR_DECIMAL" => 108,
        "TECLA_SUBTRACAO" => 109,
        "TECLA_DECIMAL" => 110,
        "TECLA_DIVISAO" => 111,
        "TECLA_DELETAR" => 127,
        "TECLA_NUM_LOCK" => 144,
        "TECLA_SCROLL_LOCK" => 145,
        "TECLA_PRINTSCREEN" => 154,
        "TECLA_INSERT" => 155,
        "TECLA_AJUDA" => 156,
        "TECLA_BACK_QUOTE" => 192,
        "TECLA_QUOTE" => 222,
        "TECLA_WINDOWS" => 524,
        "TECLA_MENU_CONTEXTO" => 525,
        "TECLA_CAPS_LOCK" => 20,
        "TECLA_META" => 157,
        "TECLA_AMPERSAND" => 150,
        "TECLA_ASTERISK" => 151,
        "TECLA_QUOTEDBL" => 152,
        "TECLA_LESS" => 153,
        "TECLA_GREATER" => 160,
        "TECLA_BRACELEFT" => 161,
        "TECLA_BRACERIGHT" => 162,
        "TECLA_AT" => 512,
        "TECLA_COLON" => 513,
        "TECLA_CIRCUMFLEX" => 514,
        "TECLA_DOLLAR" => 515,
        "TECLA_EURO_SIGN" => 516,
        "TECLA_EXCLAMATION_MARK" => 517,
        "TECLA_INVERTED_EXCLAMATION_MARK" => 518,
        "TECLA_LEFT_PARENTHESIS" => 519,
        "TECLA_NUMBER_SIGN" => 520,
        "TECLA_PLUS" => 521,
        "TECLA_RIGHT_PARENTHESIS" => 522,
        "TECLA_UNDERSCORE" => 523,
        _ => {
            let rest = name.strip_prefix("TECLA_")?;
            if let Some(digit) = rest
                .strip_suffix("_NUM")
                .or_else(|| rest.strip_prefix("NUM_"))
            {
                return digit
                    .parse::<i32>()
                    .ok()
                    .filter(|d| (0..=9).contains(d))
                    .map(|d| 96 + d);
            }
            if let Some(f) = rest.strip_prefix('F') {
                return f.parse::<i32>().ok().and_then(|n| match n {
                    1..=12 => Some(111 + n),
                    13..=24 => Some(61_427 + n),
                    _ => None,
                });
            }
            return rest
                .parse::<i32>()
                .ok()
                .filter(|d| (0..=9).contains(d))
                .map(|d| 48 + d);
        }
    };
    Some(code)
}

pub(super) fn mouse_constant(name: &str) -> Option<i32> {
    match name {
        "BOTAO_ESQUERDO" => Some(0),
        "BOTAO_DIREITO" => Some(1),
        "BOTAO_MEIO" => Some(2),
        _ => None,
    }
}

pub(super) fn to_hir_span(value: Span) -> hir::Span {
    hir::Span::new(value.start, value.end)
}
pub(super) fn scalar_type(name: &str) -> Option<hir::Type> {
    Some(match name {
        "inteiro" => hir::Type::Integer,
        "real" => hir::Type::Real,
        "logico" => hir::Type::Boolean,
        "cadeia" => hir::Type::String,
        "caracter" => hir::Type::Character,
        _ => return None,
    })
}
pub(super) fn type_label(ty: hir::Type) -> &'static str {
    match ty {
        hir::Type::Unit => "vazio",
        hir::Type::Integer => "inteiro",
        hir::Type::Real => "real",
        hir::Type::Boolean => "logico",
        hir::Type::String => "cadeia",
        hir::Type::Character => "caracter",
        hir::Type::VectorInteger => "vetor<inteiro>",
        hir::Type::VectorReal => "vetor<real>",
        hir::Type::VectorBoolean => "vetor<logico>",
        hir::Type::VectorString => "vetor<cadeia>",
        hir::Type::VectorCharacter => "vetor<caracter>",
        hir::Type::Record(_) => "registro",
        hir::Type::VectorRecord(_) => "vetor<registro>",
        hir::Type::Enum(_) => "enum",
        hir::Type::VectorEnum(_) => "vetor<enum>",
    }
}
pub(super) fn syntax_span(value: hir::Span) -> Span {
    Span::new(value.start, value.end)
}
pub(super) fn join_hir(a: hir::Span, b: hir::Span) -> hir::Span {
    hir::Span::new(a.start.min(b.start), a.end.max(b.end))
}

pub(super) fn is_reserved(name: &str) -> bool {
    matches!(
        name,
        "funcao"
            | "inicio"
            | "seja"
            | "mutavel"
            | "inteiro"
            | "real"
            | "logico"
            | "cadeia"
            | "caracter"
            | "vetor"
            | "tamanho"
            | "adicionar"
            | "remover"
            | "obter"
            | "verdadeiro"
            | "falso"
            | "escreva"
            | "se"
            | "senao"
            | "enquanto"
            | "para"
            | "pare"
            | "continue"
            | "faca"
            | "escolha"
            | "leia"
            | "retorne"
            | "registro"
            | "enum"
            | "caso"
            | "Opcao"
            | "Resultado"
    )
}

pub(super) fn guarantees_return(statements: &[hir::Stmt]) -> bool {
    statements.iter().any(|statement| match &statement.kind {
        hir::StmtKind::Return { .. } => true,
        hir::StmtKind::If {
            then_branch,
            else_branch: Some(else_branch),
            ..
        } => guarantees_return(then_branch) && guarantees_return(else_branch),
        hir::StmtKind::Choose { arms, .. } => {
            !arms.is_empty() && arms.iter().all(|arm| guarantees_return(&arm.body))
        }
        _ => false,
    })
}

pub(super) fn binary(kind: &Kind) -> Option<(hir::BinaryOp, u8)> {
    Some(match kind {
        Kind::Or => (hir::BinaryOp::Or, 1),
        Kind::And => (hir::BinaryOp::And, 2),
        Kind::Pipe => (hir::BinaryOp::BitOr, 3),
        Kind::Caret => (hir::BinaryOp::Xor, 4),
        Kind::Amp => (hir::BinaryOp::BitAnd, 5),
        Kind::Equal => (hir::BinaryOp::Equal, 6),
        Kind::NotEqual => (hir::BinaryOp::NotEqual, 6),
        Kind::Greater => (hir::BinaryOp::Greater, 7),
        Kind::Less => (hir::BinaryOp::Less, 7),
        Kind::GreaterEqual => (hir::BinaryOp::GreaterEqual, 7),
        Kind::LessEqual => (hir::BinaryOp::LessEqual, 7),
        Kind::Shl => (hir::BinaryOp::ShiftLeft, 8),
        Kind::Shr => (hir::BinaryOp::ShiftRight, 8),
        Kind::Plus => (hir::BinaryOp::Add, 9),
        Kind::Minus => (hir::BinaryOp::Subtract, 9),
        Kind::Star => (hir::BinaryOp::Multiply, 10),
        Kind::Slash => (hir::BinaryOp::Divide, 10),
        Kind::Percent => (hir::BinaryOp::Remainder, 10),
        _ => return None,
    })
}

pub(super) fn binary_type(
    op: hir::BinaryOp,
    left: hir::Type,
    right: hir::Type,
) -> Option<hir::Type> {
    use hir::Type::*;
    if left.is_vector() || right.is_vector() {
        return None;
    }
    Some(match op {
        hir::BinaryOp::Add
            if left == String && right == String || left == Character && right == Character =>
        {
            String
        }
        hir::BinaryOp::Add
        | hir::BinaryOp::Subtract
        | hir::BinaryOp::Multiply
        | hir::BinaryOp::Divide
            if left == right && matches!(left, Integer | Real) =>
        {
            left
        }
        hir::BinaryOp::Remainder
        | hir::BinaryOp::BitAnd
        | hir::BinaryOp::BitOr
        | hir::BinaryOp::Xor
        | hir::BinaryOp::ShiftLeft
        | hir::BinaryOp::ShiftRight
            if left == Integer && right == Integer =>
        {
            Integer
        }
        hir::BinaryOp::And | hir::BinaryOp::Or if left == Boolean && right == Boolean => Boolean,
        hir::BinaryOp::Equal | hir::BinaryOp::NotEqual
            if left == right && !matches!(left, Record(_) | Enum(_)) =>
        {
            Boolean
        }
        hir::BinaryOp::Greater
        | hir::BinaryOp::Less
        | hir::BinaryOp::GreaterEqual
        | hir::BinaryOp::LessEqual
            if left == right && matches!(left, Integer | Real | Character) =>
        {
            Boolean
        }
        _ => return None,
    })
}
