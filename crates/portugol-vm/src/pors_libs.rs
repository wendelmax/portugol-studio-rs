//! Bibliotecas padrão nativas do dialeto Portugol-RS (`Matematica`, `Texto`, `Tipos`,
//! `Util`, `Calendario`, `Arquivos`, `Json`, `Internet`).
//!
//! Compiladas sempre junto ao núcleo mínimo da VM (inclusive sem a feature `stdlib`
//! e em `wasm32-unknown-unknown`), sem dependências externas, usando índices Unicode
//! (`char`), tipos explícitos (`Opcao<T>` / `Resultado<T, cadeia>`), relógio UTC
//! determinístico e contabilização estrita do orçamento de memória de cadeias antes
//! de cada alocação.

use std::cell::RefCell;
#[cfg(not(target_arch = "wasm32"))]
use std::path::{Component, Path, PathBuf};
use std::rc::Rc;
#[cfg(not(target_arch = "wasm32"))]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use crate::value::{Value, java_double};

/// Limite rígido por operação para evitar OOM mesmo quando a VM não configurou
/// um orçamento cumulativo de cadeias.
const MAX_SINGLE_STRING_BYTES: usize = 16 * 1024 * 1024;
/// Limite máximo de espera por chamada em `Util.aguarde` (60 s).
const MAX_SLEEP_MS: i32 = 60_000;
/// Limite máximo por arquivo em `Arquivos.*` (2 MiB).
#[cfg(not(target_arch = "wasm32"))]
const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
/// Limite máximo de entradas listadas por `Arquivos.listar`.
#[cfg(not(target_arch = "wasm32"))]
const MAX_LIST_ENTRIES: usize = 1_024;
/// Profundidade máxima de aninhamento aceita por `Json.*`.
const MAX_JSON_DEPTH: usize = 64;
/// Limite máximo em bytes de entrada ou saída por chamada em `Json.*` (2 MiB).
const MAX_JSON_BYTES: usize = 2 * 1024 * 1024;
/// Limite máximo de resposta HTTP em `Internet.*` (1 MiB).
#[cfg(not(target_arch = "wasm32"))]
const MAX_HTTP_BYTES: usize = 1024 * 1024;
/// Timeout padrão de requisições HTTP em `Internet.*` (5 s).
#[cfg(not(target_arch = "wasm32"))]
const DEFAULT_HTTP_TIMEOUT_MS: u64 = 5_000;
/// Timeout mínimo e máximo permitidos para `Internet.definir_tempo_limite`.
#[cfg(not(target_arch = "wasm32"))]
const MIN_HTTP_TIMEOUT_MS: u64 = 100;
#[cfg(not(target_arch = "wasm32"))]
const MAX_HTTP_TIMEOUT_MS: u64 = 30_000;

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
    workspace_dir: Option<PathBuf>,
    #[cfg(not(target_arch = "wasm32"))]
    network_enabled: bool,
    #[cfg(not(target_arch = "wasm32"))]
    http_timeout_ms: u64,
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

    fn next_u64(&mut self) -> u64 {
        let mut x = self.rng;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.rng = x;
        x.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn elapsed_ms(&self) -> u64 {
        match &self.clock {
            #[cfg(not(target_arch = "wasm32"))]
            PorsClock::Real(started) => started.elapsed().as_millis() as u64,
            PorsClock::Simulated { elapsed_ms, .. } => *elapsed_ms,
        }
    }

    fn now_epoch_ms(&self) -> i64 {
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

    fn sleep_ms(
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
        "PorsMatematica" => Some(call_matematica(name, args)),
        "PorsTexto" => Some(call_texto(name, args, charge_string)),
        "PorsTipos" => Some(call_tipos(name, args, charge_string)),
        "PorsUtil" => Some(call_util(name, args, runtime, charge_string, check_cancel)),
        "PorsCalendario" => Some(call_calendario(name, args, runtime, charge_string)),
        "PorsArquivos" => Some(call_arquivos(name, args, runtime, charge_string)),
        "PorsJson" => Some(call_json(name, args, charge_string)),
        "PorsInternet" => Some(call_internet(
            name,
            args,
            runtime,
            charge_string,
            check_cancel,
        )),
        _ => None,
    }
}

fn check_and_charge(
    bytes: usize,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<(), String> {
    charge_string(bytes)?;
    if bytes > MAX_SINGLE_STRING_BYTES {
        return Err("O limite de memória para cadeias foi excedido.".to_string());
    }
    Ok(())
}

fn some_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

fn none_value() -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(1)])))
}

fn ok_value(value: Value) -> Value {
    Value::Array(Rc::new(RefCell::new(vec![Value::Int(0), value])))
}

fn err_string(
    message: String,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    check_and_charge(message.len(), charge_string)?;
    Ok(Value::Array(Rc::new(RefCell::new(vec![
        Value::Int(1),
        Value::str(message),
    ]))))
}

fn arity(args: &[Value], expected: usize, qualified: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "`{qualified}` espera {expected} argumento(s), mas recebeu {}.",
            args.len()
        ))
    }
}

