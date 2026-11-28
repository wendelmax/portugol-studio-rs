//! `Dialogo` (ADR 0018, Fase 5): caixas do sistema chamadas das funções de
//! uma tela. Quem mostra as caixas é o host; fora de uma tela, cada função
//! devolve um erro que explica onde ela funciona.

use std::rc::Rc;

use crate::host::FileFilter;
use crate::value::Value;

use super::{expect_str, none_value, some_value};

/// `"*.txt;*.csv"` (o `Filtro` do `<DialogoArquivo>`) em filtros do sistema.
/// Vazio aceita qualquer arquivo.
pub(crate) fn filters(raw: &str) -> Vec<FileFilter> {
    let extensions: Vec<String> = raw
        .split(';')
        .map(|part| {
            part.trim()
                .trim_start_matches('*')
                .trim_start_matches('.')
                .to_owned()
        })
        .filter(|extension| !extension.is_empty() && extension != "*")
        .collect();
    if extensions.is_empty() {
        return Vec::new();
    }
    vec![FileFilter {
        description: extensions
            .iter()
            .map(|e| format!("*.{e}"))
            .collect::<Vec<_>>()
            .join(", "),
        extensions,
    }]
}

fn text_argument<'a>(args: &'a [Value], index: usize, name: &str) -> Result<&'a str, String> {
    args.get(index)
        .ok_or_else(|| format!("`Dialogo.{name}` recebeu argumentos de menos."))
        .and_then(expect_str)
}

pub(crate) fn call(
    name: &str,
    args: &[Value],
    ctx: &mut crate::stdlib::NativeContext,
) -> Result<Value, String> {
    match name {
        "mensagem" => {
            ctx.host.show_message(text_argument(args, 0, name)?)?;
            Ok(Value::Void)
        }
        "confirmar" => Ok(Value::Bool(
            ctx.host.confirm(text_argument(args, 0, name)?)?,
        )),
        "abrir_arquivo" | "salvar_arquivo" => {
            let title = text_argument(args, 0, name)?;
            let filter = text_argument(args, 1, name)?;
            let chosen = ctx
                .host
                .file_dialog(name == "salvar_arquivo", title, &filters(filter))?;
            Ok(match chosen {
                Some(path) => some_value(Value::Str(Rc::from(path.as_str()))),
                None => none_value(),
            })
        }
        other => Err(format!("Função desconhecida em Dialogo: `{other}`.")),
    }
}

#[cfg(test)]
mod tests {
    use super::filters;

    #[test]
    fn filtro_do_dialogo_vira_extensoes() {
        let parsed = filters("*.txt; *.CSV;.md");
        assert_eq!(parsed[0].extensions, ["txt", "CSV", "md"]);
        assert_eq!(parsed[0].description, "*.txt, *.CSV, *.md");
        assert!(filters("").is_empty());
        assert!(filters("*.*").is_empty());
    }
}
