//! Interpretador do bytecode.
//!
//! A execução é feita em fatias ([`Vm::step`]) para que a IDE possa
//! interromper o programa e, na fase do depurador, parar em pontos de parada.

use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
#[cfg(not(target_arch = "wasm32"))]
use std::time::Instant;

use portugol_bytecode::{Module, Op, ValueType};
use portugol_syntax::Span;
use portugol_syntax::ast::Type;

use crate::debug::{DebugControl, DebugFrame, DebugVariable, PausedInfo};
use crate::io::Io;
#[cfg(feature = "stdlib")]
use crate::stdlib::{self, NativeContext, NativeFn};
use crate::value::{Matrix, Reference, Value};

#[cfg(not(target_arch = "wasm32"))]
struct RuntimeClock {
    started: Instant,
    excluded: Duration,
}

#[cfg(not(target_arch = "wasm32"))]
impl RuntimeClock {
    fn start() -> Self {
        Self {
            started: Instant::now(),
            excluded: Duration::ZERO,
        }
    }

    fn active_elapsed(&self) -> Duration {
        self.started.elapsed().saturating_sub(self.excluded)
    }

    fn begin_pause(&self) -> Instant {
        Instant::now()
    }

    fn end_pause(&mut self, paused_at: Instant) {
        self.excluded += paused_at.elapsed();
    }
}

#[cfg(target_arch = "wasm32")]
struct RuntimeClock;

#[cfg(target_arch = "wasm32")]
impl RuntimeClock {
    fn start() -> Self {
        Self
    }

    fn active_elapsed(&self) -> Duration {
        Duration::ZERO
    }

    fn begin_pause(&self) {}

    fn end_pause(&mut self, _paused_at: ()) {}
}

/// Maior vetor/matriz aceito, igual ao limite da análise semântica.
const MAX_ELEMENTS: usize = 16_777_216;
/// Profundidade máxima de chamadas antes de "estouro de pilha".
const MAX_DEPTH: usize = 100_000;

#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeError {
    pub message: String,
    pub span: Span,
    /// Pilha de chamadas, da função atual para a de entrada.
    pub trace: Vec<(String, Span)>,
    /// Motivo tipado quando a execução parou por um limite configurado.
    pub limit: Option<RuntimeLimitReason>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeLimitReason {
    Time,
    StringMemory,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    Running,
    Finished,
}

#[derive(Debug, Clone, Copy)]
struct Frame {
    function: u32,
    ip: usize,
    base: usize,
}

/// Callback opcional de host para resolver chamadas nativas (`Op::CallNative`),
/// inclusive no núcleo mínimo sem a feature `stdlib` (ex.: `UI.*` em `wasm32`).
/// Retornar `None` delega para o tratamento padrão da VM.
pub type HostNativeHandler<'a> =
    dyn FnMut(&str, &str, &[Value]) -> Option<Result<Value, String>> + 'a;

pub struct Vm<'a> {
    module: &'a Module,
    io: &'a mut dyn Io,
    stack: Vec<Value>,
    frames: Vec<Frame>,
    active_locals: Vec<Vec<bool>>,
    tracks_local_visibility: bool,
    globals: Vec<Value>,
    strings: Vec<Rc<str>>,
    #[cfg(feature = "stdlib")]
    natives: Vec<Option<NativeFn>>,
    #[cfg(feature = "stdlib")]
    consts: Vec<Option<Value>>,
    #[cfg(feature = "stdlib")]
    ctx: NativeContext,
    native_handler: Option<&'a mut HostNativeHandler<'a>>,
    interrupt: Arc<AtomicBool>,
    args: Vec<String>,
    debug: Option<DebugControl>,
    runtime_limit: Option<Duration>,
    runtime_clock: RuntimeClock,
    string_memory_limit: Option<usize>,
    string_memory_used: usize,
    pending_limit: Option<RuntimeLimitReason>,
    last_return: Value,
}

/// Executa o programa até o fim.
pub fn run(module: &Module, io: &mut dyn Io) -> Result<(), RuntimeError> {
    Vm::new(module, io).run()
}

type Exec<T> = Result<T, String>;

/// Nome do tipo mostrado no inspetor, incluindo dimensões e `&`.
fn type_label(ty: Type, dims: u8, by_ref: bool) -> String {
    let mut label = String::new();
    if by_ref {
        label.push('&');
    }
    label.push_str(ty.as_str());
    for _ in 0..dims {
        label.push_str("[]");
    }
    label
}

/// Texto de um valor escalar no inspetor.
fn debug_scalar(value: &Value) -> String {
    match value {
        Value::Void => "<vazio>".to_string(),
        Value::Ref(_) => "[referência]".to_string(),
        Value::Array(_) => "[vetor]".to_string(),
        Value::Matrix(_) => "[matriz]".to_string(),
        other => other.display(),
    }
}

/// Acrescenta uma variável ao inspetor, expandindo vetores e matrizes.
fn debug_value(
    out: &mut Vec<DebugVariable>,
    name: &str,
    ty: &str,
    value: &Value,
    scope: &'static str,
    max_items: usize,
    pors_debug: Option<&portugol_bytecode::PorsDebugType>,
) {
    if let Some(debug_type) = pors_debug {
        debug_nominal_value(out, name, value, scope, max_items, debug_type);
        return;
    }
    match value {
        Value::Array(cells) => {
            let cells = cells.borrow();
            let element_ty = ty.strip_suffix("[]").unwrap_or(ty);
            out.push(DebugVariable {
                name: name.to_string(),
                ty: ty.to_string(),
                value: format!("{} elemento(s)", cells.len()),
                scope,
            });
            for (index, cell) in cells.iter().take(max_items).enumerate() {
                out.push(DebugVariable {
                    name: format!("{name}[{index}]"),
                    ty: element_ty.to_string(),
                    value: debug_scalar(cell),
                    scope,
                });
            }
        }
        Value::Matrix(matrix) => {
            let matrix = matrix.borrow();
            let element_ty = ty.strip_suffix("[]").unwrap_or(ty);
            out.push(DebugVariable {
                name: name.to_string(),
                ty: ty.to_string(),
                value: format!("{}x{}", matrix.rows, matrix.cols),
                scope,
            });
            let cols = matrix.cols.max(1);
            for (index, cell) in matrix.data.iter().take(max_items).enumerate() {
                out.push(DebugVariable {
                    name: format!("{name}[{},{}]", index / cols, index % cols),
                    ty: element_ty.to_string(),
                    value: debug_scalar(cell),
                    scope,
                });
            }
        }
        _ => out.push(DebugVariable {
            name: name.to_string(),
            ty: ty.to_string(),
            value: debug_scalar(value),
            scope,
        }),
    }
}

