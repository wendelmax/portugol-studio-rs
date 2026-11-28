pub(super) const PORS_UI_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_ui {
    use std::cell::RefCell;
    use std::fmt::Write as _;

    const MAX_NODES: usize = 1_024;
    const MAX_DEPTH: usize = 32;

    #[derive(Clone)]
    pub enum UiNode {
        Column(Vec<UiNode>),
        Row(Vec<UiNode>),
        Text(String),
        Button { id: String, label: String },
        Input { id: String, value: String, placeholder: String },
        List(Vec<UiNode>),
    }

    impl UiNode {
        pub fn to_json(&self) -> String {
            let mut out = String::new();
            self.write_json(&mut out);
            out
        }

        fn children(&self) -> &[UiNode] {
            match self {
                Self::Column(items) | Self::Row(items) | Self::List(items) => items,
                Self::Text(_) | Self::Button { .. } | Self::Input { .. } => &[],
            }
        }

        fn validate(&self, depth: usize, count: &mut usize) {
            if depth > MAX_DEPTH {
                eprintln!("Erro de UI: profundidade máxima de {MAX_DEPTH} níveis excedida.");
                std::process::exit(1);
            }
            *count += 1;
            if *count > MAX_NODES {
                eprintln!("Erro de UI: limite máximo de {MAX_NODES} nós excedido.");
                std::process::exit(1);
            }
            for child in self.children() {
                child.validate(depth + 1, count);
            }
        }

        fn write_json(&self, out: &mut String) {
            match self {
                Self::Column(children) => {
                    out.push_str("{\"kind\":\"coluna\",\"children\":");
                    write_nodes(children, out);
                    out.push('}');
                }
                Self::Row(children) => {
                    out.push_str("{\"kind\":\"linha\",\"children\":");
                    write_nodes(children, out);
                    out.push('}');
                }
                Self::Text(content) => {
                    out.push_str("{\"kind\":\"texto\",\"content\":");
                    write_json_str(content, out);
                    out.push('}');
                }
                Self::Button { id, label } => {
                    out.push_str("{\"kind\":\"botao\",\"id\":");
                    write_json_str(id, out);
                    out.push_str(",\"label\":");
                    write_json_str(label, out);
                    out.push('}');
                }
                Self::Input { id, value, placeholder } => {
                    out.push_str("{\"kind\":\"entrada\",\"id\":");
                    write_json_str(id, out);
                    out.push_str(",\"value\":");
                    write_json_str(value, out);
                    out.push_str(",\"placeholder\":");
                    write_json_str(placeholder, out);
                    out.push('}');
                }
                Self::List(items) => {
                    out.push_str("{\"kind\":\"lista\",\"items\":");
                    write_nodes(items, out);
                    out.push('}');
                }
            }
        }
    }

    fn write_nodes(nodes: &[UiNode], out: &mut String) {
        out.push('[');
        for (i, node) in nodes.iter().enumerate() {
            if i > 0 { out.push(','); }
            node.write_json(out);
        }
        out.push(']');
    }

    fn write_json_str(value: &str, out: &mut String) {
        out.push('"');
        for ch in value.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                '\x08' => out.push_str("\\b"),
                '\x0c' => out.push_str("\\f"),
                c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); }
                other => out.push(other),
            }
        }
        out.push('"');
    }

    thread_local! {
        static NODES: RefCell<Vec<UiNode>> = const { RefCell::new(Vec::new()) };
    }

    pub fn begin_frame() {
        NODES.with(|nodes| nodes.borrow_mut().clear());
    }

    fn push(node: UiNode) -> i32 {
        NODES.with(|nodes| {
            let mut list = nodes.borrow_mut();
            if list.len() >= MAX_NODES {
                eprintln!("Erro de UI: limite máximo de {MAX_NODES} nós excedido.");
                std::process::exit(1);
            }
            let handle = list.len() as i32;
            list.push(node);
            handle
        })
    }

    fn resolve(handles: &[i32]) -> Vec<UiNode> {
        NODES.with(|nodes| {
            let list = nodes.borrow();
            let mut out = Vec::with_capacity(handles.len());
            for &handle in handles {
                let Some(node) = usize::try_from(handle).ok().and_then(|i| list.get(i)) else {
                    eprintln!("Erro de UI: identificador de nó inválido ({handle}).");
                    std::process::exit(1);
                };
                out.push(node.clone());
            }
            out
        })
    }

    pub fn finish_frame(root: i32) -> UiNode {
        let node = NODES.with(|nodes| {
            let list = nodes.borrow();
            usize::try_from(root)
                .ok()
                .and_then(|i| list.get(i))
                .cloned()
                .unwrap_or_else(|| {
                    eprintln!("Erro de UI: nó raiz inválido ({root}).");
                    std::process::exit(1);
                })
        });
        let mut count = 0usize;
        node.validate(1, &mut count);
        node
    }

    pub fn texto(parts: &[String]) -> i32 {
        let mut content = String::new();
        for part in parts {
            content.push_str(part);
        }
        push(UiNode::Text(content))
    }

    pub fn botao(id: &str, label: &str) -> i32 {
        push(UiNode::Button { id: id.to_owned(), label: label.to_owned() })
    }

    pub fn entrada(id: &str, value: &str, placeholder: &str) -> i32 {
        push(UiNode::Input {
            id: id.to_owned(),
            value: value.to_owned(),
            placeholder: placeholder.to_owned(),
        })
    }

    pub fn coluna(children: &[i32]) -> i32 {
        push(UiNode::Column(resolve(children)))
    }

    pub fn linha(children: &[i32]) -> i32 {
        push(UiNode::Row(resolve(children)))
    }

    pub fn lista(items: &[i32]) -> i32 {
        push(UiNode::List(resolve(items)))
    }

    pub fn parse_event_line(line: &str) -> Option<(String, String, String)> {
        if line.starts_with('{') {
            let kind = json_string_field(line, "kind")?;
            let id = json_string_field(line, "id")?;
            let value = json_string_field(line, "value").unwrap_or_default();
            return Some((kind, id, value));
        }
        if let Some(id) = line.strip_prefix("clique:") {
            return Some(("clique".to_owned(), id.to_owned(), String::new()));
        }
        if let Some(rest) = line.strip_prefix("entrada_alterada:")
            && let Some((id, value)) = rest.split_once(':')
        {
            return Some(("entrada_alterada".to_owned(), id.to_owned(), value.to_owned()));
        }
        None
    }

    fn json_string_field(raw: &str, key: &str) -> Option<String> {
        let needle = format!("\"{key}\"");
        let after_key = raw.split_once(&needle)?.1;
        let after_colon = after_key.trim_start().strip_prefix(':')?.trim_start();
        let mut chars = after_colon.strip_prefix('"')?.chars();
        let mut out = String::new();
        while let Some(ch) = chars.next() {
            match ch {
                '"' => return Some(out),
                '\\' => match chars.next()? {
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'n' => out.push('\n'),
                    'r' => out.push('\r'),
                    't' => out.push('\t'),
                    'b' => out.push('\x08'),
                    'f' => out.push('\x0c'),
                    other => out.push(other),
                },
                other => out.push(other),
            }
        }
        None
    }
}
"#;

