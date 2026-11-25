#[cfg(not(target_arch = "wasm32"))]
use std::time::Duration;

use crate::value::Value;

#[cfg(not(target_arch = "wasm32"))]
use super::json::JsonParser;
use super::{PorsRuntime, arity, err_string, expect_int};
#[cfg(not(target_arch = "wasm32"))]
use super::{check_and_charge, expect_str, ok_value};

/// Limite máximo de resposta HTTP em `Internet.*` (1 MiB).
#[cfg(not(target_arch = "wasm32"))]
const MAX_HTTP_BYTES: usize = 1024 * 1024;
/// Timeout mínimo e máximo permitidos para `Internet.definir_tempo_limite`.
#[cfg(not(target_arch = "wasm32"))]
const MIN_HTTP_TIMEOUT_MS: u64 = 100;
#[cfg(not(target_arch = "wasm32"))]
const MAX_HTTP_TIMEOUT_MS: u64 = 30_000;

#[cfg(target_arch = "wasm32")]
pub(super) fn call(
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
pub(super) fn call(
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
