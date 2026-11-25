use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::{
    arity, check_and_charge, err_string, expect_char, expect_int, expect_str, none_value, ok_value,
    some_value,
};

pub(super) fn call(
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
