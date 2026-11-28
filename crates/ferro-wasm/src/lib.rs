//! Fachada WebAssembly sem dependências externas para o núcleo FERRO mínimo e
//! o runtime declarativo MVU de UI (ADR 0017).
//!
//! Compila nativamente para `wasm32-unknown-unknown` usando `ferro-vm` sem
//! a feature `stdlib`, expondo:
//! - validação (`ferro_validate`), formatação canônica (`ferro_format`) e execução
//!   com orçamentos de passos/memória/saída (`ferro_run`) de programas `.fer`;
//! - gerenciamento de sessões MVU (`ferro_ui_start`, `ferro_ui_dispatch`,
//!   `ferro_ui_snapshot`, `ferro_ui_close`) sobre [`UiSession`];
//! - respostas em JSON determinístico (RFC 8259) alocadas em memória linear via
//!   buffer reutilizável por thread (`ferro_result_ptr` / `ferro_result_len`).

use std::alloc::{Layout, alloc, dealloc};
use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::fmt::Write as _;
use std::slice;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use ferro_bytecode::{Module, ValueType};
use ferro_frontend::WhitespaceEdit;
use ferro_hir::Diagnostic;
use ferro_vm::ui::{DEFAULT_MAX_STRING_BYTES, UiBudget, UiError, UiEvent, UiSession};
use ferro_vm::{Io, RuntimeLimitReason, Status, Vm};

pub const DEFAULT_RUN_MAX_STEPS: usize = 1_000_000;
pub const DEFAULT_RUN_MAX_OUTPUT_BYTES: usize = 1024 * 1024;
pub const DEFAULT_UI_MAX_OUTPUT_BYTES: usize = 256 * 1024;

/// Limites explícitos para execução de programas `.fer` no host WASM.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunBudget {
    pub max_steps: usize,
    pub max_string_bytes: usize,
    pub max_output_bytes: usize,
}

impl Default for RunBudget {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_RUN_MAX_STEPS,
            max_string_bytes: DEFAULT_MAX_STRING_BYTES,
            max_output_bytes: DEFAULT_RUN_MAX_OUTPUT_BYTES,
        }
    }
}

/// Registro de sessões MVU ativas em um host persistente (WASM ou testes).
#[derive(Default)]
pub struct SessionStore {
    next_id: u32,
    sessions: BTreeMap<u32, UiSession<'static>>,
}

