//! `Objetos.java`: objetos com propriedades, criados vazios ou a partir de
//! JSON/XML, identificados por um endereço inteiro.
//!
//! Comportamentos do original mantidos:
//! * o endereço é a posição numa lista; `liberar_objeto` remove da lista e
//!   desloca os endereços seguintes;
//! * um subobjeto obtido com `obter_propriedade_tipo_objeto` compartilha a
//!   estrutura com o objeto de origem (alterar um altera o outro);
//! * números inteiros do JSON que cabem em `int` são `inteiro`; os demais
//!   inteiros não têm tipo do Portugol;
//! * JSON ou XML inválido resulta em objeto vazio, sem erro;
//! * `obter_json` imprime como o `DefaultPrettyPrinter` do Jackson.
//!
//! Diferenças: propriedades ficam na ordem de inserção (no original, objetos
//! criados com `criar_objeto` usam a ordem de um `HashMap`); caracteres e
//! vetores atribuídos como propriedade podem ser lidos de volta como
//! caracter e vetor (no original isso dava erro de tipo).

use std::cell::RefCell;
use std::rc::Rc;

use super::{NativeContext, NativeFn, arity, int, text};
use crate::value::{Value, java_double};

type Map = Rc<RefCell<Vec<(String, Json)>>>;
type List = Rc<RefCell<Vec<Json>>>;

#[derive(Clone)]
enum Json {
    Null,
    Bool(bool),
    Int(i32),
    /// Inteiro que não cabe em `int` (Long/BigInteger no Jackson).
    Long(i64),
    Real(f64),
    Str(String),
    Char(char),
    List(List),
    Map(Map),
}

#[derive(Default)]
pub struct Objects {
    list: Vec<Map>,
}

pub fn function(name: &str) -> Option<NativeFn> {
    Some(match name {
        "criar_objeto" => criar_objeto,
        "criar_objeto_via_json" => criar_objeto_via_json,
        "criar_objeto_via_xml" => criar_objeto_via_xml,
        "atribuir_propriedade" => atribuir_propriedade,
        "obter_propriedade_tipo_inteiro" => {
            |c, a| get(c, a, "obter_propriedade_tipo_inteiro", as_int)
        }
        "obter_propriedade_tipo_real" => |c, a| get(c, a, "obter_propriedade_tipo_real", as_real),
        "obter_propriedade_tipo_logico" => {
            |c, a| get(c, a, "obter_propriedade_tipo_logico", as_bool)
        }
        "obter_propriedade_tipo_caracter" => {
            |c, a| get(c, a, "obter_propriedade_tipo_caracter", as_char)
        }
        "obter_propriedade_tipo_cadeia" => obter_propriedade_tipo_cadeia,
        "obter_propriedade_tipo_objeto" => obter_propriedade_tipo_objeto,
        "obter_propriedade_tipo_inteiro_em_vetor" => {
            |c, a| get_at(c, a, "obter_propriedade_tipo_inteiro_em_vetor", as_int)
        }
        "obter_propriedade_tipo_real_em_vetor" => {
            |c, a| get_at(c, a, "obter_propriedade_tipo_real_em_vetor", as_real)
        }
        "obter_propriedade_tipo_logico_em_vetor" => {
            |c, a| get_at(c, a, "obter_propriedade_tipo_logico_em_vetor", as_bool)
        }
        "obter_propriedade_tipo_caracter_em_vetor" => {
            |c, a| get_at(c, a, "obter_propriedade_tipo_caracter_em_vetor", as_char)
        }
        "obter_propriedade_tipo_cadeia_em_vetor" => {
            |c, a| get_at(c, a, "obter_propriedade_tipo_cadeia_em_vetor", as_str)
        }
        "obter_propriedade_tipo_objeto_em_vetor" => obter_propriedade_tipo_objeto_em_vetor,
        "obter_tamanho_vetor_propriedade" => obter_tamanho_vetor_propriedade,
        "liberar_objeto" => liberar_objeto,
        "liberar" => liberar,
        "obter_json" => obter_json,
        "contem_propriedade" => contem_propriedade,
        "tipo_propriedade" => tipo_propriedade,
        _ => return None,
    })
}

