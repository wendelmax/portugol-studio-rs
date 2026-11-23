//! Contrato declarativo de UI e ciclo de vida orientado a eventos (MVU) para
//! programas visuais em PORS, portável entre Desktop (Tauri WebView) e Web (WASM).
//!
//! Implementa a base de runtime do ADR 0017 sem dependências de sistema
//! operacional ou rede, permitindo compilar e executar sob `--no-default-features`
//! e `wasm32-unknown-unknown`.

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use portugol_bytecode::Module;

use crate::io::Io;
use crate::value::Value;
use crate::vm::{RuntimeError, Vm};

pub const DEFAULT_MAX_STEPS_PER_EVENT: usize = 200_000;
pub const DEFAULT_MAX_STRING_BYTES: usize = 16 * 1024 * 1024;
pub const DEFAULT_MAX_NODES: usize = 1_024;
pub const DEFAULT_MAX_DEPTH: usize = 32;
pub const DEFAULT_MAX_HISTORY: usize = 64;

/// Limites por evento/renderização para manter a interface responsiva e segura.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiBudget {
    pub max_steps_per_event: usize,
    pub max_string_bytes: usize,
    pub max_nodes: usize,
    pub max_depth: usize,
    pub max_history: usize,
}

impl Default for UiBudget {
    fn default() -> Self {
        Self {
            max_steps_per_event: DEFAULT_MAX_STEPS_PER_EVENT,
            max_string_bytes: DEFAULT_MAX_STRING_BYTES,
            max_nodes: DEFAULT_MAX_NODES,
            max_depth: DEFAULT_MAX_DEPTH,
            max_history: DEFAULT_MAX_HISTORY,
        }
    }
}

/// Árvore declarativa inicial de componentes do ADR 0017.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiNode {
    Column {
        children: Vec<UiNode>,
    },
    Row {
        children: Vec<UiNode>,
    },
    Text {
        content: String,
    },
    Button {
        id: String,
        label: String,
    },
    Input {
        id: String,
        value: String,
        placeholder: String,
    },
    List {
        items: Vec<UiNode>,
    },
}

impl UiNode {
    /// Quantidade total de nós na subárvore.
    pub fn node_count(&self) -> usize {
        1 + self.children().iter().map(Self::node_count).sum::<usize>()
    }

    /// Profundidade máxima da subárvore (folha = 1).
    pub fn depth(&self) -> usize {
        1 + self.children().iter().map(Self::depth).max().unwrap_or(0)
    }

    fn children(&self) -> &[UiNode] {
        match self {
            Self::Column { children } | Self::Row { children } => children,
            Self::List { items } => items,
            Self::Text { .. } | Self::Button { .. } | Self::Input { .. } => &[],
        }
    }

    /// Valida limites de nós e profundidade sem percorrer além do teto.
    pub fn validate(&self, budget: &UiBudget) -> Result<usize, UiError> {
        fn walk(
            node: &UiNode,
            depth: usize,
            budget: &UiBudget,
            count: &mut usize,
        ) -> Result<(), UiError> {
            if depth > budget.max_depth {
                return Err(UiError::DepthLimitExceeded {
                    limit: budget.max_depth,
                });
            }
            *count += 1;
            if *count > budget.max_nodes {
                return Err(UiError::NodeLimitExceeded {
                    limit: budget.max_nodes,
                });
            }
            for child in node.children() {
                walk(child, depth + 1, budget, count)?;
            }
            Ok(())
        }

        let mut total = 0usize;
        walk(self, 1, budget, &mut total)?;
        Ok(total)
    }

