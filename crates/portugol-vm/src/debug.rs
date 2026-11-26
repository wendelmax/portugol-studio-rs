//! Controle do depurador compartilhado entre a VM e quem a executa (IDE,
//! testes). A VM verifica o controle antes de cada instrução; quando deve
//! parar, publica o estado e bloqueia numa variável de condição até receber
//! um comando. Assim `Vm::run()` continua funcionando normalmente sem
//! depurador e a IDE pode controlar a execução de outra thread.

use std::sync::{Condvar, Mutex};

use portugol_hir::Span;

/// O que a VM deve fazer depois de voltar de uma pausa.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DebugCommand {
    /// Continua até o próximo ponto de parada ou pedido de pausa.
    Continue,
    /// Para na próxima instrução, mesmo que seja dentro de uma chamada.
    StepInto,
    /// Para na próxima instrução deste quadro (ou de um chamador).
    StepOver,
    /// Para quando a função atual retornar.
    StepOut,
}

/// Uma variável visível no ponto de parada.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugVariable {
    pub name: String,
    pub ty: String,
    pub value: String,
    /// `"global"` ou `"local"`.
    pub scope: &'static str,
}

/// Um quadro da pilha de chamadas, do atual para o de entrada.
#[derive(Debug, Clone, PartialEq)]
pub struct DebugFrame {
    pub function: String,
    /// Deslocamento em bytes no fonte; a fachada converte para linha/coluna.
    pub offset: u32,
}

/// Estado publicado quando a VM para.
#[derive(Debug, Clone, PartialEq)]
pub struct PausedInfo {
    pub span: Span,
    pub stack: Vec<DebugFrame>,
    pub variables: Vec<DebugVariable>,
}

type PauseCallback = Box<dyn Fn(&PausedInfo) + Send + Sync>;

#[derive(Debug)]
struct Inner {
    breakpoints: Vec<Span>,
    command: DebugCommand,
    /// Mudado a cada comando para que a pausa possa esperar por um novo.
    seq: u64,
    pause_requested: bool,
    paused_depth: usize,
    /// Profundidade do quadro em que o passo foi pedido.
    target: usize,
    /// Depois de retomar, não considerar parar na mesma instrução de novo.
    skip_span: Option<Span>,
    /// Trecho que motivou a pausa: o ponto de parada inteiro, quando foi ele
    /// que disparou, para não parar de novo na mesma linha.
    pause_span: Option<Span>,
    stopped: bool,
}

/// Alça de controle do depurador. `Clone` compartilha o mesmo estado.
#[derive(Clone)]
pub struct DebugControl {
    inner: std::sync::Arc<Mutex<Inner>>,
    changed: std::sync::Arc<Condvar>,
    on_pause: std::sync::Arc<Mutex<Option<PauseCallback>>>,
}

impl std::fmt::Debug for DebugControl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DebugControl").finish_non_exhaustive()
    }
}

impl DebugControl {
    pub fn new(on_pause: impl Fn(&PausedInfo) + Send + Sync + 'static) -> Self {
        DebugControl {
            inner: std::sync::Arc::new(Mutex::new(Inner {
                breakpoints: Vec::new(),
                command: DebugCommand::Continue,
                seq: 0,
                pause_requested: false,
                paused_depth: 0,
                target: 0,
                skip_span: None,
                pause_span: None,
                stopped: false,
            })),
            changed: std::sync::Arc::new(Condvar::new()),
            on_pause: std::sync::Arc::new(Mutex::new(Some(Box::new(on_pause)))),
        }
    }

    fn lock_inner(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(|error| error.into_inner())
    }

