//! `Matematica.java`.
//!
//! As funções usam o crate `libm` (port do musl, derivado do fdlibm, o mesmo
//! do `StrictMath` do Java) em vez da `libm` do sistema, para que os
//! resultados sejam idênticos em Windows, Linux e macOS.

use super::{NativeContext, NativeFn, arity, int, real};
use crate::value::Value;

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "potencia" => potencia,
        "raiz" => raiz,
        "arredondar" => arredondar,
        "logaritmo" => logaritmo,
        "seno" => seno,
        "cosseno" => cosseno,
        "tangente" => tangente,
        "valor_absoluto" => valor_absoluto,
        "maior_numero" => maior_numero,
        "menor_numero" => menor_numero,
        _ => return None,
    })
}

fn potencia(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "potencia")?;
    Ok(Value::Real(libm::pow(real(&args[0])?, real(&args[1])?)))
}

fn raiz(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "raiz")?;
    Ok(Value::Real(libm::pow(
        real(&args[0])?,
        1.0 / real(&args[1])?,
    )))
}

fn arredondar(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "arredondar")?;
    let numero = real(&args[0])?;
    let casas = int(&args[1])?;
    let mut factor = 1.0;
    for _ in 1..=casas {
        factor *= 10.0;
    }
    Ok(Value::Real(java_round(numero * factor) / factor))
}

/// `Math.round(double)`: arredonda meio para cima (`floor(x + 0.5)`), devolve
/// `long` e satura nos limites.
fn java_round(value: f64) -> f64 {
    if value.is_nan() {
        return 0.0;
    }
    (libm::floor(value + 0.5) as i64) as f64
}

fn logaritmo(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "logaritmo")?;
    Ok(Value::Real(
        libm::log(real(&args[0])?) / libm::log(real(&args[1])?),
    ))
}

fn seno(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "seno")?;
    Ok(Value::Real(libm::sin(real(&args[0])?)))
}

fn cosseno(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cosseno")?;
    Ok(Value::Real(libm::cos(real(&args[0])?)))
}

fn tangente(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "tangente")?;
    Ok(Value::Real(libm::tan(real(&args[0])?)))
}

fn valor_absoluto(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "valor_absoluto")?;
    Ok(Value::Real(real(&args[0])?.abs()))
}

fn maior_numero(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "maior_numero")?;
    Ok(Value::Real(java_max(real(&args[0])?, real(&args[1])?)))
}

fn menor_numero(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "menor_numero")?;
    Ok(Value::Real(java_min(real(&args[0])?, real(&args[1])?)))
}

/// `Math.max` propaga NaN, ao contrário de `f64::max`.
fn java_max(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.max(b)
    }
}

fn java_min(a: f64, b: f64) -> f64 {
    if a.is_nan() || b.is_nan() {
        f64::NAN
    } else {
        a.min(b)
    }
}