pub fn constant(name: &str) -> Option<Value> {
    let value = match name {
        "TIPO_INTEIRO" => 1,
        "TIPO_CADEIA" => 2,
        "TIPO_CARACTER" => 3,
        "TIPO_REAL" => 4,
        "TIPO_LOGICO" => 5,
        "TIPO_OBJETO" => 6,
        "TIPO_VETOR" => 7,
        _ => return None,
    };
    Some(Value::Int(value))
}

const TYPE_MISMATCH: &str = "O tipo da propriedade informada não corresponde ao tipo \
     identificado na função.\nAltere a função de chamada para o tipo correto.";

const BAD_INDEX: &str = "Você tentou acessar um índice de vetor inválido.\n\
     O índice deve ser menor que o número de elementos que o vetor possui.\n\
     Por exemplo, se foi declarado um vetor com 5 elementos (inteiro vetor[5]), o maior índice possível é 4.\n\
     Além disso, o índice de um vetor não pode ser negativo.";

fn new_map(entries: Vec<(String, Json)>) -> Map {
    Rc::new(RefCell::new(entries))
}

fn store(ctx: &mut NativeContext, map: Map) -> Value {
    ctx.objects.list.push(map);
    Value::Int(ctx.objects.list.len() as i32 - 1)
}

fn object(ctx: &NativeContext, address: i32) -> Result<Map, String> {
    usize::try_from(address)
        .ok()
        .and_then(|i| ctx.objects.list.get(i))
        .cloned()
        .ok_or_else(|| "O endereço de memória especificado não aponta para um objeto".to_string())
}

fn missing(map: &Map, property: &str) -> String {
    let keys: Vec<String> = map.borrow().iter().map(|(k, _)| k.clone()).collect();
    let mut message = format!("O objeto não contém a propriedade {property}");
    if !keys.is_empty() {
        message.push_str(". \nVocê pode acessar as seguintes propriedades: ");
        message.push_str(&keys.join(", "));
    }
    message
}

fn lookup(map: &Map, property: &str) -> Option<Json> {
    map.borrow()
        .iter()
        .find(|(k, _)| k == property)
        .map(|(_, v)| v.clone())
}

fn property(ctx: &NativeContext, args: &[Value]) -> Result<Json, String> {
    let map = object(ctx, int(&args[0])?)?;
    let name = text(&args[1])?;
    lookup(&map, name).ok_or_else(|| missing(&map, name))
}

