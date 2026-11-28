use crate::value::Value;

use super::{arity, expect_int, expect_real};

pub(super) fn call(name: &str, args: &[Value]) -> Result<Value, String> {
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
