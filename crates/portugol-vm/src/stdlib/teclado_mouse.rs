//! `Teclado.java` e `Mouse.java`: estado do teclado e do mouse na janela
//! gráfica. Os códigos de tecla são os `KeyEvent.VK_*` do Java.
//!
//! Sem janela gráfica não chegam eventos: as consultas respondem "nada
//! pressionado" e `ler_tecla`/`ler_botao`, que no original esperariam para
//! sempre, geram erro.

use super::{NativeContext, NativeFn, arity, int};
use crate::graphics::Wait;
use crate::host::unavailable;
use crate::value::Value;

pub fn teclado(name: &str) -> Option<NativeFn> {
    Some(match name {
        "tecla_pressionada" => tecla_pressionada,
        "alguma_tecla_pressionada" => alguma_tecla_pressionada,
        "ler_tecla" => ler_tecla,
        "caracter_tecla" => caracter_tecla,
        _ => return None,
    })
}

pub fn mouse(name: &str) -> Option<NativeFn> {
    Some(match name {
        "botao_pressionado" => botao_pressionado,
        "algum_botao_pressionado" => algum_botao_pressionado,
        "ler_botao" => ler_botao,
        "posicao_x" => |c, a| position(c, a, "posicao_x", |(x, _)| x),
        "posicao_y" => |c, a| position(c, a, "posicao_y", |(_, y)| y),
        "ocultar_cursor" => |c, a| cursor(c, a, "ocultar_cursor", false),
        "exibir_cursor" => |c, a| cursor(c, a, "exibir_cursor", true),
        _ => return None,
    })
}

pub fn teclado_constant(name: &str) -> Option<Value> {
    let letter = name
        .strip_prefix("TECLA_")
        .filter(|rest| rest.len() == 1 && rest.as_bytes()[0].is_ascii_uppercase());
    if let Some(letter) = letter {
        return Some(Value::Int(letter.as_bytes()[0] as i32));
    }
    let code = match name {
        "TECLA_ENTER" => 10,
        "TECLA_BACKSPACE" => 8,
        "TECLA_TAB" => 9,
        "TECLA_CANCELAR" => 3,
        "TECLA_LIMPAR" => 12,
        "TECLA_SHIFT" => 16,
        "TECLA_CONTROL" => 17,
        "TECLA_ALT" => 18,
        "TECLA_PAUSE" => 19,
        "TECLA_ESC" => 27,
        "TECLA_ESPACO" => 32,
        "TECLA_PAGE_UP" => 33,
        "TECLA_PAGE_DOWN" => 34,
        "TECLA_END" => 35,
        "TECLA_HOME" => 36,
        "TECLA_SETA_ESQUERDA" => 37,
        "TECLA_SETA_ACIMA" => 38,
        "TECLA_SETA_DIREITA" => 39,
        "TECLA_SETA_ABAIXO" => 40,
        "TECLA_VIRGULA" => 44,
        "TECLA_MENOS" => 45,
        "TECLA_PONTO_FINAL" => 46,
        "TECLA_BARRA" => 47,
        "TECLA_PONTO_VIRGULA" => 59,
        "TECLA_IGUAL" => 61,
        "TECLA_ABRE_COLCHETES" => 91,
        "TECLA_BARRA_INVERTIDA" => 92,
        "TECLA_FECHA_COLCHETES" => 93,
        "TECLA_MULTIPLICACAO" => 106,
        "TECLA_ADICAO" => 107,
        "TECLA_NUM_SEPARADOR_DECIMAL" => 108,
        "TECLA_SUBTRACAO" => 109,
        "TECLA_DECIMAL" => 110,
        "TECLA_DIVISAO" => 111,
        "TECLA_DELETAR" => 127,
        "TECLA_NUM_LOCK" => 144,
        "TECLA_SCROLL_LOCK" => 145,
        "TECLA_PRINTSCREEN" => 154,
        "TECLA_INSERT" => 155,
        "TECLA_AJUDA" => 156,
        "TECLA_BACK_QUOTE" => 192,
        "TECLA_QUOTE" => 222,
        "TECLA_WINDOWS" => 524,
        "TECLA_MENU_CONTEXTO" => 525,
        "TECLA_CAPS_LOCK" => 20,
        "TECLA_META" => 157,
        "TECLA_AMPERSAND" => 150,
        "TECLA_ASTERISK" => 151,
        "TECLA_QUOTEDBL" => 152,
        "TECLA_LESS" => 153,
        "TECLA_GREATER" => 160,
        "TECLA_BRACELEFT" => 161,
        "TECLA_BRACERIGHT" => 162,
        "TECLA_AT" => 512,
        "TECLA_COLON" => 513,
        "TECLA_CIRCUMFLEX" => 514,
        "TECLA_DOLLAR" => 515,
        "TECLA_EURO_SIGN" => 516,
        "TECLA_EXCLAMATION_MARK" => 517,
        "TECLA_INVERTED_EXCLAMATION_MARK" => 518,
        "TECLA_LEFT_PARENTHESIS" => 519,
        "TECLA_NUMBER_SIGN" => 520,
        "TECLA_PLUS" => 521,
        "TECLA_RIGHT_PARENTHESIS" => 522,
        "TECLA_UNDERSCORE" => 523,
        _ => {
            // As duas formas para o teclado numérico são aceitas.
            let rest = name.strip_prefix("TECLA_")?;
            if let Some(digit) = rest
                .strip_suffix("_NUM")
                .or_else(|| rest.strip_prefix("NUM_"))
            {
                return digit
                    .parse::<i32>()
                    .ok()
                    .filter(|d| (0..=9).contains(d))
                    .map(|d| Value::Int(96 + d));
            }
            if let Some(f) = rest.strip_prefix('F') {
                return f.parse::<i32>().ok().and_then(|n| match n {
                    1..=12 => Some(Value::Int(111 + n)),
                    13..=24 => Some(Value::Int(61_427 + n)),
                    _ => None,
                });
            }
            return rest
                .parse::<i32>()
                .ok()
                .filter(|d| (0..=9).contains(d))
                .map(|d| Value::Int(48 + d));
        }
    };
    Some(Value::Int(code))
}

