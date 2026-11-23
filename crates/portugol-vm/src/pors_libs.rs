//! Bibliotecas padrão nativas do dialeto Portugol-RS (`Matematica`, `Texto`, `Tipos`).
//!
//! Compiladas sempre junto ao núcleo mínimo da VM (inclusive sem a feature `stdlib`
//! e em `wasm32-unknown-unknown`), sem dependências externas, usando índices Unicode
//! (`char`), tipos explícitos (`Opcao<T>` / `Resultado<T, cadeia>`) e contabilização
//! estrita do orçamento de memória de cadeias antes de cada alocação.

use std::cell::RefCell;
use std::rc::Rc;

use crate::value::{Value, java_double};

/// Limite rígido por operação para evitar OOM mesmo quando a VM não configurou
/// um orçamento cumulativo de cadeias.
const MAX_SINGLE_STRING_BYTES: usize = 16 * 1024 * 1024;

pub(crate) fn call(
    library: &str,
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Option<Result<Value, String>> {
    match library {
        "PorsMatematica" => Some(call_matematica(name, args)),
        "PorsTexto" => Some(call_texto(name, args, charge_string)),
        "PorsTipos" => Some(call_tipos(name, args, charge_string)),
        _ => None,
    }
}

fn check_and_charge(
    bytes: usize,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<(), String> {
    charge_string(bytes)?;
    if bytes > MAX_SINGLE_STRING_BYTES {
        return Err("O limite de memória para cadeias foi excedido.".to_string());
    }
    Ok(())
}

fn some_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

fn none_value() -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])))
}

fn ok_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

fn err_string(
    message: String,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    check_and_charge(message.len(), charge_string)?;
    Ok(Value::Array(Rc::new(RefCell::new(vec![
        Value::Int(1),
        Value::str(message),
    ]))))
}

fn arity(args: &[Value], expected: usize, qualified: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "`{qualified}` espera {expected} argumento(s), mas recebeu {}.",
            args.len()
        ))
    }
}

