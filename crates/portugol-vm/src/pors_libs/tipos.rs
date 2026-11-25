use crate::value::{Value, java_double};

use super::{
    arity, check_and_charge, err_string, expect_bool, expect_char, expect_int, expect_real,
    expect_str, ok_value,
};

fn parse_pors_int_base(texto: &str, base: i32) -> Result<i32, String> {
    if !matches!(base, 2 | 10 | 16) {
        return Err(format!("Base inválida ({base}); use 2, 10 ou 16"));
    }
    let trimmed = texto.trim();
    let (neg, rest) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed),
    };
    let digits = match base {
        2 => rest
            .strip_prefix("0b")
            .or_else(|| rest.strip_prefix("0B"))
            .unwrap_or(rest),
        16 => rest
            .strip_prefix("0x")
            .or_else(|| rest.strip_prefix("0X"))
            .unwrap_or(rest),
        _ => rest,
    };
    if digits.is_empty() || digits.starts_with('+') || digits.starts_with('-') {
        return Err(format!("O valor '{texto}' não é um inteiro válido"));
    }
    let candidate = if neg {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    i32::from_str_radix(&candidate, base as u32)
        .map_err(|_| format!("O valor '{texto}' não é um inteiro válido"))
}

pub(super) fn call(
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "para_inteiro" => {
            arity(args, 1, "Tipos.para_inteiro")?;
            let texto = expect_str(&args[0])?;
            match texto.trim().parse::<i32>() {
                Ok(valor) => Ok(ok_value(Value::Int(valor))),
                Err(_) => err_string(
                    format!("O valor '{texto}' não é um inteiro válido"),
                    charge_string,
                ),
            }
        }
        "para_inteiro_base" => {
            arity(args, 2, "Tipos.para_inteiro_base")?;
            let texto = expect_str(&args[0])?;
            let base = expect_int(&args[1])?;
            match parse_pors_int_base(texto, base) {
                Ok(valor) => Ok(ok_value(Value::Int(valor))),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "para_real" => {
            arity(args, 1, "Tipos.para_real")?;
            let texto = expect_str(&args[0])?;
            match texto.trim().parse::<f64>() {
                Ok(valor) if valor.is_finite() => Ok(ok_value(Value::Real(valor))),
                _ => err_string(
                    format!("O valor '{texto}' não é um número real válido"),
                    charge_string,
                ),
            }
        }
        "para_logico" => {
            arity(args, 1, "Tipos.para_logico")?;
            let texto = expect_str(&args[0])?;
            match texto.trim() {
                "verdadeiro" => Ok(ok_value(Value::Bool(true))),
                "falso" => Ok(ok_value(Value::Bool(false))),
                _ => err_string(
                    format!("O valor '{texto}' não é um valor lógico válido"),
                    charge_string,
                ),
            }
        }
        "para_caracter" => {
            arity(args, 1, "Tipos.para_caracter")?;
            let texto = expect_str(&args[0])?;
            let mut chars = texto.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(ok_value(Value::Char(ch))),
                _ => err_string(
                    format!("O valor '{texto}' não é um caracter válido"),
                    charge_string,
                ),
            }
        }
        "inteiro_para_real" => {
            arity(args, 1, "Tipos.inteiro_para_real")?;
            Ok(Value::Real(expect_int(&args[0])? as f64))
        }
        "real_para_inteiro" => {
            arity(args, 1, "Tipos.real_para_inteiro")?;
            Ok(Value::Int(expect_real(&args[0])? as i32))
        }
        "inteiro_para_cadeia" => {
            arity(args, 1, "Tipos.inteiro_para_cadeia")?;
            let out = expect_int(&args[0])?.to_string();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "inteiro_para_cadeia_base" => {
            arity(args, 2, "Tipos.inteiro_para_cadeia_base")?;
            let valor = expect_int(&args[0])?;
            let base = expect_int(&args[1])?;
            let out = match base {
                2 => format!("{:b}", valor as u32),
                10 => valor.to_string(),
                16 => format!("{:X}", valor as u32),
                _ => {
                    return err_string(
                        format!("Base inválida ({base}); use 2, 10 ou 16"),
                        charge_string,
                    );
                }
            };
            check_and_charge(out.len(), charge_string)?;
            Ok(ok_value(Value::str(out)))
        }
        "real_para_cadeia" => {
            arity(args, 1, "Tipos.real_para_cadeia")?;
            let out = java_double(expect_real(&args[0])?);
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "logico_para_cadeia" => {
            arity(args, 1, "Tipos.logico_para_cadeia")?;
            let out = if expect_bool(&args[0])? {
                "verdadeiro"
            } else {
                "falso"
            };
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "caracter_para_cadeia" => {
            arity(args, 1, "Tipos.caracter_para_cadeia")?;
            let out = expect_char(&args[0])?.to_string();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "logico_para_inteiro" => {
            arity(args, 1, "Tipos.logico_para_inteiro")?;
            Ok(Value::Int(i32::from(expect_bool(&args[0])?)))
        }
        "caracter_para_inteiro" => {
            arity(args, 1, "Tipos.caracter_para_inteiro")?;
            let ch = expect_char(&args[0])?;
            match ch.to_digit(10) {
                Some(digit) => Ok(ok_value(Value::Int(digit as i32))),
                None => err_string(
                    format!("O caracter '{ch}' não é um dígito válido"),
                    charge_string,
                ),
            }
        }
        "inteiro_para_caracter" => {
            arity(args, 1, "Tipos.inteiro_para_caracter")?;
            let valor = expect_int(&args[0])?;
            if (0..=9).contains(&valor) {
                Ok(ok_value(Value::Char(char::from(b'0' + valor as u8))))
            } else {
                err_string(
                    format!("O inteiro '{valor}' não corresponde a um dígito (0 a 9)"),
                    charge_string,
                )
            }
        }
        "cadeia_e_inteiro" => {
            arity(args, 1, "Tipos.cadeia_e_inteiro")?;
            Ok(Value::Bool(
                expect_str(&args[0])?.trim().parse::<i32>().is_ok(),
            ))
        }
        "cadeia_e_real" => {
            arity(args, 1, "Tipos.cadeia_e_real")?;
            Ok(Value::Bool(
                expect_str(&args[0])?
                    .trim()
                    .parse::<f64>()
                    .is_ok_and(f64::is_finite),
            ))
        }
        "cadeia_e_logico" => {
            arity(args, 1, "Tipos.cadeia_e_logico")?;
            Ok(Value::Bool(matches!(
                expect_str(&args[0])?.trim(),
                "verdadeiro" | "falso"
            )))
        }
        "cadeia_e_caracter" => {
            arity(args, 1, "Tipos.cadeia_e_caracter")?;
            Ok(Value::Bool(expect_str(&args[0])?.chars().count() == 1))
        }
        other => Err(format!("Função desconhecida em Tipos: `{other}`.")),
    }
}
