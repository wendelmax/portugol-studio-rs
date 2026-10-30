//! `Arquivos.java`: leitura e escrita de arquivos de texto.
//!
//! Como no original: até 10 arquivos abertos ao mesmo tempo, identificados
//! pelo índice devolvido por `abrir_arquivo` (o primeiro livre a partir do
//! fim); leitura e escrita em UTF-8, uma linha por chamada; caminhos
//! relativos partem da pasta do programa.

use std::fs::{self, File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

use super::{NativeContext, NativeFn, arity, boolean, int, text, text_array};
use crate::host::FileFilter;
use crate::value::Value;

const MAX_FILES: usize = 10;
const LINE_SEPARATOR: &str = if cfg!(windows) { "\r\n" } else { "\n" };

#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    Read,
    Write,
    Append,
}

struct OpenFile {
    path: PathBuf,
    mode: Mode,
    reader: Option<BufReader<File>>,
    writer: Option<File>,
    eof: bool,
}

/// Arquivos abertos pelo programa; fechados quando a execução termina.
#[derive(Default)]
pub struct OpenFiles {
    slots: [Option<OpenFile>; MAX_FILES],
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "abrir_arquivo" => abrir_arquivo,
        "fechar_arquivo" => fechar_arquivo,
        "fim_arquivo" => fim_arquivo,
        "ler_linha" => ler_linha,
        "escrever_linha" => escrever_linha,
        "substituir_texto" => substituir_texto,
        "arquivo_existe" => arquivo_existe,
        "apagar_arquivo" => apagar_arquivo,
        "selecionar_arquivo" => selecionar_arquivo,
        "listar_pastas" => listar_pastas,
        "listar_arquivos" => listar_arquivos,
        "listar_arquivos_por_tipo" => listar_arquivos_por_tipo,
        "criar_pasta" => criar_pasta,
        _ => return None,
    })
}

pub fn constant(name: &str) -> Option<Value> {
    match name {
        "MODO_LEITURA" => Some(Value::Int(0)),
        "MODO_ESCRITA" => Some(Value::Int(1)),
        "MODO_ACRESCENTAR" => Some(Value::Int(2)),
        _ => None,
    }
}

/// `programa.resolverCaminho` + `getAbsolutePath`.
fn resolve(ctx: &NativeContext, path: &str) -> PathBuf {
    let path = Path::new(path);
    let full = if path.is_absolute() {
        path.to_path_buf()
    } else {
        ctx.work_dir.join(path)
    };
    std::path::absolute(&full).unwrap_or(full)
}

fn shown(path: &Path) -> String {
    path.display().to_string()
}

fn slot(ctx: &mut NativeContext, address: i32) -> Result<&mut OpenFile, String> {
    usize::try_from(address)
        .ok()
        .and_then(|i| ctx.files.slots.get_mut(i))
        .and_then(Option::as_mut)
        .ok_or_else(|| "O endereço de memória especificado não aponta para um arquivo".to_string())
}

fn abrir_arquivo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "abrir_arquivo")?;
    let path = resolve(ctx, text(&args[0])?);
    let mode = int(&args[1])?;

    if ctx.files.slots.iter().flatten().any(|f| f.path == path) {
        return Err(format!("O arquivo '{}' já está aberto", shown(&path)));
    }
    let mode = match mode {
        0 => Mode::Read,
        1 => Mode::Write,
        2 => Mode::Append,
        other => return Err(format!("Modo de acesso inválido: {other}")),
    };
    let Some(index) = (0..MAX_FILES).rev().find(|&i| ctx.files.slots[i].is_none()) else {
        return Err(
            "O número máximo de arquivos que podem ser abertos ao mesmo tempo foi atingido".into(),
        );
    };

    let mut file = OpenFile {
        path: path.clone(),
        mode,
        reader: None,
        writer: None,
        eof: false,
    };
    match mode {
        Mode::Read => {
            let handle = File::open(&path).map_err(|_| {
                format!(
                    "Não foi possível abrir o arquivo '{}' para leitura",
                    shown(&path)
                )
            })?;
            file.reader = Some(BufReader::new(handle));
        }
        Mode::Write | Mode::Append => {
            if let Some(parent) = path.parent() {
                let _ = fs::create_dir_all(parent);
            }
            let handle = OpenOptions::new()
                .write(true)
                .create(true)
                .append(mode == Mode::Append)
                .truncate(mode == Mode::Write)
                .open(&path)
                .map_err(|_| {
                    format!(
                        "Não foi possível abrir o arquivo '{}' para escrita",
                        shown(&path)
                    )
                })?;
            file.writer = Some(handle);
        }
    }
    ctx.files.slots[index] = Some(file);
    Ok(Value::Int(index as i32))
}