fn expect_int(value: &Value) -> Result<i32, String> {
    match value {
        Value::Int(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `inteiro`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_real(value: &Value) -> Result<f64, String> {
    match value {
        Value::Real(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `real`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `logico`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_char(value: &Value) -> Result<char, String> {
    match value {
        Value::Char(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `caracter`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_str(value: &Value) -> Result<&str, String> {
    match value {
        Value::Str(v) => Ok(v),
        other => Err(format!(
            "Esperado valor `cadeia`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn call_matematica(name: &str, args: &[Value]) -> Result<Value, String> {
    match name {
        "potencia" => {
            arity(args, 2, "Matematica.potencia")?;
            Ok(Value::Real(
                expect_real(&args[0])?.powf(expect_real(&args[1])?),
            ))
        }
        "raiz" => {
            arity(args, 2, "Matematica.raiz")?;
            Ok(Value::Real(
                expect_real(&args[0])?.powf(1.0 / expect_real(&args[1])?),
            ))
        }
        "raiz_quadrada" => {
            arity(args, 1, "Matematica.raiz_quadrada")?;
            Ok(Value::Real(expect_real(&args[0])?.sqrt()))
        }
        "arredondar" => {
            arity(args, 2, "Matematica.arredondar")?;
            let numero = expect_real(&args[0])?;
            let casas = expect_int(&args[1])?.clamp(0, 15);
            let factor = 10_f64.powi(casas);
            Ok(Value::Real((numero * factor).round() / factor))
        }
        "piso" => {
            arity(args, 1, "Matematica.piso")?;
            Ok(Value::Real(expect_real(&args[0])?.floor()))
        }
        "teto" => {
            arity(args, 1, "Matematica.teto")?;
            Ok(Value::Real(expect_real(&args[0])?.ceil()))
        }
        "logaritmo" => {
            arity(args, 2, "Matematica.logaritmo")?;
            Ok(Value::Real(
                expect_real(&args[0])?.ln() / expect_real(&args[1])?.ln(),
            ))
        }
        "seno" => {
            arity(args, 1, "Matematica.seno")?;
            Ok(Value::Real(expect_real(&args[0])?.sin()))
        }
        "cosseno" => {
            arity(args, 1, "Matematica.cosseno")?;
            Ok(Value::Real(expect_real(&args[0])?.cos()))
        }
        "tangente" => {
            arity(args, 1, "Matematica.tangente")?;
            Ok(Value::Real(expect_real(&args[0])?.tan()))
        }
        "valor_absoluto" => {
            arity(args, 1, "Matematica.valor_absoluto")?;
            Ok(Value::Real(expect_real(&args[0])?.abs()))
        }
        "valor_absoluto_inteiro" => {
            arity(args, 1, "Matematica.valor_absoluto_inteiro")?;
            Ok(Value::Int(expect_int(&args[0])?.wrapping_abs()))
        }
        "maior_numero" => {
            arity(args, 2, "Matematica.maior_numero")?;
            let a = expect_real(&args[0])?;
            let b = expect_real(&args[1])?;
            Ok(Value::Real(if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                a.max(b)
            }))
        }
        "menor_numero" => {
            arity(args, 2, "Matematica.menor_numero")?;
            let a = expect_real(&args[0])?;
            let b = expect_real(&args[1])?;
            Ok(Value::Real(if a.is_nan() || b.is_nan() {
                f64::NAN
            } else {
                a.min(b)
            }))
        }
        "maior_inteiro" => {
            arity(args, 2, "Matematica.maior_inteiro")?;
            Ok(Value::Int(expect_int(&args[0])?.max(expect_int(&args[1])?)))
        }
        "menor_inteiro" => {
            arity(args, 2, "Matematica.menor_inteiro")?;
            Ok(Value::Int(expect_int(&args[0])?.min(expect_int(&args[1])?)))
        }
        other => Err(format!("Função desconhecida em Matematica: `{other}`.")),
    }
}

fn call_texto(
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "numero_caracteres" => {
            arity(args, 1, "Texto.numero_caracteres")?;
            Ok(Value::Int(expect_str(&args[0])?.chars().count() as i32))
        }
        "caixa_alta" => {
            arity(args, 1, "Texto.caixa_alta")?;
            let out = expect_str(&args[0])?.to_uppercase();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "caixa_baixa" => {
            arity(args, 1, "Texto.caixa_baixa")?;
            let out = expect_str(&args[0])?.to_lowercase();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "aparar" => {
            arity(args, 1, "Texto.aparar")?;
            let out = expect_str(&args[0])?.trim();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "contem" => {
            arity(args, 2, "Texto.contem")?;
            Ok(Value::Bool(
                expect_str(&args[0])?.contains(expect_str(&args[1])?),
            ))
        }
        "comeca_com" => {
            arity(args, 2, "Texto.comeca_com")?;
            Ok(Value::Bool(
                expect_str(&args[0])?.starts_with(expect_str(&args[1])?),
            ))
        }
        "termina_com" => {
            arity(args, 2, "Texto.termina_com")?;
            Ok(Value::Bool(
                expect_str(&args[0])?.ends_with(expect_str(&args[1])?),
            ))
        }
        "substituir" => {
            arity(args, 3, "Texto.substituir")?;
            let texto = expect_str(&args[0])?;
            let antigo = expect_str(&args[1])?;
            let novo = expect_str(&args[2])?;
            let expected = if antigo.is_empty() {
                let slots = texto.chars().count().saturating_add(1);
                texto.len().saturating_add(slots.saturating_mul(novo.len()))
            } else {
                let count = texto.matches(antigo).count();
                let removed = count.saturating_mul(antigo.len());
                let added = count.saturating_mul(novo.len());
                texto.len().saturating_sub(removed).saturating_add(added)
            };
            check_and_charge(expected, charge_string)?;
            Ok(Value::str(texto.replace(antigo, novo)))
        }
        "preencher_a_esquerda" => {
            arity(args, 3, "Texto.preencher_a_esquerda")?;
            let car = expect_char(&args[0])?;
            let tamanho = expect_int(&args[1])?;
            let texto = expect_str(&args[2])?;
            let len = texto.chars().count();
            if tamanho <= 0 || len >= tamanho as usize {
                check_and_charge(texto.len(), charge_string)?;
                return Ok(Value::str(texto));
            }
            let pad = tamanho as usize - len;
            let expected = pad
                .saturating_mul(car.len_utf8())
                .saturating_add(texto.len());
            check_and_charge(expected, charge_string)?;
            let mut out = String::with_capacity(expected);
            for _ in 0..pad {
                out.push(car);
            }
            out.push_str(texto);
            Ok(Value::str(out))
        }
        "obter_caracter" => {
            arity(args, 2, "Texto.obter_caracter")?;
            let texto = expect_str(&args[0])?;
            let indice = expect_int(&args[1])?;
            if indice < 0 {
                return Ok(none_value());
            }
            match texto.chars().nth(indice as usize) {
                Some(ch) => Ok(some_value(Value::Char(ch))),
                None => Ok(none_value()),
            }
        }
        "posicao_texto" => {
            arity(args, 3, "Texto.posicao_texto")?;
            let trecho: Vec<char> = expect_str(&args[0])?.chars().collect();
            let texto: Vec<char> = expect_str(&args[1])?.chars().collect();
            let inicio = expect_int(&args[2])?.max(0) as usize;
            if inicio > texto.len() {
                return Ok(none_value());
            }
            if trecho.is_empty() {
                return Ok(some_value(Value::Int(inicio as i32)));
            }
            if inicio == texto.len() {
                return Ok(none_value());
            }
            match texto[inicio..]
                .windows(trecho.len())
                .position(|window| window == trecho.as_slice())
            {
                Some(at) => Ok(some_value(Value::Int((inicio + at) as i32))),
                None => Ok(none_value()),
            }
        }
        "extrair_subtexto" => {
            arity(args, 3, "Texto.extrair_subtexto")?;
            let chars: Vec<char> = expect_str(&args[0])?.chars().collect();
            let inicio = expect_int(&args[1])?;
            let fim = expect_int(&args[2])?;
            if inicio < 0 || fim < inicio || fim as usize > chars.len() {
                return err_string(
                    "Posição inicial ou final inválida para extrair subtexto".to_string(),
                    charge_string,
                );
            }
            let slice: String = chars[inicio as usize..fim as usize].iter().collect();
            check_and_charge(slice.len(), charge_string)?;
            Ok(ok_value(Value::str(slice)))
        }
        "dividir" => {
            arity(args, 2, "Texto.dividir")?;
            let texto = expect_str(&args[0])?;
            let separador = expect_str(&args[1])?;
            if separador.is_empty() {
                check_and_charge(texto.len(), charge_string)?;
                let parts: Vec<Value> =
                    texto.chars().map(|ch| Value::str(ch.to_string())).collect();
                return Ok(Value::Array(Rc::new(RefCell::new(parts))));
            }
            let raw_parts: Vec<&str> = texto.split(separador).collect();
            let total_bytes: usize = raw_parts.iter().map(|part| part.len()).sum();
            check_and_charge(total_bytes, charge_string)?;
            let parts: Vec<Value> = raw_parts.into_iter().map(Value::str).collect();
            Ok(Value::Array(Rc::new(RefCell::new(parts))))
        }
        other => Err(format!("Função desconhecida em Texto: `{other}`.")),
    }
}

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

fn call_tipos(
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
