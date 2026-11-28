use crate::value::Value;

use super::{FerroRuntime, arity, err_string, expect_int, ok_value};

/// Limite máximo de espera por chamada em `Util.aguarde` (60 s).
const MAX_SLEEP_MS: i32 = 60_000;

pub(super) fn call(
    name: &str,
    args: &[Value],
    runtime: &mut FerroRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
    check_cancel: &mut impl FnMut() -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "sorteia" => {
            arity(args, 2, "Util.sorteia")?;
            let min = expect_int(&args[0])?;
            let max = expect_int(&args[1])?;
            if min > max {
                return err_string(
                    format!("O valor mínimo ({min}) não pode ser maior que o máximo ({max})"),
                    charge_string,
                );
            }
            let span = (max as i64 - min as i64 + 1) as u64;
            let offset = (runtime.next_u64() % span) as i64;
            Ok(ok_value(Value::Int((min as i64 + offset) as i32)))
        }
        "definir_semente" => {
            arity(args, 1, "Util.definir_semente")?;
            let seed = expect_int(&args[0])? as u32 as u64;
            runtime.set_seed(seed);
            Ok(Value::Void)
        }
        "tempo_decorrido" => {
            arity(args, 0, "Util.tempo_decorrido")?;
            Ok(Value::Int(runtime.elapsed_ms().min(i32::MAX as u64) as i32))
        }
        "aguarde" => {
            arity(args, 1, "Util.aguarde")?;
            let ms = expect_int(&args[0])?;
            if ms < 0 {
                return Err(
                    "O intervalo de espera em `Util.aguarde` não pode ser negativo.".into(),
                );
            }
            if ms > MAX_SLEEP_MS {
                return Err(format!(
                    "O intervalo de espera ({ms} ms) excede o limite de {MAX_SLEEP_MS} ms por chamada."
                ));
            }
            runtime.sleep_ms(ms as u64, check_cancel)?;
            Ok(Value::Void)
        }
        other => Err(format!("Função desconhecida em Util: `{other}`.")),
    }
}