fn fechar_arquivo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "fechar_arquivo")?;
    let address = int(&args[0])?;
    slot(ctx, address)?;
    ctx.files.slots[address as usize] = None;
    Ok(Value::Void)
}

fn fim_arquivo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "fim_arquivo")?;
    let file = slot(ctx, int(&args[0])?)?;
    if file.mode != Mode::Read {
        return Err(format!(
            "O arquivo '{}' está aberto em modo de escrita",
            shown(&file.path)
        ));
    }
    Ok(Value::Bool(file.eof))
}

/// `BufferedReader.readLine`: no fim devolve "" e marca o fim do arquivo.
fn ler_linha(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "ler_linha")?;
    let file = slot(ctx, int(&args[0])?)?;
    if file.mode != Mode::Read {
        return Err(format!(
            "O arquivo '{}' está aberto em modo de escrita",
            shown(&file.path)
        ));
    }
    let reader = file.reader.as_mut().expect("aberto para leitura");
    let mut line = String::new();
    match reader.read_line(&mut line) {
        Ok(0) => {
            file.eof = true;
            Ok(Value::str(""))
        }
        Ok(_) => {
            while line.ends_with('\n') || line.ends_with('\r') {
                line.pop();
            }
            Ok(Value::str(line))
        }
        Err(_) => Err(format!(
            "Não foi possível ler a próxima linha do arquivo '{}'",
            shown(&file.path)
        )),
    }
}

fn escrever_linha(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "escrever_linha")?;
    let line = text(&args[0])?.replace(['\n', '\r'], "");
    let file = slot(ctx, int(&args[1])?)?;
    if file.mode == Mode::Read {
        return Err(format!(
            "O arquivo '{}' está aberto em modo de leitura",
            shown(&file.path)
        ));
    }
    let writer = file.writer.as_mut().expect("aberto para escrita");
    writer
        .write_all(line.as_bytes())
        .and_then(|_| writer.write_all(LINE_SEPARATOR.as_bytes()))
        .map_err(|_| {
            format!(
                "Não foi possível escrever no arquivo '{}'",
                shown(&file.path)
            )
        })?;
    Ok(Value::Void)
}

/// Como no original, `texto_pesquisa` é uma expressão regular e o arquivo é
/// lido e gravado em ISO-8859-1.
fn substituir_texto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 4, "substituir_texto")?;
    let path = resolve(ctx, text(&args[0])?);
    let pattern = text(&args[1])?;
    let replacement = text(&args[2])?;
    let only_first = boolean(&args[3])?;
    let failure = || format!("Não foi possível substituir no arquivo '{}'", shown(&path));

    let bytes = fs::read(&path).map_err(|_| failure())?;
    let content: String = bytes.iter().map(|&b| b as char).collect();
    let regex = regex::Regex::new(pattern).map_err(|e| format!("Expressão inválida: {e}"))?;
    let replaced = if only_first {
        regex.replace(&content, replacement)
    } else {
        regex.replace_all(&content, replacement)
    };
    let out: Vec<u8> = replaced
        .chars()
        .map(|c| if (c as u32) < 256 { c as u8 } else { b'?' })
        .collect();
    fs::write(&path, out).map_err(|_| failure())?;
    Ok(Value::Void)
}