    /// Serialização JSON canônica e determinística (RFC 8259) sem dependências externas.
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.write_json(&mut out);
        out
    }

    fn write_json(&self, out: &mut String) {
        match self {
            Self::Column { children } => {
                out.push_str("{\"kind\":\"coluna\",\"children\":");
                write_nodes_json(children, out);
                out.push('}');
            }
            Self::Row { children } => {
                out.push_str("{\"kind\":\"linha\",\"children\":");
                write_nodes_json(children, out);
                out.push('}');
            }
            Self::Text { content } => {
                out.push_str("{\"kind\":\"texto\",\"content\":");
                write_json_string(content, out);
                out.push('}');
            }
            Self::Button { id, label } => {
                out.push_str("{\"kind\":\"botao\",\"id\":");
                write_json_string(id, out);
                out.push_str(",\"label\":");
                write_json_string(label, out);
                out.push('}');
            }
            Self::Input {
                id,
                value,
                placeholder,
            } => {
                out.push_str("{\"kind\":\"entrada\",\"id\":");
                write_json_string(id, out);
                out.push_str(",\"value\":");
                write_json_string(value, out);
                out.push_str(",\"placeholder\":");
                write_json_string(placeholder, out);
                out.push('}');
            }
            Self::List { items } => {
                out.push_str("{\"kind\":\"lista\",\"items\":");
                write_nodes_json(items, out);
                out.push('}');
            }
        }
    }
}

/// Evento enviado pelo host (Desktop WebView ou Web DOM) para a sessão MVU.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiEvent {
    Click { id: String },
    InputChanged { id: String, value: String },
}

impl UiEvent {
    pub fn kind_name(&self) -> &'static str {
        match self {
            Self::Click { .. } => "clique",
            Self::InputChanged { .. } => "entrada_alterada",
        }
    }

    pub fn target_id(&self) -> &str {
        match self {
            Self::Click { id } | Self::InputChanged { id, .. } => id,
        }
    }

    pub fn payload_value(&self) -> &str {
        match self {
            Self::Click { .. } => "",
            Self::InputChanged { value, .. } => value,
        }
    }

    pub fn to_json(&self) -> String {
        let mut out = String::new();
        match self {
            Self::Click { id } => {
                out.push_str("{\"kind\":\"clique\",\"id\":");
                write_json_string(id, &mut out);
                out.push('}');
            }
            Self::InputChanged { id, value } => {
                out.push_str("{\"kind\":\"entrada_alterada\",\"id\":");
                write_json_string(id, &mut out);
                out.push_str(",\"value\":");
                write_json_string(value, &mut out);
                out.push('}');
            }
        }
        out
    }
}

/// Fotografia imutável de cada transição MVU (base para histórico e depuração).
#[derive(Debug, Clone)]
pub struct UiSnapshot {
    pub step: u64,
    pub event: Option<UiEvent>,
    pub state: Value,
    pub tree: UiNode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum UiError {
    Runtime(RuntimeError),
    NodeLimitExceeded { limit: usize },
    DepthLimitExceeded { limit: usize },
    InvalidNodeHandle(i32),
    InvalidRootValue(String),
    MissingFunction(&'static str),
    UnsupportedArity { function: &'static str, arity: u32 },
    Cancelled,
}

impl From<RuntimeError> for UiError {
    fn from(error: RuntimeError) -> Self {
        Self::Runtime(error)
    }
}

impl std::fmt::Display for UiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Runtime(error) => write!(f, "{}", error.message),
            Self::NodeLimitExceeded { limit } => {
                write!(
                    f,
                    "O limite máximo de {limit} nós de interface foi excedido."
                )
            }
            Self::DepthLimitExceeded { limit } => {
                write!(
                    f,
                    "A profundidade máxima de {limit} níveis da interface foi excedida."
                )
            }
            Self::InvalidNodeHandle(handle) => {
                write!(f, "Identificador de nó de UI inválido: {handle}.")
            }
            Self::InvalidRootValue(ty) => {
                write!(
                    f,
                    "\"visualizar\" deve retornar um identificador inteiro de nó, mas retornou {ty}."
                )
            }
            Self::MissingFunction(name) => {
                write!(f, "O programa visual precisa definir a função \"{name}\".")
            }
            Self::UnsupportedArity { function, arity } => {
                write!(
                    f,
                    "A função \"{function}\" possui quantidade de parâmetros não suportada ({arity})."
                )
            }
            Self::Cancelled => write!(f, "Sessão visual cancelada."),
        }
    }
}

