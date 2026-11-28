//! Serviços do sistema das telas desktop (ADR 0018, Fase 5): `Dialogo`,
//! `AreaTransferencia`, `Notificacao` e `Telas` (Emenda 5). Quem fala com o sistema é o host;
//! fora de uma tela, cada função devolve um erro que explica onde funciona.

use std::rc::Rc;

use crate::value::Value;

use super::{expect_str, none_value, some_value};

fn text(args: &[Value], index: usize, function: &str) -> Result<String, String> {
    args.get(index)
        .ok_or_else(|| format!("`{function}` recebeu argumentos de menos."))
        .and_then(expect_str)
        .map(str::to_owned)
}

pub(crate) fn call(
    library: &str,
    name: &str,
    args: &[Value],
    ctx: &mut crate::stdlib::NativeContext,
) -> Result<Value, String> {
    match (library, name) {
        ("FerroDialogo", _) => super::dialogo::call(name, args, ctx),
        ("FerroAreaTransferencia", "copiar") => {
            ctx.host
                .clipboard_write(&text(args, 0, "AreaTransferencia.copiar")?)?;
            Ok(Value::Void)
        }
        ("FerroAreaTransferencia", "colar") => Ok(match ctx.host.clipboard_read()? {
            Some(text) => some_value(Value::Str(Rc::from(text.as_str()))),
            None => none_value(),
        }),
        ("FerroNotificacao", "mostrar") => {
            let title = text(args, 0, "Notificacao.mostrar")?;
            let body = text(args, 1, "Notificacao.mostrar")?;
            ctx.host.notify(&title, &body)?;
            Ok(Value::Void)
        }
        ("FerroTelas", "abrir") => {
            let screen = text(args, 0, "Telas.abrir")?;
            let (data, callback) = match args.len() {
                1 => (None, None),
                _ => (
                    Some((
                        args.get(1).cloned().unwrap_or(Value::Void),
                        text(args, 3, "Telas.abrir")?,
                    )),
                    Some(text(args, 2, "Telas.abrir")?),
                ),
            };
            ctx.host.screen_request(crate::host::ScreenRequest::Open {
                screen,
                data,
                callback,
            })?;
            Ok(Value::Void)
        }
        ("FerroTelas", "concluir" | "fechar") => {
            ctx.host.screen_request(crate::host::ScreenRequest::Close {
                accept: name == "concluir",
            })?;
            Ok(Value::Void)
        }
        (library, name) => Err(format!(
            "Função desconhecida em {}: `{name}`.",
            library.trim_start_matches("Ferro")
        )),
    }
}
