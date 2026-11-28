//! Bibliotecas padrão nativas do dialeto Ferro (`Matematica`, `Texto`, `Tipos`,
//! `Util`, `Calendario`, `Arquivos`, `Json`, `Internet`).
//!
//! Compiladas sempre junto ao núcleo mínimo da VM (inclusive sem a feature `stdlib`
//! e em `wasm32-unknown-unknown`), sem dependências externas, usando índices Unicode
//! (`char`), tipos explícitos (`Opcao<T>` / `Resultado<T, cadeia>`), relógio UTC
//! determinístico e contabilização estrita do orçamento de memória de cadeias antes
//! de cada alocação.

mod arquivos;
mod calendario;
mod internet;
mod json;
mod matematica;
mod multimedia;
mod texto;
mod tipos;
mod util;

use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::path::PathBuf;
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::value::Value;

#[cfg(feature = "stdlib")]
pub(crate) use multimedia::call_multimedia;
#[cfg(not(feature = "stdlib"))]
pub(crate) use multimedia::call_multimedia_unavailable;

/// Limite rígido por operação para evitar OOM mesmo quando a VM não configurou
/// um orçamento cumulativo de cadeias.
pub(super) const MAX_SINGLE_STRING_BYTES: usize = 16 * 1024 * 1024;
/// Timeout padrão de requisições HTTP em `Internet.*` (5 s).
#[cfg(not(target_arch = "wasm32"))]
pub(super) const DEFAULT_HTTP_TIMEOUT_MS: u64 = 5_000;

enum PorsClock {
    #[cfg(not(target_arch = "wasm32"))]
    Real(Instant),
    Simulated {
        elapsed_ms: u64,
        epoch_ms: u64,
    },
}

pub(crate) struct PorsRuntime {
    rng: u64,
    clock: PorsClock,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) workspace_dir: Option<PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) network_enabled: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(super) http_timeout_ms: u64,
}

impl PorsRuntime {
    pub(crate) fn new() -> Self {
        #[cfg(not(target_arch = "wasm32"))]
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        #[cfg(target_arch = "wasm32")]
        let seed = 0x9E37_79B9_7F4A_7C15;

        Self {
            rng: seed | 1,
            #[cfg(not(target_arch = "wasm32"))]
            clock: PorsClock::Real(Instant::now()),
            #[cfg(target_arch = "wasm32")]
            clock: PorsClock::Simulated {
                elapsed_ms: 0,
                epoch_ms: 0,
            },
            #[cfg(not(target_arch = "wasm32"))]
            workspace_dir: None,
            #[cfg(not(target_arch = "wasm32"))]
            network_enabled: false,
            #[cfg(not(target_arch = "wasm32"))]
            http_timeout_ms: DEFAULT_HTTP_TIMEOUT_MS,
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn set_workspace_dir(&mut self, dir: PathBuf) {
        self.workspace_dir = Some(dir);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn set_network_enabled(&mut self, enabled: bool) {
        self.network_enabled = enabled;
    }

    pub(crate) fn set_seed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }

    pub(crate) fn simulate_time(&mut self, epoch_ms: u64) {
        self.clock = PorsClock::Simulated {
            elapsed_ms: 0,
            epoch_ms,
        };
    }

    pub(super) fn next_u64(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    pub(super) fn elapsed_ms(&self) -> u64 {
        match &self.clock {
            #[cfg(not(target_arch = "wasm32"))]
            PorsClock::Real(started) => started.elapsed().as_millis() as u64,
            PorsClock::Simulated { elapsed_ms, .. } => *elapsed_ms,
        }
    }

    pub(super) fn now_epoch_ms(&self) -> i64 {
        match &self.clock {
            #[cfg(not(target_arch = "wasm32"))]
            PorsClock::Real(_) => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            PorsClock::Simulated {
                elapsed_ms,
                epoch_ms,
            } => epoch_ms.saturating_add(*elapsed_ms) as i64,
        }
    }

    pub(super) fn sleep_ms(
        &mut self,
        ms: u64,
        check_cancel: &mut impl FnMut() -> Result<(), String>,
    ) -> Result<(), String> {
        match &mut self.clock {
            PorsClock::Simulated { elapsed_ms, .. } => {
                check_cancel()?;
                *elapsed_ms = elapsed_ms.saturating_add(ms);
                Ok(())
            }
            #[cfg(not(target_arch = "wasm32"))]
            PorsClock::Real(_) => {
                let target = Instant::now() + Duration::from_millis(ms);
                loop {
                    check_cancel()?;
                    let now = Instant::now();
                    if now >= target {
                        return Ok(());
                    }
                    let slice = (target - now).min(Duration::from_millis(10));
                    std::thread::sleep(slice);
                }
            }
        }
    }
}

pub(crate) fn call(
    library: &str,
    name: &str,
    args: &[Value],
    runtime: &mut PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
    check_cancel: &mut impl FnMut() -> Result<(), String>,
) -> Option<Result<Value, String>> {
    match library {
        "PorsMatematica" => Some(matematica::call(name, args)),
        "PorsTexto" => Some(texto::call(name, args, charge_string)),
        "PorsTipos" => Some(tipos::call(name, args, charge_string)),
        "PorsUtil" => Some(util::call(name, args, runtime, charge_string, check_cancel)),
        "PorsCalendario" => Some(calendario::call(name, args, runtime, charge_string)),
        "PorsArquivos" => Some(arquivos::call(name, args, runtime, charge_string)),
        "PorsJson" => Some(json::call(name, args, charge_string)),
        "PorsInternet" => Some(internet::call(
            name,
            args,
            runtime,
            charge_string,
            check_cancel,
        )),
        _ => None,
    }
}

pub(super) fn check_and_charge(
    bytes: usize,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<(), String> {
    charge_string(bytes)?;
    if bytes > MAX_SINGLE_STRING_BYTES {
        return Err("O limite de memória para cadeias foi excedido.".to_string());
    }
    Ok(())
}

pub(super) fn some_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

pub(super) fn none_value() -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])))
}

pub(super) fn ok_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

pub(super) fn err_string(
    message: String,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    check_and_charge(message.len(), charge_string)?;
    Ok(Value::Array(Rc::new(RefCell::new(vec![
        Value::Int(1),
        Value::str(message),
    ]))))
}

pub(super) fn arity(args: &[Value], expected: usize, qualified: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "`{qualified}` espera {expected} argumento(s), mas recebeu {}.",
            args.len()
        ))
    }
}

pub(super) fn expect_int(value: &Value) -> Result<i32, String> {
    match value {
        Value::Int(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `inteiro`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

pub(super) fn expect_real(value: &Value) -> Result<f64, String> {
    match value {
        Value::Real(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `real`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

pub(super) fn expect_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `logico`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

pub(super) fn expect_char(value: &Value) -> Result<char, String> {
    match value {
        Value::Char(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `caracter`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

pub(super) fn expect_str(value: &Value) -> Result<&str, String> {
    match value {
        Value::Str(v) => Ok(v),
        other => Err(format!(
            "Esperado valor `cadeia`, mas recebeu {}.",
            other.type_name()
        )),
    }
}
