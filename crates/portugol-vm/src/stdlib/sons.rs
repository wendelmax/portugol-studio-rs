//! Biblioteca Sons. A VM controla os endereços e volumes, e o host faz a
//! decodificação e reprodução do áudio.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use super::{NativeContext, NativeFn, arity, boolean, int, text};
use crate::{Sound, Value};

struct Entry {
    path: PathBuf,
    sound: Box<dyn Sound>,
    volume: i32,
}

pub(crate) struct Sounds {
    entries: HashMap<i32, Entry>,
    next: i32,
    volume: i32,
}

impl Default for Sounds {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            next: 0,
            volume: 100,
        }
    }
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "carregar_som" => load,
        "liberar_som" => release,
        "reproduzir_som" => play,
        "pausar_som" => pause,
        "interromper_som" => stop,
        "obter_tamanho_musica" => duration,
        "obter_posicao_atual_musica" => position,
        "definir_posicao_atual_musica" => seek,
        "definir_volume" => set_volume,
        "definir_volume_reproducao" => set_clip_volume,
        "obter_volume" => volume,
        "obter_volume_reproducao" => clip_volume,
        _ => return None,
    })
}

fn gain(master: i32, clip: i32) -> f32 {
    // SonsUtils.linearParaExponencial usa a terceira potência.
    (master.clamp(0, 100) as f32 * clip as f32 / 10_000.0).powi(3)
}

fn milliseconds(duration: Duration) -> i32 {
    i32::try_from(duration.as_millis()).unwrap_or(i32::MAX)
}

fn load(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "carregar_som")?;
    let path = super::resolve_path(ctx, text(&args[0])?);
    let path = path.canonicalize().unwrap_or(path);
    if let Some((&id, _)) = ctx
        .sounds
        .entries
        .iter()
        .find(|(_, entry)| entry.path == path)
    {
        return Ok(Value::Int(id));
    }
    let mut sound = ctx.host.load_sound(&path)?;
    let id = ctx.sounds.next;
    ctx.sounds.next = id
        .checked_add(1)
        .ok_or("Limite de sons carregados atingido.")?;
    sound.set_gain(gain(ctx.sounds.volume, 100));
    ctx.sounds.entries.insert(
        id,
        Entry {
            path,
            sound,
            volume: 100,
        },
    );
    Ok(Value::Int(id))
}

fn release(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "liberar_som")?;
    ctx.sounds.entries.remove(&int(&args[0])?);
    Ok(Value::Void)
}

fn play(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "reproduzir_som")?;
    let id = int(&args[0])?;
    let looping = boolean(&args[1])?;
    let entry = ctx
        .sounds
        .entries
        .get_mut(&id)
        .ok_or_else(|| format!("Endereço de som inválido ({id})!"))?;
    entry.sound.play(looping)?;
    Ok(Value::Int(id))
}

fn pause(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "pausar_som")?;
    if let Some(entry) = ctx.sounds.entries.get_mut(&int(&args[0])?) {
        entry.sound.pause();
    }
    Ok(Value::Void)
}

fn stop(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "interromper_som")?;
    if let Some(entry) = ctx.sounds.entries.get_mut(&int(&args[0])?) {
        entry.sound.stop();
    }
    Ok(Value::Void)
}

fn duration(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_tamanho_musica")?;
    Ok(Value::Int(
        ctx.sounds
            .entries
            .get(&int(&args[0])?)
            .map_or(0, |entry| milliseconds(entry.sound.duration())),
    ))
}

fn position(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_posicao_atual_musica")?;
    Ok(Value::Int(
        ctx.sounds
            .entries
            .get(&int(&args[0])?)
            .map_or(0, |entry| milliseconds(entry.sound.position())),
    ))
}

fn seek(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "definir_posicao_atual_musica")?;
    let id = int(&args[0])?;
    let millis = int(&args[1])?.max(0) as u64;
    if let Some(entry) = ctx.sounds.entries.get_mut(&id) {
        entry.sound.seek(Duration::from_millis(millis))?;
    }
    Ok(Value::Void)
}

fn set_volume(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "definir_volume")?;
    let volume = int(&args[0])?;
    ctx.sounds.volume = volume;
    for entry in ctx.sounds.entries.values_mut() {
        entry.sound.set_gain(gain(volume, entry.volume));
    }
    Ok(Value::Void)
}

fn set_clip_volume(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "definir_volume_reproducao")?;
    let id = int(&args[0])?;
    let volume = int(&args[1])?.clamp(0, 100);
    if let Some(entry) = ctx.sounds.entries.get_mut(&id) {
        entry.volume = volume;
        entry.sound.set_gain(gain(ctx.sounds.volume, volume));
    }
    Ok(Value::Void)
}

fn volume(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "obter_volume")?;
    Ok(Value::Int(ctx.sounds.volume))
}

fn clip_volume(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_volume_reproducao")?;
    Ok(Value::Int(
        ctx.sounds
            .entries
            .get(&int(&args[0])?)
            .map_or(-1, |entry| entry.volume),
    ))
}