fn debug_nominal_value(
    out: &mut Vec<DebugVariable>,
    name: &str,
    value: &Value,
    scope: &'static str,
    max_items: usize,
    debug_type: &portugol_bytecode::PorsDebugType,
) {
    if debug_type.is_vector
        && let Value::Array(items) = value
    {
        let items = items.borrow();
        out.push(DebugVariable {
            name: name.into(),
            ty: format!("vetor<{}>", debug_type.name),
            value: format!("{} elemento(s)", items.len()),
            scope,
        });
        for (index, item) in items.iter().take(max_items).enumerate() {
            debug_nominal_one(
                out,
                &format!("{name}[{index}]"),
                item,
                scope,
                max_items,
                debug_type,
            );
        }
        return;
    }
    debug_nominal_one(out, name, value, scope, max_items, debug_type);
}

fn debug_nominal_one(
    out: &mut Vec<DebugVariable>,
    name: &str,
    value: &Value,
    scope: &'static str,
    max_items: usize,
    debug_type: &portugol_bytecode::PorsDebugType,
) {
    use portugol_bytecode::PorsDebugShape;
    match (&debug_type.shape, value) {
        (PorsDebugShape::Record { fields }, Value::Array(cells)) => {
            let cells = cells.borrow();
            out.push(DebugVariable {
                name: name.into(),
                ty: debug_type.name.clone(),
                value: "registro".into(),
                scope,
            });
            for (field, field_value) in fields.iter().zip(cells.iter()) {
                let field_name = format!("{name}.{}", field.name);
                if let Some(nested) = &field.nested {
                    debug_nominal_value(out, &field_name, field_value, scope, max_items, nested);
                } else {
                    out.push(DebugVariable {
                        name: field_name,
                        ty: field.ty.clone(),
                        value: debug_scalar(field_value),
                        scope,
                    });
                }
            }
        }
        (PorsDebugShape::Enum { variants }, Value::Array(cells)) => {
            let cells = cells.borrow();
            let tag = cells.first().and_then(|value| match value {
                Value::Int(tag) => usize::try_from(*tag).ok(),
                _ => None,
            });
            if let Some(variant) = tag.and_then(|tag| variants.get(tag)) {
                out.push(DebugVariable {
                    name: name.into(),
                    ty: debug_type.name.clone(),
                    value: format!("{}::{}", debug_type.name, variant.name),
                    scope,
                });
                for (index, (field, field_value)) in
                    variant.payload.iter().zip(cells.iter().skip(1)).enumerate()
                {
                    let payload_name = format!("{name}.{}[{index}]", variant.name);
                    if let Some(nested) = &field.nested {
                        debug_nominal_value(
                            out,
                            &payload_name,
                            field_value,
                            scope,
                            max_items,
                            nested,
                        );
                    } else {
                        out.push(DebugVariable {
                            name: payload_name,
                            ty: field.ty.clone(),
                            value: debug_scalar(field_value),
                            scope,
                        });
                    }
                }
            } else {
                out.push(DebugVariable {
                    name: name.into(),
                    ty: debug_type.name.clone(),
                    value: "[enum inválido]".into(),
                    scope,
                });
            }
        }
        _ => out.push(DebugVariable {
            name: name.into(),
            ty: debug_type.name.clone(),
            value: debug_scalar(value),
            scope,
        }),
    }
}

impl<'a> Vm<'a> {
    pub fn new(module: &'a Module, io: &'a mut dyn Io) -> Self {
        let globals = module
            .globals
            .iter()
            .map(|g| Value::default_for(g.ty.into()))
            .collect();
        let strings = module
            .strings
            .iter()
            .map(|s| Rc::from(s.as_str()))
            .collect();
        #[cfg(feature = "stdlib")]
        let natives = module
            .natives
            .iter()
            .map(|n| stdlib::native(&n.library, &n.name))
            .collect();
        #[cfg(feature = "stdlib")]
        let consts = module
            .native_consts
            .iter()
            .map(|n| stdlib::constant(&n.library, &n.name))
            .collect();

        let interrupt = Arc::new(AtomicBool::new(false));
        #[cfg(feature = "stdlib")]
        let mut ctx = NativeContext::new();
        #[cfg(feature = "stdlib")]
        {
            ctx.interrupt = Arc::clone(&interrupt);
        }

        let entry = &module.functions[module.entry as usize];
        let stack = vec![Value::Void; entry.locals.len()];
        let tracks_local_visibility = module.functions.iter().any(|function| {
            function
                .code
                .iter()
                .any(|op| matches!(op, Op::SetLocalActive { .. }))
        });
        let active_locals = vec![vec![!tracks_local_visibility; entry.locals.len()]];

        Vm {
            module,
            io,
            stack,
            frames: vec![Frame {
                function: module.entry,
                ip: 0,
                base: 0,
            }],
            active_locals,
            tracks_local_visibility,
            globals,
            strings,
            #[cfg(feature = "stdlib")]
            natives,
            #[cfg(feature = "stdlib")]
            consts,
            #[cfg(feature = "stdlib")]
            ctx,
            native_handler: None,
            interrupt,
            args: Vec::new(),
            debug: None,
            runtime_limit: None,
            runtime_clock: RuntimeClock::start(),
            string_memory_limit: None,
            string_memory_used: 0,
            pending_limit: None,
            last_return: Value::Void,
        }
    }

