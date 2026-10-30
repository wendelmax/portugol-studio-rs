//! `Matematica.java`.

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
    Ok(Value::Real(real(&args[0])?.powf(real(&args[1])?)))
}

fn raiz(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "raiz")?;
    Ok(Value::Real(real(&args[0])?.powf(1.0 / real(&args[1])?)))
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
    ((value + 0.5).floor() as i64) as f64
}

fn logaritmo(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "logaritmo")?;
    Ok(Value::Real(real(&args[0])?.ln() / real(&args[1])?.ln()))
}

fn seno(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "seno")?;
    Ok(Value::Real(real(&args[0])?.sin()))
}

fn cosseno(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "cosseno")?;
    Ok(Value::Real(real(&args[0])?.cos()))
}

fn tangente(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "tangente")?;
    Ok(Value::Real(real(&args[0])?.tan()))
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
