use crate::value::Value;

use super::{PorsRuntime, arity, check_and_charge, err_string, expect_bool, expect_int, ok_value};

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

pub(super) fn call(
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