    /// Registra um resolvedor nativo do host (compatível com `--no-default-features` e `wasm32`).
    pub fn with_native_handler(mut self, handler: &'a mut HostNativeHandler<'a>) -> Self {
        self.native_handler = Some(handler);
        self
    }

    /// Fixa a semente do gerador de números aleatórios (útil em testes).
    #[cfg(feature = "stdlib")]
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.ctx.reseed(seed);
        self
    }

    /// Pasta usada como base para caminhos relativos (a do programa, na IDE).
    #[cfg(feature = "stdlib")]
    pub fn with_working_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.ctx.work_dir = dir.into();
        self
    }

    /// Diretório devolvido por `Util.obter_diretorio_usuario` nesta execução.
    /// Permite isolar as gravações dos exemplos na suíte de regressão.
    #[cfg(feature = "stdlib")]
    pub fn with_user_dir(mut self, dir: impl Into<std::path::PathBuf>) -> Self {
        self.ctx.user_dir = Some(dir.into());
        self
    }

    /// Recursos que dependem do ambiente (diálogos, gráficos, som...).
    #[cfg(feature = "stdlib")]
    pub fn with_host(mut self, host: Box<dyn crate::host::Host>) -> Self {
        self.ctx.host = host;
        self
    }

    /// Desliga o acesso à rede (Internet e ServicosWeb falham como sem
    /// conexão). Usado nos testes de regressão.
    #[cfg(feature = "stdlib")]
    pub fn offline(mut self) -> Self {
        self.ctx.offline = true;
        self
    }

    /// Relógio simulado a partir de `epoch_ms` (ms desde 1970 UTC): `aguarde`
    /// não dorme e `tempo_decorrido`/Calendario ficam determinísticos.
    #[cfg(feature = "stdlib")]
    pub fn with_simulated_time(mut self, epoch_ms: u64) -> Self {
        self.ctx.simulate_time(epoch_ms);
        self
    }

    /// Parâmetros recebidos por `funcao inicio(cadeia params[])`.
    pub fn with_args(mut self, args: Vec<String>) -> Self {
        self.args = args;
        self
    }

    /// Liga o depurador. A VM passa a parar nos pontos de parada e a responder
    /// aos comandos de passo; `info` é publicada antes de cada pausa.
    pub fn with_debug(mut self, debug: DebugControl) -> Self {
        self.debug = Some(debug);
        self
    }

    /// Limita o tempo de CPU lógico da execução. Pausas no debugger e espera
    /// por entrada não entram no orçamento.
    pub fn with_runtime_limit(mut self, limit: Duration) -> Self {
        self.runtime_limit = Some(limit);
        self
    }

    /// Limita cumulativamente as alocações de cadeia feitas pela VM.
    pub fn with_string_memory_limit(mut self, bytes: usize) -> Self {
        self.string_memory_limit = Some(bytes);
        self
    }

    /// Usa um sinalizador de interrupção criado fora da VM (por exemplo,
    /// antes de mover a execução para outra thread).
    pub fn with_interrupt(mut self, interrupt: Arc<AtomicBool>) -> Self {
        #[cfg(feature = "stdlib")]
        {
            self.ctx.interrupt = Arc::clone(&interrupt);
        }
        self.interrupt = interrupt;
        self
    }

    /// Sinalizador para interromper a execução a partir de outra thread.
    pub fn interrupt_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.interrupt)
    }

    pub fn run(&mut self) -> Result<(), RuntimeError> {
        while self.step(100_000)? == Status::Running {}
        Ok(())
    }

    /// Invoca uma função do módulo pelo nome com orçamento explícito de instruções,
    /// devolvendo o valor retornado. Ideal para ciclos orientados a eventos (MVU)
    /// em Desktop e WASM sem depender de timeout fixo de janela.
    pub fn call_function(
        &mut self,
        name: &str,
        args: Vec<Value>,
        step_budget: usize,
    ) -> Result<Value, RuntimeError> {
        let Some((index, function)) = self
            .module
            .functions
            .iter()
            .enumerate()
            .find(|(_, function)| function.name == name)
        else {
            return Err(self.error(format!("Função não encontrada no módulo: \"{name}\".")));
        };
        if args.len() != function.arity as usize {
            return Err(self.error(format!(
                "A função \"{name}\" espera {} argumento(s), mas recebeu {}.",
                function.arity,
                args.len()
            )));
        }

        self.stack.clear();
        self.frames.clear();
        self.active_locals.clear();
        self.pending_limit = None;
        self.last_return = Value::Void;

        let local_slots = function.locals.len().max(args.len());
        self.stack.extend(args);
        self.stack.resize(local_slots, Value::Void);
        self.active_locals
            .push(vec![!self.tracks_local_visibility; function.locals.len()]);
        self.frames.push(Frame {
            function: index as u32,
            ip: 0,
            base: 0,
        });

        match self.step(step_budget)? {
            Status::Finished => Ok(std::mem::replace(&mut self.last_return, Value::Void)),
            Status::Running => {
                self.pending_limit = Some(RuntimeLimitReason::Time);
                Err(self.error("O limite de passos do evento foi excedido.".to_string()))
            }
        }
    }

    /// Executa até `budget` instruções.
    pub fn step(&mut self, budget: usize) -> Result<Status, RuntimeError> {
        let module = self.module;
        // O quadro atual fica numa cópia local durante o laço; `self.frames`
        // só é sincronizado em chamadas, retornos, erros e no fim da fatia.
        let Some(mut frame) = self.frames.last().copied() else {
            return Ok(Status::Finished);
        };
        let mut code: &[Op] = &module.functions[frame.function as usize].code;

        for executed in 0..budget {
            if executed % 1024 == 0 && self.interrupt.load(Ordering::Relaxed) {
                self.sync(frame);
                return Err(self.error("O programa foi interrompido.".to_string()));
            }
            if executed % 1024 == 0
                && self
                    .runtime_limit
                    .is_some_and(|limit| self.runtime_clock.active_elapsed() >= limit)
            {
                self.sync(frame);
                self.pending_limit = Some(RuntimeLimitReason::Time);
                return Err(self.error("O limite de tempo da execução foi excedido.".to_string()));
            }
            if let Some(debug) = &self.debug {
                let function = &module.functions[frame.function as usize];
                let span = function.spans.get(frame.ip).copied().unwrap_or_default();
                // As instruções sintéticas de entrada usam o trecho de todo o
                // programa; parar nelas não corresponde a nenhuma linha útil.
                let synthetic = (frame.function == module.entry && span == function.span)
                    || matches!(code.get(frame.ip), Some(Op::SetLocalActive { .. }));
                if !synthetic && debug.should_pause(self.frames.len(), span) {
                    let info = self.debug_info(&frame);
                    let pause_started = self.runtime_clock.begin_pause();
                    let resumed = debug.pause_and_wait(&info);
                    self.runtime_clock.end_pause(pause_started);
                    if !resumed {
                        self.sync(frame);
                        return Err(self.error("O programa foi interrompido.".to_string()));
                    }
                }
            }
            let op = code[frame.ip];
            frame.ip += 1;
            match self.exec(op, &mut frame) {
                Ok(false) => {}
                Ok(true) => {
                    if self.frames.is_empty() {
                        return Ok(Status::Finished);
                    }
                    code = &module.functions[frame.function as usize].code;
                }
                Err(message) => {
                    self.sync(frame);
                    return Err(self.error(message));
                }
            }
        }
        self.sync(frame);
        Ok(Status::Running)
    }

    fn sync(&mut self, frame: Frame) {
        if let Some(top) = self.frames.last_mut() {
            *top = frame;
        }
    }

    /// Trecho de código da instrução em execução (ou que acabou de falhar).
    pub fn current_span(&self) -> Option<Span> {
        let frame = self.frames.last()?;
        let function = &self.module.functions[frame.function as usize];
        function.spans.get(frame.ip.saturating_sub(1)).copied()
    }

    /// Estado exposto ao depurador no ponto de parada.
    fn debug_info(&self, frame: &Frame) -> PausedInfo {
        const MAX_ITEMS: usize = 100;

        let function = &self.module.functions[frame.function as usize];
        let span = function
            .spans
            .get(frame.ip)
            .copied()
            .unwrap_or(function.span);

        let mut stack = vec![DebugFrame {
            function: function.name.clone(),
            offset: span.start,
        }];
        for caller in self.frames.iter().rev().skip(1) {
            let caller_function = &self.module.functions[caller.function as usize];
            let offset = caller_function
                .spans
                .get(caller.ip.saturating_sub(1))
                .copied()
                .unwrap_or(caller_function.span)
                .start;
            stack.push(DebugFrame {
                function: caller_function.name.clone(),
                offset,
            });
        }

        let mut variables = Vec::new();
        for (info, value) in self.module.globals.iter().zip(self.globals.iter()) {
            debug_value(
                &mut variables,
                &info.name,
                &type_label(info.ty, info.dims, false),
                value,
                "global",
                MAX_ITEMS,
                None,
            );
        }
        for (slot, info) in function.locals.iter().enumerate() {
            if info.name.is_empty() {
                continue; // temporário do compilador
            }
            if !self.active_locals.last().unwrap()[slot] {
                continue;
            }
            let value = &self.stack[frame.base + slot];
            debug_value(
                &mut variables,
                &info.name,
                &info
                    .pors_debug
                    .as_ref()
                    .map(|debug| {
                        if debug.is_vector {
                            format!("vetor<{}>", debug.name)
                        } else {
                            debug.name.clone()
                        }
                    })
                    .unwrap_or_else(|| type_label(info.ty, info.dims, info.by_ref)),
                value,
                "local",
                MAX_ITEMS,
                info.pors_debug.as_ref(),
            );
        }

        PausedInfo {
            span,
            stack,
            variables,
        }
    }

    fn error(&self, message: String) -> RuntimeError {
        let trace: Vec<_> = self
            .frames
            .iter()
            .rev()
            .map(|frame| {
                let function = &self.module.functions[frame.function as usize];
                let span = function
                    .spans
                    .get(frame.ip.saturating_sub(1))
                    .copied()
                    .unwrap_or(function.span);
                (function.name.clone(), span)
            })
            .collect();
        RuntimeError {
            message,
            span: trace.first().map(|(_, span)| *span).unwrap_or_default(),
            trace,
            limit: self.pending_limit,
        }
    }

    fn charge_string(&mut self, bytes: usize) -> Exec<()> {
        let next = self.string_memory_used.saturating_add(bytes);
        if self.string_memory_limit.is_some_and(|limit| next > limit) {
            self.pending_limit = Some(RuntimeLimitReason::StringMemory);
            return Err("O limite de memória para cadeias foi excedido.".to_string());
        }
        self.string_memory_used = next;
        Ok(())
    }

    fn clone_pors_value(&mut self, value: &Value) -> Exec<Value> {
        match value {
            Value::Str(text) => {
                self.charge_string(text.len())?;
                Ok(Value::Str(Rc::clone(text)))
            }
            Value::Array(array) => {
                let items = array.borrow().clone();
                let mut cloned = Vec::with_capacity(items.len());
                for item in &items {
                    cloned.push(self.clone_pors_value(item)?);
                }
                Ok(Value::Array(Rc::new(RefCell::new(cloned))))
            }
            Value::Matrix(matrix) => {
                let matrix = matrix.borrow().clone();
                let mut data = Vec::with_capacity(matrix.data.len());
                for item in &matrix.data {
                    data.push(self.clone_pors_value(item)?);
                }
                Ok(Value::Matrix(Rc::new(RefCell::new(Matrix {
                    rows: matrix.rows,
                    cols: matrix.cols,
                    data,
                }))))
            }
            _ => Ok(value.clone()),
        }
    }

    fn pop(&mut self) -> Value {
        self.stack.pop().expect("pilha de operandos vazia")
    }

    fn pop_n(&mut self, n: u32) -> Vec<Value> {
        let at = self.stack.len() - n as usize;
        self.stack.split_off(at)
    }

    fn push(&mut self, value: Value) {
        self.stack.push(value);
    }

    /// Executa uma instrução. Devolve `true` quando o quadro atual mudou
    /// (chamada ou retorno).
    #[inline(always)]
    fn exec(&mut self, op: Op, frame: &mut Frame) -> Exec<bool> {
        let module = self.module;
        let base = frame.base;

        match op {
            Op::PushInt(value) => self.push(Value::Int(value)),
            Op::PushReal(value) => self.push(Value::Real(value)),
            Op::PushBool(value) => self.push(Value::Bool(value)),
            Op::PushChar(value) => self.push(Value::Char(value)),
            Op::PushStr(index) => {
                // Literais pertencem ao pool do módulo. `Rc::clone` só cria
                // outra referência ao mesmo texto, não uma nova cadeia.
                let value = Value::Str(Rc::clone(&self.strings[index as usize]));
                self.push(value);
            }
            Op::PushVoid => self.push(Value::Void),
            Op::PushDefault(ty) => self.push(Value::default_for(ty)),
            Op::Pop => {
                self.pop();
            }
            Op::Dup => {
                let top = self.stack.last().cloned().expect("pilha vazia");
                self.push(top);
            }

            Op::LoadLocal(slot) => {
                let value = self.stack[base + slot as usize].clone();
                self.push(value);
            }
            Op::StoreLocal(slot) => {
                let value = self.pop();
                self.stack[base + slot as usize] = value;
            }
            Op::SetLocalActive { local, active } => {
                self.active_locals.last_mut().expect("frame ativo")[local as usize] = active;
            }
            Op::LoadGlobal(slot) => {
                let value = self.globals[slot as usize].clone();
                self.push(value);
            }
            Op::StoreGlobal(slot) => {
                let value = self.pop();
                self.globals[slot as usize] = value;
            }
            Op::LoadDeref(slot) => {
                let reference = self.reference_in(base + slot as usize)?;
                let value = self.read_ref(&reference);
                self.push(value);
            }
            Op::StoreDeref(slot) => {
                let reference = self.reference_in(base + slot as usize)?;
                let value = self.pop();
                self.write_ref(&reference, value);
            }
            Op::RefLocal(slot) => {
                self.push(Value::Ref(Rc::new(Reference::Stack(base + slot as usize))));
            }
            Op::RefGlobal(slot) => {
                self.push(Value::Ref(Rc::new(Reference::Global(slot as usize))));
            }
            Op::RefElem => {
                let index = self.pop();
                let array = self.pop();
                let Value::Array(array) = array else {
                    return Err(not_a("vetor", &array));
                };
                let index = check_index(&index, array.borrow().len())?;
                self.push(Value::Ref(Rc::new(Reference::Elem(array, index))));
            }
            Op::RefMatElem => {
                let col = self.pop();
                let row = self.pop();
                let matrix = self.pop();
                let Value::Matrix(matrix) = matrix else {
                    return Err(not_a("matriz", &matrix));
                };
                let index = matrix_index(&matrix.borrow(), &row, &col)?;
                self.push(Value::Ref(Rc::new(Reference::MatElem(matrix, index))));
            }

            Op::NewArray(ty) => {
                let size = self.pop();
                let size = check_size(&size)?;
                let array = vec![Value::default_for(ty); size];
                self.push(Value::Array(Rc::new(RefCell::new(array))));
            }
            Op::NewMatrix(ty) => {
                let cols = self.pop();
                let rows = self.pop();
                let rows = check_size(&rows)?;
                let cols = check_size(&cols)?;
                if rows.saturating_mul(cols) > MAX_ELEMENTS {
                    return Err(out_of_memory());
                }
                let matrix = Matrix {
                    rows,
                    cols,
                    data: vec![Value::default_for(ty); rows * cols],
                };
                self.push(Value::Matrix(Rc::new(RefCell::new(matrix))));
            }
            Op::ArrayLit(count) => {
                let values = self.pop_n(count);
                self.push(Value::Array(Rc::new(RefCell::new(values))));
            }
            Op::ArrayLen => {
                let array = self.pop();
                let Value::Array(array) = array else {
                    return Err(not_a("vetor", &array));
                };
                self.push(Value::Int(array.borrow().len() as i32));
            }
            Op::CloneArray => {
                let array = self.pop();
                let Value::Array(array) = array else {
                    return Err(not_a("vetor", &array));
                };
                let values = array.borrow().clone();
                let mut cloned = Vec::with_capacity(values.len());
                for value in values.iter() {
                    cloned.push(self.clone_pors_value(value)?);
                }
                self.push(Value::Array(Rc::new(RefCell::new(cloned))));
            }
            Op::CloneString => {
                let value = self.pop();
                let Value::Str(text) = value else {
                    return Err(not_a("cadeia", &value));
                };
                self.charge_string(text.len())?;
                self.push(Value::Str(Rc::from(text.as_ref())));
            }
            Op::MatrixLit { rows, cols } => {
                let data = self.pop_n(rows * cols);
                let matrix = Matrix {
                    rows: rows as usize,
                    cols: cols as usize,
                    data,
                };
                self.push(Value::Matrix(Rc::new(RefCell::new(matrix))));
            }
            Op::LoadElem => {
                let index = self.pop();
                let array = self.pop();
                let Value::Array(array) = array else {
                    return Err(not_a("vetor", &array));
                };
                let array = array.borrow();
                let index = check_index(&index, array.len())?;
                let value = array[index].clone();
                drop(array);
                self.push(value);
            }
            Op::StoreElem => {
                let index = self.pop();
                let array = self.pop();
                let value = self.pop();
                let Value::Array(array) = array else {
                    return Err(not_a("vetor", &array));
                };
                let mut array = array.borrow_mut();
                let index = check_index(&index, array.len())?;
                array[index] = value;
            }
            Op::LoadMatElem => {
                let col = self.pop();
                let row = self.pop();
                let matrix = self.pop();
                let Value::Matrix(matrix) = matrix else {
                    return Err(not_a("matriz", &matrix));
                };
                let matrix = matrix.borrow();
                let index = matrix_index(&matrix, &row, &col)?;
                let value = matrix.data[index].clone();
                drop(matrix);
                self.push(value);
            }
            Op::StoreMatElem => {
                let col = self.pop();
                let row = self.pop();
                let matrix = self.pop();
                let value = self.pop();
                let Value::Matrix(matrix) = matrix else {
                    return Err(not_a("matriz", &matrix));
                };
                let mut matrix = matrix.borrow_mut();
                let index = matrix_index(&matrix, &row, &col)?;
                matrix.data[index] = value;
            }

            Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Mod
            | Op::BitAnd
            | Op::BitOr
            | Op::BitXor
            | Op::Shl
            | Op::Shr
            | Op::Eq
            | Op::Ne
            | Op::Lt
            | Op::Le
            | Op::Gt
            | Op::Ge => {
                // Caminho rápido: dois inteiros no topo, resultado no lugar.
                let len = self.stack.len();
                if let [.., Value::Int(a), Value::Int(b)] = self.stack.as_slice()
                    && let Some(result) = int_binary(op, *a, *b)
                {
                    // Os descartados são `Int`, sem nada a liberar: `forget`
                    // evita a chamada à cola de `drop` de `Value`.
                    std::mem::forget(self.stack.pop());
                    std::mem::forget(std::mem::replace(&mut self.stack[len - 2], result));
                    return Ok(false);
                }
                let right = self.pop();
                let left = self.pop();
                let result = binary(op, &left, &right)?;
                if let Value::Str(text) = &result {
                    self.charge_string(text.len())?;
                }
                self.push(result);
            }
            Op::Neg => {
                let value = self.pop();
                let result = match num(&value) {
                    Some(Num::Int(v)) => Value::Int(v.wrapping_neg()),
                    Some(Num::Real(v)) => Value::Real(-v),
                    None => return Err(incompatible("menos unário", &value)),
                };
                self.push(result);
            }
            Op::Not => {
                let value = self.pop();
                let Value::Bool(v) = value else {
                    return Err(incompatible("nao", &value));
                };
                self.push(Value::Bool(!v));
            }
            Op::BitNot => {
                let value = self.pop();
                let Value::Int(v) = value else {
                    return Err(incompatible("~", &value));
                };
                self.push(Value::Int(!v));
            }
            Op::Convert(ty) => {
                let top = self.stack.last_mut().expect("pilha vazia");
                if let Some(converted) = convert(top, ty) {
                    *top = converted;
                }
            }

            Op::Jump(target) => frame.ip = target as usize,
            Op::JumpIfFalse(target) => {
                if !self.pop_bool()? {
                    frame.ip = target as usize;
                }
            }
            Op::JumpIfTrue(target) => {
                if self.pop_bool()? {
                    frame.ip = target as usize;
                }
            }
            Op::JumpIfFalseKeep(target) => {
                if !self.peek_bool()? {
                    frame.ip = target as usize;
                } else {
                    self.pop();
                }
            }
            Op::JumpIfTrueKeep(target) => {
                if self.peek_bool()? {
                    frame.ip = target as usize;
                } else {
                    self.pop();
                }
            }
            Op::Call { function, argc } => {
                if self.frames.len() >= MAX_DEPTH {
                    return Err("Ocorreu um estouro de pilha de memória no programa. \
                         Isto significa que existe alguma função do programa que está \
                         sendo chamada de forma recursiva sem uma condição de parada."
                        .to_string());
                }
                let callee = &module.functions[function as usize];
                let base = self.stack.len() - argc as usize;
                self.stack
                    .resize(base + callee.locals.len().max(argc as usize), Value::Void);
                self.active_locals
                    .push(vec![!self.tracks_local_visibility; callee.locals.len()]);
                // O topo de `self.frames` guarda o ponto de retorno do chamador.
                self.sync(*frame);
                *frame = Frame {
                    function,
                    ip: 0,
                    base,
                };
                self.frames.push(*frame);
                return Ok(true);
            }
            Op::CallNative { native, argc } => {
                let args = self.pop_n(argc);
                let native_ref = &module.natives[native as usize];
                if let Some(handler) = self.native_handler.as_deref_mut()
                    && let Some(outcome) = handler(&native_ref.library, &native_ref.name, &args)
                {
                    let result = outcome?;
                    if let Value::Str(text) = &result {
                        self.charge_string(text.len())?;
                    }
                    self.push(result);
                    return Ok(false);
                }
                if let Some(outcome) = crate::pors_libs::call(
                    &native_ref.library,
                    &native_ref.name,
                    &args,
                    &mut |bytes| self.charge_string(bytes),
                ) {
                    let result = outcome?;
                    self.push(result);
                    return Ok(false);
                }
                #[cfg(feature = "stdlib")]
                {
                    let Some(function) = self.natives[native as usize] else {
                        return Err(format!(
                            "A função \"{}.{}\" ainda não está disponível nesta versão.",
                            native_ref.library, native_ref.name
                        ));
                    };
                    let result = function(&mut self.ctx, &args)?;
                    self.push(result);
                }
                #[cfg(not(feature = "stdlib"))]
                {
                    return Err(format!(
                        "A função \"{}.{}\" não está habilitada neste núcleo mínimo da VM.",
                        native_ref.library, native_ref.name
                    ));
                }
            }
            Op::LoadNativeConst(index) => {
                #[cfg(feature = "stdlib")]
                {
                    let Some(value) = self.consts[index as usize].clone() else {
                        let constant = &module.native_consts[index as usize];
                        return Err(format!(
                            "A constante \"{}.{}\" ainda não está disponível nesta versão.",
                            constant.library, constant.name
                        ));
                    };
                    self.push(value);
                }
                #[cfg(not(feature = "stdlib"))]
                {
                    let constant = &module.native_consts[index as usize];
                    return Err(format!(
                        "A constante \"{}.{}\" não está habilitada neste núcleo mínimo da VM.",
                        constant.library, constant.name
                    ));
                }
            }
            Op::Return => {
                let value = self.pop();
                self.frames.pop();
                self.active_locals.pop();
                self.stack.truncate(frame.base);
                if let Some(caller) = self.frames.last() {
                    *frame = *caller;
                    self.push(value);
                } else {
                    self.last_return = value;
                }
                return Ok(true);
            }

            Op::ProgramArgs => {
                let args = self.args.iter().map(|a| Value::str(a.as_str())).collect();
                self.push(Value::Array(Rc::new(RefCell::new(args))));
            }

            Op::Write(argc) => {
                let args = self.pop_n(argc);
                for arg in args {
                    self.io.write_value(&arg);
                }
            }
            Op::Read(ty) => {
                let wait_started = self.runtime_clock.begin_pause();
                let line = self.io.read_line(ty);
                self.runtime_clock.end_pause(wait_started);
                let value = parse_input(ty, line)?;
                if let Value::Str(text) = &value {
                    self.charge_string(text.len())?;
                }
                self.push(value);
            }
            Op::Clear => self.io.clear(),
            Op::Random => {
                let max = self.pop();
                let min = self.pop();
                let (Value::Int(_min), Value::Int(_max)) = (&min, &max) else {
                    return Err("\"sorteia\" espera dois valores inteiros".to_string());
                };
                #[cfg(feature = "stdlib")]
                {
                    let value = stdlib::sorteia(&mut self.ctx, *_min, *_max)?;
                    self.push(Value::Int(value));
                }
                #[cfg(not(feature = "stdlib"))]
                {
                    return Err(
                        "\"sorteia\" não está habilitada neste núcleo mínimo da VM.".to_string()
                    );
                }
            }
        }
        Ok(false)
    }

    fn pop_bool(&mut self) -> Exec<bool> {
        match self.pop() {
            Value::Bool(value) => Ok(value),
            other => Err(not_a("logico", &other)),
        }
    }

    fn peek_bool(&self) -> Exec<bool> {
        match self.stack.last() {
            Some(Value::Bool(value)) => Ok(*value),
            Some(other) => Err(not_a("logico", other)),
            None => Err("pilha vazia".to_string()),
        }
    }

    fn reference_in(&self, slot: usize) -> Exec<Rc<Reference>> {
        match &self.stack[slot] {
            Value::Ref(reference) => Ok(Rc::clone(reference)),
            other => Err(not_a("referência", other)),
        }
    }

    fn read_ref(&self, reference: &Reference) -> Value {
        match reference {
            Reference::Stack(slot) => self.stack[*slot].clone(),
            Reference::Global(slot) => self.globals[*slot].clone(),
            Reference::Elem(array, index) => array.borrow()[*index].clone(),
            Reference::MatElem(matrix, index) => matrix.borrow().data[*index].clone(),
        }
    }

    fn write_ref(&mut self, reference: &Reference, value: Value) {
        match reference {
            Reference::Stack(slot) => self.stack[*slot] = value,
            Reference::Global(slot) => self.globals[*slot] = value,
            Reference::Elem(array, index) => array.borrow_mut()[*index] = value,
            Reference::MatElem(matrix, index) => matrix.borrow_mut().data[*index] = value,
        }
    }
}