pub fn mouse_constant(name: &str) -> Option<Value> {
    match name {
        "BOTAO_ESQUERDO" => Some(Value::Int(0)),
        "BOTAO_DIREITO" => Some(Value::Int(1)),
        "BOTAO_MEIO" => Some(Value::Int(2)),
        _ => None,
    }
}

fn tecla_pressionada(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "tecla_pressionada")?;
    let code = int(&args[0])?;
    if !(0..=525).contains(&code) && !(61_440..=61_451).contains(&code) && code != 65_406 {
        return Err("Você tentou acessar um índice de vetor ou matriz inválido.".into());
    }
    Ok(Value::Bool(
        ctx.host.input().is_some_and(|i| i.is_key_down(code)),
    ))
}

fn alguma_tecla_pressionada(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "alguma_tecla_pressionada")?;
    Ok(Value::Bool(
        ctx.host.input().is_some_and(|i| i.any_key_down()),
    ))
}

fn wait_result(result: Wait) -> Result<Value, String> {
    match result {
        Wait::Got(value) => Ok(Value::Int(value)),
        Wait::Interrupted => Err("O programa foi interrompido.".into()),
        Wait::Closed => Err("A janela do modo gráfico foi fechada.".into()),
    }
}

fn ler_tecla(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "ler_tecla")?;
    let input = ctx
        .host
        .input()
        .ok_or_else(|| unavailable("a leitura do teclado"))?;
    wait_result(input.wait_key(&ctx.interrupt))
}

fn caracter_tecla(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "caracter_tecla")?;
    let code = int(&args[0])? as u32 & 0xFFFF;
    Ok(Value::Char(char::from_u32(code).unwrap_or('\u{FFFD}')))
}

fn botao_pressionado(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "botao_pressionado")?;
    let button = int(&args[0])?;
    if !(0..3).contains(&button) {
        return Err(format!(
            "O código '{button}' não é um código de botão válido"
        ));
    }
    Ok(Value::Bool(
        ctx.host.input().is_some_and(|i| i.is_button_down(button)),
    ))
}

fn algum_botao_pressionado(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "algum_botao_pressionado")?;
    Ok(Value::Bool(
        ctx.host.input().is_some_and(|i| i.any_button_down()),
    ))
}

fn ler_botao(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "ler_botao")?;
    let input = ctx
        .host
        .input()
        .ok_or_else(|| unavailable("a leitura do mouse"))?;
    wait_result(input.wait_button(&ctx.interrupt))
}

fn position(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    pick: fn((i32, i32)) -> i32,
) -> Result<Value, String> {
    arity(args, 0, name)?;
    Ok(Value::Int(
        ctx.host.input().map_or(0, |i| pick(i.mouse_position())),
    ))
}

/// Sem janela gráfica não há cursor a alterar; como no original, nada acontece.
fn cursor(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    visible: bool,
) -> Result<Value, String> {
    arity(args, 0, name)?;
    let _ = ctx
        .host
        .graphics_window(crate::graphics::WindowCommand::CursorVisible(visible));
    Ok(Value::Void)
}