pub(super) const PORS_MATEMATICA_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_matematica {
    pub fn potencia(base: f64, expoente: f64) -> f64 { base.powf(expoente) }
    pub fn raiz(radicando: f64, indice: f64) -> f64 { radicando.powf(1.0 / indice) }
    pub fn raiz_quadrada(numero: f64) -> f64 { numero.sqrt() }
    pub fn arredondar(numero: f64, casas: i32) -> f64 {
        let factor = 10_f64.powi(casas.clamp(0, 15));
        (numero * factor).round() / factor
    }
    pub fn piso(numero: f64) -> f64 { numero.floor() }
    pub fn teto(numero: f64) -> f64 { numero.ceil() }
    pub fn logaritmo(numero: f64, base: f64) -> f64 { numero.ln() / base.ln() }
    pub fn seno(angulo: f64) -> f64 { angulo.sin() }
    pub fn cosseno(angulo: f64) -> f64 { angulo.cos() }
    pub fn tangente(angulo: f64) -> f64 { angulo.tan() }
    pub fn valor_absoluto(numero: f64) -> f64 { numero.abs() }
    pub fn valor_absoluto_inteiro(numero: i32) -> i32 { numero.wrapping_abs() }
    pub fn maior_numero(a: f64, b: f64) -> f64 { if a.is_nan() || b.is_nan() { f64::NAN } else { a.max(b) } }
    pub fn menor_numero(a: f64, b: f64) -> f64 { if a.is_nan() || b.is_nan() { f64::NAN } else { a.min(b) } }
    pub fn maior_inteiro(a: i32, b: i32) -> i32 { a.max(b) }
    pub fn menor_inteiro(a: i32, b: i32) -> i32 { a.min(b) }
}
"#;

pub(super) const PORS_TEXTO_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_texto {
    pub fn numero_caracteres(texto: &str) -> i32 { texto.chars().count() as i32 }
    pub fn caixa_alta(texto: &str) -> String {
        let out = texto.to_uppercase();
        super::__ferro_account(out.len());
        out
    }
    pub fn caixa_baixa(texto: &str) -> String {
        let out = texto.to_lowercase();
        super::__ferro_account(out.len());
        out
    }
    pub fn aparar(texto: &str) -> String { super::__ferro_string(texto.trim()) }
    pub fn contem(texto: &str, trecho: &str) -> bool { texto.contains(trecho) }
    pub fn comeca_com(texto: &str, prefixo: &str) -> bool { texto.starts_with(prefixo) }
    pub fn termina_com(texto: &str, sufixo: &str) -> bool { texto.ends_with(sufixo) }
    pub fn substituir(texto: &str, antigo: &str, novo: &str) -> String {
        let expected = if antigo.is_empty() {
            let slots = texto.chars().count().saturating_add(1);
            texto.len().saturating_add(slots.saturating_mul(novo.len()))
        } else {
            let count = texto.matches(antigo).count();
            let removed = count.saturating_mul(antigo.len());
            let added = count.saturating_mul(novo.len());
            texto.len().saturating_sub(removed).saturating_add(added)
        };
        super::__ferro_account(expected);
        texto.replace(antigo, novo)
    }
    pub fn preencher_a_esquerda(car: char, tamanho: i32, texto: &str) -> String {
        let len = texto.chars().count();
        if tamanho <= 0 || len >= tamanho as usize {
            return super::__ferro_string(texto);
        }
        let pad = tamanho as usize - len;
        let expected = pad.saturating_mul(car.len_utf8()).saturating_add(texto.len());
        super::__ferro_account(expected);
        let mut out = String::with_capacity(expected);
        for _ in 0..pad {
            out.push(car);
        }
        out.push_str(texto);
        out
    }
    pub fn obter_caracter(texto: &str, indice: i32) -> Option<char> {
        if indice < 0 {
            return None;
        }
        texto.chars().nth(indice as usize)
    }
    pub fn posicao_texto(trecho: &str, texto: &str, inicio: i32) -> Option<i32> {
        let chars: Vec<char> = texto.chars().collect();
        let needle: Vec<char> = trecho.chars().collect();
        let start = inicio.max(0) as usize;
        if start > chars.len() {
            return None;
        }
        if needle.is_empty() {
            return Some(start as i32);
        }
        if start == chars.len() {
            return None;
        }
        chars[start..]
            .windows(needle.len())
            .position(|window| window == needle.as_slice())
            .map(|at| (start + at) as i32)
    }
    pub fn extrair_subtexto(texto: &str, inicio: i32, fim: i32) -> Result<String, String> {
        let chars: Vec<char> = texto.chars().collect();
        if inicio < 0 || fim < inicio || fim as usize > chars.len() {
            return Err(super::__ferro_string(
                "Posição inicial ou final inválida para extrair subtexto",
            ));
        }
        let slice: String = chars[inicio as usize..fim as usize].iter().collect();
        super::__ferro_account(slice.len());
        Ok(slice)
    }
    pub fn dividir(texto: &str, separador: &str) -> Vec<String> {
        if separador.is_empty() {
            super::__ferro_account(texto.len());
            return texto.chars().map(|ch| ch.to_string()).collect();
        }
        let parts: Vec<&str> = texto.split(separador).collect();
        let total: usize = parts.iter().map(|part| part.len()).sum();
        super::__ferro_account(total);
        parts.into_iter().map(str::to_owned).collect()
    }
}
"#;