impl SessionStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Inicia uma sessão MVU a partir de um [`Module`] compilado e retorna a
    /// resposta JSON canônica com o identificador da sessão e a árvore inicial.
    pub fn start_module(&mut self, module: Module, budget: UiBudget) -> String {
        let interrupt = Arc::new(AtomicBool::new(false));
        let mut io = BoundedBufferIo::new("", DEFAULT_UI_MAX_OUTPUT_BYTES, Arc::clone(&interrupt));
        match UiSession::start_owned(module, &mut io, budget, interrupt) {
            Ok(session) => {
                self.next_id = self.next_id.wrapping_add(1).max(1);
                while self.sessions.contains_key(&self.next_id) {
                    self.next_id = self.next_id.wrapping_add(1).max(1);
                }
                let id = self.next_id;
                let step = session.step();
                let tree_json = session.tree().to_json();
                let history_len = session.history().len();
                self.sessions.insert(id, session);
                session_ok_json(id, step, history_len, &tree_json, &io.output)
            }
            Err(error) => ui_error_json(&error, &io.output, io.truncated),
        }
    }

    /// Compila um fonte `.fer` e inicia uma sessão MVU com o orçamento informado.
    pub fn start_ferro(&mut self, source: &str, budget: UiBudget) -> String {
        let program = match ferro_frontend::lower(source) {
            Ok(program) => program,
            Err(diagnostics) => return diagnostics_error_json(&diagnostics),
        };
        let module = match ferro_bytecode::compile_ferro(&program) {
            Ok(module) => module,
            Err(error) => {
                return simple_error_json(
                    &error.message,
                    Some("compile"),
                    Some(error.span.start),
                    Some(error.span.end),
                );
            }
        };
        self.start_module(module, budget)
    }

    /// Despacha um evento (`clique` ou `entrada_alterada`) para a sessão indicada.
    pub fn dispatch(
        &mut self,
        session_id: u32,
        kind: &str,
        target_id: &str,
        value: &str,
    ) -> String {
        let event = match parse_ui_event(kind, target_id, value) {
            Ok(event) => event,
            Err(message) => return simple_error_json(&message, Some("invalid_event"), None, None),
        };
        let Some(session) = self.sessions.get_mut(&session_id) else {
            return simple_error_json(
                &format!("Sessão visual {session_id} não encontrada."),
                Some("session_not_found"),
                None,
                None,
            );
        };
        let interrupt = session.interrupt_handle();
        interrupt.store(false, Ordering::Relaxed);
        let mut io = BoundedBufferIo::new("", DEFAULT_UI_MAX_OUTPUT_BYTES, interrupt);
        match session.dispatch(event, &mut io) {
            Ok(snapshot) => {
                let step = snapshot.step;
                let tree_json = snapshot.tree.to_json();
                let history_len = session.history().len();
                session_ok_json(session_id, step, history_len, &tree_json, &io.output)
            }
            Err(error) => ui_error_json(&error, &io.output, io.truncated),
        }
    }

    /// Retorna o snapshot JSON atual da sessão, sem disparar novos eventos.
    pub fn snapshot(&self, session_id: u32) -> String {
        let Some(session) = self.sessions.get(&session_id) else {
            return simple_error_json(
                &format!("Sessão visual {session_id} não encontrada."),
                Some("session_not_found"),
                None,
                None,
            );
        };
        session_ok_json(
            session_id,
            session.step(),
            session.history().len(),
            &session.tree().to_json(),
            "",
        )
    }

    /// Encerra e libera a sessão indicada.
    pub fn close(&mut self, session_id: u32) -> bool {
        self.sessions.remove(&session_id).is_some()
    }

    pub fn active_count(&self) -> usize {
        self.sessions.len()
    }
}

/// Valida um programa `.fer` e devolve os diagnósticos em JSON.
pub fn validate_ferro_json(source: &str) -> String {
    match ferro_frontend::lower(source) {
        Ok(_) => "{\"ok\":true,\"diagnostics\":[]}".into(),
        Err(diagnostics) => {
            let mut out = String::from("{\"ok\":false,\"diagnostics\":");
            write_diagnostics_json(&diagnostics, &mut out);
            out.push('}');
            out
        }
    }
}

/// Formata um programa `.fer` com o formatador canônico UTF-16 e devolve JSON.
pub fn format_ferro_json(source: &str) -> String {
    match ferro_frontend::format(source) {
        Some(edits) => {
            let changed = !edits.is_empty();
            match apply_utf16_edits(source, &edits) {
                Some(formatted) => {
                    let mut out = String::from("{\"ok\":true,\"changed\":");
                    out.push_str(if changed { "true" } else { "false" });
                    out.push_str(",\"formatted\":");
                    write_json_string(&formatted, &mut out);
                    out.push('}');
                    out
                }
                None => simple_error_json(
                    "Falha interna ao aplicar edições UTF-16 de formatação.",
                    Some("format_internal"),
                    None,
                    None,
                ),
            }
        }
        None => match ferro_frontend::lower(source) {
            Err(diagnostics) => diagnostics_error_json(&diagnostics),
            Ok(_) => simple_error_json(
                "Não foi possível formatar o código-fonte .fer.",
                Some("format_error"),
                None,
                None,
            ),
        },
    }
}

/// Gera código Rust equivalente a um programa `.fer` e devolve JSON canônico.
pub fn export_rust_ferro_json(source: &str) -> String {
    match ferro_rust::export_ferro_rs(source) {
        Ok(code) => {
            let mut out = String::from("{\"ok\":true,\"code\":");
            write_json_string(&code, &mut out);
            out.push_str(",\"diagnostics\":[]}");
            out
        }
        Err(diagnostics) => diagnostics_error_json(&diagnostics),
    }
}

