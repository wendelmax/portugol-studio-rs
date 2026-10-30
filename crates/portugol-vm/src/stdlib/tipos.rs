//! `Tipos.java`. Peculiaridades do original foram mantidas de propósito e
//! estão comentadas onde aparecem.

use super::{NativeContext, NativeFn, arity, boolean, character, int, real, text};
use crate::value::{Value, java_double};

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "cadeia_e_inteiro" => cadeia_e_inteiro,
        "cadeia_e_real" => cadeia_e_real,
        "cadeia_e_logico" => cadeia_e_logico,
        "cadeia_e_caracter" => cadeia_e_caracter,
        "cadeia_para_caracter" => cadeia_para_caracter,
        "cadeia_para_inteiro" => cadeia_para_inteiro,
        "cadeia_para_real" => cadeia_para_real,
        "cadeia_para_logico" => cadeia_para_logico,
        "inteiro_e_caracter" => inteiro_e_caracter,
        "inteiro_para_cadeia" => inteiro_para_cadeia,
        "inteiro_para_caracter" => inteiro_para_caracter,
        "inteiro_para_logico" => inteiro_para_logico,
        "inteiro_para_real" => inteiro_para_real,
        "caracter_e_inteiro" => caracter_e_inteiro,
        "caracter_e_logico" => caracter_e_logico,
        "caracter_para_cadeia" => caracter_para_cadeia,
        "caracter_para_inteiro" => caracter_para_inteiro,
        "caracter_para_logico" => caracter_para_logico,
        "logico_para_cadeia" => logico_para_cadeia,
        "logico_para_inteiro" => logico_para_inteiro,
        "logico_para_caracter" => logico_para_caracter,
        "real_para_cadeia" => real_para_cadeia,
        "real_para_inteiro" => real_para_inteiro,
        _ => return None,
    })
}

fn invalid_base(base: i32) -> String {
    format!(
        "A base informada ({base}) é inválida, a base deve ser um dos seguintes valores: 2; 10; 16"
    )
}

fn is_integer(cad: &str, base: i32) -> Result<bool, String> {
    Ok(match base {
        // ^(0b|0B)?[0-1]+$
        2 => {
            let digits = cad
                .strip_prefix("0b")
                .or_else(|| cad.strip_prefix("0B"))
                .unwrap_or(cad);
            !digits.is_empty() && digits.chars().all(|c| c == '0' || c == '1')
        }
        // ^-?\d+$
        10 => {
            let digits = cad.strip_prefix('-').unwrap_or(cad);
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit())
        }
        // ^(0x|0X)?([0-9]|[a-f]|[A-F])+$
        16 => {
            let digits = cad
                .strip_prefix("0x")
                .or_else(|| cad.strip_prefix("0X"))
                .unwrap_or(cad);
            !digits.is_empty() && digits.chars().all(|c| c.is_ascii_hexdigit())
        }
        _ => return Err(invalid_base(base)),
    })
}

fn parse_integer(valor: &str, base: i32) -> Result<i32, String> {
    if !matches!(base, 2 | 10 | 16) {
        return Err(invalid_base(base));
    }
    // `replaceFirst` do original: remove a primeira ocorrência em qualquer
    // posição, não só no início.
    let valor = match base {
        16 => valor.replacen("0X", "", 1).replacen("0x", "", 1),
        2 => valor.replacen("0B", "", 1).replacen("0b", "", 1),
        _ => valor.to_string(),
    };
    match i64::from_str_radix(&valor, base as u32) {
        // Valores acima de `int` dão a volta (permite "FFFFFFFF" = -1).
        Ok(value) => Ok(value as i32),
        Err(_) => Err(format!("o valor '{valor}' não é um número inteiro válido")),
    }
}

fn cadeia_e_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "cadeia_e_inteiro")?;
    Ok(Value::Bool(is_integer(text(&args[0])?, int(&args[1])?)?))
}

/// `^-?\d+\.\d+$`
fn cadeia_e_real(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_e_real")?;
    let cad = text(&args[0])?;
    let unsigned = cad.strip_prefix('-').unwrap_or(cad);
    let ok = unsigned.split_once('.').is_some_and(|(int, frac)| {
        !int.is_empty()
            && !frac.is_empty()
            && int.chars().all(|c| c.is_ascii_digit())
            && frac.chars().all(|c| c.is_ascii_digit())
    });
    Ok(Value::Bool(ok))
}