pub(super) const PORS_TIPOS_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_tipos {
    pub fn para_inteiro(texto: &str) -> Result<i32, String> {
        texto.trim().parse::<i32>().map_err(|_| {
            super::__ferro_string(&format!("O valor '{texto}' não é um inteiro válido"))
        })
    }
    pub fn para_inteiro_base(texto: &str, base: i32) -> Result<i32, String> {
        if !matches!(base, 2 | 10 | 16) {
            return Err(super::__ferro_string(&format!(
                "Base inválida ({base}); use 2, 10 ou 16"
            )));
        }
        let trimmed = texto.trim();
        let (neg, rest) = match trimmed.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, trimmed),
        };
        let digits = match base {
            2 => rest
                .strip_prefix("0b")
                .or_else(|| rest.strip_prefix("0B"))
                .unwrap_or(rest),
            16 => rest
                .strip_prefix("0x")
                .or_else(|| rest.strip_prefix("0X"))
                .unwrap_or(rest),
            _ => rest,
        };
        if digits.is_empty() || digits.starts_with('+') || digits.starts_with('-') {
            return Err(super::__ferro_string(&format!(
                "O valor '{texto}' não é um inteiro válido"
            )));
        }
        let candidate = if neg {
            format!("-{digits}")
        } else {
            digits.to_owned()
        };
        i32::from_str_radix(&candidate, base as u32).map_err(|_| {
            super::__ferro_string(&format!("O valor '{texto}' não é um inteiro válido"))
        })
    }
    pub fn para_real(texto: &str) -> Result<f64, String> {
        match texto.trim().parse::<f64>() {
            Ok(valor) if valor.is_finite() => Ok(valor),
            _ => Err(super::__ferro_string(&format!(
                "O valor '{texto}' não é um número real válido"
            ))),
        }
    }
    pub fn para_logico(texto: &str) -> Result<bool, String> {
        match texto.trim() {
            "verdadeiro" => Ok(true),
            "falso" => Ok(false),
            _ => Err(super::__ferro_string(&format!(
                "O valor '{texto}' não é um valor lógico válido"
            ))),
        }
    }
    pub fn para_caracter(texto: &str) -> Result<char, String> {
        let mut chars = texto.chars();
        match (chars.next(), chars.next()) {
            (Some(ch), None) => Ok(ch),
            _ => Err(super::__ferro_string(&format!(
                "O valor '{texto}' não é um caracter válido"
            ))),
        }
    }
    pub fn inteiro_para_real(valor: i32) -> f64 { valor as f64 }
    pub fn real_para_inteiro(valor: f64) -> i32 { valor as i32 }
    pub fn inteiro_para_cadeia(valor: i32) -> String { super::__ferro_string(&valor.to_string()) }
    pub fn inteiro_para_cadeia_base(valor: i32, base: i32) -> Result<String, String> {
        let out = match base {
            2 => format!("{:b}", valor as u32),
            10 => valor.to_string(),
            16 => format!("{:X}", valor as u32),
            _ => {
                return Err(super::__ferro_string(&format!(
                    "Base inválida ({base}); use 2, 10 ou 16"
                )));
            }
        };
        super::__ferro_account(out.len());
        Ok(out)
    }
    pub fn real_para_cadeia(valor: f64) -> String {
        super::__ferro_string(&super::__ferro_java_double(valor))
    }
    pub fn logico_para_cadeia(valor: bool) -> String {
        super::__ferro_string(if valor { "verdadeiro" } else { "falso" })
    }
    pub fn caracter_para_cadeia(valor: char) -> String { super::__ferro_string(&valor.to_string()) }
    pub fn logico_para_inteiro(valor: bool) -> i32 { i32::from(valor) }
    pub fn caracter_para_inteiro(valor: char) -> Result<i32, String> {
        match valor.to_digit(10) {
            Some(digit) => Ok(digit as i32),
            None => Err(super::__ferro_string(&format!(
                "O caracter '{valor}' não é um dígito válido"
            ))),
        }
    }
    pub fn inteiro_para_caracter(valor: i32) -> Result<char, String> {
        if (0..=9).contains(&valor) {
            Ok(char::from(b'0' + valor as u8))
        } else {
            Err(super::__ferro_string(&format!(
                "O inteiro '{valor}' não corresponde a um dígito (0 a 9)"
            )))
        }
    }
    pub fn cadeia_e_inteiro(texto: &str) -> bool { texto.trim().parse::<i32>().is_ok() }
    pub fn cadeia_e_real(texto: &str) -> bool {
        texto.trim().parse::<f64>().is_ok_and(f64::is_finite)
    }
    pub fn cadeia_e_logico(texto: &str) -> bool {
        matches!(texto.trim(), "verdadeiro" | "falso")
    }
    pub fn cadeia_e_caracter(texto: &str) -> bool { texto.chars().count() == 1 }
}
"#;

pub(super) const PORS_CLOCK_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_clock {
    use std::cell::RefCell;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    enum ClockState {
        Real(Instant),
        Simulated { elapsed_ms: u64, epoch_ms: u64 },
    }

    thread_local! {
        static CLOCK: RefCell<ClockState> = RefCell::new(match std::env::var("FERRO_SIMULATED_EPOCH_MS").ok().and_then(|v| v.parse::<u64>().ok()) {
            Some(epoch_ms) => ClockState::Simulated { elapsed_ms: 0, epoch_ms },
            None => ClockState::Real(Instant::now()),
        });
    }

    pub fn elapsed_ms() -> u64 {
        CLOCK.with(|cell| match &*cell.borrow() {
            ClockState::Real(started) => started.elapsed().as_millis() as u64,
            ClockState::Simulated { elapsed_ms, .. } => *elapsed_ms,
        })
    }

    pub fn now_epoch_ms() -> i64 {
        CLOCK.with(|cell| match &*cell.borrow() {
            ClockState::Real(_) => SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_millis() as i64)
                .unwrap_or(0),
            ClockState::Simulated { elapsed_ms, epoch_ms } => epoch_ms.saturating_add(*elapsed_ms) as i64,
        })
    }

    pub fn sleep_ms(ms: u64) {
        let simulated = CLOCK.with(|cell| {
            if let ClockState::Simulated { elapsed_ms, .. } = &mut *cell.borrow_mut() {
                *elapsed_ms = elapsed_ms.saturating_add(ms);
                true
            } else {
                false
            }
        });
        if !simulated {
            std::thread::sleep(Duration::from_millis(ms));
        }
    }
}
"#;

pub(super) const PORS_UTIL_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_util {
    use std::cell::Cell;
    use std::time::{SystemTime, UNIX_EPOCH};

    const MAX_SLEEP_MS: i32 = 60_000;

    thread_local! {
        static RNG: Cell<u64> = Cell::new(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map(|d| d.as_nanos() as u64)
                .unwrap_or(0x9E37_79B9_7F4A_7C15)
                | 1,
        );
    }

    fn next_u64() -> u64 {
        RNG.with(|cell| {
            let mut x = cell.get();
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            cell.set(x);
            x.wrapping_mul(0x2545_F491_4F6C_DD1D)
        })
    }

    pub fn definir_semente(semente: i32) {
        RNG.with(|cell| cell.set((semente as u32 as u64) | 1));
    }

    pub fn sorteia(min: i32, max: i32) -> Result<i32, String> {
        if min > max {
            return Err(super::__ferro_string(&format!(
                "O valor mínimo ({min}) não pode ser maior que o máximo ({max})"
            )));
        }
        let span = (max as i64 - min as i64 + 1) as u64;
        let offset = (next_u64() % span) as i64;
        Ok((min as i64 + offset) as i32)
    }

    pub fn tempo_decorrido() -> i32 {
        super::__ferro_clock::elapsed_ms().min(i32::MAX as u64) as i32
    }

    pub fn aguarde(ms: i32) {
        if ms < 0 {
            eprintln!("Erro de execução: o intervalo de espera em `Util.aguarde` não pode ser negativo.");
            std::process::exit(1);
        }
        if ms > MAX_SLEEP_MS {
            eprintln!("Erro de execução: o intervalo de espera ({ms} ms) excede o limite de {MAX_SLEEP_MS} ms por chamada.");
            std::process::exit(1);
        }
        super::__ferro_clock::sleep_ms(ms as u64);
    }
}
"#;

