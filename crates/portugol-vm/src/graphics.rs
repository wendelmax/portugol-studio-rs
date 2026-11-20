//! Tipos compartilhados entre a biblioteca Graficos (na VM) e quem exibe a
//! janela (o host): o protocolo binário dos quadros, os comandos de janela e
//! o estado de teclado e mouse.
//!
//! # Protocolo dos quadros
//!
//! Cada chamada a `Graficos.renderizar()` produz uma mensagem: uma sequência
//! de operações, cada uma começando por um código de 1 byte. Inteiros e
//! `f32` são little-endian; cadeias são `u32` (bytes) + UTF-8. A janela
//! executa as operações em ordem, mantendo o estado (cor, gradiente, fonte)
//! como o `Graphics2D` do original, e mostra o resultado.
//!
//! | código | operação | argumentos |
//! |---|---|---|
//! | 1 | carregar imagem | uid `u32`, largura `u32`, altura `u32`, RGBA |
//! | 2 | liberar imagem | uid `u32` |
//! | 3 | carregar fonte | nome, `u32` + bytes TTF |
//! | 10 | limpar | largura `i32`, altura `i32` (preenche com a pintura atual) |
//! | 11 | cor | ARGB `u32` (remove o gradiente) |
//! | 12 | gradiente | tipo `u8`, ARGB `u32`, ARGB `u32` |
//! | 13 | fonte | nome, tamanho `f32`, estilo `u8` (1 itálico, 2 negrito, 4 sublinhado) |
//! | 20 | retângulo | x, y, l, a `i32`, cantos `u8`, preencher `u8`, [rotação] |
//! | 21 | elipse | x, y, l, a `i32`, preencher `u8`, [rotação] |
//! | 22 | linha | x1, y1, x2, y2 `i32`, [rotação] |
//! | 23 | ponto | x, y `i32` |
//! | 24 | polígono | n `u32`, n × (x, y) `i32`, preencher `u8`, [rotação] |
//! | 25 | texto | x, linha de base `i32`, texto, [rotação] |
//! | 26 | imagem | uid `u32`, x, y `i32`, opacidade `u8`, [rotação] |
//! | 27 | porção de imagem | uid `u32`, x, y, xi, yi, l, a `i32`, opacidade `u8`, [rotação] |
//!
//! `[rotação]` é ângulo em radianos `f32` seguido do centro (cx, cy `i32`).

use std::collections::HashSet;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Condvar, Mutex};
use std::time::Duration;

pub mod op {
    pub const UPLOAD_IMAGE: u8 = 1;
    pub const RELEASE_IMAGE: u8 = 2;
    pub const LOAD_FONT: u8 = 3;
    pub const CLEAR: u8 = 10;
    pub const COLOR: u8 = 11;
    pub const GRADIENT: u8 = 12;
    pub const FONT: u8 = 13;
    pub const RECT: u8 = 20;
    pub const ELLIPSE: u8 = 21;
    pub const LINE: u8 = 22;
    pub const POINT: u8 = 23;
    pub const POLYGON: u8 = 24;
    pub const TEXT: u8 = 25;
    pub const IMAGE: u8 = 26;
    pub const IMAGE_PART: u8 = 27;
}

/// Comandos de janela de Graficos e Mouse.
#[derive(Debug, Clone, PartialEq)]
pub enum WindowCommand {
    Resize {
        width: i32,
        height: i32,
    },
    Title(String),
    Minimize,
    Restore,
    Decorations(bool),
    FullScreen(bool),
    Icon {
        width: u32,
        height: u32,
        rgba: Vec<u8>,
    },
    CursorVisible(bool),
}

/// Teclado e mouse da janela gráfica. O host chama os métodos de evento; a
/// VM consulta e, em `ler_tecla`/`ler_botao`, espera o próximo evento.
#[derive(Default)]
pub struct InputState {
    state: Mutex<Input>,
    changed: Condvar,
}

struct Input {
    keys: HashSet<i32>,
    any_key: bool,
    last_key: i32,
    key_releases: u64,
    mouse: (i32, i32),
    buttons: [bool; 3],
    buttons_down: i32,
    last_button: i32,
    button_releases: u64,
    closed: bool,
}

impl Default for Input {
    fn default() -> Self {
        Input {
            keys: HashSet::new(),
            any_key: false,
            last_key: -1,
            key_releases: 0,
            mouse: (0, 0),
            buttons: [false; 3],
            buttons_down: 0,
            last_button: -1,
            button_releases: 0,
            closed: false,
        }
    }
}