fn arquivo_existe(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "arquivo_existe")?;
    Ok(Value::Bool(resolve(ctx, text(&args[0])?).is_file()))
}

fn apagar_arquivo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "apagar_arquivo")?;
    let path = resolve(ctx, text(&args[0])?);
    if path.is_file() && fs::remove_file(&path).is_ok() {
        Ok(Value::Void)
    } else {
        Err(format!(
            "Não foi possível apagar o arquivo '{}'",
            shown(&path)
        ))
    }
}

/// Formatos no padrão `"Descrição|ext1,ext2"`.
fn selecionar_arquivo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "selecionar_arquivo")?;
    let formats = text_array(&args[0])?;
    let accept_all = boolean(&args[1])?;
    let mut filters = Vec::new();
    for format in formats.borrow().iter() {
        let format = text(format)?;
        let Some((description, extensions)) = format.split_once('|') else {
            return Err(format!("O formato de arquivo '{format}' é inválido"));
        };
        filters.push(FileFilter {
            description: description.to_string(),
            extensions: extensions
                .split(',')
                .map(|e| e.trim().to_lowercase())
                .collect(),
        });
    }
    let accept_all = accept_all || filters.is_empty();
    let selected = ctx.host.select_file(&filters, accept_all)?;
    Ok(Value::str(selected.unwrap_or_default()))
}

enum Listing<'a> {
    Folders,
    Files,
    FilesOfType(&'a [String]),
}

/// Preenche o vetor com os nomes e completa com "". Como no original, o
/// tamanho é comparado com todas as entradas da pasta (pastas e arquivos).
fn list(ctx: &mut NativeContext, args: &[Value], kind: Listing) -> Result<Value, String> {
    let parent = text(&args[0])?;
    let dir = resolve(ctx, parent);
    let target = text_array(&args[1])?;
    let Ok(entries) = fs::read_dir(&dir) else {
        return Err(format!("{parent} não é um caminho possível"));
    };
    let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
    entries.sort();
    if let Listing::FilesOfType(types) = kind {
        entries.retain(|p| {
            let name = p.to_string_lossy().to_lowercase();
            types.iter().any(|t| name.ends_with(&t.to_lowercase()))
        });
    }

    let capacity = target.borrow().len();
    if entries.len() > capacity {
        let what = if matches!(kind, Listing::Folders) {
            "pastas"
        } else {
            "arquivos"
        };
        return Err(format!(
            "Não foi possível listar os {what} pois o vetor passado é muito pequeno. O diretório \
             escolhido possui {} {what}, mas o vetor passado comporta apenas {capacity} \
             elementos. Aumente o tamanho do vetor ou escolha outro diretório para listar os {what}.",
            entries.len()
        ));
    }

    let names: Vec<String> = entries
        .iter()
        .filter(|p| match kind {
            Listing::Folders => p.is_dir(),
            _ => p.is_file(),
        })
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    let mut target = target.borrow_mut();
    for (index, slot) in target.iter_mut().enumerate() {
        *slot = Value::str(names.get(index).map(String::as_str).unwrap_or(""));
    }
    Ok(Value::Void)
}

fn listar_pastas(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "listar_pastas")?;
    list(ctx, args, Listing::Folders)
}

fn listar_arquivos(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "listar_arquivos")?;
    list(ctx, args, Listing::Files)
}

fn listar_arquivos_por_tipo(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "listar_arquivos_por_tipo")?;
    let types: Vec<String> = text_array(&args[2])?
        .borrow()
        .iter()
        .map(|v| text(v).map(str::to_string))
        .collect::<Result<_, _>>()?;
    list(ctx, args, Listing::FilesOfType(&types))
}

fn criar_pasta(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "criar_pasta")?;
    let raw = text(&args[0])?;
    let dir = resolve(ctx, raw);
    fs::create_dir_all(&dir)
        .map(|_| Value::Void)
        .map_err(|_| format!("Não foi possivel criar um diretorio em {raw}"))
}