/// Compila e executa um programa `.fer` na VM mínima com entrada e limites explícitos.
pub fn run_ferro_json(source: &str, input: &str, budget: RunBudget) -> String {
    let program = match ferro_frontend::lower(source) {
        Ok(program) => program,
        Err(diagnostics) => return diagnostics_error_json(&diagnostics),
    };
    let module = match ferro_bytecode::compile_ferro(&program) {
        Ok(module) => module,
        Err(error) => {
            return simple_error_json(
                &error.message,
                Some("compile"),
                Some(error.span.start),
                Some(error.span.end),
            );
        }
    };

    let interrupt = Arc::new(AtomicBool::new(false));
    let mut io = BoundedBufferIo::new(input, budget.max_output_bytes, Arc::clone(&interrupt));
    let mut vm = Vm::new(&module, &mut io)
        .with_interrupt(interrupt)
        .with_string_memory_limit(budget.max_string_bytes);

    match vm.step(budget.max_steps) {
        Ok(Status::Finished) => {
            let mut out = String::from("{\"ok\":true,\"stdout\":");
            write_json_string(&io.output, &mut out);
            out.push_str(",\"diagnostics\":[]}");
            out
        }
        Ok(Status::Running) => run_error_json(
            &format!(
                "Limite máximo de {} passos de execução excedido.",
                budget.max_steps
            ),
            Some("steps"),
            &io.output,
        ),
        Err(error) => {
            if io.truncated {
                run_error_json(
                    &format!(
                        "Limite máximo de {} bytes de saída excedido.",
                        budget.max_output_bytes
                    ),
                    Some("output"),
                    &io.output,
                )
            } else {
                let limit = error.limit.map(limit_reason_name);
                run_error_json(&error.message, limit, &io.output)
            }
        }
    }
}

struct BoundedBufferIo {
    input: VecDeque<String>,
    output: String,
    max_output_bytes: usize,
    truncated: bool,
    interrupt: Arc<AtomicBool>,
}

impl BoundedBufferIo {
    fn new(input: &str, max_output_bytes: usize, interrupt: Arc<AtomicBool>) -> Self {
        Self {
            input: input.lines().map(str::to_string).collect(),
            output: String::new(),
            max_output_bytes,
            truncated: false,
            interrupt,
        }
    }
}

impl Io for BoundedBufferIo {
    fn write(&mut self, text: &str) {
        if self.truncated {
            return;
        }
        if self.output.len().saturating_add(text.len()) > self.max_output_bytes {
            let remaining = self.max_output_bytes.saturating_sub(self.output.len());
            let mut cut = remaining.min(text.len());
            while cut > 0 && !text.is_char_boundary(cut) {
                cut -= 1;
            }
            self.output.push_str(&text[..cut]);
            self.truncated = true;
            self.interrupt.store(true, Ordering::Relaxed);
            return;
        }
        self.output.push_str(text);
    }

    fn read_line(&mut self, _expected: ValueType) -> Option<String> {
        self.input.pop_front()
    }

    fn clear(&mut self) {
        self.output.clear();
    }
}

fn parse_ui_event(kind: &str, target_id: &str, value: &str) -> Result<UiEvent, String> {
    match kind {
        "clique" => Ok(UiEvent::Click {
            id: target_id.into(),
        }),
        "entrada_alterada" => Ok(UiEvent::InputChanged {
            id: target_id.into(),
            value: value.into(),
        }),
        other => Err(format!(
            "Tipo de evento de UI desconhecido: \"{other}\". Use \"clique\" ou \"entrada_alterada\"."
        )),
    }
}

fn apply_utf16_edits(source: &str, edits: &[WhitespaceEdit]) -> Option<String> {
    let mut units: Vec<u16> = source.encode_utf16().collect();
    for edit in edits.iter().rev() {
        if edit.from > edit.to || edit.to > units.len() {
            return None;
        }
        units.splice(edit.from..edit.to, edit.insert.encode_utf16());
    }
    String::from_utf16(&units).ok()
}