/// Construtor de árvore de visão acionado por chamadas nativas `UI.*` durante
/// `visualizar(estado)`.
pub struct UiTreeBuilder {
    nodes: Vec<UiNode>,
    budget: UiBudget,
}

impl UiTreeBuilder {
    pub fn new(budget: UiBudget) -> Self {
        Self {
            nodes: Vec::new(),
            budget,
        }
    }

    /// Resolve chamadas nativas da biblioteca `"UI"`. Devolve `None` para outras bibliotecas.
    pub fn handle_native(
        &mut self,
        library: &str,
        name: &str,
        args: &[Value],
    ) -> Option<Result<Value, String>> {
        if library != "UI" {
            return None;
        }
        Some(self.dispatch(name, args))
    }

    fn dispatch(&mut self, name: &str, args: &[Value]) -> Result<Value, String> {
        let node = match name {
            "texto" => {
                if args.is_empty() {
                    return Err("\"UI.texto\" espera pelo menos 1 argumento.".into());
                }
                let mut content = String::new();
                for arg in args {
                    let piece = match arg {
                        Value::Str(text) => text.as_ref().to_owned(),
                        Value::Int(_) | Value::Real(_) | Value::Bool(_) | Value::Char(_) => {
                            arg.display()
                        }
                        other => {
                            return Err(format!(
                                "\"UI.texto\" espera valores escalares (cadeia, inteiro, real, logico ou caracter), mas recebeu {}.",
                                other.type_name()
                            ));
                        }
                    };
                    if content.len().saturating_add(piece.len()) > self.budget.max_string_bytes {
                        return Err(format!(
                            "O limite máximo de {} bytes de texto em \"UI.texto\" foi excedido.",
                            self.budget.max_string_bytes
                        ));
                    }
                    content.push_str(&piece);
                }
                UiNode::Text { content }
            }
            "botao" => {
                check_arity(args, 2, "UI.botao")?;
                UiNode::Button {
                    id: expect_string(&args[0], "UI.botao")?,
                    label: expect_string(&args[1], "UI.botao")?,
                }
            }
            "entrada" => {
                check_arity(args, 3, "UI.entrada")?;
                UiNode::Input {
                    id: expect_string(&args[0], "UI.entrada")?,
                    value: expect_string(&args[1], "UI.entrada")?,
                    placeholder: expect_string(&args[2], "UI.entrada")?,
                }
            }
            "coluna" => {
                check_arity(args, 1, "UI.coluna")?;
                UiNode::Column {
                    children: self.resolve_children(&args[0], "UI.coluna")?,
                }
            }
            "linha" => {
                check_arity(args, 1, "UI.linha")?;
                UiNode::Row {
                    children: self.resolve_children(&args[0], "UI.linha")?,
                }
            }
            "lista" => {
                check_arity(args, 1, "UI.lista")?;
                UiNode::List {
                    items: self.resolve_children(&args[0], "UI.lista")?,
                }
            }
            other => {
                return Err(format!(
                    "O componente \"UI.{other}\" não faz parte do contrato visual atual."
                ));
            }
        };

        if self.nodes.len() >= self.budget.max_nodes {
            return Err(format!(
                "O limite máximo de {} nós de interface foi excedido.",
                self.budget.max_nodes
            ));
        }
        let handle = self.nodes.len() as i32;
        self.nodes.push(node);
        Ok(Value::Int(handle))
    }

    fn resolve_children(&self, value: &Value, function: &str) -> Result<Vec<UiNode>, String> {
        let Value::Array(handles) = value else {
            return Err(format!(
                "\"{function}\" espera um vetor de identificadores de nós."
            ));
        };
        let handles = handles.borrow();
        let mut children = Vec::with_capacity(handles.len());
        for entry in handles.iter() {
            let Value::Int(handle) = entry else {
                return Err(format!(
                    "\"{function}\" espera identificadores inteiros de nós."
                ));
            };
            let node = self
                .node(*handle)
                .ok_or_else(|| format!("Identificador de nó de UI inválido: {handle}."))?;
            children.push(node.clone());
        }
        Ok(children)
    }

