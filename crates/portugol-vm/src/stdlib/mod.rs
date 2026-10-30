//! Bibliotecas padrão de console, portadas de
//! `core/.../nucleo/bibliotecas` do Portugol Studio.
//!
//! As mensagens de erro reproduzem as do original. Bibliotecas que dependem
//! de interface gráfica (Graficos, Teclado, Mouse, Sons) ficam para a fase de
//! paridade; chamá-las gera um erro de execução claro.

mod matematica;
mod texto;
mod tipos;
mod util;

use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::value::Value;

pub type NativeFn = fn(&mut NativeContext, &[Value]) -> Result<Value, String>;

/// Estado compartilhado pelas funções nativas.
pub struct NativeContext {
    rng: u64,
    pub(crate) start: Instant,
}

impl NativeContext {
    pub fn new() -> Self {
        let seed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x9E37_79B9_7F4A_7C15);
        Self::with_seed(seed)
    }

    pub fn with_seed(seed: u64) -> Self {
        NativeContext {
            // xorshift não aceita estado zero.
            rng: seed | 1,
            start: Instant::now(),
        }
    }

    /// xorshift64*: suficiente para sorteios didáticos.
    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x >> 12;
        x ^= x << 25;
        x ^= x >> 27;
        self.rng = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// Inteiro uniforme em `[0, bound)`.
    fn below(&mut self, bound: u64) -> u64 {
        // Rejeição para evitar viés do módulo.
        let zone = u64::MAX - u64::MAX % bound;
        loop {
            let value = self.next_u64();
            if value < zone {
                return value % bound;
            }
        }
    }
}

impl Default for NativeContext {
    fn default() -> Self {
        Self::new()
    }
}

pub fn native(library: &str, name: &str) -> Option<NativeFn> {
    match library {
        "Matematica" => matematica::function(name),
        "Texto" => texto::function(name),
        "Tipos" => tipos::function(name),
        "Util" => util::function(name),
        _ => None,
    }
}

pub fn constant(library: &str, name: &str) -> Option<Value> {
    match (library, name) {
        ("Matematica", "PI") => Some(Value::Real(std::f64::consts::PI)),
        _ => None,
    }
}

/// `sorteia` da linguagem e `Util.sorteia`: intervalo fechado `[min, max]`.
pub fn sorteia(ctx: &mut NativeContext, min: i32, max: i32) -> Result<i32, String> {
    if min > max {
        return Err(format!(
            "O valor mínimo ({min}) é maior do que o valor máximo ({max})"
        ));
    }
    if min == max {
        return Err(format!("Os valores mínimo e máximo são iguais: {min}"));
    }
    let span = (max as i64 - min as i64 + 1) as u64;
    Ok((min as i64 + ctx.below(span) as i64) as i32)
}

// ---------------------------------------------------------------------
// Extração de argumentos. A análise semântica já garante tipos compatíveis;
// estas funções aplicam as conversões que o Java faria na chamada.
// ---------------------------------------------------------------------

fn arity(args: &[Value], expected: usize, name: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "a função \"{name}\" espera {expected} argumento(s), mas recebeu {}",
            args.len()
        ))
    }
}

fn real(value: &Value) -> Result<f64, String> {
    match value {
        Value::Real(v) => Ok(*v),
        Value::Int(v) => Ok(*v as f64),
        Value::Char(c) => Ok(*c as u32 as f64),
        other => Err(expected("real", other)),
    }
}

fn int(value: &Value) -> Result<i32, String> {
    match value {
        Value::Int(v) => Ok(*v),
        Value::Char(c) => Ok(*c as i32),
        other => Err(expected("inteiro", other)),
    }
}

fn text(value: &Value) -> Result<&str, String> {
    match value {
        Value::Str(s) => Ok(s),
        other => Err(expected("cadeia", other)),
    }
}

fn character(value: &Value) -> Result<char, String> {
    match value {
        Value::Char(c) => Ok(*c),
        other => Err(expected("caracter", other)),
    }
}

fn boolean(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        other => Err(expected("logico", other)),
    }
}

fn expected(ty: &str, value: &Value) -> String {
    format!(
        "esperava um argumento do tipo {ty}, mas recebeu {}",
        value.type_name()
    )
}