fn limit_reason_name(reason: RuntimeLimitReason) -> &'static str {
    match reason {
        RuntimeLimitReason::Time => "time",
        RuntimeLimitReason::StringMemory => "string_memory",
    }
}

fn session_ok_json(
    session_id: u32,
    step: u64,
    history_len: usize,
    tree_json: &str,
    stdout: &str,
) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "{{\"ok\":true,\"session\":{session_id},\"step\":{step},\"history_len\":{history_len},\"tree\":{tree_json},\"stdout\":"
    );
    write_json_string(stdout, &mut out);
    out.push('}');
    out
}

fn ui_error_json(error: &UiError, stdout: &str, output_truncated: bool) -> String {
    let code = if output_truncated {
        "output"
    } else {
        match error {
            UiError::Runtime(runtime) => runtime.limit.map(limit_reason_name).unwrap_or("runtime"),
            UiError::NodeLimitExceeded { .. } => "node_limit",
            UiError::DepthLimitExceeded { .. } => "depth_limit",
            UiError::InvalidNodeHandle(_) => "invalid_node_handle",
            UiError::InvalidRootValue(_) => "invalid_root_value",
            UiError::MissingFunction(_) => "missing_function",
            UiError::UnsupportedArity { .. } => "unsupported_arity",
            UiError::Cancelled => "cancelled",
        }
    };
    let message = if output_truncated {
        format!(
            "Limite máximo de {DEFAULT_UI_MAX_OUTPUT_BYTES} bytes de saída excedido durante a sessão visual."
        )
    } else {
        error.to_string()
    };
    let mut out = String::from("{\"ok\":false,\"error\":");
    write_json_string(&message, &mut out);
    out.push_str(",\"code\":");
    write_json_string(code, &mut out);
    out.push_str(",\"stdout\":");
    write_json_string(stdout, &mut out);
    out.push('}');
    out
}

fn run_error_json(message: &str, limit: Option<&str>, stdout: &str) -> String {
    let mut out = String::from("{\"ok\":false,\"stdout\":");
    write_json_string(stdout, &mut out);
    out.push_str(",\"error\":");
    write_json_string(message, &mut out);
    out.push_str(",\"limit\":");
    match limit {
        Some(name) => write_json_string(name, &mut out),
        None => out.push_str("null"),
    }
    out.push_str(",\"diagnostics\":[]}");
    out
}

fn diagnostics_error_json(diagnostics: &[Diagnostic]) -> String {
    let first_message = diagnostics
        .first()
        .map(|d| d.message.as_str())
        .unwrap_or("Erro de sintaxe ou semântica no código .fer.");
    let mut out = String::from("{\"ok\":false,\"error\":");
    write_json_string(first_message, &mut out);
    out.push_str(",\"code\":\"diagnostics\",\"diagnostics\":");
    write_diagnostics_json(diagnostics, &mut out);
    out.push('}');
    out
}

fn simple_error_json(
    message: &str,
    code: Option<&str>,
    start: Option<u32>,
    end: Option<u32>,
) -> String {
    let mut out = String::from("{\"ok\":false,\"error\":");
    write_json_string(message, &mut out);
    if let Some(code) = code {
        out.push_str(",\"code\":");
        write_json_string(code, &mut out);
    }
    if let (Some(start), Some(end)) = (start, end) {
        let _ = write!(out, ",\"span\":{{\"start\":{start},\"end\":{end}}}");
    }
    out.push_str(",\"diagnostics\":[]}");
    out
}

fn write_diagnostics_json(diagnostics: &[Diagnostic], out: &mut String) {
    out.push('[');
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push_str("{\"code\":");
        write_json_string(diagnostic.code, out);
        out.push_str(",\"message\":");
        write_json_string(&diagnostic.message, out);
        let _ = write!(
            out,
            ",\"start\":{},\"end\":{}}}",
            diagnostic.span.start, diagnostic.span.end
        );
    }
    out.push(']');
}

fn write_json_string(value: &str, out: &mut String) {
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
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}

thread_local! {
    static RESULT_BUFFER: RefCell<Vec<u8>> = const { RefCell::new(Vec::new()) };
    static SESSIONS: RefCell<SessionStore> = RefCell::new(SessionStore::new());
}