    pub fn node(&self, handle: i32) -> Option<&UiNode> {
        usize::try_from(handle)
            .ok()
            .and_then(|index| self.nodes.get(index))
    }

    /// Materializa e valida o nó raiz retornado por `visualizar(estado)`.
    pub fn finish(&self, root_value: &Value) -> Result<UiNode, UiError> {
        let Value::Int(handle) = root_value else {
            return Err(UiError::InvalidRootValue(root_value.type_name().into()));
        };
        let root = self
            .node(*handle)
            .ok_or(UiError::InvalidNodeHandle(*handle))?
            .clone();
        root.validate(&self.budget)?;
        Ok(root)
    }
}

enum ModuleStorage<'a> {
    Borrowed(&'a Module),
    Owned(Module),
}

impl ModuleStorage<'_> {
    fn as_ref(&self) -> &Module {
        match self {
            Self::Borrowed(module) => module,
            Self::Owned(module) => module,
        }
    }
}

/// Sessão MVU orientada a eventos para um módulo compilado (`iniciar`,
/// `atualizar` e `visualizar`), com orçamento isolado por evento e cancelamento.
pub struct UiSession<'a> {
    module: ModuleStorage<'a>,
    budget: UiBudget,
    interrupt: Arc<AtomicBool>,
    state: Value,
    tree: UiNode,
    step: u64,
    history: Vec<UiSnapshot>,
}

impl UiSession<'static> {
    /// Inicia uma sessão MVU assumindo posse do módulo compilado (útil para
    /// hosts persistentes como WASM e Tauri).
    pub fn start_owned(
        module: Module,
        io: &mut dyn Io,
        budget: UiBudget,
        interrupt: Arc<AtomicBool>,
    ) -> Result<Self, UiError> {
        Self::start_with_storage(ModuleStorage::Owned(module), io, budget, interrupt)
    }
}

impl<'a> UiSession<'a> {
    /// Inicia a sessão executando `iniciar()` e a renderização inicial `visualizar(estado)`.
    pub fn start(
        module: &'a Module,
        io: &mut dyn Io,
        budget: UiBudget,
        interrupt: Arc<AtomicBool>,
    ) -> Result<Self, UiError> {
        Self::start_with_storage(ModuleStorage::Borrowed(module), io, budget, interrupt)
    }

    fn start_with_storage(
        module: ModuleStorage<'a>,
        io: &mut dyn Io,
        budget: UiBudget,
        interrupt: Arc<AtomicBool>,
    ) -> Result<Self, UiError> {
        if interrupt.load(Ordering::Relaxed) {
            return Err(UiError::Cancelled);
        }
        let module_ref = module.as_ref();
        let init = module_ref
            .function("iniciar")
            .ok_or(UiError::MissingFunction("iniciar"))?;
        if init.arity != 0 {
            return Err(UiError::UnsupportedArity {
                function: "iniciar",
                arity: init.arity,
            });
        }
        let update = module_ref
            .function("atualizar")
            .ok_or(UiError::MissingFunction("atualizar"))?;
        if !(2..=4).contains(&update.arity) {
            return Err(UiError::UnsupportedArity {
                function: "atualizar",
                arity: update.arity,
            });
        }
        let view = module_ref
            .function("visualizar")
            .ok_or(UiError::MissingFunction("visualizar"))?;
        if view.arity != 1 {
            return Err(UiError::UnsupportedArity {
                function: "visualizar",
                arity: view.arity,
            });
        }

        let state = {
            let mut vm = Vm::new(module_ref, io)
                .with_interrupt(Arc::clone(&interrupt))
                .with_string_memory_limit(budget.max_string_bytes);
            vm.call_function("iniciar", Vec::new(), budget.max_steps_per_event)?
        };
        let tree = render_view(module_ref, io, &state, budget, &interrupt)?;

        let snapshot = UiSnapshot {
            step: 0,
            event: None,
            state: state.clone(),
            tree: tree.clone(),
        };
        let mut history = Vec::new();
        if budget.max_history > 0 {
            history.push(snapshot);
        }

        Ok(Self {
            module,
            budget,
            interrupt,
            state,
            tree,
            step: 0,
            history,
        })
    }