    fn lock_on_pause(&self) -> std::sync::MutexGuard<'_, Option<PauseCallback>> {
        self.on_pause
            .lock()
            .unwrap_or_else(|error| error.into_inner())
    }

    /// Substitui os pontos de parada. Cada `Span` cobre uma linha inteira do
    /// fonte (a fachada converte o número da linha); a VM compara com o trecho
    /// que gerou cada instrução.
    pub fn set_breakpoints(&self, breakpoints: Vec<Span>) {
        self.lock_inner().breakpoints = breakpoints;
    }

    /// Pede uma pausa na próxima instrução, mesmo sem ponto de parada.
    pub fn request_pause(&self) {
        let mut inner = self.lock_inner();
        inner.pause_requested = true;
        self.changed.notify_all();
    }

    /// Ordena que a VM siga (ou dê um passo) depois da pausa atual.
    pub fn command(&self, command: DebugCommand) {
        let mut inner = self.lock_inner();
        if command != DebugCommand::Continue {
            let depth = inner.paused_depth;
            inner.target_depth(depth);
        }
        inner.command = command;
        inner.seq += 1;
        self.changed.notify_all();
    }

    /// Desbloqueia a VM e faz o laço de execução terminar em "interrompido".
    pub fn stop(&self) {
        let mut inner = self.lock_inner();
        inner.stopped = true;
        self.changed.notify_all();
    }

    /// Chamado pela VM antes de executar a instrução em `span`.
    pub(crate) fn should_pause(&self, depth: usize, span: Span) -> bool {
        let mut inner = self.lock_inner();
        if inner.stopped {
            return false;
        }
        // Um comando só começa a valer depois que a instrução (ou a linha)
        // onde estávamos parados terminar de executar.
        if inner
            .skip_span
            .is_some_and(|skip| skip.start <= span.start && span.end <= skip.end)
        {
            return false;
        }
        inner.skip_span = None;
        let mut pause_span = None;
        let pause = match inner.command {
            DebugCommand::Continue => {
                if inner.pause_requested {
                    pause_span = Some(span);
                    true
                } else if let Some(bp) = inner
                    .breakpoints
                    .iter()
                    .find(|bp| bp.start <= span.start && span.end <= bp.end)
                {
                    pause_span = Some(*bp);
                    true
                } else {
                    false
                }
            }
            DebugCommand::StepInto => {
                pause_span = Some(span);
                true
            }
            DebugCommand::StepOver => {
                pause_span = Some(span);
                depth <= inner.target
            }
            DebugCommand::StepOut => {
                pause_span = Some(span);
                depth < inner.target
            }
        };
        if pause {
            inner.paused_depth = depth;
            inner.pause_requested = false;
            inner.pause_span = pause_span;
        }
        pause
    }

    /// Publica `info` e espera até que um novo comando (ou `stop`) chegue.
    /// Devolve `false` se a execução foi interrompida.
    ///
    /// O callback é chamado com o bloqueio `inner` tomado: assim um comando
    /// enviado pela IDE (que também precisa de `inner`) só é processado depois
    /// que a espera estiver registrada, sem perder o sinal da variável de
    /// condição. O callback não deve chamar métodos deste controle.
    pub(crate) fn pause_and_wait(&self, info: &PausedInfo) -> bool {
        let mut inner = self.lock_inner();
        let seq = inner.seq;
        if let Some(callback) = self.lock_on_pause().as_ref() {
            callback(info);
        }
        while !inner.stopped && inner.seq == seq {
            inner = self
                .changed
                .wait(inner)
                .unwrap_or_else(|error| error.into_inner());
        }
        let stopped = inner.stopped;
        inner.skip_span = inner.pause_span.or(Some(info.span));
        !stopped
    }
}

impl Inner {
    fn target_depth(&mut self, depth: usize) {
        // `target_depth` é o quadro em que o passo foi pedido. Guardado num
        // campo separado para não confundir com a profundidade atual.
        self.target = depth;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_gracefully_after_mutex_poisoning() {
        let control = DebugControl::new(|_| panic!("falha simulada no callback"));
        let cloned = control.clone();
        let join = std::thread::spawn(move || {
            let info = PausedInfo {
                span: Span::new(0, 1),
                stack: Vec::new(),
                variables: Vec::new(),
            };
            cloned.pause_and_wait(&info);
        });
        assert!(join.join().is_err());

        // Chamadas subsequentes não devem entrar em pânico por PoisonError.
        control.set_breakpoints(vec![Span::new(0, 10)]);
        control.request_pause();
        control.command(DebugCommand::Continue);
        control.stop();
        assert!(!control.should_pause(0, Span::new(0, 5)));
    }
}
