//! Serviço thread-safe de sessões MVU declarativas para a IDE desktop (ADR 0017).
//!
//! Mantém o armazenamento das instâncias [`UiSession`] (não-`Send` por usar `Rc` na VM)
//! em uma thread dedicada e expõe o mesmo contrato JSON canônico da fachada `ferro-wasm`,
//! permitindo compartilhar o mesmo renderizador DOM seguro entre Desktop (Tauri) e Web (WASM).

use std::collections::{BTreeMap, HashMap, VecDeque};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;

use ferro_bytecode::ValueType;
use ferro_hir::Diagnostic;
use ferro_vm::ui::{UiBudget, UiError, UiEvent, UiSession};
use ferro_vm::{Io, RuntimeLimitReason};
use serde_json::{Value as JsonValue, json};

const DEFAULT_UI_MAX_OUTPUT_BYTES: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, Default)]
pub struct PorsUiBudgetRequest {
    pub max_steps: Option<usize>,
    pub max_nodes: Option<usize>,
    pub max_depth: Option<usize>,
}

impl PorsUiBudgetRequest {
    fn resolve(self) -> UiBudget {
        let mut budget = UiBudget::default();
        if let Some(steps) = self.max_steps.filter(|&v| v > 0) {
            budget.max_steps_per_event = steps;
        }
        if let Some(nodes) = self.max_nodes.filter(|&v| v > 0) {
            budget.max_nodes = nodes;
        }
        if let Some(depth) = self.max_depth.filter(|&v| v > 0) {
            budget.max_depth = depth;
        }
        budget
    }
}

enum UiRequest {
    Start {
        source: String,
        budget: UiBudget,
        reply: mpsc::Sender<String>,
    },
    Dispatch {
        session_id: u32,
        kind: String,
        target_id: String,
        value: String,
        reply: mpsc::Sender<String>,
    },
    Close {
        session_id: u32,
        reply: mpsc::Sender<bool>,
    },
}

/// Gerenciador `Send + Sync` de sessões MVU PORS para hosts desktop (Tauri).
pub struct PorsUiService {
    sender: Mutex<mpsc::Sender<UiRequest>>,
    interrupts: Arc<Mutex<HashMap<u32, Arc<AtomicBool>>>>,
}

impl Default for PorsUiService {
    fn default() -> Self {
        Self::new()
    }
}

impl PorsUiService {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel::<UiRequest>();
        let interrupts = Arc::new(Mutex::new(HashMap::<u32, Arc<AtomicBool>>::new()));
        let worker_interrupts = Arc::clone(&interrupts);
        let _ = thread::Builder::new()
            .name("pors-ui-worker".into())
            .spawn(move || run_worker(rx, worker_interrupts));
        Self {
            sender: Mutex::new(tx),
            interrupts,
        }
    }

    pub fn start(&self, source: String, budget: PorsUiBudgetRequest) -> String {
        let (reply_tx, reply_rx) = mpsc::channel();
        let sent = self
            .sender
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .send(UiRequest::Start {
                source,
                budget: budget.resolve(),
                reply: reply_tx,
            });
        if sent.is_err() {
            return simple_error_json("Serviço de preview visual indisponível.", "worker_closed");
        }
        reply_rx.recv().unwrap_or_else(|_| {
            simple_error_json("Serviço de preview visual encerrado.", "worker_closed")
        })
    }

    pub fn dispatch(
        &self,
        session_id: u32,
        kind: String,
        target_id: String,
        value: String,
    ) -> String {
        let (reply_tx, reply_rx) = mpsc::channel();
        let sent =
            self.sender
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .send(UiRequest::Dispatch {
                    session_id,
                    kind,
                    target_id,
                    value,
                    reply: reply_tx,
                });
        if sent.is_err() {
            return simple_error_json("Serviço de preview visual indisponível.", "worker_closed");
        }
        reply_rx.recv().unwrap_or_else(|_| {
            simple_error_json("Serviço de preview visual encerrado.", "worker_closed")
        })
    }

    /// Sinaliza interrupção imediata da VM (mesmo durante um evento em execução)
    /// e enfileira o fechamento da sessão.
    pub fn cancel(&self, session_id: u32) -> bool {
        let flagged = {
            let map = self.interrupts.lock().unwrap_or_else(|e| e.into_inner());
            if let Some(flag) = map.get(&session_id) {
                flag.store(true, Ordering::Release);
                true
            } else {
                false
            }
        };
        let closed = self.close(session_id);
        flagged || closed
    }

    pub fn close(&self, session_id: u32) -> bool {
        let (reply_tx, reply_rx) = mpsc::channel();
        let sent = self
            .sender
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .send(UiRequest::Close {
                session_id,
                reply: reply_tx,
            });
        if sent.is_err() {
            return false;
        }
        reply_rx.recv().unwrap_or(false)
    }
}