/// O padrão original é `^verdadeiro|falso$`, que por precedência significa
/// "começa com verdadeiro" ou "termina com falso".
fn cadeia_e_logico(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_e_logico")?;
    let cad = text(&args[0])?;
    Ok(Value::Bool(
        cad.starts_with("verdadeiro") || cad.ends_with("falso"),
    ))
}

fn cadeia_e_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_e_caracter")?;
    Ok(Value::Bool(text(&args[0])?.chars().count() == 1))
}

fn cadeia_para_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_para_caracter")?;
    let valor = text(&args[0])?;
    let mut chars = valor.chars();
    match (chars.next(), chars.next()) {
        (Some(c), None) => Ok(Value::Char(c)),
        _ => Err(format!("o valor '{valor}' não é um caracter válido")),
    }
}

fn cadeia_para_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "cadeia_para_inteiro")?;
    Ok(Value::Int(parse_integer(text(&args[0])?, int(&args[1])?)?))
}

fn cadeia_para_real(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_para_real")?;
    let valor = text(&args[0])?;
    valor
        .trim()
        .parse()
        .map(Value::Real)
        .map_err(|_| format!("o valor '{valor}' não é um número real válido"))
}

fn cadeia_para_logico(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cadeia_para_logico")?;
    match text(&args[0])? {
        "verdadeiro" => Ok(Value::Bool(true)),
        "falso" => Ok(Value::Bool(false)),
        valor => Err(format!("o valor '{valor}' não é um valor lógico válido")),
    }
}

/// O original considera "caracter" apenas os dígitos 0 a 9.
fn inteiro_e_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "inteiro_e_caracter")?;
    Ok(Value::Bool((0..=9).contains(&int(&args[0])?)))
}

fn inteiro_para_cadeia(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "inteiro_para_cadeia")?;
    let valor = int(&args[0])?;
    let text = match int(&args[1])? {
        2 => format!("{:032b}", valor as u32),
        10 => valor.to_string(),
        16 => format!("{:08X}", valor as u32),
        base => return Err(invalid_base(base)),
    };
    Ok(Value::str(text))
}

fn inteiro_para_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "inteiro_para_caracter")?;
    let valor = int(&args[0])?;
    if (0..=9).contains(&valor) {
        Ok(Value::Char(char::from(b'0' + valor as u8)))
    } else {
        Err(format!("o valor '{valor}' não é um caracter válido"))
    }
}

fn inteiro_para_logico(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "inteiro_para_logico")?;
    Ok(Value::Bool(int(&args[0])? > 0))
}

fn inteiro_para_real(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "inteiro_para_real")?;
    Ok(Value::Real(int(&args[0])? as f64))
}

fn caracter_e_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_e_inteiro")?;
    Ok(Value::Bool(character(&args[0])?.is_ascii_digit()))
}

fn caracter_e_logico(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_e_logico")?;
    Ok(Value::Bool(matches!(
        character(&args[0])?,
        'S' | 's' | 'N' | 'n'
    )))
}

fn caracter_para_cadeia(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_para_cadeia")?;
    Ok(Value::str(character(&args[0])?.to_string()))
}

fn caracter_para_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_para_inteiro")?;
    Ok(Value::Int(parse_integer(
        &character(&args[0])?.to_string(),
        10,
    )?))
}

fn caracter_para_logico(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_para_logico")?;
    match character(&args[0])? {
        'S' | 's' => Ok(Value::Bool(true)),
        'N' | 'n' => Ok(Value::Bool(false)),
        valor => Err(format!("o valor '{valor}' não é um valor lógico válido")),
    }
}

fn logico_para_cadeia(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "logico_para_cadeia")?;
    Ok(Value::str(if boolean(&args[0])? {
        "verdadeiro"
    } else {
        "falso"
    }))
}

fn logico_para_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "logico_para_inteiro")?;
    Ok(Value::Int(boolean(&args[0])? as i32))
}

fn logico_para_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "logico_para_caracter")?;
    Ok(Value::Char(if boolean(&args[0])? { 'S' } else { 'N' }))
}

fn real_para_cadeia(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "real_para_cadeia")?;
    Ok(Value::str(java_double(real(&args[0])?)))
}

fn real_para_inteiro(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "real_para_inteiro")?;
    Ok(Value::Int(real(&args[0])? as i32))
}