fn store_result(json: String) -> usize {
    let bytes = json.into_bytes();
    let len = bytes.len();
    RESULT_BUFFER.with(|slot| {
        *slot.borrow_mut() = bytes;
    });
    len
}

unsafe fn read_utf8_arg<'a>(ptr: *const u8, len: usize) -> Result<&'a str, String> {
    if len == 0 {
        return Ok("");
    }
    if ptr.is_null() {
        return Err("Ponteiro nulo recebido pela interface WASM.".into());
    }
    // SAFETY: O chamador do host WASM garante um intervalo válido de `len` bytes
    // previamente alocado com `ferro_alloc`.
    let bytes = unsafe { slice::from_raw_parts(ptr, len) };
    std::str::from_utf8(bytes).map_err(|_| "Entrada UTF-8 inválida recebida pelo host WASM.".into())
}

/// Aloca `len` bytes na memória linear WASM para passagem de strings UTF-8 pelo host.
#[unsafe(no_mangle)]
pub extern "C" fn ferro_alloc(len: usize) -> *mut u8 {
    if len == 0 {
        return std::ptr::dangling_mut();
    }
    let Ok(layout) = Layout::from_size_align(len, 1) else {
        return std::ptr::null_mut();
    };
    // SAFETY: `len > 0` e alinhamento 1 é sempre válido.
    unsafe { alloc(layout) }
}

/// Libera um bloco previamente alocado por [`ferro_alloc`].
///
/// # Safety
/// `ptr` deve ter sido retornado por [`ferro_alloc`] com o mesmo `len` e ainda não
/// ter sido liberado.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_dealloc(ptr: *mut u8, len: usize) {
    if len == 0 || ptr.is_null() {
        return;
    }
    if let Ok(layout) = Layout::from_size_align(len, 1) {
        // SAFETY: Contrato de segurança garantido pelo chamador.
        unsafe { dealloc(ptr, layout) };
    }
}

/// Ponteiro para o último resultado JSON gravado em UTF-8.
#[unsafe(no_mangle)]
pub extern "C" fn ferro_result_ptr() -> *const u8 {
    RESULT_BUFFER.with(|slot| slot.borrow().as_ptr())
}

/// Tamanho em bytes do último resultado JSON gravado em UTF-8.
#[unsafe(no_mangle)]
pub extern "C" fn ferro_result_len() -> usize {
    RESULT_BUFFER.with(|slot| slot.borrow().len())
}

/// Valida um fonte `.fer` e grava o resultado JSON no buffer de resposta.
///
/// # Safety
/// `src_ptr` deve apontar para `src_len` bytes válidos na memória linear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_validate(src_ptr: *const u8, src_len: usize) -> usize {
    let json = match unsafe { read_utf8_arg(src_ptr, src_len) } {
        Ok(source) => validate_ferro_json(source),
        Err(message) => simple_error_json(&message, Some("invalid_utf8"), None, None),
    };
    store_result(json)
}

/// Formata um fonte `.fer` e grava o resultado JSON no buffer de resposta.
///
/// # Safety
/// `src_ptr` deve apontar para `src_len` bytes válidos na memória linear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_format(src_ptr: *const u8, src_len: usize) -> usize {
    let json = match unsafe { read_utf8_arg(src_ptr, src_len) } {
        Ok(source) => format_ferro_json(source),
        Err(message) => simple_error_json(&message, Some("invalid_utf8"), None, None),
    };
    store_result(json)
}

/// Exporta um fonte `.fer` para código Rust e grava o resultado JSON no buffer de resposta.
///
/// # Safety
/// `src_ptr` deve apontar para `src_len` bytes válidos na memória linear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_export_rust(src_ptr: *const u8, src_len: usize) -> usize {
    let json = match unsafe { read_utf8_arg(src_ptr, src_len) } {
        Ok(source) => export_rust_ferro_json(source),
        Err(message) => simple_error_json(&message, Some("invalid_utf8"), None, None),
    };
    store_result(json)
}