fn expect_int(value: &Value) -> Result<i32, String> {
    match value {
        Value::Int(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `inteiro`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_real(value: &Value) -> Result<f64, String> {
    match value {
        Value::Real(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `real`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `logico`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_char(value: &Value) -> Result<char, String> {
    match value {
        Value::Char(v) => Ok(*v),
        other => Err(format!(
            "Esperado valor `caracter`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn expect_str(value: &Value) -> Result<&str, String> {
    match value {
        Value::Str(v) => Ok(v),
        other => Err(format!(
            "Esperado valor `cadeia`, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn call_matematica(name: &str, args: &[Value]) -> Result<Value, String> {
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

fn call_texto(
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

fn parse_pors_int_base(texto: &str, base: i32) -> Result<i32, String> {
    if !matches!(base, 2 | 10 | 16) {
        return Err(format!("Base inválida ({base}); use 2, 10 ou 16"));
    }
    let trimmed = texto.trim();
    let (neg, rest) = match trimmed.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, trimmed),
    };
    let digits = match base {
        2 => rest
            .strip_prefix("0b")
            .or_else(|| rest.strip_prefix("0B"))
            .unwrap_or(rest),
        16 => rest
            .strip_prefix("0x")
            .or_else(|| rest.strip_prefix("0X"))
            .unwrap_or(rest),
        _ => rest,
    };
    if digits.is_empty() || digits.starts_with('+') || digits.starts_with('-') {
        return Err(format!("O valor '{texto}' não é um inteiro válido"));
    }
    let candidate = if neg {
        format!("-{digits}")
    } else {
        digits.to_owned()
    };
    i32::from_str_radix(&candidate, base as u32)
        .map_err(|_| format!("O valor '{texto}' não é um inteiro válido"))
}

fn call_tipos(
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "para_inteiro" => {
            arity(args, 1, "Tipos.para_inteiro")?;
            let texto = expect_str(&args[0])?;
            match texto.trim().parse::<i32>() {
                Ok(valor) => Ok(ok_value(Value::Int(valor))),
                Err(_) => err_string(
                    format!("O valor '{texto}' não é um inteiro válido"),
                    charge_string,
                ),
            }
        }
        "para_inteiro_base" => {
            arity(args, 2, "Tipos.para_inteiro_base")?;
            let texto = expect_str(&args[0])?;
            let base = expect_int(&args[1])?;
            match parse_pors_int_base(texto, base) {
                Ok(valor) => Ok(ok_value(Value::Int(valor))),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "para_real" => {
            arity(args, 1, "Tipos.para_real")?;
            let texto = expect_str(&args[0])?;
            match texto.trim().parse::<f64>() {
                Ok(valor) if valor.is_finite() => Ok(ok_value(Value::Real(valor))),
                _ => err_string(
                    format!("O valor '{texto}' não é um número real válido"),
                    charge_string,
                ),
            }
        }
        "para_logico" => {
            arity(args, 1, "Tipos.para_logico")?;
            let texto = expect_str(&args[0])?;
            match texto.trim() {
                "verdadeiro" => Ok(ok_value(Value::Bool(true))),
                "falso" => Ok(ok_value(Value::Bool(false))),
                _ => err_string(
                    format!("O valor '{texto}' não é um valor lógico válido"),
                    charge_string,
                ),
            }
        }
        "para_caracter" => {
            arity(args, 1, "Tipos.para_caracter")?;
            let texto = expect_str(&args[0])?;
            let mut chars = texto.chars();
            match (chars.next(), chars.next()) {
                (Some(ch), None) => Ok(ok_value(Value::Char(ch))),
                _ => err_string(
                    format!("O valor '{texto}' não é um caracter válido"),
                    charge_string,
                ),
            }
        }
        "inteiro_para_real" => {
            arity(args, 1, "Tipos.inteiro_para_real")?;
            Ok(Value::Real(expect_int(&args[0])? as f64))
        }
        "real_para_inteiro" => {
            arity(args, 1, "Tipos.real_para_inteiro")?;
            Ok(Value::Int(expect_real(&args[0])? as i32))
        }
        "inteiro_para_cadeia" => {
            arity(args, 1, "Tipos.inteiro_para_cadeia")?;
            let out = expect_int(&args[0])?.to_string();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "inteiro_para_cadeia_base" => {
            arity(args, 2, "Tipos.inteiro_para_cadeia_base")?;
            let valor = expect_int(&args[0])?;
            let base = expect_int(&args[1])?;
            let out = match base {
                2 => format!("{:b}", valor as u32),
                10 => valor.to_string(),
                16 => format!("{:X}", valor as u32),
                _ => {
                    return err_string(
                        format!("Base inválida ({base}); use 2, 10 ou 16"),
                        charge_string,
                    );
                }
            };
            check_and_charge(out.len(), charge_string)?;
            Ok(ok_value(Value::str(out)))
        }
        "real_para_cadeia" => {
            arity(args, 1, "Tipos.real_para_cadeia")?;
            let out = java_double(expect_real(&args[0])?);
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "logico_para_cadeia" => {
            arity(args, 1, "Tipos.logico_para_cadeia")?;
            let out = if expect_bool(&args[0])? {
                "verdadeiro"
            } else {
                "falso"
            };
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "caracter_para_cadeia" => {
            arity(args, 1, "Tipos.caracter_para_cadeia")?;
            let out = expect_char(&args[0])?.to_string();
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "logico_para_inteiro" => {
            arity(args, 1, "Tipos.logico_para_inteiro")?;
            Ok(Value::Int(i32::from(expect_bool(&args[0])?)))
        }
        "caracter_para_inteiro" => {
            arity(args, 1, "Tipos.caracter_para_inteiro")?;
            let ch = expect_char(&args[0])?;
            match ch.to_digit(10) {
                Some(digit) => Ok(ok_value(Value::Int(digit as i32))),
                None => err_string(
                    format!("O caracter '{ch}' não é um dígito válido"),
                    charge_string,
                ),
            }
        }
        "inteiro_para_caracter" => {
            arity(args, 1, "Tipos.inteiro_para_caracter")?;
            let valor = expect_int(&args[0])?;
            if (0..=9).contains(&valor) {
                Ok(ok_value(Value::Char(char::from(b'0' + valor as u8))))
            } else {
                err_string(
                    format!("O inteiro '{valor}' não corresponde a um dígito (0 a 9)"),
                    charge_string,
                )
            }
        }
        "cadeia_e_inteiro" => {
            arity(args, 1, "Tipos.cadeia_e_inteiro")?;
            Ok(Value::Bool(
                expect_str(&args[0])?.trim().parse::<i32>().is_ok(),
            ))
        }
        "cadeia_e_real" => {
            arity(args, 1, "Tipos.cadeia_e_real")?;
            Ok(Value::Bool(
                expect_str(&args[0])?
                    .trim()
                    .parse::<f64>()
                    .is_ok_and(f64::is_finite),
            ))
        }
        "cadeia_e_logico" => {
            arity(args, 1, "Tipos.cadeia_e_logico")?;
            Ok(Value::Bool(matches!(
                expect_str(&args[0])?.trim(),
                "verdadeiro" | "falso"
            )))
        }
        "cadeia_e_caracter" => {
            arity(args, 1, "Tipos.cadeia_e_caracter")?;
            Ok(Value::Bool(expect_str(&args[0])?.chars().count() == 1))
        }
        other => Err(format!("Função desconhecida em Tipos: `{other}`.")),
    }
}

fn call_util(
    name: &str,
    args: &[Value],
    runtime: &mut PorsRuntime,
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

struct UtcDateTime {
    year: i32,
    month: u32,
    day: u32,
    weekday_1_to_7: i32,
    hour: u32,
    minute: u32,
    second: u32,
    millis: u32,
}

fn utc_datetime_from_epoch_ms(epoch_ms: i64) -> UtcDateTime {
    let millis = epoch_ms.rem_euclid(1_000) as u32;
    let total_secs = epoch_ms.div_euclid(1_000);
    let secs_of_day = total_secs.rem_euclid(86_400) as u32;
    let days = total_secs.div_euclid(86_400);
    let (year, month, day) = civil_from_days(days);
    let weekday_1_to_7 = ((days + 4).rem_euclid(7) + 1) as i32;
    let hour = secs_of_day / 3_600;
    let minute = (secs_of_day % 3_600) / 60;
    let second = secs_of_day % 60;
    UtcDateTime {
        year,
        month,
        day,
        weekday_1_to_7,
        hour,
        minute,
        second,
        millis,
    }
}

fn civil_from_days(z: i64) -> (i32, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = (y + i64::from(m <= 2)) as i32;
    (year, m as u32, d as u32)
}

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn format_weekday_result(
    names: [&str; 7],
    args: &[Value],
    qualified: &str,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    arity(args, 3, qualified)?;
    let dia = expect_int(&args[0])?;
    let alta = expect_bool(&args[1])?;
    let baixa = expect_bool(&args[2])?;
    if !(1..=7).contains(&dia) {
        return err_string(
            format!("'{dia}' não corresponde a um dia da semana válido (1 a 7)"),
            charge_string,
        );
    }
    let raw = names[(dia - 1) as usize];
    let formatted = if alta {
        raw.to_uppercase()
    } else if baixa {
        raw.to_lowercase()
    } else {
        raw.to_owned()
    };
    check_and_charge(formatted.len(), charge_string)?;
    Ok(ok_value(Value::str(formatted)))
}

fn call_calendario(
    name: &str,
    args: &[Value],
    runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    let dt = || utc_datetime_from_epoch_ms(runtime.now_epoch_ms());
    match name {
        "ano_atual" => {
            arity(args, 0, "Calendario.ano_atual")?;
            Ok(Value::Int(dt().year))
        }
        "mes_atual" => {
            arity(args, 0, "Calendario.mes_atual")?;
            Ok(Value::Int(dt().month as i32))
        }
        "dia_mes_atual" => {
            arity(args, 0, "Calendario.dia_mes_atual")?;
            Ok(Value::Int(dt().day as i32))
        }
        "dia_semana_atual" => {
            arity(args, 0, "Calendario.dia_semana_atual")?;
            Ok(Value::Int(dt().weekday_1_to_7))
        }
        "hora_atual" => {
            arity(args, 1, "Calendario.hora_atual")?;
            let h = dt().hour as i32;
            Ok(Value::Int(if expect_bool(&args[0])? { h % 12 } else { h }))
        }
        "minuto_atual" => {
            arity(args, 0, "Calendario.minuto_atual")?;
            Ok(Value::Int(dt().minute as i32))
        }
        "segundo_atual" => {
            arity(args, 0, "Calendario.segundo_atual")?;
            Ok(Value::Int(dt().second as i32))
        }
        "milisegundo_atual" => {
            arity(args, 0, "Calendario.milisegundo_atual")?;
            Ok(Value::Int(dt().millis as i32))
        }
        "dia_semana_completo" => format_weekday_result(
            [
                "Domingo",
                "Segunda-Feira",
                "Terça-Feira",
                "Quarta-Feira",
                "Quinta-Feira",
                "Sexta-Feira",
                "Sábado",
            ],
            args,
            "Calendario.dia_semana_completo",
            charge_string,
        ),
        "dia_semana_curto" => format_weekday_result(
            [
                "Domingo", "Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado",
            ],
            args,
            "Calendario.dia_semana_curto",
            charge_string,
        ),
        "dia_semana_abreviado" => format_weekday_result(
            ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"],
            args,
            "Calendario.dia_semana_abreviado",
            charge_string,
        ),
        "nome_mes" => {
            arity(args, 1, "Calendario.nome_mes")?;
            const MESES: [&str; 12] = [
                "Janeiro",
                "Fevereiro",
                "Março",
                "Abril",
                "Maio",
                "Junho",
                "Julho",
                "Agosto",
                "Setembro",
                "Outubro",
                "Novembro",
                "Dezembro",
            ];
            let mes = expect_int(&args[0])?;
            if !(1..=12).contains(&mes) {
                return err_string(
                    format!("'{mes}' não corresponde a um mês válido (1 a 12)"),
                    charge_string,
                );
            }
            let nome = MESES[(mes - 1) as usize];
            check_and_charge(nome.len(), charge_string)?;
            Ok(ok_value(Value::str(nome)))
        }
        "eh_bissexto" => {
            arity(args, 1, "Calendario.eh_bissexto")?;
            Ok(Value::Bool(is_leap_year(expect_int(&args[0])?)))
        }
        "dias_no_mes" => {
            arity(args, 2, "Calendario.dias_no_mes")?;
            let ano = expect_int(&args[0])?;
            let mes = expect_int(&args[1])?;
            let dias = match mes {
                1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
                4 | 6 | 9 | 11 => 30,
                2 if is_leap_year(ano) => 29,
                2 => 28,
                _ => {
                    return err_string(
                        format!("'{mes}' não corresponde a um mês válido (1 a 12)"),
                        charge_string,
                    );
                }
            };
            Ok(ok_value(Value::Int(dias)))
        }
        "data_iso_atual" => {
            arity(args, 0, "Calendario.data_iso_atual")?;
            let now = dt();
            let iso = format!("{:04}-{:02}-{:02}", now.year, now.month, now.day);
            check_and_charge(iso.len(), charge_string)?;
            Ok(Value::str(iso))
        }
        other => Err(format!("Função desconhecida em Calendario: `{other}`.")),
    }
}

#[cfg(target_arch = "wasm32")]
fn call_arquivos(
    name: &str,
    _args: &[Value],
    _runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    if matches!(
        name,
        "ler_texto"
            | "ler_linhas"
            | "escrever_texto"
            | "acrescentar_texto"
            | "existe"
            | "remover"
            | "listar"
    ) {
        return err_string(
            "Acesso a arquivos não está disponível no ambiente WebAssembly.".to_string(),
            charge_string,
        );
    }
    Err(format!("Função desconhecida em Arquivos: `{name}`."))
}

#[cfg(not(target_arch = "wasm32"))]
fn resolve_workspace_path(
    runtime: &PorsRuntime,
    raw: &str,
    allow_root: bool,
) -> Result<(PathBuf, PathBuf), String> {
    let Some(root) = runtime.workspace_dir.as_deref() else {
        return Err("Acesso a arquivos requer um diretório de workspace configurado.".to_string());
    };
    if raw.contains('\0') {
        return Err("Caminho inválido: contém caractere nulo.".to_string());
    }
    let trimmed = raw.trim();
    if trimmed.is_empty() && !allow_root {
        return Err("O caminho do arquivo não pode ser vazio.".to_string());
    }
    let rel = Path::new(trimmed);
    if rel.is_absolute() {
        return Err("Apenas caminhos relativos ao workspace são permitidos.".to_string());
    }
    let mut clean = PathBuf::new();
    for component in rel.components() {
        match component {
            Component::Normal(part) => clean.push(part),
            Component::CurDir => {}
            Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                return Err(
                    "Caminho fora do workspace não é permitido (`..` ou raiz absoluta)."
                        .to_string(),
                );
            }
        }
    }
    if clean.as_os_str().is_empty() && !allow_root {
        return Err("O caminho deve apontar para um arquivo dentro do workspace.".to_string());
    }
    let canonical_root =
        std::fs::canonicalize(root).map_err(|err| format!("Workspace inacessível: {err}"))?;
    let target = canonical_root.join(&clean);

    // Garante que nenhum ancestral existente escape do workspace via symlink.
    let mut probe = target.as_path();
    while !probe.exists() {
        let Some(parent) = probe.parent() else {
            break;
        };
        probe = parent;
    }
    if probe.exists() {
        let canonical_probe = std::fs::canonicalize(probe)
            .map_err(|err| format!("Não foi possível validar o caminho: {err}"))?;
        if !canonical_probe.starts_with(&canonical_root) {
            return Err("Caminho escapa do workspace (link simbólico bloqueado).".to_string());
        }
    }
    Ok((canonical_root, target))
}

#[cfg(not(target_arch = "wasm32"))]
fn call_arquivos(
    name: &str,
    args: &[Value],
    runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "ler_texto" => {
            arity(args, 1, "Arquivos.ler_texto")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let meta = match std::fs::metadata(&target) {
                Ok(meta) => meta,
                Err(err) => {
                    return err_string(
                        format!("Não foi possível abrir '{path}': {err}"),
                        charge_string,
                    );
                }
            };
            if !meta.is_file() {
                return err_string(format!("'{path}' não é um arquivo regular."), charge_string);
            }
            if meta.len() as usize > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excede o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            check_and_charge(meta.len() as usize, charge_string)?;
            match std::fs::read_to_string(&target) {
                Ok(content) => Ok(ok_value(Value::str(content))),
                Err(err) => err_string(format!("Erro ao ler '{path}': {err}"), charge_string),
            }
        }
        "ler_linhas" => {
            arity(args, 1, "Arquivos.ler_linhas")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let meta = match std::fs::metadata(&target) {
                Ok(meta) => meta,
                Err(err) => {
                    return err_string(
                        format!("Não foi possível abrir '{path}': {err}"),
                        charge_string,
                    );
                }
            };
            if !meta.is_file() {
                return err_string(format!("'{path}' não é um arquivo regular."), charge_string);
            }
            if meta.len() as usize > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excede o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            check_and_charge(meta.len() as usize, charge_string)?;
            match std::fs::read_to_string(&target) {
                Ok(content) => {
                    let lines: Vec<Value> = content.lines().map(Value::str).collect();
                    Ok(ok_value(Value::Array(Rc::new(RefCell::new(lines)))))
                }
                Err(err) => err_string(format!("Erro ao ler '{path}': {err}"), charge_string),
            }
        }
        "escrever_texto" => {
            arity(args, 2, "Arquivos.escrever_texto")?;
            let path = expect_str(&args[0])?;
            let content = expect_str(&args[1])?;
            if content.len() > MAX_FILE_BYTES {
                return err_string(
                    format!("O conteúdo excede o limite de {MAX_FILE_BYTES} bytes por arquivo."),
                    charge_string,
                );
            }
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            if let Some(parent) = target.parent()
                && let Err(err) = std::fs::create_dir_all(parent)
            {
                return err_string(
                    format!("Não foi possível criar diretório para '{path}': {err}"),
                    charge_string,
                );
            }
            match std::fs::write(&target, content) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(format!("Erro ao gravar '{path}': {err}"), charge_string),
            }
        }
        "acrescentar_texto" => {
            arity(args, 2, "Arquivos.acrescentar_texto")?;
            let path = expect_str(&args[0])?;
            let content = expect_str(&args[1])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let existing_len = std::fs::metadata(&target)
                .map(|m| m.len() as usize)
                .unwrap_or(0);
            if existing_len.saturating_add(content.len()) > MAX_FILE_BYTES {
                return err_string(
                    format!("O arquivo '{path}' excederia o limite de {MAX_FILE_BYTES} bytes."),
                    charge_string,
                );
            }
            if let Some(parent) = target.parent()
                && let Err(err) = std::fs::create_dir_all(parent)
            {
                return err_string(
                    format!("Não foi possível criar diretório para '{path}': {err}"),
                    charge_string,
                );
            }
            use std::io::Write;
            let file = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(&target);
            match file.and_then(|mut handle| handle.write_all(content.as_bytes())) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(
                    format!("Erro ao acrescentar em '{path}': {err}"),
                    charge_string,
                ),
            }
        }
        "existe" => {
            arity(args, 1, "Arquivos.existe")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, true) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            Ok(ok_value(Value::Bool(target.exists())))
        }
        "remover" => {
            arity(args, 1, "Arquivos.remover")?;
            let path = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, path, false) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            match std::fs::remove_file(&target) {
                Ok(()) => Ok(ok_value(Value::Bool(true))),
                Err(err) => err_string(format!("Erro ao remover '{path}': {err}"), charge_string),
            }
        }
        "listar" => {
            arity(args, 1, "Arquivos.listar")?;
            let dir = expect_str(&args[0])?;
            let (_, target) = match resolve_workspace_path(runtime, dir, true) {
                Ok(pair) => pair,
                Err(msg) => return err_string(msg, charge_string),
            };
            let entries = match std::fs::read_dir(&target) {
                Ok(entries) => entries,
                Err(err) => {
                    return err_string(format!("Erro ao listar '{dir}': {err}"), charge_string);
                }
            };
            let mut names = Vec::new();
            let mut total_bytes = 0usize;
            for entry in entries {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(err) => {
                        return err_string(format!("Erro ao listar '{dir}': {err}"), charge_string);
                    }
                };
                if names.len() >= MAX_LIST_ENTRIES {
                    return err_string(
                        format!("Diretório excede o limite de {MAX_LIST_ENTRIES} entradas."),
                        charge_string,
                    );
                }
                let item = entry.file_name().to_string_lossy().into_owned();
                total_bytes = total_bytes.saturating_add(item.len());
                names.push(item);
            }
            names.sort();
            check_and_charge(total_bytes, charge_string)?;
            let values: Vec<Value> = names.into_iter().map(Value::str).collect();
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(values)))))
        }
        other => Err(format!("Função desconhecida em Arquivos: `{other}`.")),
    }
}

enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

struct JsonParser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    fn parse(src: &'a str) -> Result<JsonValue, String> {
        if src.len() > MAX_JSON_BYTES {
            return Err(format!(
                "Documento JSON excede o limite de {MAX_JSON_BYTES} bytes."
            ));
        }
        let mut parser = Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        };
        parser.skip_ws();
        let value = parser.parse_value(0)?;
        parser.skip_ws();
        if parser.pos != parser.bytes.len() {
            return Err("Conteúdo inesperado após o término do JSON.".to_string());
        }
        Ok(value)
    }

    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(depth + 1),
            Some(b'[') => self.parse_array(depth + 1),
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b't') => {
                self.expect_bytes(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.expect_bytes(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.expect_bytes(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(format!(
                "Caractere inesperado no JSON na posição {}.",
                self.pos
            )),
            None => Err("Fim inesperado do JSON.".to_string()),
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.pos..self.pos + expected.len()) == Some(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Token JSON inválido na posição {}.", self.pos))
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.pos += 1; // '{'
        self.skip_ws();
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(format!(
                    "Esperada chave de texto no objeto JSON na posição {}.",
                    self.pos
                ));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(format!(
                    "Esperado `:` após chave JSON na posição {}.",
                    self.pos
                ));
            }
            self.pos += 1;
            let val = self.parse_value(depth)?;
            if let Some(existing) = entries.iter_mut().find(|(k, _)| k == &key) {
                existing.1 = val;
            } else {
                entries.push((key, val));
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => {
                    return Err(format!(
                        "Esperado `,` ou `}}` no objeto JSON na posição {}.",
                        self.pos
                    ));
                }
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.pos += 1; // '['
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let item = self.parse_value(depth)?;
            items.push(item);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => {
                    return Err(format!(
                        "Esperado `,` ou `]` na lista JSON na posição {}.",
                        self.pos
                    ));
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        let Some(slice) = self.src.get(self.pos..self.pos + 4) else {
            return Err("Escape Unicode `\\u` incompleto no JSON.".to_string());
        };
        let code = u16::from_str_radix(slice, 16)
            .map_err(|_| "Escape Unicode `\\u` inválido no JSON.".to_string())?;
        self.pos += 4;
        Ok(code)
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // '"'
        let mut out = String::new();
        let mut chunk_start = self.pos;
        while let Some(&b) = self.bytes.get(self.pos) {
            match b {
                b'"' => {
                    out.push_str(&self.src[chunk_start..self.pos]);
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    out.push_str(&self.src[chunk_start..self.pos]);
                    self.pos += 1;
                    let esc = self
                        .peek()
                        .ok_or_else(|| "Escape incompleto na cadeia JSON.".to_string())?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.parse_hex4()?;
                            if (0xD800..=0xDBFF).contains(&first) {
                                if self.bytes.get(self.pos..self.pos + 2) != Some(b"\\u") {
                                    return Err(
                                        "Par substituto UTF-16 incompleto no JSON.".to_string()
                                    );
                                }
                                self.pos += 2;
                                let second = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(
                                        "Par substituto UTF-16 inválido no JSON.".to_string()
                                    );
                                }
                                let scalar = 0x10000
                                    + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                                let ch = char::from_u32(scalar).ok_or_else(|| {
                                    "Código Unicode inválido no JSON.".to_string()
                                })?;
                                out.push(ch);
                            } else if (0xDC00..=0xDFFF).contains(&first) {
                                return Err(
                                    "Substituto UTF-16 isolado inválido no JSON.".to_string()
                                );
                            } else {
                                let ch = char::from_u32(first as u32).ok_or_else(|| {
                                    "Código Unicode inválido no JSON.".to_string()
                                })?;
                                out.push(ch);
                            }
                        }
                        _ => return Err("Sequência de escape inválida na cadeia JSON.".to_string()),
                    }
                    chunk_start = self.pos;
                }
                0x00..=0x1F => {
                    return Err("Caractere de controle não escapado na cadeia JSON.".to_string());
                }
                _ => self.pos += 1,
            }
        }
        Err("Cadeia JSON não fechada.".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(format!("Número JSON inválido na posição {}.", self.pos)),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(format!(
                    "Fração decimal inválida no JSON na posição {}.",
                    self.pos
                ));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(format!(
                    "Expoente inválido no JSON na posição {}.",
                    self.pos
                ));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let token = &self.src[start..self.pos];
        if !is_float && let Ok(n) = token.parse::<i64>() {
            return Ok(JsonValue::Int(n));
        }
        let f = token
            .parse::<f64>()
            .map_err(|_| format!("Número JSON inválido: `{token}`."))?;
        if !f.is_finite() {
            return Err(format!("Número JSON fora do intervalo finito: `{token}`."));
        }
        Ok(JsonValue::Float(f))
    }
}

fn json_escape_into(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_json(
    value: &JsonValue,
    pretty: bool,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    if out.len() > MAX_JSON_BYTES {
        return Err(format!(
            "Saída JSON excede o limite de {MAX_JSON_BYTES} bytes."
        ));
    }
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Int(n) => {
            use std::fmt::Write;
            let _ = write!(out, "{n}");
        }
        JsonValue::Float(f) => {
            if !f.is_finite() {
                return Err("Valores NaN ou infinitos não são válidos em JSON.".to_string());
            }
            let rendered = format!("{f}");
            out.push_str(&rendered);
            if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                out.push_str(".0");
            }
        }
        JsonValue::Str(s) => json_escape_into(s, out),
        JsonValue::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent + 2 {
                        out.push(' ');
                    }
                }
                write_json(item, pretty, indent + 2, out)?;
            }
            if pretty {
                out.push('\n');
                for _ in 0..indent {
                    out.push(' ');
                }
            }
            out.push(']');
        }
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            out.push('{');
            for (index, (k, v)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent + 2 {
                        out.push(' ');
                    }
                }
                json_escape_into(k, out);
                out.push(':');
                if pretty {
                    out.push(' ');
                }
                write_json(v, pretty, indent + 2, out)?;
            }
            if pretty {
                out.push('\n');
                for _ in 0..indent {
                    out.push(' ');
                }
            }
            out.push('}');
        }
    }
    if out.len() > MAX_JSON_BYTES {
        return Err(format!(
            "Saída JSON excede o limite de {MAX_JSON_BYTES} bytes."
        ));
    }
    Ok(())
}