    /// Despacha um evento do host, executando `atualizar` e `visualizar` com
    /// orçamentos novos por evento (sem teto fixo de tempo ocioso da janela).
    pub fn dispatch(&mut self, event: UiEvent, io: &mut dyn Io) -> Result<&UiSnapshot, UiError> {
        if self.interrupt.load(Ordering::Relaxed) {
            return Err(UiError::Cancelled);
        }
        let module_ref = self.module.as_ref();
        let update = module_ref
            .function("atualizar")
            .ok_or(UiError::MissingFunction("atualizar"))?;
        let args = match update.arity {
            2 => vec![self.state.clone(), Value::str(event.target_id())],
            3 => vec![
                self.state.clone(),
                Value::str(event.kind_name()),
                Value::str(event.target_id()),
            ],
            4 => vec![
                self.state.clone(),
                Value::str(event.kind_name()),
                Value::str(event.target_id()),
                Value::str(event.payload_value()),
            ],
            arity => {
                return Err(UiError::UnsupportedArity {
                    function: "atualizar",
                    arity,
                });
            }
        };

        let next_state = {
            let mut vm = Vm::new(module_ref, io)
                .with_interrupt(Arc::clone(&self.interrupt))
                .with_string_memory_limit(self.budget.max_string_bytes);
            vm.call_function("atualizar", args, self.budget.max_steps_per_event)?
        };
        let next_tree = render_view(module_ref, io, &next_state, self.budget, &self.interrupt)?;

        self.step += 1;
        self.state = next_state;
        self.tree = next_tree;

        let snapshot = UiSnapshot {
            step: self.step,
            event: Some(event),
            state: self.state.clone(),
            tree: self.tree.clone(),
        };
        if self.budget.max_history > 0 {
            if self.history.len() >= self.budget.max_history {
                self.history.remove(0);
            }
            self.history.push(snapshot);
            Ok(self.history.last().expect("snapshot recém-inserido"))
        } else {
            self.history.clear();
            self.history.push(snapshot);
            Ok(self.history.last().expect("snapshot ativo"))
        }
    }

    pub fn step(&self) -> u64 {
        self.step
    }

    pub fn state(&self) -> &Value {
        &self.state
    }

    pub fn tree(&self) -> &UiNode {
        &self.tree
    }

    pub fn history(&self) -> &[UiSnapshot] {
        &self.history
    }

    pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt)
    }
}

fn render_view(
    module: &Module,
    io: &mut dyn Io,
    state: &Value,
    budget: UiBudget,
    interrupt: &Arc<AtomicBool>,
) -> Result<UiNode, UiError> {
    if interrupt.load(Ordering::Relaxed) {
        return Err(UiError::Cancelled);
    }
    let mut builder = UiTreeBuilder::new(budget);
    let root_value = {
        let mut handler =
            |library: &str, name: &str, args: &[Value]| builder.handle_native(library, name, args);
        let mut vm = Vm::new(module, io)
            .with_interrupt(Arc::clone(interrupt))
            .with_string_memory_limit(budget.max_string_bytes)
            .with_native_handler(&mut handler);
        vm.call_function(
            "visualizar",
            vec![state.clone()],
            budget.max_steps_per_event,
        )?
    };
    builder.finish(&root_value)
}

fn check_arity(args: &[Value], expected: usize, function: &str) -> Result<(), String> {
    if args.len() == expected {
        Ok(())
    } else {
        Err(format!(
            "\"{function}\" espera {expected} argumento(s), mas recebeu {}.",
            args.len()
        ))
    }
}

fn expect_string(value: &Value, function: &str) -> Result<String, String> {
    match value {
        Value::Str(text) => Ok(text.to_string()),
        other => Err(format!(
            "\"{function}\" espera uma cadeia, mas recebeu {}.",
            other.type_name()
        )),
    }
}

fn write_nodes_json(nodes: &[UiNode], out: &mut String) {
    out.push('[');
    for (index, node) in nodes.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        node.write_json(out);
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
                use std::fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            other => out.push(other),
        }
    }
    out.push('"');
}
