use std::cell::RefCell;
use std::rc::Rc;

use crate::value::Value;

use super::{
    arity, check_and_charge, err_string, expect_bool, expect_int, expect_real, expect_str, ok_value,
};

/// Profundidade máxima de aninhamento aceita por `Json.*`.
const MAX_JSON_DEPTH: usize = 64;
/// Limite máximo em bytes de entrada ou saída por chamada em `Json.*` (2 MiB).
const MAX_JSON_BYTES: usize = 2 * 1024 * 1024;

pub(super) enum JsonValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    Array(Vec<JsonValue>),
    Object(Vec<(String, JsonValue)>),
}

pub(super) struct JsonParser<'a> {
    src: &'a str,
    bytes: &'a [u8],
    pos: usize,
}

impl<'a> JsonParser<'a> {
    pub(super) fn parse(src: &'a str) -> Result<JsonValue, String> {
        if src.len() > MAX_JSON_BYTES {
            return Err(format!(
                "Documento JSON excede o limite de {MAX_JSON_BYTES} bytes."
            ));
        }
        let mut parser = Self {
            src,
            bytes: src.as_bytes(),
            pos: 0,
        };
        parser.skip_ws();
        let value = parser.parse_value(0)?;
        parser.skip_ws();
        if parser.pos != parser.bytes.len() {
            return Err("Conteúdo inesperado após o término do JSON.".to_string());
        }
        Ok(value)
    }

    fn skip_ws(&mut self) {
        while let Some(&b) = self.bytes.get(self.pos) {
            if matches!(b, b' ' | b'\t' | b'\n' | b'\r') {
                self.pos += 1;
            } else {
                break;
            }
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    fn parse_value(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.skip_ws();
        match self.peek() {
            Some(b'{') => self.parse_object(depth + 1),
            Some(b'[') => self.parse_array(depth + 1),
            Some(b'"') => self.parse_string().map(JsonValue::Str),
            Some(b't') => {
                self.expect_bytes(b"true")?;
                Ok(JsonValue::Bool(true))
            }
            Some(b'f') => {
                self.expect_bytes(b"false")?;
                Ok(JsonValue::Bool(false))
            }
            Some(b'n') => {
                self.expect_bytes(b"null")?;
                Ok(JsonValue::Null)
            }
            Some(b'-' | b'0'..=b'9') => self.parse_number(),
            Some(_) => Err(format!(
                "Caractere inesperado no JSON na posição {}.",
                self.pos
            )),
            None => Err("Fim inesperado do JSON.".to_string()),
        }
    }

    fn expect_bytes(&mut self, expected: &[u8]) -> Result<(), String> {
        if self.bytes.get(self.pos..self.pos + expected.len()) == Some(expected) {
            self.pos += expected.len();
            Ok(())
        } else {
            Err(format!("Token JSON inválido na posição {}.", self.pos))
        }
    }

    fn parse_object(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.pos += 1; // '{'
        self.skip_ws();
        let mut entries: Vec<(String, JsonValue)> = Vec::new();
        if self.peek() == Some(b'}') {
            self.pos += 1;
            return Ok(JsonValue::Object(entries));
        }
        loop {
            self.skip_ws();
            if self.peek() != Some(b'"') {
                return Err(format!(
                    "Esperada chave de texto no objeto JSON na posição {}.",
                    self.pos
                ));
            }
            let key = self.parse_string()?;
            self.skip_ws();
            if self.peek() != Some(b':') {
                return Err(format!(
                    "Esperado `:` após chave JSON na posição {}.",
                    self.pos
                ));
            }
            self.pos += 1;
            let val = self.parse_value(depth)?;
            if let Some(existing) = entries.iter_mut().find(|(k, _)| k == &key) {
                existing.1 = val;
            } else {
                entries.push((key, val));
            }
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    return Ok(JsonValue::Object(entries));
                }
                _ => {
                    return Err(format!(
                        "Esperado `,` ou `}}` no objeto JSON na posição {}.",
                        self.pos
                    ));
                }
            }
        }
    }