fn serialize_json_value(value: &JsonValue, pretty: bool) -> Result<String, String> {
    let mut out = String::new();
    write_json(value, pretty, 0, &mut out)?;
    Ok(out)
}

fn parse_json_object(src: &str) -> Result<Vec<(String, JsonValue)>, String> {
    match JsonParser::parse(src)? {
        JsonValue::Object(entries) => Ok(entries),
        _ => Err("O documento JSON precisa ser um objeto (`{ ... }`).".to_string()),
    }
}

fn json_get_entry<'a>(
    entries: &'a [(String, JsonValue)],
    key: &str,
) -> Result<&'a JsonValue, String> {
    entries
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("Chave '{key}' não encontrada no objeto JSON."))
}

fn json_set_key(
    src: &str,
    key: &str,
    value: JsonValue,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    let mut entries = match parse_json_object(src) {
        Ok(entries) => entries,
        Err(msg) => return err_string(msg, charge_string),
    };
    if let Some(existing) = entries.iter_mut().find(|(k, _)| k == key) {
        existing.1 = value;
    } else {
        entries.push((key.to_owned(), value));
    }
    let rendered = match serialize_json_value(&JsonValue::Object(entries), false) {
        Ok(text) => text,
        Err(msg) => return err_string(msg, charge_string),
    };
    check_and_charge(rendered.len(), charge_string)?;
    Ok(ok_value(Value::str(rendered)))
}