pub(super) const PORS_CALENDARIO_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_calendario {
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

    fn now_utc() -> UtcDateTime {
        let epoch_ms = super::__ferro_clock::now_epoch_ms();
        let millis = epoch_ms.rem_euclid(1_000) as u32;
        let total_secs = epoch_ms.div_euclid(1_000);
        let secs_of_day = total_secs.rem_euclid(86_400) as u32;
        let days = total_secs.div_euclid(86_400);
        let (year, month, day) = civil_from_days(days);
        let weekday_1_to_7 = ((days + 4).rem_euclid(7) + 1) as i32;
        UtcDateTime {
            year,
            month,
            day,
            weekday_1_to_7,
            hour: secs_of_day / 3_600,
            minute: (secs_of_day % 3_600) / 60,
            second: secs_of_day % 60,
            millis,
        }
    }

    pub fn ano_atual() -> i32 { now_utc().year }
    pub fn mes_atual() -> i32 { now_utc().month as i32 }
    pub fn dia_mes_atual() -> i32 { now_utc().day as i32 }
    pub fn dia_semana_atual() -> i32 { now_utc().weekday_1_to_7 }
    pub fn hora_atual(formato_12h: bool) -> i32 {
        let h = now_utc().hour as i32;
        if formato_12h { h % 12 } else { h }
    }
    pub fn minuto_atual() -> i32 { now_utc().minute as i32 }
    pub fn segundo_atual() -> i32 { now_utc().second as i32 }
    pub fn milisegundo_atual() -> i32 { now_utc().millis as i32 }
    pub fn eh_bissexto(ano: i32) -> bool {
        (ano % 4 == 0 && ano % 100 != 0) || ano % 400 == 0
    }
    pub fn dias_no_mes(ano: i32, mes: i32) -> Result<i32, String> {
        match mes {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => Ok(31),
            4 | 6 | 9 | 11 => Ok(30),
            2 if eh_bissexto(ano) => Ok(29),
            2 => Ok(28),
            _ => Err(super::__ferro_string(&format!(
                "'{mes}' não corresponde a um mês válido (1 a 12)"
            ))),
        }
    }
    pub fn nome_mes(mes: i32) -> Result<String, String> {
        const MESES: [&str; 12] = [
            "Janeiro", "Fevereiro", "Março", "Abril", "Maio", "Junho",
            "Julho", "Agosto", "Setembro", "Outubro", "Novembro", "Dezembro",
        ];
        if !(1..=12).contains(&mes) {
            return Err(super::__ferro_string(&format!(
                "'{mes}' não corresponde a um mês válido (1 a 12)"
            )));
        }
        Ok(super::__ferro_string(MESES[(mes - 1) as usize]))
    }
    fn format_day(names: [&str; 7], dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        if !(1..=7).contains(&dia) {
            return Err(super::__ferro_string(&format!(
                "'{dia}' não corresponde a um dia da semana válido (1 a 7)"
            )));
        }
        let raw = names[(dia - 1) as usize];
        let out = if alta {
            raw.to_uppercase()
        } else if baixa {
            raw.to_lowercase()
        } else {
            raw.to_owned()
        };
        super::__ferro_account(out.len());
        Ok(out)
    }
    pub fn dia_semana_completo(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(
            ["Domingo", "Segunda-Feira", "Terça-Feira", "Quarta-Feira", "Quinta-Feira", "Sexta-Feira", "Sábado"],
            dia,
            alta,
            baixa,
        )
    }
    pub fn dia_semana_curto(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(
            ["Domingo", "Segunda", "Terça", "Quarta", "Quinta", "Sexta", "Sábado"],
            dia,
            alta,
            baixa,
        )
    }
    pub fn dia_semana_abreviado(dia: i32, alta: bool, baixa: bool) -> Result<String, String> {
        format_day(["Dom", "Seg", "Ter", "Qua", "Qui", "Sex", "Sáb"], dia, alta, baixa)
    }
    pub fn data_iso_atual() -> String {
        let now = now_utc();
        let iso = format!("{:04}-{:02}-{:02}", now.year, now.month, now.day);
        super::__ferro_account(iso.len());
        iso
    }
}
"#;

pub(super) const PORS_ARQUIVOS_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_arquivos {
    use std::io::Write as _;
    use std::path::{Component, Path, PathBuf};

    const MAX_FILE_BYTES: usize = 2 * 1024 * 1024;
    const MAX_LIST_ENTRIES: usize = 1_024;

    fn resolve(raw: &str, allow_root: bool) -> Result<PathBuf, String> {
        let root = std::env::var_os("FERRO_WORKSPACE_DIR")
            .map(PathBuf::from)
            .or_else(|| std::env::current_dir().ok())
            .ok_or_else(|| super::__ferro_string("Diretório de workspace indisponível."))?;
        if raw.contains('\0') {
            return Err(super::__ferro_string("Caminho inválido: contém caractere nulo."));
        }
        let trimmed = raw.trim();
        if trimmed.is_empty() && !allow_root {
            return Err(super::__ferro_string("O caminho do arquivo não pode ser vazio."));
        }
        let rel = Path::new(trimmed);
        if rel.is_absolute() {
            return Err(super::__ferro_string("Apenas caminhos relativos ao workspace são permitidos."));
        }
        let mut clean = PathBuf::new();
        for component in rel.components() {
            match component {
                Component::Normal(part) => clean.push(part),
                Component::CurDir => {}
                Component::ParentDir | Component::RootDir | Component::Prefix(_) => {
                    return Err(super::__ferro_string(
                        "Caminho fora do workspace não é permitido (`..` ou raiz absoluta).",
                    ));
                }
            }
        }
        if clean.as_os_str().is_empty() && !allow_root {
            return Err(super::__ferro_string("O caminho deve apontar para um arquivo dentro do workspace."));
        }
        let canonical_root = std::fs::canonicalize(&root)
            .map_err(|err| super::__ferro_string(&format!("Workspace inacessível: {err}")))?;
        let target = canonical_root.join(&clean);
        let mut probe = target.as_path();
        while !probe.exists() {
            let Some(parent) = probe.parent() else { break };
            probe = parent;
        }
        if probe.exists() {
            let canonical_probe = std::fs::canonicalize(probe)
                .map_err(|err| super::__ferro_string(&format!("Não foi possível validar o caminho: {err}")))?;
            if !canonical_probe.starts_with(&canonical_root) {
                return Err(super::__ferro_string("Caminho escapa do workspace (link simbólico bloqueado)."));
            }
        }
        Ok(target)
    }

    pub fn ler_texto(caminho: &str) -> Result<String, String> {
        let target = resolve(caminho, false)?;
        let meta = std::fs::metadata(&target)
            .map_err(|err| super::__ferro_string(&format!("Não foi possível abrir '{caminho}': {err}")))?;
        if !meta.is_file() {
            return Err(super::__ferro_string(&format!("'{caminho}' não é um arquivo regular.")));
        }
        if meta.len() as usize > MAX_FILE_BYTES {
            return Err(super::__ferro_string(&format!(
                "O arquivo '{caminho}' excede o limite de {MAX_FILE_BYTES} bytes."
            )));
        }
        super::__ferro_account(meta.len() as usize);
        std::fs::read_to_string(&target)
            .map_err(|err| super::__ferro_string(&format!("Erro ao ler '{caminho}': {err}")))
    }

    pub fn ler_linhas(caminho: &str) -> Result<Vec<String>, String> {
        let content = ler_texto(caminho)?;
        Ok(content.lines().map(str::to_owned).collect())
    }

    pub fn escrever_texto(caminho: &str, conteudo: &str) -> Result<bool, String> {
        if conteudo.len() > MAX_FILE_BYTES {
            return Err(super::__ferro_string(&format!(
                "O conteúdo excede o limite de {MAX_FILE_BYTES} bytes por arquivo."
            )));
        }
        let target = resolve(caminho, false)?;
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                super::__ferro_string(&format!("Não foi possível criar diretório para '{caminho}': {err}"))
            })?;
        }
        std::fs::write(&target, conteudo)
            .map(|()| true)
            .map_err(|err| super::__ferro_string(&format!("Erro ao gravar '{caminho}': {err}")))
    }

    pub fn acrescentar_texto(caminho: &str, conteudo: &str) -> Result<bool, String> {
        let target = resolve(caminho, false)?;
        let existing_len = std::fs::metadata(&target).map(|m| m.len() as usize).unwrap_or(0);
        if existing_len.saturating_add(conteudo.len()) > MAX_FILE_BYTES {
            return Err(super::__ferro_string(&format!(
                "O arquivo '{caminho}' excederia o limite de {MAX_FILE_BYTES} bytes."
            )));
        }
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|err| {
                super::__ferro_string(&format!("Não foi possível criar diretório para '{caminho}': {err}"))
            })?;
        }
        let mut handle = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&target)
            .map_err(|err| super::__ferro_string(&format!("Erro ao acrescentar em '{caminho}': {err}")))?;
        handle
            .write_all(conteudo.as_bytes())
            .map(|()| true)
            .map_err(|err| super::__ferro_string(&format!("Erro ao acrescentar em '{caminho}': {err}")))
    }

    pub fn existe(caminho: &str) -> Result<bool, String> {
        Ok(resolve(caminho, true)?.exists())
    }

    pub fn remover(caminho: &str) -> Result<bool, String> {
        let target = resolve(caminho, false)?;
        std::fs::remove_file(&target)
            .map(|()| true)
            .map_err(|err| super::__ferro_string(&format!("Erro ao remover '{caminho}': {err}")))
    }

    pub fn listar(diretorio: &str) -> Result<Vec<String>, String> {
        let target = resolve(diretorio, true)?;
        let entries = std::fs::read_dir(&target)
            .map_err(|err| super::__ferro_string(&format!("Erro ao listar '{diretorio}': {err}")))?;
        let mut names = Vec::new();
        let mut total_bytes = 0usize;
        for entry in entries {
            let entry = entry.map_err(|err| super::__ferro_string(&format!("Erro ao listar '{diretorio}': {err}")))?;
            if names.len() >= MAX_LIST_ENTRIES {
                return Err(super::__ferro_string(&format!(
                    "Diretório excede o limite de {MAX_LIST_ENTRIES} entradas."
                )));
            }
            let item = entry.file_name().to_string_lossy().into_owned();
            total_bytes = total_bytes.saturating_add(item.len());
            names.push(item);
        }
        names.sort();
        super::__ferro_account(total_bytes);
        Ok(names)
    }
}
"#;