    fn parse_array(&mut self, depth: usize) -> Result<JsonValue, String> {
        if depth > MAX_JSON_DEPTH {
            return Err(format!(
                "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
            ));
        }
        self.pos += 1; // '['
        self.skip_ws();
        let mut items = Vec::new();
        if self.peek() == Some(b']') {
            self.pos += 1;
            return Ok(JsonValue::Array(items));
        }
        loop {
            let item = self.parse_value(depth)?;
            items.push(item);
            self.skip_ws();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b']') => {
                    self.pos += 1;
                    return Ok(JsonValue::Array(items));
                }
                _ => {
                    return Err(format!(
                        "Esperado `,` ou `]` na lista JSON na posição {}.",
                        self.pos
                    ));
                }
            }
        }
    }

    fn parse_hex4(&mut self) -> Result<u16, String> {
        let Some(slice) = self.src.get(self.pos..self.pos + 4) else {
            return Err("Escape Unicode `\\u` incompleto no JSON.".to_string());
        };
        let code = u16::from_str_radix(slice, 16)
            .map_err(|_| "Escape Unicode `\\u` inválido no JSON.".to_string())?;
        self.pos += 4;
        Ok(code)
    }

    fn parse_string(&mut self) -> Result<String, String> {
        self.pos += 1; // '"'
        let mut out = String::new();
        let mut chunk_start = self.pos;
        while let Some(&b) = self.bytes.get(self.pos) {
            match b {
                b'"' => {
                    out.push_str(&self.src[chunk_start..self.pos]);
                    self.pos += 1;
                    return Ok(out);
                }
                b'\\' => {
                    out.push_str(&self.src[chunk_start..self.pos]);
                    self.pos += 1;
                    let esc = self
                        .peek()
                        .ok_or_else(|| "Escape incompleto na cadeia JSON.".to_string())?;
                    self.pos += 1;
                    match esc {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{0008}'),
                        b'f' => out.push('\u{000C}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let first = self.parse_hex4()?;
                            if (0xD800..=0xDBFF).contains(&first) {
                                if self.bytes.get(self.pos..self.pos + 2) != Some(b"\\u") {
                                    return Err(
                                        "Par substituto UTF-16 incompleto no JSON.".to_string()
                                    );
                                }
                                self.pos += 2;
                                let second = self.parse_hex4()?;
                                if !(0xDC00..=0xDFFF).contains(&second) {
                                    return Err(
                                        "Par substituto UTF-16 inválido no JSON.".to_string()
                                    );
                                }
                                let scalar = 0x10000
                                    + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                                let ch = char::from_u32(scalar).ok_or_else(|| {
                                    "Código Unicode inválido no JSON.".to_string()
                                })?;
                                out.push(ch);
                            } else if (0xDC00..=0xDFFF).contains(&first) {
                                return Err(
                                    "Substituto UTF-16 isolado inválido no JSON.".to_string()
                                );
                            } else {
                                let ch = char::from_u32(first as u32).ok_or_else(|| {
                                    "Código Unicode inválido no JSON.".to_string()
                                })?;
                                out.push(ch);
                            }
                        }
                        _ => return Err("Sequência de escape inválida na cadeia JSON.".to_string()),
                    }
                    chunk_start = self.pos;
                }
                0x00..=0x1F => {
                    return Err("Caractere de controle não escapado na cadeia JSON.".to_string());
                }
                _ => self.pos += 1,
            }
        }
        Err("Cadeia JSON não fechada.".to_string())
    }

    fn parse_number(&mut self) -> Result<JsonValue, String> {
        let start = self.pos;
        if self.peek() == Some(b'-') {
            self.pos += 1;
        }
        match self.peek() {
            Some(b'0') => self.pos += 1,
            Some(b'1'..=b'9') => {
                self.pos += 1;
                while matches!(self.peek(), Some(b'0'..=b'9')) {
                    self.pos += 1;
                }
            }
            _ => return Err(format!("Número JSON inválido na posição {}.", self.pos)),
        }
        let mut is_float = false;
        if self.peek() == Some(b'.') {
            is_float = true;
            self.pos += 1;
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(format!(
                    "Fração decimal inválida no JSON na posição {}.",
                    self.pos
                ));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        if matches!(self.peek(), Some(b'e' | b'E')) {
            is_float = true;
            self.pos += 1;
            if matches!(self.peek(), Some(b'+' | b'-')) {
                self.pos += 1;
            }
            if !matches!(self.peek(), Some(b'0'..=b'9')) {
                return Err(format!(
                    "Expoente inválido no JSON na posição {}.",
                    self.pos
                ));
            }
            while matches!(self.peek(), Some(b'0'..=b'9')) {
                self.pos += 1;
            }
        }
        let token = &self.src[start..self.pos];
        if !is_float && let Ok(n) = token.parse::<i64>() {
            return Ok(JsonValue::Int(n));
        }
        let f = token
            .parse::<f64>()
            .map_err(|_| format!("Número JSON inválido: `{token}`."))?;
        if !f.is_finite() {
            return Err(format!("Número JSON fora do intervalo finito: `{token}`."));
        }
        Ok(JsonValue::Float(f))
    }
}