fn call_json(
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "valido" => {
            arity(args, 1, "Json.valido")?;
            Ok(Value::Bool(
                JsonParser::parse(expect_str(&args[0])?).is_ok(),
            ))
        }
        "formatar" => {
            arity(args, 1, "Json.formatar")?;
            let value = match JsonParser::parse(expect_str(&args[0])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(&value, true) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "minificar" => {
            arity(args, 1, "Json.minificar")?;
            let value = match JsonParser::parse(expect_str(&args[0])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(&value, false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "escapar_cadeia" => {
            arity(args, 1, "Json.escapar_cadeia")?;
            let raw = expect_str(&args[0])?;
            let mut out = String::with_capacity(raw.len().saturating_add(2));
            json_escape_into(raw, &mut out);
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "contem_chave" => {
            arity(args, 2, "Json.contem_chave")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            Ok(ok_value(Value::Bool(entries.iter().any(|(k, _)| k == key))))
        }
        "obter_chaves" => {
            arity(args, 1, "Json.obter_chaves")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let total_bytes: usize = entries.iter().map(|(k, _)| k.len()).sum();
            check_and_charge(total_bytes, charge_string)?;
            let keys: Vec<Value> = entries.into_iter().map(|(k, _)| Value::str(k)).collect();
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(keys)))))
        }
        "obter_cadeia" => {
            arity(args, 2, "Json.obter_cadeia")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Str(s)) => {
                    check_and_charge(s.len(), charge_string)?;
                    Ok(ok_value(Value::str(s.clone())))
                }
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém uma cadeia no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_inteiro" => {
            arity(args, 2, "Json.obter_inteiro")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Int(n)) => match i32::try_from(*n) {
                    Ok(v) => Ok(ok_value(Value::Int(v))),
                    Err(_) => err_string(
                        format!("O valor de '{key}' excede o intervalo de `inteiro` (32 bits)."),
                        charge_string,
                    ),
                },
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um número inteiro no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_real" => {
            arity(args, 2, "Json.obter_real")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Int(n)) => Ok(ok_value(Value::Real(*n as f64))),
                Ok(JsonValue::Float(f)) => Ok(ok_value(Value::Real(*f))),
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um número no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_logico" => {
            arity(args, 2, "Json.obter_logico")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Bool(b)) => Ok(ok_value(Value::Bool(*b))),
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um valor lógico no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_json" => {
            arity(args, 2, "Json.obter_json")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let value = match json_get_entry(&entries, key) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(value, false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "obter_lista_cadeia" => {
            arity(args, 2, "Json.obter_lista_cadeia")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let items = match json_get_entry(&entries, key) {
                Ok(JsonValue::Array(items)) => items,
                Ok(_) => {
                    return err_string(
                        format!("A chave '{key}' não contém uma lista no JSON."),
                        charge_string,
                    );
                }
                Err(msg) => return err_string(msg, charge_string),
            };
            let mut out = Vec::with_capacity(items.len());
            let mut total_bytes = 0usize;
            for item in items {
                match item {
                    JsonValue::Str(s) => {
                        total_bytes = total_bytes.saturating_add(s.len());
                        out.push(Value::str(s.clone()));
                    }
                    _ => {
                        return err_string(
                            format!("A lista em '{key}' contém elementos que não são cadeias."),
                            charge_string,
                        );
                    }
                }
            }
            check_and_charge(total_bytes, charge_string)?;
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(out)))))
        }
        "obter_lista_inteiro" => {
            arity(args, 2, "Json.obter_lista_inteiro")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let items = match json_get_entry(&entries, key) {
                Ok(JsonValue::Array(items)) => items,
                Ok(_) => {
                    return err_string(
                        format!("A chave '{key}' não contém uma lista no JSON."),
                        charge_string,
                    );
                }
                Err(msg) => return err_string(msg, charge_string),
            };
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    JsonValue::Int(n) => match i32::try_from(*n) {
                        Ok(v) => out.push(Value::Int(v)),
                        Err(_) => {
                            return err_string(
                                format!(
                                    "A lista em '{key}' contém inteiro fora do intervalo de 32 bits."
                                ),
                                charge_string,
                            );
                        }
                    },
                    _ => {
                        return err_string(
                            format!("A lista em '{key}' contém elementos que não são inteiros."),
                            charge_string,
                        );
                    }
                }
            }
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(out)))))
        }
        "definir_cadeia" => {
            arity(args, 3, "Json.definir_cadeia")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Str(expect_str(&args[2])?.to_owned()),
                charge_string,
            )
        }
        "definir_inteiro" => {
            arity(args, 3, "Json.definir_inteiro")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Int(i64::from(expect_int(&args[2])?)),
                charge_string,
            )
        }
        "definir_real" => {
            arity(args, 3, "Json.definir_real")?;
            let f = expect_real(&args[2])?;
            if !f.is_finite() {
                return err_string(
                    "Valores NaN ou infinitos não podem ser gravados em JSON.".to_string(),
                    charge_string,
                );
            }
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Float(f),
                charge_string,
            )
        }
        "definir_logico" => {
            arity(args, 3, "Json.definir_logico")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Bool(expect_bool(&args[2])?),
                charge_string,
            )
        }
        "definir_json" => {
            arity(args, 3, "Json.definir_json")?;
            let nested = match JsonParser::parse(expect_str(&args[2])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                nested,
                charge_string,
            )
        }
        "remover_chave" => {
            arity(args, 2, "Json.remover_chave")?;
            let mut entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            entries.retain(|(k, _)| k != key);
            let rendered = match serialize_json_value(&JsonValue::Object(entries), false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        other => Err(format!("Função desconhecida em Json: `{other}`.")),
    }
}