fn run_worker(
    rx: mpsc::Receiver<UiRequest>,
    interrupts: Arc<Mutex<HashMap<u32, Arc<AtomicBool>>>>,
) {
    let mut sessions = BTreeMap::<u32, UiSession<'static>>::new();
    let mut next_id = 0u32;

    while let Ok(request) = rx.recv() {
        match request {
            UiRequest::Start {
                source,
                budget,
                reply,
            } => {
                let response =
                    handle_start(&source, budget, &mut next_id, &mut sessions, &interrupts);
                let _ = reply.send(response);
            }
            UiRequest::Dispatch {
                session_id,
                kind,
                target_id,
                value,
                reply,
            } => {
                let response =
                    handle_dispatch(session_id, &kind, &target_id, &value, &mut sessions);
                let _ = reply.send(response);
            }
            UiRequest::Close { session_id, reply } => {
                let existed = sessions.remove(&session_id).is_some();
                interrupts
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .remove(&session_id);
                let _ = reply.send(existed);
            }
        }
    }
}

fn handle_start(
    source: &str,
    budget: UiBudget,
    next_id: &mut u32,
    sessions: &mut BTreeMap<u32, UiSession<'static>>,
    interrupts: &Arc<Mutex<HashMap<u32, Arc<AtomicBool>>>>,
) -> String {
    let program = match ferro_frontend::lower(source) {
        Ok(program) => program,
        Err(diagnostics) => return diagnostics_error_json(&diagnostics),
    };
    let module = match ferro_bytecode::compile_pors(&program) {
        Ok(module) => module,
        Err(error) => {
            return json!({
                "ok": false,
                "error": error.message,
                "code": "compile",
                "span": { "start": error.span.start, "end": error.span.end },
                "diagnostics": []
            })
            .to_string();
        }
    };

    let interrupt = Arc::new(AtomicBool::new(false));
    let mut io = BoundedBufferIo::new("", DEFAULT_UI_MAX_OUTPUT_BYTES, Arc::clone(&interrupt));
    match UiSession::start_owned(module, &mut io, budget, Arc::clone(&interrupt)) {
        Ok(session) => {
            *next_id = next_id.wrapping_add(1).max(1);
            while sessions.contains_key(next_id) {
                *next_id = next_id.wrapping_add(1).max(1);
            }
            let id = *next_id;
            let step = session.step();
            let history_len = session.history().len();
            let tree_json = session.tree().to_json();
            sessions.insert(id, session);
            interrupts
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(id, interrupt);
            session_ok_json(id, step, history_len, &tree_json, &io.output)
        }
        Err(error) => ui_error_json(&error, &io.output, io.truncated),
    }
}