/// Compila e executa um fonte `.fer` na VM mínima com limites configuráveis
/// (`0` aplica o valor padrão de cada orçamento).
///
/// # Safety
/// `src_ptr`/`src_len` e `input_ptr`/`input_len` devem apontar para fatias válidas.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_run(
    src_ptr: *const u8,
    src_len: usize,
    input_ptr: *const u8,
    input_len: usize,
    max_steps: usize,
    max_string_bytes: usize,
    max_output_bytes: usize,
) -> usize {
    let json = match (unsafe { read_utf8_arg(src_ptr, src_len) }, unsafe {
        read_utf8_arg(input_ptr, input_len)
    }) {
        (Ok(source), Ok(input)) => {
            let defaults = RunBudget::default();
            let budget = RunBudget {
                max_steps: if max_steps == 0 {
                    defaults.max_steps
                } else {
                    max_steps
                },
                max_string_bytes: if max_string_bytes == 0 {
                    defaults.max_string_bytes
                } else {
                    max_string_bytes
                },
                max_output_bytes: if max_output_bytes == 0 {
                    defaults.max_output_bytes
                } else {
                    max_output_bytes
                },
            };
            run_ferro_json(source, input, budget)
        }
        (Err(message), _) | (_, Err(message)) => {
            simple_error_json(&message, Some("invalid_utf8"), None, None)
        }
    };
    store_result(json)
}

/// Inicia uma sessão MVU a partir de um fonte `.fer` (`0` aplica orçamento padrão).
///
/// # Safety
/// `src_ptr` deve apontar para `src_len` bytes válidos na memória linear.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_ui_start(
    src_ptr: *const u8,
    src_len: usize,
    max_steps: usize,
    max_nodes: usize,
    max_depth: usize,
) -> usize {
    let json = match unsafe { read_utf8_arg(src_ptr, src_len) } {
        Ok(source) => {
            let mut budget = UiBudget::default();
            if max_steps > 0 {
                budget.max_steps_per_event = max_steps;
            }
            if max_nodes > 0 {
                budget.max_nodes = max_nodes;
            }
            if max_depth > 0 {
                budget.max_depth = max_depth;
            }
            SESSIONS.with(|store| store.borrow_mut().start_ferro(source, budget))
        }
        Err(message) => simple_error_json(&message, Some("invalid_utf8"), None, None),
    };
    store_result(json)
}

/// Despacha um evento (`clique` ou `entrada_alterada`) para uma sessão MVU ativa.
///
/// # Safety
/// Os ponteiros e comprimentos fornecidos devem representar fatias válidas.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn ferro_ui_dispatch(
    session_id: u32,
    kind_ptr: *const u8,
    kind_len: usize,
    target_ptr: *const u8,
    target_len: usize,
    value_ptr: *const u8,
    value_len: usize,
) -> usize {
    let json = match (
        unsafe { read_utf8_arg(kind_ptr, kind_len) },
        unsafe { read_utf8_arg(target_ptr, target_len) },
        unsafe { read_utf8_arg(value_ptr, value_len) },
    ) {
        (Ok(kind), Ok(target_id), Ok(value)) => SESSIONS.with(|store| {
            store
                .borrow_mut()
                .dispatch(session_id, kind, target_id, value)
        }),
        (Err(message), _, _) | (_, Err(message), _) | (_, _, Err(message)) => {
            simple_error_json(&message, Some("invalid_utf8"), None, None)
        }
    };
    store_result(json)
}

/// Lê o snapshot atual de uma sessão MVU ativa.
#[unsafe(no_mangle)]
pub extern "C" fn ferro_ui_snapshot(session_id: u32) -> usize {
    let json = SESSIONS.with(|store| store.borrow().snapshot(session_id));
    store_result(json)
}

/// Encerra uma sessão MVU ativa. Retorna `1` se existia, `0` caso contrário.
#[unsafe(no_mangle)]
pub extern "C" fn ferro_ui_close(session_id: u32) -> u32 {
    let closed = SESSIONS.with(|store| store.borrow_mut().close(session_id));
    u32::from(closed)
}