// ---------------------------------------------------------------------
// Operações
// ---------------------------------------------------------------------

#[derive(Debug, Clone, Copy)]
enum Num {
    Int(i32),
    Real(f64),
}

impl Num {
    fn real(self) -> f64 {
        match self {
            Num::Int(v) => v as f64,
            Num::Real(v) => v,
        }
    }
}

/// `caracter` participa de contas como o código do caractere, igual ao `char`
/// do Java.
fn num(value: &Value) -> Option<Num> {
    match value {
        Value::Int(v) => Some(Num::Int(*v)),
        Value::Real(v) => Some(Num::Real(*v)),
        Value::Char(c) => Some(Num::Int(*c as i32)),
        _ => None,
    }
}

fn binary(op: Op, left: &Value, right: &Value) -> Exec<Value> {
    use Value::*;

    // Concatenação: basta um dos lados ser cadeia. `caracter + caracter`
    // também concatena, seguindo o tipo `cadeia` dado pela análise semântica.
    if matches!(op, Op::Add)
        && (matches!(left, Str(_))
            || matches!(right, Str(_))
            || matches!((left, right), (Char(_), Char(_))))
    {
        let mut text = left.concat_text();
        text.push_str(&right.concat_text());
        return Ok(Value::str(text));
    }

    match op {
        Op::Eq | Op::Ne => {
            let equal = match (left, right) {
                (Str(a), Str(b)) => a == b,
                (Bool(a), Bool(b)) => a == b,
                (Char(a), Char(b)) => a == b,
                _ => match (num(left), num(right)) {
                    (Some(Num::Int(a)), Some(Num::Int(b))) => a == b,
                    (Some(a), Some(b)) => a.real() == b.real(),
                    _ => return Err(incompatible_pair(op, left, right)),
                },
            };
            return Ok(Bool(if op == Op::Eq { equal } else { !equal }));
        }
        Op::BitAnd | Op::BitOr | Op::BitXor => {
            if let (Bool(a), Bool(b)) = (left, right) {
                return Ok(Bool(match op {
                    Op::BitAnd => a & b,
                    Op::BitOr => a | b,
                    _ => a ^ b,
                }));
            }
        }
        _ => {}
    }

    let (Some(a), Some(b)) = (num(left), num(right)) else {
        return Err(incompatible_pair(op, left, right));
    };

    Ok(match (a, b) {
        (Num::Int(a), Num::Int(b)) => match op {
            Op::Add => Int(a.wrapping_add(b)),
            Op::Sub => Int(a.wrapping_sub(b)),
            Op::Mul => Int(a.wrapping_mul(b)),
            Op::Div => {
                if b == 0 {
                    return Err(division_by_zero());
                }
                Int(a.wrapping_div(b))
            }
            Op::Mod => {
                if b == 0 {
                    return Err(division_by_zero());
                }
                Int(a.wrapping_rem(b))
            }
            Op::BitAnd => Int(a & b),
            Op::BitOr => Int(a | b),
            Op::BitXor => Int(a ^ b),
            // Como no Java, só os 5 bits menos significativos contam.
            Op::Shl => Int(a.wrapping_shl(b as u32)),
            Op::Shr => Int(a.wrapping_shr(b as u32)),
            Op::Lt => Bool(a < b),
            Op::Le => Bool(a <= b),
            Op::Gt => Bool(a > b),
            Op::Ge => Bool(a >= b),
            _ => unreachable!("operador binário: {op:?}"),
        },
        (a, b) => {
            let (a, b) = (a.real(), b.real());
            match op {
                Op::Add => Real(a + b),
                Op::Sub => Real(a - b),
                Op::Mul => Real(a * b),
                // Divisão real por zero dá infinito, como no Java.
                Op::Div => Real(a / b),
                Op::Mod => Real(a % b),
                Op::Lt => Bool(a < b),
                Op::Le => Bool(a <= b),
                Op::Gt => Bool(a > b),
                Op::Ge => Bool(a >= b),
                _ => return Err(incompatible_pair(op, left, right)),
            }
        }
    })
}

