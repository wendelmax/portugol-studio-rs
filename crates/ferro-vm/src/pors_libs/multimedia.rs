#[cfg(feature = "stdlib")]
use std::cell::RefCell;
#[cfg(feature = "stdlib")]
use std::rc::Rc;

use crate::value::Value;

#[cfg(feature = "stdlib")]
use super::arquivos::resolve_workspace_path;
use super::err_string;
#[cfg(feature = "stdlib")]
use super::{PorsRuntime, arity, expect_str, ok_value};

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