#[cfg(target_arch = "wasm32")]
fn call_internet(
    name: &str,
    args: &[Value],
    _runtime: &mut PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
    _check_cancel: &mut impl FnMut() -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "definir_tempo_limite" => {
            arity(args, 1, "Internet.definir_tempo_limite")?;
            let _ = expect_int(&args[0])?;
            Ok(Value::Void)
        }
        "obter_texto" | "enviar_json" | "endereco_disponivel" => err_string(
            "Acesso direto à rede não está disponível no ambiente WebAssembly.".to_string(),
            charge_string,
        ),
        other => Err(format!("Função desconhecida em Internet: `{other}`.")),
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn validate_http_url(url: &str) -> Result<(), String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("A URL não pode ser vazia.".to_string());
    }
    if trimmed
        .bytes()
        .any(|b| matches!(b, b'\r' | b'\n' | b'\0' | b' '))
    {
        return Err("URL inválida: contém espaços ou caracteres de controle.".to_string());
    }
    if !(trimmed.starts_with("http://") || trimmed.starts_with("https://")) {
        return Err("Apenas endereços http:// ou https:// são permitidos.".to_string());
    }
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
enum HttpOutcome {
    Text(String),
    Available(bool),
}

#[cfg(not(target_arch = "wasm32"))]
fn run_cancellable_http(
    check_cancel: &mut impl FnMut() -> Result<(), String>,
    work: impl FnOnce() -> Result<HttpOutcome, String> + Send + 'static,
) -> Result<Result<HttpOutcome, String>, String> {
    check_cancel()?;
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    std::thread::spawn(move || {
        let _ = tx.send(work());
    });
    loop {
        match rx.recv_timeout(Duration::from_millis(25)) {
            Ok(outcome) => return Ok(outcome),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                check_cancel()?;
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                return Ok(Err(
                    "Falha interna ao aguardar resposta da requisição HTTP.".to_string(),
                ));
            }
        }
    }
}