/// Inteiros com resultado definido; `None` manda para o caminho genérico
/// (inclusive divisão por zero, que vira erro lá).
#[inline(always)]
fn int_binary(op: Op, a: i32, b: i32) -> Option<Value> {
    Some(match op {
        Op::Add => Value::Int(a.wrapping_add(b)),
        Op::Sub => Value::Int(a.wrapping_sub(b)),
        Op::Mul => Value::Int(a.wrapping_mul(b)),
        Op::Div if b != 0 => Value::Int(a.wrapping_div(b)),
        Op::Mod if b != 0 => Value::Int(a.wrapping_rem(b)),
        Op::Eq => Value::Bool(a == b),
        Op::Ne => Value::Bool(a != b),
        Op::Lt => Value::Bool(a < b),
        Op::Le => Value::Bool(a <= b),
        Op::Gt => Value::Bool(a > b),
        Op::Ge => Value::Bool(a >= b),
        _ => return None,
    })
}

/// Valor convertido, ou `None` quando nada muda.
fn convert(value: &Value, ty: ValueType) -> Option<Value> {
    Some(match (ty, value) {
        // `(int)` do Java: trunca, satura nos limites e NaN vira 0 — o mesmo
        // que `as i32` em Rust.
        (ValueType::Inteiro, Value::Real(v)) => Value::Int(*v as i32),
        (ValueType::Inteiro, Value::Char(c)) => Value::Int(*c as i32),
        (ValueType::Real, Value::Int(v)) => Value::Real(*v as f64),
        (ValueType::Real, Value::Char(c)) => Value::Real(*c as u32 as f64),
        (ValueType::Caracter, Value::Int(v)) => {
            Value::Char(char::from_u32(*v as u32 & 0xFFFF).unwrap_or('\u{FFFD}'))
        }
        _ => return None,
    })
}

