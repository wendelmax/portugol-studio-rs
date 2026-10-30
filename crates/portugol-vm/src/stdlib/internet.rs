//! `Internet.java`: baixar texto e imagens da web.

use std::io::Read;
use std::time::Duration;

use super::{NativeContext, NativeFn, arity, int, text};
use crate::value::Value;

/// Estado da biblioteca: o tempo limite de conexão e leitura.
pub struct Internet {
    timeout_ms: u64,
}

impl Default for Internet {
    fn default() -> Self {
        Internet { timeout_ms: 2000 }
    }
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "definir_tempo_limite" => definir_tempo_limite,
        "obter_texto" => obter_texto,
        "baixar_imagem" => baixar_imagem,
        "endereco_disponivel" => endereco_disponivel,
        _ => return None,
    })
}

fn agent(ctx: &NativeContext) -> ureq::Agent {
    let timeout = Duration::from_millis(ctx.internet.timeout_ms.max(1));
    let mut builder = ureq::AgentBuilder::new()
        .timeout_connect(timeout)
        .timeout_read(timeout);
    if ctx.offline {
        builder = builder.resolver(|_: &str| Err(std::io::Error::other("sem rede")));
    }
    builder.build()
}

fn definir_tempo_limite(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_tempo_limite")?;
    ctx.internet.timeout_ms = int(&args[0])?.max(0) as u64;
    Ok(Value::Void)
}

/// No original, qualquer falha (inclusive conteúdo vazio) vira a mesma
/// mensagem genérica.
fn obter_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_texto")?;
    let url = text(&args[0])?;
    let failure = || format!("Não foi possível obter o conteúdo de {url}");
    let body = agent(ctx)
        .get(url)
        .call()
        .map_err(|_| failure())?
        .into_string()
        .map_err(|_| failure())?;
    if body.is_empty() {
        return Err(failure());
    }
    Ok(Value::str(body))
}

/// Baixa a imagem e grava em `caminho.png` ou `caminho.jpg`, conforme o
/// `Content-Type`; devolve a extensão usada.
fn baixar_imagem(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "baixar_imagem")?;
    let url = text(&args[0])?;
    let target = text(&args[1])?;
    let failure = || format!("Não foi possível obter o conteúdo de {url}");
    let response = agent(ctx).get(url).call().map_err(|_| failure())?;
    let kind = match response.content_type() {
        "image/jpg" | "image/jpeg" => "jpg",
        _ => "png",
    };
    let mut bytes = Vec::new();
    response
        .into_reader()
        .take(64 * 1024 * 1024)
        .read_to_end(&mut bytes)
        .map_err(|_| failure())?;
    let image = image::load_from_memory(&bytes).map_err(|_| failure())?;

    let path = super::arquivos::resolve(ctx, &format!("{target}.{kind}"));
    let format = if kind == "jpg" {
        image::ImageFormat::Jpeg
    } else {
        image::ImageFormat::Png
    };
    // JPEG não tem canal alfa.
    let saved = if kind == "jpg" {
        image::DynamicImage::ImageRgb8(image.to_rgb8()).save_with_format(&path, format)
    } else {
        image.save_with_format(&path, format)
    };
    saved.map_err(|_| {
        format!(
            "Não foi possivel salvar a imagem em {}\nGaranta que o caminho é valido e todas as pastas existem",
            path.display()
        )
    })?;
    Ok(Value::str(kind))
}

fn endereco_disponivel(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "endereco_disponivel")?;
    let available = match agent(ctx).head(text(&args[0])?).call() {
        Ok(response) => response.header("Content-Length") != Some("0"),
        // O original só considera indisponível o 404; outros códigos contam
        // como disponíveis.
        Err(ureq::Error::Status(code, _)) => code != 404,
        Err(_) => false,
    };
    Ok(Value::Bool(available))
}
