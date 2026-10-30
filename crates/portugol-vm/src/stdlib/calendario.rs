//! `Calendario.java`: data e hora do computador, no fuso local.

use chrono::{DateTime, Datelike, Local, NaiveDateTime, Timelike};

use super::{NativeContext, NativeFn, arity, boolean, int};
use crate::value::Value;

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "dia_mes_atual" => dia_mes_atual,
        "dia_semana_atual" => dia_semana_atual,
        "mes_atual" => mes_atual,
        "ano_atual" => ano_atual,
        "hora_atual" => hora_atual,
        "minuto_atual" => minuto_atual,
        "segundo_atual" => segundo_atual,
        "milisegundo_atual" => milisegundo_atual,
        "dia_semana_completo" => dia_semana_completo,
        "dia_semana_curto" => dia_semana_curto,
        "dia_semana_abreviado" => dia_semana_abreviado,
        _ => return None,
    })
}

pub fn constant(name: &str) -> Option<Value> {
    const DIAS: [&str; 7] = [
        "DIA_DOMINGO",
        "DIA_SEGUNDA_FEIRA",
        "DIA_TERCA_FEIRA",
        "DIA_QUARTA_FEIRA",
        "DIA_QUINTA_FEIRA",
        "DIA_SEXTA_FEIRA",
        "DIA_SABADO",
    ];
    const MESES: [&str; 12] = [
        "MES_JANEIRO",
        "MES_FEVEREIRO",
        "MES_MARCO",
        "MES_ABRIL",
        "MES_MAIO",
        "MES_JUNHO",
        "MES_JULHO",
        "MES_AGOSTO",
        "MES_SETEMBRO",
        "MES_OUTUBRO",
        "MES_NOVEMBRO",
        "MES_DEZEMBRO",
    ];
    let position = DIAS
        .iter()
        .position(|d| *d == name)
        .or_else(|| MESES.iter().position(|m| *m == name))?;
    Some(Value::Int(position as i32 + 1))
}

/// Data e hora atuais. Com relógio real usa o fuso do sistema; com relógio
/// simulado, UTC (para testes reprodutíveis em qualquer máquina).
fn now(ctx: &NativeContext) -> NaiveDateTime {
    let ms = ctx.now_ms() as i64;
    if ctx.is_simulated() {
        DateTime::from_timestamp_millis(ms)
            .map(|d| d.naive_utc())
            .unwrap_or_default()
    } else {
        Local::now().naive_local()
    }
}

fn dia_mes_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "dia_mes_atual")?;
    Ok(Value::Int(now(ctx).day() as i32))
}

/// Domingo = 1, como `Calendar.DAY_OF_WEEK`.
fn dia_semana_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "dia_semana_atual")?;
    Ok(Value::Int(
        now(ctx).weekday().num_days_from_sunday() as i32 + 1,
    ))
}

fn mes_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "mes_atual")?;
    Ok(Value::Int(now(ctx).month() as i32))
}

fn ano_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "ano_atual")?;
    Ok(Value::Int(now(ctx).year()))
}

/// `Calendar.HOUR` (0 a 11) no formato de 12 horas.
fn hora_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "hora_atual")?;
    let hour = now(ctx).hour() as i32;
    Ok(Value::Int(if boolean(&args[0])? {
        hour % 12
    } else {
        hour
    }))
}

fn minuto_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "minuto_atual")?;
    Ok(Value::Int(now(ctx).minute() as i32))
}

fn segundo_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "segundo_atual")?;
    Ok(Value::Int(now(ctx).second() as i32))
}

fn milisegundo_atual(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "milisegundo_atual")?;
    Ok(Value::Int(
        (now(ctx).nanosecond() / 1_000_000).min(999) as i32
    ))
}

/// Nomes como no original (inclusive "Sabado" sem acento).
fn day_name(names: [&str; 7], args: &[Value], function: &str) -> Result<Value, String> {
    arity(args, 3, function)?;
    let dia = int(&args[0])?;
    let alta = boolean(&args[1])?;
    let baixa = boolean(&args[2])?;
    if !(1..=7).contains(&dia) {
        return Err(format!(
            "'{dia}' não corresponde a um dia da semana válido."
        ));
    }
    let name = names[dia as usize - 1];
    Ok(Value::str(if alta {
        name.to_uppercase()
    } else if baixa {
        name.to_lowercase()
    } else {
        name.to_string()
    }))
}

fn dia_semana_completo(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    day_name(
        [
            "Domingo",
            "Segunda-Feira",
            "Terça-Feira",
            "Quarta-Feira",
            "Quinta-Feira",
            "Sexta-Feira",
            "Sabado",
        ],
        args,
        "dia_semana_completo",
    )
}

fn dia_semana_curto(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    day_name(
        [
            "Domingo", "Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sabado",
        ],
        args,
        "dia_semana_curto",
    )
}

fn dia_semana_abreviado(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    day_name(
        ["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sab"],
        args,
        "dia_semana_abreviado",
    )
}