fn json_escape_into(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\u{0008}' => out.push_str("\\b"),
            '\u{000C}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn write_json(
    value: &JsonValue,
    pretty: bool,
    indent: usize,
    out: &mut String,
) -> Result<(), String> {
    if out.len() > MAX_JSON_BYTES {
        return Err(format!(
            "Saída JSON excede o limite de {MAX_JSON_BYTES} bytes."
        ));
    }
    match value {
        JsonValue::Null => out.push_str("null"),
        JsonValue::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        JsonValue::Int(n) => {
            use std::fmt::Write;
            let _ = write!(out, "{n}");
        }
        JsonValue::Float(f) => {
            if !f.is_finite() {
                return Err("Valores NaN ou infinitos não são válidos em JSON.".to_string());
            }
            let rendered = format!("{f}");
            out.push_str(&rendered);
            if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                out.push_str(".0");
            }
        }
        JsonValue::Str(s) => json_escape_into(s, out),
        JsonValue::Array(items) => {
            if items.is_empty() {
                out.push_str("[]");
                return Ok(());
            }
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent + 2 {
                        out.push(' ');
                    }
                }
                write_json(item, pretty, indent + 2, out)?;
            }
            if pretty {
                out.push('\n');
                for _ in 0..indent {
                    out.push(' ');
                }
            }
            out.push(']');
        }
        JsonValue::Object(entries) => {
            if entries.is_empty() {
                out.push_str("{}");
                return Ok(());
            }
            out.push('{');
            for (index, (k, v)) in entries.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent + 2 {
                        out.push(' ');
                    }
                }
                json_escape_into(k, out);
                out.push(':');
                if pretty {
                    out.push(' ');
                }
                write_json(v, pretty, indent + 2, out)?;
            }
            if pretty {
                out.push('\n');
                for _ in 0..indent {
                    out.push(' ');
                }
            }
            out.push('}');
        }
    }
    if out.len() > MAX_JSON_BYTES {
        return Err(format!(
            "Saída JSON excede o limite de {MAX_JSON_BYTES} bytes."
        ));
    }
    Ok(())
}

fn serialize_json_value(value: &JsonValue, pretty: bool) -> Result<String, String> {
    let mut out = String::new();
    write_json(value, pretty, 0, &mut out)?;
    Ok(out)
}

fn parse_json_object(src: &str) -> Result<Vec<(String, JsonValue)>, String> {
    match JsonParser::parse(src)? {
        JsonValue::Object(entries) => Ok(entries),
        _ => Err("O documento JSON precisa ser um objeto (`{ ... }`).".to_string()),
    }
}

