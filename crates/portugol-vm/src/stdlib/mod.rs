//! Bibliotecas multimídia da VM (`Graficos`, `Teclado`, `Mouse`, `Sons`)
//! e contexto nativo de host compartilhado pela execução PORS.
//!
//! As bibliotecas de console PORS (`Matematica`, `Texto`, `Tipos`, `Util`,
//! `Calendario`, `Arquivos`, `Json`, `Internet`) vivem em [`crate::pors_libs`].

mod fontes;
mod graficos;
mod sons;
mod teclado_mouse;

use std::path::{Path, PathBuf};
use std::time::{Instant, SystemTime, UNIX_EPOCH};

use crate::host::{Host, NoHost};
use crate::value::Value;

pub type NativeFn = fn(&mut NativeContext, &[Value]) -> Result<Value, String>;

/// Relógio usado para animações de GIF e testes determinísticos.
enum Clock {
    Real(Instant),
    /// Tempo simulado em milissegundos desde o início, a partir de um instante fixo.
    Simulated {
        elapsed_ms: u64,
    },
}

/// Estado compartilhado pelas funções nativas de multimídia e host.
pub struct NativeContext {
    rng: u64,
    clock: Clock,
    /// Base dos caminhos relativos para carregamento de imagens, fontes e sons.
    pub(crate) work_dir: PathBuf,
    pub(crate) host: Box<dyn Host>,
    pub(crate) graphics: graficos::Graphics,
    pub(crate) sounds: sons::Sounds,
    /// Sinal de interrupção da VM, para esperas como `ler_tecla`.
    pub(crate) interrupt: std::sync::Arc<std::sync::atomic::AtomicBool>,
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
            clock: Clock::Real(Instant::now()),
            work_dir: std::env::current_dir().unwrap_or_default(),
            host: Box::new(NoHost),
            graphics: graficos::Graphics::default(),
            sounds: sons::Sounds::default(),
            interrupt: Default::default(),
        }
    }

    pub(crate) fn reseed(&mut self, seed: u64) {
        self.rng = seed | 1;
    }

    /// Troca o relógio real por um simulado (para testes determinísticos).
    pub fn simulate_time(&mut self, _epoch_ms: u64) {
        self.clock = Clock::Simulated { elapsed_ms: 0 };
    }

    /// Milissegundos desde o início da execução.
    pub(crate) fn elapsed_ms(&self) -> u64 {
        match &self.clock {
            Clock::Real(start) => start.elapsed().as_millis() as u64,
            Clock::Simulated { elapsed_ms } => *elapsed_ms,
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

pub(super) fn resolve_path(ctx: &NativeContext, path: &str) -> PathBuf {
    let path = Path::new(path);
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        ctx.work_dir.join(path)
    };
    std::path::absolute(&full).unwrap_or(full)
}

pub fn native(library: &str, name: &str) -> Option<NativeFn> {
    match library {
        "Graficos" => graficos::function(name),
        "Teclado" => teclado_mouse::teclado(name),
        "Mouse" => teclado_mouse::mouse(name),
        "Sons" => sons::function(name),
        _ => None,
    }
}

pub fn constant(library: &str, name: &str) -> Option<Value> {
    match (library, name) {
        ("Graficos", name) => graficos::constant(name),
        ("Teclado", name) => teclado_mouse::teclado_constant(name),
        ("Mouse", name) => teclado_mouse::mouse_constant(name),
        _ => None,
    }
}

/// `sorteia`: intervalo fechado `[min, max]`.
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
        Value::Real(v) => Ok(*v as i32),
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
