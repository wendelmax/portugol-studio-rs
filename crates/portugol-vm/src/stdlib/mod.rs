//! Bibliotecas padrão de console, portadas de
//! `core/.../nucleo/bibliotecas` do Portugol Studio.
//!
//! As mensagens de erro reproduzem as do original. Bibliotecas que dependem
//! de interface gráfica (Graficos, Teclado, Mouse, Sons) ficam para a fase de
//! paridade; chamá-las gera um erro de execução claro.

mod arquivos;
mod calendario;
mod internet;
mod matematica;
mod objetos;
mod servicos_web;
mod texto;
mod tipos;
mod util;

use std::path::PathBuf;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::host::{Host, NoHost};
use crate::value::Value;

pub type NativeFn = fn(&mut NativeContext, &[Value]) -> Result<Value, String>;

/// Relógio das funções de tempo (`aguarde`, `tempo_decorrido`, Calendario).
enum Clock {
    Real(Instant),
    /// Tempo simulado em milissegundos desde o início, a partir de um instante
    /// fixo; `aguarde` só avança o contador. Torna testes determinísticos.
    Simulated {
        elapsed_ms: u64,
        epoch_ms: u64,
    },
}

/// Estado compartilhado pelas funções nativas.
pub struct NativeContext {
    rng: u64,
    clock: Clock,
    /// Base dos caminhos relativos (no original, a pasta do programa).
    pub(crate) work_dir: PathBuf,
    pub(crate) host: Box<dyn Host>,
    pub(crate) files: arquivos::OpenFiles,
    pub(crate) objects: objetos::Objects,
    pub(crate) internet: internet::Internet,
    pub(crate) web: servicos_web::WebServices,
    /// Sem acesso à rede: Internet e ServicosWeb falham como sem conexão.
    pub(crate) offline: bool,
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
            files: arquivos::OpenFiles::default(),
            objects: objetos::Objects::default(),
            internet: internet::Internet::default(),
            web: servicos_web::WebServices::default(),
            offline: false,
        }
    }

    /// Troca o relógio real por um simulado que começa em `epoch_ms`
    /// (milissegundos desde 1970-01-01 UTC).
    pub fn simulate_time(&mut self, epoch_ms: u64) {
        self.clock = Clock::Simulated {
            elapsed_ms: 0,
            epoch_ms,
        };
    }

    /// Milissegundos desde o início da execução.
    pub(crate) fn elapsed_ms(&self) -> u64 {
        match &self.clock {
            Clock::Real(start) => start.elapsed().as_millis() as u64,
            Clock::Simulated { elapsed_ms, .. } => *elapsed_ms,
        }
    }

    /// Instante atual em milissegundos desde 1970-01-01 UTC.
    pub(crate) fn now_ms(&self) -> u64 {
        match &self.clock {
            Clock::Real(_) => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(0),
            Clock::Simulated {
                elapsed_ms,
                epoch_ms,
            } => epoch_ms + elapsed_ms,
        }
    }

    pub(crate) fn is_simulated(&self) -> bool {
        matches!(self.clock, Clock::Simulated { .. })
    }

    pub(crate) fn sleep(&mut self, ms: u64) {
        match &mut self.clock {
            Clock::Real(_) => std::thread::sleep(Duration::from_millis(ms)),
            Clock::Simulated { elapsed_ms, .. } => *elapsed_ms += ms,
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
        "Arquivos" => arquivos::function(name),
        "Calendario" => calendario::function(name),
        "Matematica" => matematica::function(name),
        "Objetos" => objetos::function(name),
        "Internet" => internet::function(name),
        "ServicosWeb" => servicos_web::function(name),
        "Texto" => texto::function(name),
        "Tipos" => tipos::function(name),
        "Util" => util::function(name),
        _ => None,
    }
}

pub fn constant(library: &str, name: &str) -> Option<Value> {
    match (library, name) {
        ("Matematica", "PI") => Some(Value::Real(std::f64::consts::PI)),
        ("Calendario", name) => calendario::constant(name),
        ("Arquivos", name) => arquivos::constant(name),
        ("Objetos", name) => objetos::constant(name),
        ("ServicosWeb", name) => servicos_web::constant(name),
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

fn text_array(value: &Value) -> Result<crate::value::ArrayRef, String> {
    match value {
        Value::Array(array) => Ok(array.clone()),
        other => Err(expected("vetor de cadeias", other)),
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