pub(super) const PORS_JSON_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_json {
    use std::fmt::Write as _;

    const MAX_JSON_DEPTH: usize = 64;
    const MAX_JSON_BYTES: usize = 2 * 1024 * 1024;

    enum Value {
        Null,
        Bool(bool),
        Int(i64),
        Float(f64),
        Str(String),
        Array(Vec<Value>),
        Object(Vec<(String, Value)>),
    }

    struct Parser<'a> {
        src: &'a str,
        bytes: &'a [u8],
        pos: usize,
    }

    impl<'a> Parser<'a> {
        fn parse(src: &'a str) -> Result<Value, String> {
            if src.len() > MAX_JSON_BYTES {
                return Err(super::__ferro_string(&format!(
                    "Documento JSON excede o limite de {MAX_JSON_BYTES} bytes."
                )));
            }
            let mut p = Self { src, bytes: src.as_bytes(), pos: 0 };
            p.skip_ws();
            let v = p.parse_value(0)?;
            p.skip_ws();
            if p.pos != p.bytes.len() {
                return Err(super::__ferro_string("Conteúdo inesperado após o término do JSON."));
            }
            Ok(v)
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

        fn parse_value(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__ferro_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.skip_ws();
            match self.peek() {
                Some(b'{') => self.parse_object(depth + 1),
                Some(b'[') => self.parse_array(depth + 1),
                Some(b'"') => self.parse_string().map(Value::Str),
                Some(b't') => { self.expect(b"true")?; Ok(Value::Bool(true)) }
                Some(b'f') => { self.expect(b"false")?; Ok(Value::Bool(false)) }
                Some(b'n') => { self.expect(b"null")?; Ok(Value::Null) }
                Some(b'-' | b'0'..=b'9') => self.parse_number(),
                Some(_) => Err(super::__ferro_string(&format!(
                    "Caractere inesperado no JSON na posição {}.",
                    self.pos
                ))),
                None => Err(super::__ferro_string("Fim inesperado do JSON.")),
            }
        }

        fn expect(&mut self, lit: &[u8]) -> Result<(), String> {
            if self.bytes.get(self.pos..self.pos + lit.len()) == Some(lit) {
                self.pos += lit.len();
                Ok(())
            } else {
                Err(super::__ferro_string(&format!("Token JSON inválido na posição {}.", self.pos)))
            }
        }

        fn parse_object(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__ferro_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.pos += 1;
            self.skip_ws();
            let mut entries: Vec<(String, Value)> = Vec::new();
            if self.peek() == Some(b'}') {
                self.pos += 1;
                return Ok(Value::Object(entries));
            }
            loop {
                self.skip_ws();
                if self.peek() != Some(b'"') {
                    return Err(super::__ferro_string(&format!(
                        "Esperada chave de texto no objeto JSON na posição {}.",
                        self.pos
                    )));
                }
                let key = self.parse_string()?;
                self.skip_ws();
                if self.peek() != Some(b':') {
                    return Err(super::__ferro_string(&format!(
                        "Esperado `:` após chave JSON na posição {}.",
                        self.pos
                    )));
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
                    Some(b'}') => { self.pos += 1; return Ok(Value::Object(entries)); }
                    _ => return Err(super::__ferro_string(&format!(
                        "Esperado `,` ou `}}` no objeto JSON na posição {}.",
                        self.pos
                    ))),
                }
            }
        }

        fn parse_array(&mut self, depth: usize) -> Result<Value, String> {
            if depth > MAX_JSON_DEPTH {
                return Err(super::__ferro_string(&format!(
                    "Profundidade máxima de JSON excedida (limite: {MAX_JSON_DEPTH})."
                )));
            }
            self.pos += 1;
            self.skip_ws();
            let mut items = Vec::new();
            if self.peek() == Some(b']') {
                self.pos += 1;
                return Ok(Value::Array(items));
            }
            loop {
                items.push(self.parse_value(depth)?);
                self.skip_ws();
                match self.peek() {
                    Some(b',') => self.pos += 1,
                    Some(b']') => { self.pos += 1; return Ok(Value::Array(items)); }
                    _ => return Err(super::__ferro_string(&format!(
                        "Esperado `,` ou `]` na lista JSON na posição {}.",
                        self.pos
                    ))),
                }
            }
        }

        fn parse_hex4(&mut self) -> Result<u16, String> {
            let Some(slice) = self.src.get(self.pos..self.pos + 4) else {
                return Err(super::__ferro_string("Escape Unicode `\\u` incompleto no JSON."));
            };
            let code = u16::from_str_radix(slice, 16)
                .map_err(|_| super::__ferro_string("Escape Unicode `\\u` inválido no JSON."))?;
            self.pos += 4;
            Ok(code)
        }

        fn parse_string(&mut self) -> Result<String, String> {
            self.pos += 1;
            let mut out = String::new();
            let mut start = self.pos;
            while let Some(&b) = self.bytes.get(self.pos) {
                match b {
                    b'"' => {
                        out.push_str(&self.src[start..self.pos]);
                        self.pos += 1;
                        return Ok(out);
                    }
                    b'\\' => {
                        out.push_str(&self.src[start..self.pos]);
                        self.pos += 1;
                        let esc = self.peek().ok_or_else(|| super::__ferro_string("Escape incompleto na cadeia JSON."))?;
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
                                        return Err(super::__ferro_string("Par substituto UTF-16 incompleto no JSON."));
                                    }
                                    self.pos += 2;
                                    let second = self.parse_hex4()?;
                                    if !(0xDC00..=0xDFFF).contains(&second) {
                                        return Err(super::__ferro_string("Par substituto UTF-16 inválido no JSON."));
                                    }
                                    let scalar = 0x10000 + (((first as u32 - 0xD800) << 10) | (second as u32 - 0xDC00));
                                    let ch = char::from_u32(scalar)
                                        .ok_or_else(|| super::__ferro_string("Código Unicode inválido no JSON."))?;
                                    out.push(ch);
                                } else if (0xDC00..=0xDFFF).contains(&first) {
                                    return Err(super::__ferro_string("Substituto UTF-16 isolado inválido no JSON."));
                                } else {
                                    let ch = char::from_u32(first as u32)
                                        .ok_or_else(|| super::__ferro_string("Código Unicode inválido no JSON."))?;
                                    out.push(ch);
                                }
                            }
                            _ => return Err(super::__ferro_string("Sequência de escape inválida na cadeia JSON.")),
                        }
                        start = self.pos;
                    }
                    0x00..=0x1F => return Err(super::__ferro_string("Caractere de controle não escapado na cadeia JSON.")),
                    _ => self.pos += 1,
                }
            }
            Err(super::__ferro_string("Cadeia JSON não fechada."))
        }

        fn parse_number(&mut self) -> Result<Value, String> {
            let start = self.pos;
            if self.peek() == Some(b'-') { self.pos += 1; }
            match self.peek() {
                Some(b'0') => self.pos += 1,
                Some(b'1'..=b'9') => {
                    self.pos += 1;
                    while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
                }
                _ => return Err(super::__ferro_string(&format!("Número JSON inválido na posição {}.", self.pos))),
            }
            let mut is_float = false;
            if self.peek() == Some(b'.') {
                is_float = true;
                self.pos += 1;
                if !matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(super::__ferro_string(&format!("Fração decimal inválida no JSON na posição {}.", self.pos)));
                }
                while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
            }
            if matches!(self.peek(), Some(b'e' | b'E')) {
                is_float = true;
                self.pos += 1;
                if matches!(self.peek(), Some(b'+' | b'-')) { self.pos += 1; }
                if !matches!(self.peek(), Some(b'0'..=b'9')) {
                    return Err(super::__ferro_string(&format!("Expoente inválido no JSON na posição {}.", self.pos)));
                }
                while matches!(self.peek(), Some(b'0'..=b'9')) { self.pos += 1; }
            }
            let token = &self.src[start..self.pos];
            if !is_float {
                if let Ok(n) = token.parse::<i64>() {
                    return Ok(Value::Int(n));
                }
            }
            let f = token.parse::<f64>()
                .map_err(|_| super::__ferro_string(&format!("Número JSON inválido: `{token}`.")))?;
            if !f.is_finite() {
                return Err(super::__ferro_string(&format!("Número JSON fora do intervalo finito: `{token}`.")));
            }
            Ok(Value::Float(f))
        }
    }

    fn escape_into(text: &str, out: &mut String) {
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
                c if (c as u32) < 0x20 => { let _ = write!(out, "\\u{:04x}", c as u32); }
                c => out.push(c),
            }
        }
        out.push('"');
    }

    fn write_val(value: &Value, pretty: bool, indent: usize, out: &mut String) -> Result<(), String> {
        if out.len() > MAX_JSON_BYTES {
            return Err(super::__ferro_string(&format!("Saída JSON excede o limite de {MAX_JSON_BYTES} bytes.")));
        }
        match value {
            Value::Null => out.push_str("null"),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Int(n) => { let _ = write!(out, "{n}"); }
            Value::Float(f) => {
                if !f.is_finite() {
                    return Err(super::__ferro_string("Valores NaN ou infinitos não são válidos em JSON."));
                }
                let rendered = format!("{f}");
                out.push_str(&rendered);
                if !rendered.contains('.') && !rendered.contains('e') && !rendered.contains('E') {
                    out.push_str(".0");
                }
            }
            Value::Str(s) => escape_into(s, out),
            Value::Array(items) => {
                if items.is_empty() { out.push_str("[]"); return Ok(()); }
                out.push('[');
                for (i, item) in items.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    if pretty {
                        out.push('\n');
                        for _ in 0..indent + 2 { out.push(' '); }
                    }
                    write_val(item, pretty, indent + 2, out)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent { out.push(' '); }
                }
                out.push(']');
            }
            Value::Object(entries) => {
                if entries.is_empty() { out.push_str("{}"); return Ok(()); }
                out.push('{');
                for (i, (k, v)) in entries.iter().enumerate() {
                    if i > 0 { out.push(','); }
                    if pretty {
                        out.push('\n');
                        for _ in 0..indent + 2 { out.push(' '); }
                    }
                    escape_into(k, out);
                    out.push(':');
                    if pretty { out.push(' '); }
                    write_val(v, pretty, indent + 2, out)?;
                }
                if pretty {
                    out.push('\n');
                    for _ in 0..indent { out.push(' '); }
                }
                out.push('}');
            }
        }
        if out.len() > MAX_JSON_BYTES {
            return Err(super::__ferro_string(&format!("Saída JSON excede o limite de {MAX_JSON_BYTES} bytes.")));
        }
        Ok(())
    }

    fn serialize(value: &Value, pretty: bool) -> Result<String, String> {
        let mut out = String::new();
        write_val(value, pretty, 0, &mut out)?;
        super::__ferro_account(out.len());
        Ok(out)
    }

    fn parse_object(src: &str) -> Result<Vec<(String, Value)>, String> {
        match Parser::parse(src)? {
            Value::Object(entries) => Ok(entries),
            _ => Err(super::__ferro_string("O documento JSON precisa ser um objeto (`{ ... }`).")),
        }
    }

    fn get_entry<'a>(entries: &'a [(String, Value)], key: &str) -> Result<&'a Value, String> {
        entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            .ok_or_else(|| super::__ferro_string(&format!("Chave '{key}' não encontrada no objeto JSON.")))
    }

    fn set_key(src: &str, key: &str, val: Value) -> Result<String, String> {
        let mut entries = parse_object(src)?;
        if let Some(existing) = entries.iter_mut().find(|(k, _)| k == key) {
            existing.1 = val;
        } else {
            entries.push((key.to_owned(), val));
        }
        serialize(&Value::Object(entries), false)
    }

    pub fn valido(texto: &str) -> bool { Parser::parse(texto).is_ok() }
    pub fn formatar(texto: &str) -> Result<String, String> { serialize(&Parser::parse(texto)?, true) }
    pub fn minificar(texto: &str) -> Result<String, String> { serialize(&Parser::parse(texto)?, false) }
    pub fn escapar_cadeia(texto: &str) -> String {
        let mut out = String::with_capacity(texto.len().saturating_add(2));
        escape_into(texto, &mut out);
        super::__ferro_account(out.len());
        out
    }
    pub fn contem_chave(json: &str, chave: &str) -> Result<bool, String> {
        Ok(parse_object(json)?.iter().any(|(k, _)| k == chave))
    }
    pub fn obter_chaves(json: &str) -> Result<Vec<String>, String> {
        let entries = parse_object(json)?;
        let total: usize = entries.iter().map(|(k, _)| k.len()).sum();
        super::__ferro_account(total);
        Ok(entries.into_iter().map(|(k, _)| k).collect())
    }
    pub fn obter_cadeia(json: &str, chave: &str) -> Result<String, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Str(s) => Ok(super::__ferro_string(s)),
            _ => Err(super::__ferro_string(&format!("A chave '{chave}' não contém uma cadeia no JSON."))),
        }
    }
    pub fn obter_inteiro(json: &str, chave: &str) -> Result<i32, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Int(n) => i32::try_from(*n).map_err(|_| {
                super::__ferro_string(&format!("O valor de '{chave}' excede o intervalo de `inteiro` (32 bits)."))
            }),
            _ => Err(super::__ferro_string(&format!("A chave '{chave}' não contém um número inteiro no JSON."))),
        }
    }
    pub fn obter_real(json: &str, chave: &str) -> Result<f64, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Int(n) => Ok(*n as f64),
            Value::Float(f) => Ok(*f),
            _ => Err(super::__ferro_string(&format!("A chave '{chave}' não contém um número no JSON."))),
        }
    }
    pub fn obter_logico(json: &str, chave: &str) -> Result<bool, String> {
        let entries = parse_object(json)?;
        match get_entry(&entries, chave)? {
            Value::Bool(b) => Ok(*b),
            _ => Err(super::__ferro_string(&format!("A chave '{chave}' não contém um valor lógico no JSON."))),
        }
    }
    pub fn obter_json(json: &str, chave: &str) -> Result<String, String> {
        let entries = parse_object(json)?;
        serialize(get_entry(&entries, chave)?, false)
    }
    pub fn obter_lista_cadeia(json: &str, chave: &str) -> Result<Vec<String>, String> {
        let entries = parse_object(json)?;
        let Value::Array(items) = get_entry(&entries, chave)? else {
            return Err(super::__ferro_string(&format!("A chave '{chave}' não contém uma lista no JSON.")));
        };
        let mut out = Vec::with_capacity(items.len());
        let mut total = 0usize;
        for item in items {
            let Value::Str(s) = item else {
                return Err(super::__ferro_string(&format!("A lista em '{chave}' contém elementos que não são cadeias.")));
            };
            total = total.saturating_add(s.len());
            out.push(s.clone());
        }
        super::__ferro_account(total);
        Ok(out)
    }
    pub fn obter_lista_inteiro(json: &str, chave: &str) -> Result<Vec<i32>, String> {
        let entries = parse_object(json)?;
        let Value::Array(items) = get_entry(&entries, chave)? else {
            return Err(super::__ferro_string(&format!("A chave '{chave}' não contém uma lista no JSON.")));
        };
        let mut out = Vec::with_capacity(items.len());
        for item in items {
            let Value::Int(n) = item else {
                return Err(super::__ferro_string(&format!("A lista em '{chave}' contém elementos que não são inteiros.")));
            };
            let v = i32::try_from(*n).map_err(|_| {
                super::__ferro_string(&format!("A lista em '{chave}' contém inteiro fora do intervalo de 32 bits."))
            })?;
            out.push(v);
        }
        Ok(out)
    }
    pub fn definir_cadeia(json: &str, chave: &str, valor: &str) -> Result<String, String> {
        set_key(json, chave, Value::Str(valor.to_owned()))
    }
    pub fn definir_inteiro(json: &str, chave: &str, valor: i32) -> Result<String, String> {
        set_key(json, chave, Value::Int(i64::from(valor)))
    }
    pub fn definir_real(json: &str, chave: &str, valor: f64) -> Result<String, String> {
        if !valor.is_finite() {
            return Err(super::__ferro_string("Valores NaN ou infinitos não podem ser gravados em JSON."));
        }
        set_key(json, chave, Value::Float(valor))
    }
    pub fn definir_logico(json: &str, chave: &str, valor: bool) -> Result<String, String> {
        set_key(json, chave, Value::Bool(valor))
    }
    pub fn definir_json(json: &str, chave: &str, valor_json: &str) -> Result<String, String> {
        let nested = Parser::parse(valor_json)?;
        set_key(json, chave, nested)
    }
    pub fn remover_chave(json: &str, chave: &str) -> Result<String, String> {
        let mut entries = parse_object(json)?;
        entries.retain(|(k, _)| k != chave);
        serialize(&Value::Object(entries), false)
    }
}
"#;