#[cfg(all(not(target_arch = "wasm32"), feature = "stdlib"))]
fn execute_http_get(url: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    use std::io::Read;
    let timeout = Duration::from_millis(timeout_ms);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let response = agent
        .get(url)
        .set("User-Agent", "portugol-studio-pors/1.0")
        .call()
        .map_err(|err| format!("Falha ao acessar '{url}': {err}"))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take((MAX_HTTP_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Erro ao ler resposta de '{url}': {err}"))?;
    if bytes.len() > MAX_HTTP_BYTES {
        return Err(format!(
            "A resposta de '{url}' excede o limite de {MAX_HTTP_BYTES} bytes."
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("A resposta de '{url}' não é texto UTF-8 válido."))?;
    Ok(HttpOutcome::Text(text))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "stdlib"))]
fn execute_http_post_json(url: &str, body: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    use std::io::Read;
    let timeout = Duration::from_millis(timeout_ms);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let response = agent
        .post(url)
        .set("User-Agent", "portugol-studio-pors/1.0")
        .set("Content-Type", "application/json; charset=utf-8")
        .send_string(body)
        .map_err(|err| format!("Falha ao enviar JSON para '{url}': {err}"))?;
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take((MAX_HTTP_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(|err| format!("Erro ao ler resposta de '{url}': {err}"))?;
    if bytes.len() > MAX_HTTP_BYTES {
        return Err(format!(
            "A resposta de '{url}' excede o limite de {MAX_HTTP_BYTES} bytes."
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("A resposta de '{url}' não é texto UTF-8 válido."))?;
    Ok(HttpOutcome::Text(text))
}

#[cfg(all(not(target_arch = "wasm32"), feature = "stdlib"))]
fn execute_http_head(url: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    let timeout = Duration::from_millis(timeout_ms);
    let agent = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout)
        .timeout_write(timeout)
        .build();
    let available = match agent
        .head(url)
        .set("User-Agent", "portugol-studio-pors/1.0")
        .call()
    {
        Ok(resp) => (200..400).contains(&resp.status()),
        Err(ureq::Error::Status(code, _)) => (200..400).contains(&code),
        Err(_) => false,
    };
    Ok(HttpOutcome::Available(available))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn parse_plain_http_url(url: &str) -> Result<(String, u16, String), String> {
    if url.starts_with("https://") {
        return Err(
            "HTTPS requer o runtime completo (`stdlib`); na VM mínima apenas http:// está disponível."
                .to_string(),
        );
    }
    let rest = url
        .strip_prefix("http://")
        .ok_or_else(|| "Apenas endereços http:// ou https:// são permitidos.".to_string())?;
    let (authority, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };
    if authority.is_empty() {
        return Err("Endereço HTTP sem host válido.".to_string());
    }
    let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
        let port = p
            .parse::<u16>()
            .map_err(|_| format!("Porta inválida na URL: '{p}'."))?;
        (h, port)
    } else {
        (authority, 80)
    };
    Ok((host.to_owned(), port, path.to_owned()))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn decode_chunked_body(raw: &[u8]) -> Result<Vec<u8>, String> {
    let mut out = Vec::new();
    let mut pos = 0usize;
    while pos < raw.len() {
        let Some(rel_crlf) = raw[pos..].windows(2).position(|w| w == b"\r\n") else {
            return Err("Resposta HTTP chunked malformada.".to_string());
        };
        let size_str = std::str::from_utf8(&raw[pos..pos + rel_crlf])
            .map_err(|_| "Tamanho de bloco chunked inválido.".to_string())?
            .split(';')
            .next()
            .unwrap_or("")
            .trim();
        let size = usize::from_str_radix(size_str, 16)
            .map_err(|_| "Tamanho hexadecimal de bloco chunked inválido.".to_string())?;
        pos += rel_crlf + 2;
        if size == 0 {
            break;
        }
        if out.len().saturating_add(size) > MAX_HTTP_BYTES {
            return Err(format!(
                "A resposta HTTP excede o limite de {MAX_HTTP_BYTES} bytes."
            ));
        }
        if pos + size + 2 > raw.len() {
            return Err("Bloco HTTP chunked truncado.".to_string());
        }
        out.extend_from_slice(&raw[pos..pos + size]);
        pos += size + 2;
    }
    Ok(out)
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn plain_http_exchange(
    method: &str,
    url: &str,
    body: Option<&str>,
    timeout_ms: u64,
) -> Result<(u16, Vec<u8>), String> {
    use std::io::{Read, Write};
    use std::net::{TcpStream, ToSocketAddrs};
    let (host, port, path) = parse_plain_http_url(url)?;
    let timeout = Duration::from_millis(timeout_ms);
    let addr = (host.as_str(), port)
        .to_socket_addrs()
        .map_err(|err| format!("Não foi possível resolver '{host}': {err}"))?
        .next()
        .ok_or_else(|| format!("Host '{host}' não resolvido."))?;
    let mut stream = TcpStream::connect_timeout(&addr, timeout)
        .map_err(|err| format!("Falha ao conectar em '{url}': {err}"))?;
    let _ = stream.set_read_timeout(Some(timeout));
    let _ = stream.set_write_timeout(Some(timeout));
    let mut req = format!(
        "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nUser-Agent: portugol-studio-pors/1.0\r\nConnection: close\r\n"
    );
    if let Some(payload) = body {
        req.push_str(&format!(
            "Content-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n\r\n{payload}",
            payload.len()
        ));
    } else {
        req.push_str("\r\n");
    }
    stream
        .write_all(req.as_bytes())
        .map_err(|err| format!("Erro ao enviar requisição para '{url}': {err}"))?;
    let mut raw = Vec::new();
    let max_wire = MAX_HTTP_BYTES + 16 * 1024;
    stream
        .take((max_wire + 1) as u64)
        .read_to_end(&mut raw)
        .map_err(|err| format!("Erro ao ler resposta de '{url}': {err}"))?;
    let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
        return Err(format!("Resposta HTTP inválida de '{url}'."));
    };
    let head = std::str::from_utf8(&raw[..header_end])
        .map_err(|_| format!("Cabeçalhos HTTP inválidos de '{url}'."))?;
    let status = head
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .and_then(|code| code.parse::<u16>().ok())
        .ok_or_else(|| format!("Linha de status HTTP inválida de '{url}'."))?;
    let is_chunked = head.lines().any(|line| {
        line.to_ascii_lowercase().starts_with("transfer-encoding:")
            && line.to_ascii_lowercase().contains("chunked")
    });
    let body_raw = &raw[header_end + 4..];
    let body_bytes = if is_chunked {
        decode_chunked_body(body_raw)?
    } else {
        if body_raw.len() > MAX_HTTP_BYTES {
            return Err(format!(
                "A resposta de '{url}' excede o limite de {MAX_HTTP_BYTES} bytes."
            ));
        }
        body_raw.to_vec()
    };
    Ok((status, body_bytes))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn execute_http_get(url: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    let (status, bytes) = plain_http_exchange("GET", url, None, timeout_ms)?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "Requisição GET para '{url}' retornou HTTP {status}."
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("A resposta de '{url}' não é texto UTF-8 válido."))?;
    Ok(HttpOutcome::Text(text))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn execute_http_post_json(url: &str, body: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    let (status, bytes) = plain_http_exchange("POST", url, Some(body), timeout_ms)?;
    if !(200..300).contains(&status) {
        return Err(format!(
            "Requisição POST para '{url}' retornou HTTP {status}."
        ));
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| format!("A resposta de '{url}' não é texto UTF-8 válido."))?;
    Ok(HttpOutcome::Text(text))
}

#[cfg(all(not(target_arch = "wasm32"), not(feature = "stdlib")))]
fn execute_http_head(url: &str, timeout_ms: u64) -> Result<HttpOutcome, String> {
    let available = match plain_http_exchange("HEAD", url, None, timeout_ms) {
        Ok((status, _)) => (200..400).contains(&status),
        Err(_) => false,
    };
    Ok(HttpOutcome::Available(available))
}

#[cfg(not(target_arch = "wasm32"))]
fn call_internet(
    name: &str,
    args: &[Value],
    runtime: &mut PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
    check_cancel: &mut impl FnMut() -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "definir_tempo_limite" => {
            arity(args, 1, "Internet.definir_tempo_limite")?;
            let ms = expect_int(&args[0])?;
            runtime.http_timeout_ms =
                (ms.max(0) as u64).clamp(MIN_HTTP_TIMEOUT_MS, MAX_HTTP_TIMEOUT_MS);
            Ok(Value::Void)
        }
        "obter_texto" => {
            arity(args, 1, "Internet.obter_texto")?;
            if !runtime.network_enabled {
                return err_string(
                    "Acesso à rede bloqueado: habilite permissão explícita de Internet para este programa."
                        .to_string(),
                    charge_string,
                );
            }
            let url = expect_str(&args[0])?.trim().to_owned();
            if let Err(msg) = validate_http_url(&url) {
                return err_string(msg, charge_string);
            }
            let timeout = runtime.http_timeout_ms;
            match run_cancellable_http(check_cancel, move || execute_http_get(&url, timeout))? {
                Ok(HttpOutcome::Text(text)) => {
                    check_and_charge(text.len(), charge_string)?;
                    Ok(ok_value(Value::str(text)))
                }
                Ok(HttpOutcome::Available(_)) => unreachable!(),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "enviar_json" => {
            arity(args, 2, "Internet.enviar_json")?;
            if !runtime.network_enabled {
                return err_string(
                    "Acesso à rede bloqueado: habilite permissão explícita de Internet para este programa."
                        .to_string(),
                    charge_string,
                );
            }
            let url = expect_str(&args[0])?.trim().to_owned();
            if let Err(msg) = validate_http_url(&url) {
                return err_string(msg, charge_string);
            }
            let payload = expect_str(&args[1])?.to_owned();
            if payload.len() > MAX_HTTP_BYTES {
                return err_string(
                    format!("O corpo JSON excede o limite de {MAX_HTTP_BYTES} bytes."),
                    charge_string,
                );
            }
            if let Err(msg) = JsonParser::parse(&payload) {
                return err_string(format!("Corpo JSON inválido: {msg}"), charge_string);
            }
            let timeout = runtime.http_timeout_ms;
            match run_cancellable_http(check_cancel, move || {
                execute_http_post_json(&url, &payload, timeout)
            })? {
                Ok(HttpOutcome::Text(text)) => {
                    check_and_charge(text.len(), charge_string)?;
                    Ok(ok_value(Value::str(text)))
                }
                Ok(HttpOutcome::Available(_)) => unreachable!(),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "endereco_disponivel" => {
            arity(args, 1, "Internet.endereco_disponivel")?;
            if !runtime.network_enabled {
                return err_string(
                    "Acesso à rede bloqueado: habilite permissão explícita de Internet para este programa."
                        .to_string(),
                    charge_string,
                );
            }
            let url = expect_str(&args[0])?.trim().to_owned();
            if let Err(msg) = validate_http_url(&url) {
                return err_string(msg, charge_string);
            }
            let timeout = runtime.http_timeout_ms;
            match run_cancellable_http(check_cancel, move || execute_http_head(&url, timeout))? {
                Ok(HttpOutcome::Available(ok)) => Ok(ok_value(Value::Bool(ok))),
                Ok(HttpOutcome::Text(_)) => unreachable!(),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        other => Err(format!("Função desconhecida em Internet: `{other}`.")),
    }
}

fn multimedia_returns_result(library: &str, name: &str) -> bool {
    matches!(
        (library, name),
        (
            "PorsGraficos",
            "renderizar_imagem"
                | "criar_cor"
                | "desenhar_poligono"
                | "carregar_fonte"
                | "carregar_imagem"
                | "largura_imagem"
                | "altura_imagem"
                | "transformar_imagem"
                | "transformar_porcao_imagem"
                | "redimensionar_imagem"
                | "obter_cor_pixel"
                | "salvar_imagem"
                | "obter_intervalo_gif"
                | "obter_numero_quadros_gif"
                | "obter_numero_quadro_atual_gif"
                | "obter_quadro_gif"
                | "definir_quadro_gif"
        ) | ("PorsTeclado", "ler_tecla")
            | ("PorsMouse", "ler_botao")
            | (
                "PorsSons",
                "carregar_som" | "reproduzir_som" | "definir_posicao_atual_musica"
            )
    )
}

fn multimedia_display_module(library: &str) -> Option<&'static str> {
    match library {
        "PorsGraficos" => Some("Graficos"),
        "PorsTeclado" => Some("Teclado"),
        "PorsMouse" => Some("Mouse"),
        "PorsSons" => Some("Sons"),
        _ => None,
    }
}

#[cfg(feature = "stdlib")]
pub(crate) fn call_multimedia(
    library: &str,
    name: &str,
    args: &[Value],
    ctx: &mut crate::stdlib::NativeContext,
    runtime: &PorsRuntime,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Option<Result<Value, String>> {
    let stdlib_library = multimedia_display_module(library)?;
    let returns_result = multimedia_returns_result(library, name);

    if library == "PorsGraficos" && name == "desenhar_poligono" {
        return Some(call_pors_desenhar_poligono(args, ctx, charge_string));
    }

    let stdlib_name = match (library, name) {
        ("PorsGraficos", "obter_rgb") => "obter_RGB",
        _ => name,
    };

    let Some(function) = crate::stdlib::native(stdlib_library, stdlib_name) else {
        return Some(Err(format!(
            "Função desconhecida em {stdlib_library}: `{name}`."
        )));
    };

    // Confina leitura/gravação de ativos multimídia ao workspace configurado.
    let mut owned_args: Option<Vec<Value>> = None;
    let path_arg_index: Option<(usize, bool)> = match (library, name) {
        ("PorsGraficos", "carregar_imagem" | "carregar_fonte") => Some((0, false)),
        ("PorsGraficos", "salvar_imagem") => Some((1, true)),
        ("PorsSons", "carregar_som") => Some((0, false)),
        _ => None,
    };
    if let Some((index, allow_missing)) = path_arg_index
        && let Some(raw_value) = args.get(index)
    {
        let raw_path = match expect_str(raw_value) {
            Ok(path) => path,
            Err(err) => return Some(Err(err)),
        };
        let resolved = match resolve_workspace_path(runtime, raw_path, allow_missing) {
            Ok((_, path)) => path,
            Err(msg) => return Some(err_string(msg, charge_string)),
        };
        let mut updated = args.to_vec();
        updated[index] = Value::str(resolved.to_string_lossy().into_owned());
        owned_args = Some(updated);
    }

    let call_args = owned_args.as_deref().unwrap_or(args);
    let outcome = function(ctx, call_args);
    if !returns_result {
        return Some(outcome);
    }
    Some(match outcome {
        Ok(Value::Void) => Ok(ok_value(Value::Bool(true))),
        Ok(value) => Ok(ok_value(value)),
        Err(message) => {
            if ctx.interrupt.load(std::sync::atomic::Ordering::Relaxed) {
                Err("O programa foi interrompido.".to_string())
            } else {
                err_string(message, charge_string)
            }
        }
    })
}

#[cfg(feature = "stdlib")]
fn call_pors_desenhar_poligono(
    args: &[Value],
    ctx: &mut crate::stdlib::NativeContext,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    arity(args, 3, "Graficos.desenhar_poligono")?;
    let (Value::Array(xs_ref), Value::Array(ys_ref), Value::Bool(fill)) =
        (&args[0], &args[1], &args[2])
    else {
        return Err(
            "`Graficos.desenhar_poligono` espera (vetor<inteiro>, vetor<inteiro>, logico)."
                .to_string(),
        );
    };
    let xs = xs_ref.borrow();
    let ys = ys_ref.borrow();
    if xs.len() != ys.len() {
        return err_string(
            format!(
                "Os vetores de coordenadas X ({}) e Y ({}) precisam ter o mesmo tamanho.",
                xs.len(),
                ys.len()
            ),
            charge_string,
        );
    }
    if xs.len() < 3 {
        return err_string(
            "Um polígono precisa de pelo menos 3 vértices.".to_string(),
            charge_string,
        );
    }
    let mut data = Vec::with_capacity(xs.len() * 2);
    for (x, y) in xs.iter().zip(ys.iter()) {
        data.push(x.clone());
        data.push(y.clone());
    }
    let rows = xs.len();
    drop(xs);
    drop(ys);
    let matrix = Value::Matrix(Rc::new(RefCell::new(crate::value::Matrix {
        rows,
        cols: 2,
        data,
    })));
    let Some(function) = crate::stdlib::native("Graficos", "desenhar_poligono") else {
        return Err("Função desconhecida em Graficos: `desenhar_poligono`.".to_string());
    };
    match function(ctx, &[matrix, Value::Bool(*fill)]) {
        Ok(_) => Ok(ok_value(Value::Bool(true))),
        Err(msg) => err_string(msg, charge_string),
    }
}

#[cfg(not(feature = "stdlib"))]
pub(crate) fn call_multimedia_unavailable(
    library: &str,
    name: &str,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Option<Result<Value, String>> {
    let display = multimedia_display_module(library)?;
    let message = format!(
        "A biblioteca `{display}` (`{display}.{name}`) requer o host multimídia da VM (feature `stdlib`)."
    );
    if multimedia_returns_result(library, name) {
        Some(err_string(message, charge_string))
    } else {
        Some(Err(message))
    }
}