fn json_get_entry<'a>(
    entries: &'a [(String, JsonValue)],
    key: &str,
) -> Result<&'a JsonValue, String> {
    entries
        .iter()
        .find(|(k, _)| k == key)
        .map(|(_, v)| v)
        .ok_or_else(|| format!("Chave '{key}' não encontrada no objeto JSON."))
}

fn json_set_key(
    src: &str,
    key: &str,
    value: JsonValue,
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    let mut entries = match parse_json_object(src) {
        Ok(entries) => entries,
        Err(msg) => return err_string(msg, charge_string),
    };
    if let Some(existing) = entries.iter_mut().find(|(k, _)| k == key) {
        existing.1 = value;
    } else {
        entries.push((key.to_owned(), value));
    }
    let rendered = match serialize_json_value(&JsonValue::Object(entries), false) {
        Ok(text) => text,
        Err(msg) => return err_string(msg, charge_string),
    };
    check_and_charge(rendered.len(), charge_string)?;
    Ok(ok_value(Value::str(rendered)))
}

pub(super) fn call(
    name: &str,
    args: &[Value],
    charge_string: &mut impl FnMut(usize) -> Result<(), String>,
) -> Result<Value, String> {
    match name {
        "valido" => {
            arity(args, 1, "Json.valido")?;
            Ok(Value::Bool(
                JsonParser::parse(expect_str(&args[0])?).is_ok(),
            ))
        }
        "formatar" => {
            arity(args, 1, "Json.formatar")?;
            let value = match JsonParser::parse(expect_str(&args[0])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(&value, true) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "minificar" => {
            arity(args, 1, "Json.minificar")?;
            let value = match JsonParser::parse(expect_str(&args[0])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(&value, false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "escapar_cadeia" => {
            arity(args, 1, "Json.escapar_cadeia")?;
            let raw = expect_str(&args[0])?;
            let mut out = String::with_capacity(raw.len().saturating_add(2));
            json_escape_into(raw, &mut out);
            check_and_charge(out.len(), charge_string)?;
            Ok(Value::str(out))
        }
        "contem_chave" => {
            arity(args, 2, "Json.contem_chave")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            Ok(ok_value(Value::Bool(entries.iter().any(|(k, _)| k == key))))
        }
        "obter_chaves" => {
            arity(args, 1, "Json.obter_chaves")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let total_bytes: usize = entries.iter().map(|(k, _)| k.len()).sum();
            check_and_charge(total_bytes, charge_string)?;
            let keys: Vec<Value> = entries.into_iter().map(|(k, _)| Value::str(k)).collect();
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(keys)))))
        }
        "obter_cadeia" => {
            arity(args, 2, "Json.obter_cadeia")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Str(s)) => {
                    check_and_charge(s.len(), charge_string)?;
                    Ok(ok_value(Value::str(s.clone())))
                }
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém uma cadeia no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_inteiro" => {
            arity(args, 2, "Json.obter_inteiro")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Int(n)) => match i32::try_from(*n) {
                    Ok(v) => Ok(ok_value(Value::Int(v))),
                    Err(_) => err_string(
                        format!("O valor de '{key}' excede o intervalo de `inteiro` (32 bits)."),
                        charge_string,
                    ),
                },
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um número inteiro no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_real" => {
            arity(args, 2, "Json.obter_real")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Int(n)) => Ok(ok_value(Value::Real(*n as f64))),
                Ok(JsonValue::Float(f)) => Ok(ok_value(Value::Real(*f))),
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um número no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_logico" => {
            arity(args, 2, "Json.obter_logico")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            match json_get_entry(&entries, key) {
                Ok(JsonValue::Bool(b)) => Ok(ok_value(Value::Bool(*b))),
                Ok(_) => err_string(
                    format!("A chave '{key}' não contém um valor lógico no JSON."),
                    charge_string,
                ),
                Err(msg) => err_string(msg, charge_string),
            }
        }
        "obter_json" => {
            arity(args, 2, "Json.obter_json")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let value = match json_get_entry(&entries, key) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            let rendered = match serialize_json_value(value, false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        "obter_lista_cadeia" => {
            arity(args, 2, "Json.obter_lista_cadeia")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let items = match json_get_entry(&entries, key) {
                Ok(JsonValue::Array(items)) => items,
                Ok(_) => {
                    return err_string(
                        format!("A chave '{key}' não contém uma lista no JSON."),
                        charge_string,
                    );
                }
                Err(msg) => return err_string(msg, charge_string),
            };
            let mut out = Vec::with_capacity(items.len());
            let mut total_bytes = 0usize;
            for item in items {
                match item {
                    JsonValue::Str(s) => {
                        total_bytes = total_bytes.saturating_add(s.len());
                        out.push(Value::str(s.clone()));
                    }
                    _ => {
                        return err_string(
                            format!("A lista em '{key}' contém elementos que não são cadeias."),
                            charge_string,
                        );
                    }
                }
            }
            check_and_charge(total_bytes, charge_string)?;
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(out)))))
        }
        "obter_lista_inteiro" => {
            arity(args, 2, "Json.obter_lista_inteiro")?;
            let entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            let items = match json_get_entry(&entries, key) {
                Ok(JsonValue::Array(items)) => items,
                Ok(_) => {
                    return err_string(
                        format!("A chave '{key}' não contém uma lista no JSON."),
                        charge_string,
                    );
                }
                Err(msg) => return err_string(msg, charge_string),
            };
            let mut out = Vec::with_capacity(items.len());
            for item in items {
                match item {
                    JsonValue::Int(n) => match i32::try_from(*n) {
                        Ok(v) => out.push(Value::Int(v)),
                        Err(_) => {
                            return err_string(
                                format!(
                                    "A lista em '{key}' contém inteiro fora do intervalo de 32 bits."
                                ),
                                charge_string,
                            );
                        }
                    },
                    _ => {
                        return err_string(
                            format!("A lista em '{key}' contém elementos que não são inteiros."),
                            charge_string,
                        );
                    }
                }
            }
            Ok(ok_value(Value::Array(Rc::new(RefCell::new(out)))))
        }
        "definir_cadeia" => {
            arity(args, 3, "Json.definir_cadeia")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Str(expect_str(&args[2])?.to_owned()),
                charge_string,
            )
        }
        "definir_inteiro" => {
            arity(args, 3, "Json.definir_inteiro")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Int(i64::from(expect_int(&args[2])?)),
                charge_string,
            )
        }
        "definir_real" => {
            arity(args, 3, "Json.definir_real")?;
            let f = expect_real(&args[2])?;
            if !f.is_finite() {
                return err_string(
                    "Valores NaN ou infinitos não podem ser gravados em JSON.".to_string(),
                    charge_string,
                );
            }
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Float(f),
                charge_string,
            )
        }
        "definir_logico" => {
            arity(args, 3, "Json.definir_logico")?;
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                JsonValue::Bool(expect_bool(&args[2])?),
                charge_string,
            )
        }
        "definir_json" => {
            arity(args, 3, "Json.definir_json")?;
            let nested = match JsonParser::parse(expect_str(&args[2])?) {
                Ok(v) => v,
                Err(msg) => return err_string(msg, charge_string),
            };
            json_set_key(
                expect_str(&args[0])?,
                expect_str(&args[1])?,
                nested,
                charge_string,
            )
        }
        "remover_chave" => {
            arity(args, 2, "Json.remover_chave")?;
            let mut entries = match parse_json_object(expect_str(&args[0])?) {
                Ok(entries) => entries,
                Err(msg) => return err_string(msg, charge_string),
            };
            let key = expect_str(&args[1])?;
            entries.retain(|(k, _)| k != key);
            let rendered = match serialize_json_value(&JsonValue::Object(entries), false) {
                Ok(text) => text,
                Err(msg) => return err_string(msg, charge_string),
            };
            check_and_charge(rendered.len(), charge_string)?;
            Ok(ok_value(Value::str(rendered)))
        }
        other => Err(format!("Função desconhecida em Json: `{other}`.")),
    }
}