pub(super) const PORS_INTERNET_RUNTIME: &str = r#"
#[allow(dead_code)]
mod __ferro_internet {
    use std::io::{Read as _, Write as _};
    use std::net::{TcpStream, ToSocketAddrs as _};
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::Duration;

    const MAX_HTTP_BYTES: usize = 1024 * 1024;
    static TIMEOUT_MS: AtomicU64 = AtomicU64::new(5_000);

    fn allowed() -> Result<(), String> {
        match std::env::var("FERRO_ALLOW_NETWORK") {
            Ok(v) if matches!(v.trim().to_ascii_lowercase().as_str(), "1" | "true" | "sim") => Ok(()),
            _ => Err(super::__ferro_string(
                "Acesso à rede bloqueado: defina FERRO_ALLOW_NETWORK=1 para autorizar.",
            )),
        }
    }

    fn parse_url(url: &str) -> Result<(String, u16, String), String> {
        let trimmed = url.trim();
        if trimmed.is_empty() || trimmed.bytes().any(|b| matches!(b, b'\r' | b'\n' | b'\0' | b' ')) {
            return Err(super::__ferro_string("URL inválida: vazia ou com caracteres de controle."));
        }
        if trimmed.starts_with("https://") {
            return Err(super::__ferro_string(
                "HTTPS no binário Rust standalone requer um cliente TLS no projeto Cargo; use http:// ou execute na VM.",
            ));
        }
        let rest = trimmed
            .strip_prefix("http://")
            .ok_or_else(|| super::__ferro_string("Apenas endereços http:// ou https:// são permitidos."))?;
        let (authority, path) = match rest.find('/') {
            Some(idx) => (&rest[..idx], &rest[idx..]),
            None => (rest, "/"),
        };
        if authority.is_empty() {
            return Err(super::__ferro_string("Endereço HTTP sem host válido."));
        }
        let (host, port) = if let Some((h, p)) = authority.rsplit_once(':') {
            let port = p
                .parse::<u16>()
                .map_err(|_| super::__ferro_string(&format!("Porta inválida na URL: '{p}'.")))?;
            (h, port)
        } else {
            (authority, 80)
        };
        Ok((host.to_owned(), port, path.to_owned()))
    }