fn handle_dispatch(
    session_id: u32,
    kind: &str,
    target_id: &str,
    value: &str,
    sessions: &mut BTreeMap<u32, UiSession<'static>>,
) -> String {
    let event = match kind {
        "clique" => UiEvent::Click {
            id: target_id.to_owned(),
        },
        "entrada_alterada" => UiEvent::InputChanged {
            id: target_id.to_owned(),
            value: value.to_owned(),
        },
        other => {
            return simple_error_json(
                &format!(
                    "Tipo de evento de UI desconhecido: \"{other}\". Use \"clique\" ou \"entrada_alterada\"."
                ),
                "invalid_event",
            );
        }
    };
    let Some(session) = sessions.get_mut(&session_id) else {
        return simple_error_json(
            &format!("Sessão visual {session_id} não encontrada."),
            "session_not_found",
        );
    };
    let interrupt = session.interrupt_handle();
    if interrupt.load(Ordering::Acquire) {
        return ui_error_json(&UiError::Cancelled, "", false);
    }
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
            self.interrupt.store(true, Ordering::Release);
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

fn session_ok_json(
    session_id: u32,
    step: u64,
    history_len: usize,
    tree_json: &str,
    stdout: &str,
) -> String {
    let tree: JsonValue = serde_json::from_str(tree_json).unwrap_or(JsonValue::Null);
    json!({
        "ok": true,
        "session": session_id,
        "step": step,
        "history_len": history_len,
        "tree": tree,
        "stdout": stdout
    })
    .to_string()
}

fn ui_error_json(error: &UiError, stdout: &str, output_truncated: bool) -> String {
    let code = if output_truncated {
        "output"
    } else {
        match error {
            UiError::Runtime(runtime) => match runtime.limit {
                Some(RuntimeLimitReason::Time) => "time",
                Some(RuntimeLimitReason::StringMemory) => "string_memory",
                None => "runtime",
            },
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
    json!({
        "ok": false,
        "error": message,
        "code": code,
        "stdout": stdout
    })
    .to_string()
}

fn diagnostics_error_json(diagnostics: &[Diagnostic]) -> String {
    let first = diagnostics
        .first()
        .map(|d| d.message.as_str())
        .unwrap_or("Erro de sintaxe ou semântica no código .fer.");
    let list: Vec<JsonValue> = diagnostics
        .iter()
        .map(|d| {
            json!({
                "code": d.code,
                "message": d.message,
                "start": d.span.start,
                "end": d.span.end
            })
        })
        .collect();
    json!({
        "ok": false,
        "error": first,
        "code": "diagnostics",
        "diagnostics": list
    })
    .to_string()
}

fn simple_error_json(message: &str, code: &str) -> String {
    json!({
        "ok": false,
        "error": message,
        "code": code,
        "diagnostics": []
    })
    .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    const COUNTER_EXAMPLE: &str = r#"enum Humor {
    Neutro,
    Animado
}

registro Estado {
    contagem: inteiro,
    nome: cadeia,
    humor: Humor
}

funcao iniciar() -> Estado {
    retorne Estado { contagem: 0, nome: "Visitante", humor: Humor::Neutro };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "clique" && alvo == "inc") {
        retorne Estado { contagem: estado.contagem + 1, nome: estado.nome, humor: Humor::Animado };
    }
    se (tipo == "clique" && alvo == "dec") {
        retorne Estado { contagem: estado.contagem - 1, nome: estado.nome, humor: Humor::Neutro };
    }
    se (tipo == "clique" && alvo == "zerar") {
        retorne Estado { contagem: 0, nome: estado.nome, humor: Humor::Neutro };
    }
    se (tipo == "entrada_alterada" && alvo == "nome") {
        retorne Estado { contagem: estado.contagem, nome: valor, humor: estado.humor };
    }
    retorne estado;
}

funcao visualizar(estado: Estado) -> inteiro {
    seja mutavel selo = "Neutro";
    escolha estado.humor {
        caso Humor::Neutro => { selo = "Neutro"; }
        caso Humor::Animado => { selo = "Animado"; }
    }
    retorne UI.coluna([
        UI.texto("Olá, ", estado.nome, "!"),
        UI.texto("Contagem: ", estado.contagem, " (", selo, ")"),
        UI.linha([
            UI.botao("dec", "-1"),
            UI.botao("inc", "+1"),
            UI.botao("zerar", "Zerar")
        ]),
        UI.entrada("nome", estado.nome, "Digite seu nome")
    ]);
}
"#;

    const TASKS_EXAMPLE: &str = r#"registro Estado {
    rascunho: cadeia,
    tarefa1: cadeia,
    concluida1: logico,
    tarefa2: cadeia,
    concluida2: logico
}

funcao iniciar() -> Estado {
    retorne Estado {
        rascunho: "",
        tarefa1: "Estudar PORS",
        concluida1: falso,
        tarefa2: "Testar UI declarativa",
        concluida2: verdadeiro
    };
}

funcao atualizar(estado: Estado, tipo: cadeia, alvo: cadeia, valor: cadeia) -> Estado {
    se (tipo == "entrada_alterada" && alvo == "rascunho") {
        retorne Estado {
            rascunho: valor,
            tarefa1: estado.tarefa1,
            concluida1: estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: estado.concluida2
        };
    }
    se (tipo == "clique" && alvo == "salvar" && estado.rascunho != "") {
        retorne Estado {
            rascunho: "",
            tarefa1: estado.rascunho,
            concluida1: falso,
            tarefa2: estado.tarefa1,
            concluida2: estado.concluida1
        };
    }
    se (tipo == "clique" && alvo == "toggle1") {
        retorne Estado {
            rascunho: estado.rascunho,
            tarefa1: estado.tarefa1,
            concluida1: !estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: estado.concluida2
        };
    }
    se (tipo == "clique" && alvo == "toggle2") {
        retorne Estado {
            rascunho: estado.rascunho,
            tarefa1: estado.tarefa1,
            concluida1: estado.concluida1,
            tarefa2: estado.tarefa2,
            concluida2: !estado.concluida2
        };
    }
    retorne estado;
}

funcao rotulo_tarefa(texto: cadeia, feita: logico) -> cadeia {
    se (feita) {
        retorne "[x] {texto}";
    } senao {
        retorne "[ ] {texto}";
    }
}

funcao visualizar(estado: Estado) -> inteiro {
    retorne UI.coluna([
        UI.texto("Lista de tarefas PORS"),
        UI.linha([
            UI.entrada("rascunho", estado.rascunho, "Nova tarefa..."),
            UI.botao("salvar", "Adicionar")
        ]),
        UI.lista([
            UI.linha([
                UI.texto(rotulo_tarefa(estado.tarefa1, estado.concluida1)),
                UI.botao("toggle1", "Alternar")
            ]),
            UI.linha([
                UI.texto(rotulo_tarefa(estado.tarefa2, estado.concluida2)),
                UI.botao("toggle2", "Alternar")
            ])
        ])
    ]);
}
"#;

    #[test]
    fn runs_counter_and_tasks_visual_examples_through_worker_service() {
        let service = PorsUiService::new();

        let started: JsonValue = serde_json::from_str(
            &service.start(COUNTER_EXAMPLE.into(), PorsUiBudgetRequest::default()),
        )
        .unwrap();
        assert_eq!(started["ok"], true);
        let id = started["session"].as_u64().unwrap() as u32;
        assert_eq!(
            started["tree"]["children"][1]["content"],
            "Contagem: 0 (Neutro)"
        );

        let clicked: JsonValue =
            serde_json::from_str(&service.dispatch(id, "clique".into(), "inc".into(), "".into()))
                .unwrap();
        assert_eq!(clicked["ok"], true);
        assert_eq!(clicked["step"], 1);
        assert_eq!(
            clicked["tree"]["children"][1]["content"],
            "Contagem: 1 (Animado)"
        );
        assert!(service.close(id));

        let tasks_started: JsonValue = serde_json::from_str(
            &service.start(TASKS_EXAMPLE.into(), PorsUiBudgetRequest::default()),
        )
        .unwrap();
        assert_eq!(tasks_started["ok"], true);
        let tasks_id = tasks_started["session"].as_u64().unwrap() as u32;

        let typed: JsonValue = serde_json::from_str(&service.dispatch(
            tasks_id,
            "entrada_alterada".into(),
            "rascunho".into(),
            "Criar app visual".into(),
        ))
        .unwrap();
        assert_eq!(typed["ok"], true);

        let added: JsonValue = serde_json::from_str(&service.dispatch(
            tasks_id,
            "clique".into(),
            "salvar".into(),
            "".into(),
        ))
        .unwrap();
        assert_eq!(added["ok"], true);
        assert_eq!(
            added["tree"]["children"][2]["items"][0]["children"][0]["content"],
            "[ ] Criar app visual"
        );
        assert!(service.cancel(tasks_id));
    }

    #[test]
    fn enforces_budget_and_reports_diagnostics_through_worker_service() {
        let service = PorsUiService::new();

        let invalid: JsonValue = serde_json::from_str(&service.start(
            "funcao inicio() { escreva(desconhecido); }".into(),
            PorsUiBudgetRequest::default(),
        ))
        .unwrap();
        assert_eq!(invalid["ok"], false);
        assert_eq!(invalid["code"], "diagnostics");

        let loop_source = r#"
funcao iniciar() -> inteiro { retorne 0; }
funcao atualizar(estado: inteiro, alvo: cadeia) -> inteiro {
    enquanto (verdadeiro) {}
    retorne estado;
}
funcao visualizar(estado: inteiro) -> inteiro { retorne UI.texto(estado); }
"#;
        let started: JsonValue = serde_json::from_str(&service.start(
            loop_source.into(),
            PorsUiBudgetRequest {
                max_steps: Some(500),
                ..PorsUiBudgetRequest::default()
            },
        ))
        .unwrap();
        assert_eq!(started["ok"], true);
        let id = started["session"].as_u64().unwrap() as u32;

        let timed_out: JsonValue =
            serde_json::from_str(&service.dispatch(id, "clique".into(), "go".into(), "".into()))
                .unwrap();
        assert_eq!(timed_out["ok"], false);
        assert_eq!(timed_out["code"], "time");
        assert!(service.close(id));
    }
}