fn as_int(value: Json) -> Result<Value, String> {
    match value {
        Json::Int(v) => Ok(Value::Int(v)),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn as_real(value: Json) -> Result<Value, String> {
    match value {
        Json::Real(v) => Ok(Value::Real(v)),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn as_bool(value: Json) -> Result<Value, String> {
    match value {
        Json::Bool(v) => Ok(Value::Bool(v)),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

/// Primeiro caractere da cadeia.
fn as_char(value: Json) -> Result<Value, String> {
    match value {
        Json::Char(c) => Ok(Value::Char(c)),
        Json::Str(s) => s.chars().next().map(Value::Char).ok_or_else(|| {
            "Você tentou acessar um índice de vetor ou matriz inválido.".to_string()
        }),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn as_str(value: Json) -> Result<Value, String> {
    match value {
        Json::Str(s) => Ok(Value::str(s)),
        Json::Char(c) => Ok(Value::str(c.to_string())),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn get(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    convert: fn(Json) -> Result<Value, String>,
) -> Result<Value, String> {
    arity(args, 2, name)?;
    convert(property(ctx, args)?)
}

/// Elemento `indice` do vetor guardado na propriedade.
fn element(ctx: &NativeContext, args: &[Value]) -> Result<Json, String> {
    let list = list_property(ctx, args)?;
    let index = int(&args[2])?;
    let list = list.borrow();
    usize::try_from(index)
        .ok()
        .and_then(|i| list.get(i))
        .cloned()
        .ok_or_else(|| BAD_INDEX.to_string())
}

fn list_property(ctx: &NativeContext, args: &[Value]) -> Result<List, String> {
    let name = text(&args[1])?;
    match property(ctx, args)? {
        Json::List(list) => Ok(list),
        _ => Err(format!("A propriedade \"{name}\" não é um vetor.")),
    }
}

fn get_at(
    ctx: &mut NativeContext,
    args: &[Value],
    name: &str,
    convert: fn(Json) -> Result<Value, String>,
) -> Result<Value, String> {
    arity(args, 3, name)?;
    convert(element(ctx, args)?)
}

fn criar_objeto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "criar_objeto")?;
    Ok(store(ctx, new_map(Vec::new())))
}

fn criar_objeto_via_json(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "criar_objeto_via_json")?;
    let map = match serde_json::from_str::<serde_json::Value>(text(&args[0])?) {
        Ok(serde_json::Value::Object(object)) => from_serde_object(object),
        _ => new_map(Vec::new()),
    };
    Ok(store(ctx, map))
}

fn from_serde_object(object: serde_json::Map<String, serde_json::Value>) -> Map {
    new_map(
        object
            .into_iter()
            .map(|(k, v)| (k, from_serde(v)))
            .collect(),
    )
}

fn from_serde(value: serde_json::Value) -> Json {
    use serde_json::Value as V;
    match value {
        V::Null => Json::Null,
        V::Bool(b) => Json::Bool(b),
        V::Number(n) => {
            if let Some(i) = n.as_i64() {
                i32::try_from(i).map_or(Json::Long(i), Json::Int)
            } else if n.is_u64() {
                Json::Long(n.as_u64().unwrap_or(0) as i64)
            } else {
                Json::Real(n.as_f64().unwrap_or(0.0))
            }
        }
        V::String(s) => Json::Str(s),
        V::Array(items) => Json::List(Rc::new(RefCell::new(
            items.into_iter().map(from_serde).collect(),
        ))),
        V::Object(object) => Json::Map(from_serde_object(object)),
    }
}

/// Como o `XmlMapper.readTree` do Jackson: os filhos e atributos da raiz
/// viram propriedades, textos viram cadeias e elementos repetidos, vetores.
fn criar_objeto_via_xml(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "criar_objeto_via_xml")?;
    let map = match roxmltree::Document::parse(text(&args[0])?) {
        Ok(doc) => match xml_node(doc.root_element()) {
            Json::Map(map) => map,
            _ => new_map(Vec::new()),
        },
        Err(_) => new_map(Vec::new()),
    };
    Ok(store(ctx, map))
}

fn xml_node(node: roxmltree::Node) -> Json {
    let children: Vec<_> = node.children().filter(|c| c.is_element()).collect();
    if children.is_empty() && node.attributes().len() == 0 {
        return Json::Str(node.text().unwrap_or("").trim().to_string());
    }
    let mut entries: Vec<(String, Json)> = node
        .attributes()
        .map(|a| (a.name().to_string(), Json::Str(a.value().to_string())))
        .collect();
    for child in children {
        let name = child.tag_name().name().to_string();
        let value = xml_node(child);
        match entries.iter_mut().find(|(k, _)| *k == name) {
            Some((_, Json::List(list))) => list.borrow_mut().push(value),
            Some((_, existing)) => {
                let first = std::mem::replace(existing, Json::Null);
                *existing = Json::List(Rc::new(RefCell::new(vec![first, value])));
            }
            None => entries.push((name, value)),
        }
    }
    Json::Map(new_map(entries))
}

fn to_json(value: &Value) -> Result<Json, String> {
    Ok(match value {
        Value::Int(v) => Json::Int(*v),
        Value::Real(v) => Json::Real(*v),
        Value::Bool(v) => Json::Bool(*v),
        Value::Char(c) => Json::Char(*c),
        Value::Str(s) => Json::Str(s.to_string()),
        Value::Array(items) => Json::List(Rc::new(RefCell::new(
            items
                .borrow()
                .iter()
                .map(to_json)
                .collect::<Result<_, _>>()?,
        ))),
        Value::Matrix(matrix) => {
            let matrix = matrix.borrow();
            let rows = (0..matrix.rows)
                .map(|r| {
                    let row = &matrix.data[r * matrix.cols..(r + 1) * matrix.cols];
                    Ok(Json::List(Rc::new(RefCell::new(
                        row.iter().map(to_json).collect::<Result<_, String>>()?,
                    ))))
                })
                .collect::<Result<_, String>>()?;
            Json::List(Rc::new(RefCell::new(rows)))
        }
        other => {
            return Err(format!(
                "não é possível guardar um valor do tipo {}",
                other.type_name()
            ));
        }
    })
}

fn atribuir_propriedade(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 3, "atribuir_propriedade")?;
    let map = object(ctx, int(&args[0])?)?;
    let name = text(&args[1])?.to_string();
    let value = to_json(&args[2])?;
    let mut entries = map.borrow_mut();
    match entries.iter_mut().find(|(k, _)| *k == name) {
        Some((_, slot)) => *slot = value,
        None => entries.push((name, value)),
    }
    Ok(Value::Void)
}

/// No original esta função não verifica se a propriedade existe: ausente,
/// vira erro de tipo.
fn obter_propriedade_tipo_cadeia(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "obter_propriedade_tipo_cadeia")?;
    let map = object(ctx, int(&args[0])?)?;
    match lookup(&map, text(&args[1])?) {
        Some(value) => as_str(value),
        None => Err(TYPE_MISMATCH.into()),
    }
}

fn obter_propriedade_tipo_objeto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "obter_propriedade_tipo_objeto")?;
    match property(ctx, args)? {
        Json::Map(map) => Ok(store(ctx, map)),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn obter_propriedade_tipo_objeto_em_vetor(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Value, String> {
    arity(args, 3, "obter_propriedade_tipo_objeto_em_vetor")?;
    match element(ctx, args)? {
        Json::Map(map) => Ok(store(ctx, map)),
        _ => Err(TYPE_MISMATCH.into()),
    }
}

fn obter_tamanho_vetor_propriedade(
    ctx: &mut NativeContext,
    args: &[Value],
) -> Result<Value, String> {
    arity(args, 2, "obter_tamanho_vetor_propriedade")?;
    Ok(Value::Int(list_property(ctx, args)?.borrow().len() as i32))
}

fn liberar_objeto(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "liberar_objeto")?;
    let address = int(&args[0])?;
    object(ctx, address)?;
    ctx.objects.list.remove(address as usize);
    Ok(Value::Void)
}

fn liberar(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 0, "liberar")?;
    ctx.objects.list.clear();
    Ok(Value::Void)
}

fn obter_json(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 1, "obter_json")?;
    let map = object(ctx, int(&args[0])?)?;
    let mut out = String::new();
    write_json(&Json::Map(map), 0, &mut out);
    Ok(Value::str(out))
}

fn contem_propriedade(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "contem_propriedade")?;
    let map = object(ctx, int(&args[0])?)?;
    Ok(Value::Bool(lookup(&map, text(&args[1])?).is_some()))
}

/// Cadeia de um caractere conta como `TIPO_CARACTER`; sem tipo, 0.
fn tipo_propriedade(ctx: &mut NativeContext, args: &[Value]) -> Result<Value, String> {
    arity(args, 2, "tipo_propriedade")?;
    let kind = match property(ctx, args)? {
        Json::Str(s) if s.chars().count() == 1 => 3,
        Json::Char(_) => 3,
        Json::Str(_) => 2,
        Json::Int(_) => 1,
        Json::Real(_) => 4,
        Json::Bool(_) => 5,
        Json::Map(_) => 6,
        Json::List(_) => 7,
        Json::Null | Json::Long(_) => 0,
    };
    Ok(Value::Int(kind))
}

/// `DefaultPrettyPrinter` do Jackson: objetos em várias linhas com
/// `"chave" : valor` e indentação de 2 espaços; vetores numa linha só.
fn write_json(value: &Json, depth: usize, out: &mut String) {
    match value {
        Json::Null => out.push_str("null"),
        Json::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Json::Int(v) => out.push_str(&v.to_string()),
        Json::Long(v) => out.push_str(&v.to_string()),
        Json::Real(v) => out.push_str(&java_double(*v)),
        Json::Str(s) => write_string(s, out),
        Json::Char(c) => write_string(&c.to_string(), out),
        Json::List(items) => {
            let items = items.borrow();
            if items.is_empty() {
                out.push_str("[ ]");
                return;
            }
            out.push_str("[ ");
            for (i, item) in items.iter().enumerate() {
                if i > 0 {
                    out.push_str(", ");
                }
                write_json(item, depth + 1, out);
            }
            out.push_str(" ]");
        }
        Json::Map(entries) => {
            let entries = entries.borrow();
            if entries.is_empty() {
                out.push_str("{ }");
                return;
            }
            out.push('{');
            for (i, (key, item)) in entries.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push('\n');
                out.push_str(&"  ".repeat(depth + 1));
                write_string(key, out);
                out.push_str(" : ");
                write_json(item, depth + 1, out);
            }
            out.push('\n');
            out.push_str(&"  ".repeat(depth));
            out.push('}');
        }
    }
}

fn write_string(text: &str, out: &mut String) {
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04X}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
}