    fn decode_chunked(raw: &[u8]) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        let mut pos = 0usize;
        while pos < raw.len() {
            let Some(rel) = raw[pos..].windows(2).position(|w| w == b"\r\n") else {
                return Err(super::__ferro_string("Resposta HTTP chunked malformada."));
            };
            let size_str = std::str::from_utf8(&raw[pos..pos + rel])
                .map_err(|_| super::__ferro_string("Tamanho de bloco chunked inválido."))?
                .split(';')
                .next()
                .unwrap_or("")
                .trim();
            let size = usize::from_str_radix(size_str, 16)
                .map_err(|_| super::__ferro_string("Tamanho hexadecimal chunked inválido."))?;
            pos += rel + 2;
            if size == 0 { break; }
            if out.len().saturating_add(size) > MAX_HTTP_BYTES {
                return Err(super::__ferro_string(&format!("A resposta HTTP excede o limite de {MAX_HTTP_BYTES} bytes.")));
            }
            if pos + size + 2 > raw.len() {
                return Err(super::__ferro_string("Bloco HTTP chunked truncado."));
            }
            out.extend_from_slice(&raw[pos..pos + size]);
            pos += size + 2;
        }
        Ok(out)
    }

    fn exchange(method: &str, url: &str, body: Option<&str>) -> Result<(u16, Vec<u8>), String> {
        let (host, port, path) = parse_url(url)?;
        let timeout = Duration::from_millis(TIMEOUT_MS.load(Ordering::Relaxed));
        let addr = (host.as_str(), port)
            .to_socket_addrs()
            .map_err(|err| super::__ferro_string(&format!("Não foi possível resolver '{host}': {err}")))?
            .next()
            .ok_or_else(|| super::__ferro_string(&format!("Host '{host}' não resolvido.")))?;
        let mut stream = TcpStream::connect_timeout(&addr, timeout)
            .map_err(|err| super::__ferro_string(&format!("Falha ao conectar em '{url}': {err}")))?;
        let _ = stream.set_read_timeout(Some(timeout));
        let _ = stream.set_write_timeout(Some(timeout));
        let mut req = format!(
            "{method} {path} HTTP/1.1\r\nHost: {host}:{port}\r\nUser-Agent: ferro-studio/1.0\r\nConnection: close\r\n"
        );
        if let Some(payload) = body {
            req.push_str(&format!(
                "Content-Type: application/json; charset=utf-8\r\nContent-Length: {}\r\n\r\n{payload}",
                payload.len()
            ));
        } else {
            req.push_str("\r\n");
        }
        stream
            .write_all(req.as_bytes())
            .map_err(|err| super::__ferro_string(&format!("Erro ao enviar requisição para '{url}': {err}")))?;
        let mut raw = Vec::new();
        stream
            .take((MAX_HTTP_BYTES + 16 * 1024 + 1) as u64)
            .read_to_end(&mut raw)
            .map_err(|err| super::__ferro_string(&format!("Erro ao ler resposta de '{url}': {err}")))?;
        let Some(header_end) = raw.windows(4).position(|w| w == b"\r\n\r\n") else {
            return Err(super::__ferro_string(&format!("Resposta HTTP inválida de '{url}'.")));
        };
        let head = std::str::from_utf8(&raw[..header_end])
            .map_err(|_| super::__ferro_string(&format!("Cabeçalhos HTTP inválidos de '{url}'.")))?;
        let status = head
            .lines()
            .next()
            .and_then(|line| line.split_whitespace().nth(1))
            .and_then(|code| code.parse::<u16>().ok())
            .ok_or_else(|| super::__ferro_string(&format!("Linha de status HTTP inválida de '{url}'.")))?;
        let is_chunked = head.lines().any(|line| {
            let lower = line.to_ascii_lowercase();
            lower.starts_with("transfer-encoding:") && lower.contains("chunked")
        });
        let raw_body = &raw[header_end + 4..];
        let body_bytes = if is_chunked {
            decode_chunked(raw_body)?
        } else {
            if raw_body.len() > MAX_HTTP_BYTES {
                return Err(super::__ferro_string(&format!("A resposta de '{url}' excede o limite de {MAX_HTTP_BYTES} bytes.")));
            }
            raw_body.to_vec()
        };
        Ok((status, body_bytes))
    }

    pub fn definir_tempo_limite(ms: i32) {
        TIMEOUT_MS.store((ms.max(0) as u64).clamp(100, 30_000), Ordering::Relaxed);
    }

    pub fn obter_texto(url: &str) -> Result<String, String> {
        allowed()?;
        let (status, bytes) = exchange("GET", url, None)?;
        if !(200..300).contains(&status) {
            return Err(super::__ferro_string(&format!("Requisição GET para '{url}' retornou HTTP {status}.")));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| super::__ferro_string(&format!("A resposta de '{url}' não é texto UTF-8 válido.")))?;
        super::__ferro_account(text.len());
        Ok(text)
    }

    pub fn enviar_json(url: &str, corpo_json: &str) -> Result<String, String> {
        allowed()?;
        if corpo_json.len() > MAX_HTTP_BYTES {
            return Err(super::__ferro_string(&format!("O corpo JSON excede o limite de {MAX_HTTP_BYTES} bytes.")));
        }
        let (status, bytes) = exchange("POST", url, Some(corpo_json))?;
        if !(200..300).contains(&status) {
            return Err(super::__ferro_string(&format!("Requisição POST para '{url}' retornou HTTP {status}.")));
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| super::__ferro_string(&format!("A resposta de '{url}' não é texto UTF-8 válido.")))?;
        super::__ferro_account(text.len());
        Ok(text)
    }

    pub fn endereco_disponivel(url: &str) -> Result<bool, String> {
        allowed()?;
        let _ = parse_url(url)?;
        Ok(match exchange("HEAD", url, None) {
            Ok((status, _)) => (200..400).contains(&status),
            Err(_) => false,
        })
    }
}
"#;
