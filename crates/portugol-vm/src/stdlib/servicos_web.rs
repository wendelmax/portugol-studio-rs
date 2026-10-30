//! `ServicosWeb.java` (e `web/RestClient.java`): chamadas a serviços REST
//! com JSON.
//!
//! Como no original: respostas com código diferente de 200 não são erro e
//! devolvem a cadeia "Requisição falhou: código HTTP: N"; as linhas do corpo
//! da resposta são concatenadas sem quebra de linha.

use std::io::{BufRead, BufReader};
use std::time::Duration;

use super::{NativeContext, NativeFn, arity, int, text};
use crate::value::Value;

/// Conexão customizada (`abrir_conexao` + cabeçalhos) e parâmetro JSON
/// guardados para `fazer_requisicao`.
#[derive(Default)]
pub struct WebServices {
    connection: Option<Connection>,
    body: String,
}

struct Connection {
    url: String,
    headers: Vec<(String, String)>,
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "obter_dados" => obter_dados,
        "excluir_dados" => excluir_dados,
        "publicar_dados" => publicar_dados,
        "atualizar_dados" => atualizar_dados,
        "abrir_conexao" => abrir_conexao,
        "adicionar_cabecalho" => adicionar_cabecalho,
        "adicionar_parametros" => adicionar_parametros,
        "fazer_requisicao" => fazer_requisicao,
        _ => return None,
    })
}

pub fn constant(name: &str) -> Option<Value> {
    let value = match name {
        "OBTER" => 1,
        "PUBLICAR" => 2,
        "ATUALIZAR" => 3,
        "EXCLUIR" => 4,
        _ => return None,
    };
    Some(Value::Int(value))
}

const FAILURE: &str = "Ocorreu um problema na requisição.";

fn request(
    offline: bool,
    method: &str,
    url: &str,
    headers: &[(String, String)],
    body: &str,
) -> Result<Value, String> {
    if offline {
        return Err(FAILURE.to_string());
    }
    let agent = ureq::AgentBuilder::new()
        .timeout(Duration::from_secs(60))
        .build();
    let mut request = agent
        .request(method, url)
        .set("Accept", "application/json")
        .set("Content-Type", "application/json");
    for (key, value) in headers {
        request = request.set(key, value);
    }
    let result = if body.is_empty() {
        request.call()
    } else {
        request.send_string(body)
    };
    match result {
        Ok(response) if response.status() == 200 => {
            let mut joined = String::new();
            for line in BufReader::new(response.into_reader()).lines() {
                joined.push_str(&line.map_err(|_| FAILURE.to_string())?);
            }
            Ok(Value::str(joined))
        }
        Ok(response) => Ok(Value::str(format!(
            "Requisição falhou: código HTTP: {}",
            response.status()
        ))),
        Err(ureq::Error::Status(code, _)) => Ok(Value::str(format!(
            "Requisição falhou: código HTTP: {code}"
        ))),
        Err(_) => Err(FAILURE.to_string()),
    }
}

fn obter_dados(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_dados")?;
    request(ctx.offline, "GET", text(&args[0])?, &[], "")
}

fn excluir_dados(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "excluir_dados")?;
    request(ctx.offline, "DELETE", text(&args[0])?, &[], "")
}

fn publicar_dados(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "publicar_dados")?;
    request(ctx.offline, "POST", text(&args[0])?, &[], text(&args[1])?)
}

fn atualizar_dados(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "atualizar_dados")?;
    request(ctx.offline, "PUT", text(&args[0])?, &[], text(&args[1])?)
}

fn abrir_conexao(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "abrir_conexao")?;
    let url = text(&args[0])?;
    // `new URL(endereco)` do Java rejeita endereços sem protocolo conhecido.
    if !(url.starts_with("http://") || url.starts_with("https://")) {
        return Err("Ocorreu um problema na criação da requisição.".into());
    }
    ctx.web.connection = Some(Connection {
        url: url.to_string(),
        headers: Vec::new(),
    });
    Ok(Value::Void)
}

fn cached(ctx: &mut NativeContext) -> Result<&mut Connection, String> {
    ctx.web.connection.as_mut().ok_or_else(|| {
        "É necessário criar uma conexão customizada antes de adicionar o cabeçalho".to_string()
    })
}

fn adicionar_cabecalho(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "adicionar_cabecalho")?;
    let (key, value) = (text(&args[0])?.to_string(), text(&args[1])?.to_string());
    cached(ctx)?.headers.push((key, value));
    Ok(Value::Void)
}

fn adicionar_parametros(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "adicionar_parametros")?;
    ctx.web.body = text(&args[0])?.to_string();
    Ok(Value::Void)
}

/// GET e DELETE não enviam o parâmetro guardado; POST e PUT enviam.
fn fazer_requisicao(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "fazer_requisicao")?;
    let method = int(&args[0])?;
    let body = ctx.web.body.clone();
    let offline = ctx.offline;
    let connection = cached(ctx)?;
    let (verb, body) = match method {
        1 => ("GET", ""),
        2 => ("POST", body.as_str()),
        3 => ("PUT", body.as_str()),
        4 => ("DELETE", ""),
        _ => return Ok(Value::str("")),
    };
    request(offline, verb, &connection.url, &connection.headers, body)
}