pub const BUTTON_LEFT: i32 = 0;
pub const BUTTON_RIGHT: i32 = 1;
pub const BUTTON_MIDDLE: i32 = 2;

/// Resultado de esperar uma tecla ou botão.
pub enum Wait {
    Got(i32),
    Interrupted,
    Closed,
}

impl InputState {
    fn lock(&self) -> std::sync::MutexGuard<'_, Input> {
        self.state.lock().unwrap_or_else(|e| e.into_inner())
    }

    /// `code` no padrão `KeyEvent.VK_*` do Java.
    pub fn key_pressed(&self, code: i32) {
        let mut input = self.lock();
        input.keys.insert(code);
        input.any_key = true;
    }

    pub fn key_released(&self, code: i32) {
        let mut input = self.lock();
        input.keys.remove(&code);
        input.last_key = code;
        input.any_key = false;
        input.key_releases += 1;
        self.changed.notify_all();
    }

    /// Janela perdeu o foco ou foi minimizada.
    pub fn focus_lost(&self) {
        let mut input = self.lock();
        input.keys.clear();
        input.any_key = false;
        input.last_key = -1;
        input.buttons = [false; 3];
        input.buttons_down = 0;
        input.last_button = -1;
    }

    pub fn mouse_moved(&self, x: i32, y: i32) {
        self.lock().mouse = (x, y);
    }

    pub fn button_pressed(&self, button: i32) {
        let mut input = self.lock();
        if let Some(slot) = input.buttons.get_mut(button as usize) {
            *slot = true;
            input.buttons_down += 1;
        }
    }

    pub fn button_released(&self, button: i32) {
        let mut input = self.lock();
        if let Some(slot) = input.buttons.get_mut(button as usize) {
            *slot = false;
            input.buttons_down -= 1;
            input.last_button = button;
            input.button_releases += 1;
            self.changed.notify_all();
        }
    }

    /// A janela foi fechada: esperas em andamento terminam.
    pub fn close(&self) {
        self.lock().closed = true;
        self.changed.notify_all();
    }

    pub fn reopen(&self) {
        *self.lock() = Input::default();
    }

    pub fn is_key_down(&self, code: i32) -> bool {
        self.lock().keys.contains(&code)
    }

    pub fn any_key_down(&self) -> bool {
        self.lock().any_key
    }

    pub fn is_button_down(&self, button: i32) -> bool {
        self.lock()
            .buttons
            .get(button as usize)
            .copied()
            .unwrap_or(false)
    }

    pub fn any_button_down(&self) -> bool {
        self.lock().buttons_down > 0
    }

    pub fn mouse_position(&self) -> (i32, i32) {
        self.lock().mouse
    }

    pub fn wait_key(&self, interrupt: &AtomicBool) -> Wait {
        self.wait(interrupt, |i| i.key_releases, |i| i.last_key)
    }

    pub fn wait_button(&self, interrupt: &AtomicBool) -> Wait {
        self.wait(interrupt, |i| i.button_releases, |i| i.last_button)
    }

    fn wait(
        &self,
        interrupt: &AtomicBool,
        counter: fn(&Input) -> u64,
        value: fn(&Input) -> i32,
    ) -> Wait {
        let mut input = self.lock();
        let start = counter(&input);
        loop {
            if input.closed {
                return Wait::Closed;
            }
            if counter(&input) != start {
                return Wait::Got(value(&input));
            }
            if interrupt.load(Ordering::Relaxed) {
                return Wait::Interrupted;
            }
            input = self
                .changed
                .wait_timeout(input, Duration::from_millis(50))
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }
}

/// Montagem de uma mensagem do protocolo.
#[derive(Default)]
pub struct FrameWriter {
    pub bytes: Vec<u8>,
}

impl FrameWriter {
    pub fn op(&mut self, code: u8) -> &mut Self {
        self.bytes.push(code);
        self
    }

    pub fn u8(&mut self, value: u8) -> &mut Self {
        self.bytes.push(value);
        self
    }

    pub fn i32(&mut self, value: i32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn u32(&mut self, value: u32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn f32(&mut self, value: f32) -> &mut Self {
        self.bytes.extend_from_slice(&value.to_le_bytes());
        self
    }

    pub fn str(&mut self, value: &str) -> &mut Self {
        self.u32(value.len() as u32);
        self.bytes.extend_from_slice(value.as_bytes());
        self
    }

    pub fn raw(&mut self, value: &[u8]) -> &mut Self {
        self.bytes.extend_from_slice(value);
        self
    }
}