/// Converte a linha digitada como a aba de console do Portugol Studio.
fn parse_input(ty: ValueType, line: Option<String>) -> Exec<Value> {
    let invalid = || {
        format!(
            "A entrada de dados do programa esperava um valor do tipo '{}', mas nenhum valor \
             foi informado ou o valor informado é de outro tipo",
            ty.name()
        )
    };
    let Some(line) = line else {
        return Err(invalid());
    };
    if line.is_empty() && ty != ValueType::Cadeia {
        return Err(invalid());
    }
    match ty {
        ValueType::Inteiro => line.parse().map(Value::Int).map_err(|_| invalid()),
        ValueType::Real => line.trim().parse().map(Value::Real).map_err(|_| invalid()),
        ValueType::Caracter => Ok(Value::Char(line.chars().next().expect("linha não vazia"))),
        ValueType::Logico => match line.as_str() {
            "verdadeiro" => Ok(Value::Bool(true)),
            "falso" => Ok(Value::Bool(false)),
            _ => Err(invalid()),
        },
        ValueType::Cadeia => Ok(Value::str(line)),
        ValueType::Vazio => Err(invalid()),
    }
}

fn check_index(index: &Value, len: usize) -> Exec<usize> {
    let Value::Int(index) = index else {
        return Err(not_a("inteiro", index));
    };
    if *index < 0 || *index as usize >= len {
        return Err(format!(
            "Você tentou acessar um índice de vetor ou matriz inválido: o índice {index} \
             está fora do intervalo 0..{}.",
            len as i64 - 1
        ));
    }
    Ok(*index as usize)
}

