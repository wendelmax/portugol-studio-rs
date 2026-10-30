//! `Util.java`.

use std::time::Duration;

use super::{NativeContext, NativeFn, arity, int};
use crate::value::Value;

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "numero_elementos" => numero_elementos,
        "numero_linhas" => numero_linhas,
        "numero_colunas" => numero_colunas,
        "sorteia" => sorteia,
        "aguarde" => aguarde,
        "tempo_decorrido" => tempo_decorrido,
        "obter_diretorio_usuario" => obter_diretorio_usuario,
        _ => return None,
    })
}

fn numero_elementos(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "numero_elementos")?;
    match &args[0] {
        Value::Array(array) => Ok(Value::Int(array.borrow().len() as i32)),
        other => Err(format!(
            "\"numero_elementos\" espera um vetor, mas recebeu {}",
            other.type_name()
        )),
    }
}

fn numero_linhas(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "numero_linhas")?;
    match &args[0] {
        Value::Matrix(matrix) => Ok(Value::Int(matrix.borrow().rows as i32)),
        other => Err(format!(
            "\"numero_linhas\" espera uma matriz, mas recebeu {}",
            other.type_name()
        )),
    }
}

fn numero_colunas(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "numero_colunas")?;
    match &args[0] {
        Value::Matrix(matrix) => {
            let matrix = matrix.borrow();
            // `matriz[0].length` no Java: sem linhas, não há coluna 0.
            if matrix.rows == 0 {
                return Err("Você tentou acessar um índice de vetor ou matriz inválido.".into());
            }
            Ok(Value::Int(matrix.cols as i32))
        }
        other => Err(format!(
            "\"numero_colunas\" espera uma matriz, mas recebeu {}",
            other.type_name()
        )),
    }
}

fn sorteia(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "sorteia")?;
    Ok(Value::Int(super::sorteia(
        ctx,
        int(&args[0])?,
        int(&args[1])?,
    )?))
}

fn aguarde(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "aguarde")?;
    let ms = int(&args[0])?;
    if ms < 0 {
        return Err("O intervalo de espera não pode ser negativo".to_string());
    }
    std::thread::sleep(Duration::from_millis(ms as u64));
    Ok(Value::Void)
}

fn tempo_decorrido(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "tempo_decorrido")?;
    Ok(Value::Int(ctx.start.elapsed().as_millis() as i32))
}

fn obter_diretorio_usuario(_: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "obter_diretorio_usuario")?;
    std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(Value::str)
        .map_err(|_| "Não foi possível obter o diretório do usuário".to_string())
}
