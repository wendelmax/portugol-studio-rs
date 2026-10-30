//! `Texto.java`. Índices contam caracteres (o Java conta unidades UTF-16; a
//! diferença só aparece fora do plano multilíngue básico).

use super::{NativeContext, NativeFn, arity, character, int, text};
use crate::value::Value;

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "numero_caracteres" => numero_caracteres,
        "caixa_alta" => caixa_alta,
        "caixa_baixa" => caixa_baixa,
        "substituir" => substituir,
        "preencher_a_esquerda" => preencher_a_esquerda,
        "obter_caracter" => obter_caracter,
        "posicao_texto" => posicao_texto,
        "extrair_subtexto" => extrair_subtexto,
        _ => return None,
    })
}

fn numero_caracteres(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "numero_caracteres")?;
    Ok(Value::Int(text(&args[0])?.chars().count() as i32))
}

fn caixa_alta(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caixa_alta")?;
    Ok(Value::str(text(&args[0])?.to_uppercase()))
}

fn caixa_baixa(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caixa_baixa")?;
    Ok(Value::str(text(&args[0])?.to_lowercase()))
}

fn substituir(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "substituir")?;
    let cadeia = text(&args[0])?;
    Ok(Value::str(cadeia.replace(text(&args[1])?, text(&args[2])?)))
}

fn preencher_a_esquerda(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "preencher_a_esquerda")?;
    let car = character(&args[0])?;
    let tamanho = int(&args[1])?;
    let cadeia = text(&args[2])?;
    let len = cadeia.chars().count() as i64;
    if len >= tamanho as i64 {
        return Ok(Value::str(cadeia));
    }
    let mut out: String = std::iter::repeat_n(car, (tamanho as i64 - len) as usize).collect();
    out.push_str(cadeia);
    Ok(Value::str(out))
}

fn obter_caracter(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "obter_caracter")?;
    let cadeia = text(&args[0])?;
    let indice = int(&args[1])?;
    let len = cadeia.chars().count();
    if indice < 0 {
        return Err(format!("O índice do caracter ({indice}) é menor que 0"));
    }
    match cadeia.chars().nth(indice as usize) {
        Some(c) => Ok(Value::Char(c)),
        None => Err(format!(
            "O índice do caracter ({indice}) é maior que o número de caracteres na cadeia ({len})"
        )),
    }
}

/// `cadeia.indexOf(texto, posicao_inicial)`.
fn posicao_texto(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "posicao_texto")?;
    let texto: Vec<char> = text(&args[0])?.chars().collect();
    let cadeia: Vec<char> = text(&args[1])?.chars().collect();
    let inicio = int(&args[2])?.max(0) as usize;

    if inicio >= cadeia.len() {
        return Ok(Value::Int(if texto.is_empty() {
            cadeia.len() as i32
        } else {
            -1
        }));
    }
    if texto.is_empty() {
        return Ok(Value::Int(inicio as i32));
    }
    let found = cadeia[inicio..]
        .windows(texto.len())
        .position(|window| window == texto.as_slice());
    Ok(Value::Int(found.map_or(-1, |at| (inicio + at) as i32)))
}

fn extrair_subtexto(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "extrair_subtexto")?;
    let cadeia: Vec<char> = text(&args[0])?.chars().collect();
    let inicio = int(&args[1])?;
    let fim = int(&args[2])?;
    if inicio < 0 || fim < inicio || fim as usize > cadeia.len() {
        return Err(
            "Posição inicial ou final inválida. A posição deve estar entre 0 e o \
                    tamanho da cadeia"
                .to_string(),
        );
    }
    Ok(Value::str(
        cadeia[inicio as usize..fim as usize]
            .iter()
            .collect::<String>(),
    ))
}