fn matrix_index(matrix: &Matrix, row: &Value, col: &Value) -> Exec<usize> {
    let row = check_index(row, matrix.rows)?;
    let col = check_index(col, matrix.cols)?;
    Ok(row * matrix.cols + col)
}

fn check_size(size: &Value) -> Exec<usize> {
    let Value::Int(size) = size else {
        return Err(not_a("inteiro", size));
    };
    if *size < 0 {
        return Err(format!(
            "O tamanho de um vetor ou matriz não pode ser negativo ({size})."
        ));
    }
    if *size as usize > MAX_ELEMENTS {
        return Err(out_of_memory());
    }
    Ok(*size as usize)
}

fn out_of_memory() -> String {
    "O programa atingiu a quantidade máxima de memória RAM permitida.".to_string()
}

fn division_by_zero() -> String {
    "Foi efetuada uma divisão por zero.".to_string()
}

fn not_a(expected: &str, value: &Value) -> String {
    format!(
        "Erro interno: esperava um valor do tipo {expected}, mas encontrou {}.",
        value.type_name()
    )
}

fn incompatible(op: &str, value: &Value) -> String {
    format!(
        "Tipos incompatíveis: a operação \"{op}\" não aceita um valor do tipo \"{}\".",
        value.type_name()
    )
}

fn incompatible_pair(op: Op, left: &Value, right: &Value) -> String {
    format!(
        "Tipos incompatíveis: não é possível aplicar {op:?} a \"{}\" e \"{}\".",
        left.type_name(),
        right.type_name()
    )
}
