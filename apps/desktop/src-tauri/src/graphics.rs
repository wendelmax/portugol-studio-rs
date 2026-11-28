//! Janela da biblioteca Graficos. A VM publica um quadro de cada vez e espera
//! pela confirmação da webview. Quadros e capturas passam pelo IPC binário.

use std::collections::HashMap;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::time::{Duration, Instant};

use ferro_service::{
    FileFilter, Host, RunObserver, Sound,
    graphics::{InputState, WindowCommand},
};
use serde::Deserialize;
use tauri::{
    AppHandle, LogicalSize, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder, WindowEvent,
    image::Image,
    ipc::{InvokeBody, Request, Response},
};

use super::{Runs, audio::AudioOutput};

const POLL: Duration = Duration::from_millis(50);
const FRAME_TIMEOUT: Duration = Duration::from_secs(15);

#[derive(Default)]
pub struct GraphicsWindows {
    next: AtomicU32,
    sessions: Mutex<HashMap<String, Arc<Session>>>,
}

#[derive(Default)]
struct Mailbox {
    request: Option<Vec<u8>>,
    reply: Option<Result<Vec<u8>, String>>,
    closed: bool,
}

#[derive(Default)]
struct Session {
    input: Arc<InputState>,
    mailbox: Mutex<Mailbox>,
    changed: Condvar,
}

impl Session {
    fn close(&self) {
        self.input.close();
        self.mailbox
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .closed = true;
        self.changed.notify_all();
    }

    fn submit(&self, request: Vec<u8>, interrupted: &AtomicBool) -> Result<Vec<u8>, String> {
        let started = Instant::now();
        let mut mailbox = self.mailbox.lock().unwrap_or_else(|e| e.into_inner());
        if mailbox.closed || interrupted.load(Ordering::Relaxed) {
            return Err("A janela gráfica foi fechada.".into());
        }
        mailbox.request = Some(request);
        self.changed.notify_all();
        loop {
            if mailbox.closed || interrupted.load(Ordering::Relaxed) {
                return Err("A janela gráfica foi fechada.".into());
            }
            if let Some(reply) = mailbox.reply.take() {
                return reply;
            }
            if started.elapsed() >= FRAME_TIMEOUT {
                return Err("A janela gráfica não respondeu.".into());
            }
            mailbox = self
                .changed
                .wait_timeout(mailbox, POLL)
                .unwrap_or_else(|e| e.into_inner())
                .0;
        }
    }

    fn next(&self) -> Result<Vec<u8>, String> {
        let mailbox = self.mailbox.lock().unwrap_or_else(|e| e.into_inner());
        let (mut mailbox, _) = self
            .changed
            .wait_timeout_while(mailbox, Duration::from_millis(250), |mailbox| {
                !mailbox.closed && mailbox.request.is_none()
            })
            .unwrap_or_else(|e| e.into_inner());
        if mailbox.closed {
            return Err("A janela gráfica foi fechada.".into());
        }
        // Resposta vazia ao vencer a espera, para o frontend repetir a consulta.
        Ok(mailbox.request.take().unwrap_or_default())
    }

    fn reply(&self, reply: Result<Vec<u8>, String>) {
        let mut mailbox = self.mailbox.lock().unwrap_or_else(|e| e.into_inner());
        mailbox.reply = Some(reply);
        self.changed.notify_all();
    }
}

fn session(window: &WebviewWindow) -> Result<Arc<Session>, String> {
    window
        .state::<GraphicsWindows>()
        .sessions
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .get(window.label())
        .cloned()
        .ok_or_else(|| "Janela gráfica indisponível.".into())
}

#[tauri::command]
pub async fn graphics_next(window: WebviewWindow) -> Result<Response, String> {
    let session = session(&window)?;
    tauri::async_runtime::spawn_blocking(move || session.next().map(Response::new))
        .await
        .map_err(|error| error.to_string())?
}

#[tauri::command]
pub fn graphics_reply(window: WebviewWindow, request: Request<'_>) -> Result<(), String> {
    let InvokeBody::Raw(payload) = request.body() else {
        return Err("A resposta gráfica deve usar o protocolo binário.".into());
    };
    let (kind, data) = payload
        .split_first()
        .ok_or_else(|| "Resposta gráfica vazia.".to_string())?;
    let reply = match kind {
        0 => Ok(data.to_vec()),
        1 => Err(String::from_utf8_lossy(data).into_owned()),
        _ => return Err("Resposta gráfica desconhecida.".into()),
    };
    session(&window)?.reply(reply);
    Ok(())
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum InputEvent {
    KeyDown { code: i32 },
    KeyUp { code: i32 },
    MouseMove { x: i32, y: i32 },
    MouseDown { button: i32 },
    MouseUp { button: i32 },
    Blur,
}

#[tauri::command]
pub fn graphics_input(window: WebviewWindow, events: Vec<InputEvent>) -> Result<(), String> {
    let session = session(&window)?;
    for event in events {
        match event {
            InputEvent::KeyDown { code } => session.input.key_pressed(code),
            InputEvent::KeyUp { code } => session.input.key_released(code),
            InputEvent::MouseMove { x, y } => session.input.mouse_moved(x, y),
            InputEvent::MouseDown { button } => session.input.button_pressed(button),
            InputEvent::MouseUp { button } => session.input.button_released(button),
            InputEvent::Blur => session.input.focus_lost(),
        }
    }
    Ok(())
}

pub struct DesktopHost {
    app: AppHandle,
    id: u32,
    observer: Arc<dyn RunObserver>,
    interrupt: Arc<AtomicBool>,
    window: Option<(WebviewWindow, Arc<Session>)>,
    size: (i32, i32),
    windowed_size: Option<(i32, i32)>,
    audio: AudioOutput,
}

impl DesktopHost {
    pub fn new(
        app: AppHandle,
        id: u32,
        observer: Arc<dyn RunObserver>,
        interrupt: Arc<AtomicBool>,
    ) -> Self {
        Self {
            app,
            id,
            observer,
            interrupt,
            window: None,
            size: (640, 480),
            windowed_size: None,
            audio: AudioOutput::default(),
        }
    }

    fn ensure_window(&mut self) -> Result<&(WebviewWindow, Arc<Session>), String> {
        if self.window.is_none() {
            let manager = self.app.state::<GraphicsWindows>();
            let number = manager.next.fetch_add(1, Ordering::Relaxed);
            let label = format!("graphics-{number}");
            let session = Arc::new(Session::default());
            manager
                .sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .insert(label.clone(), Arc::clone(&session));

            let window = WebviewWindowBuilder::new(
                &self.app,
                &label,
                WebviewUrl::App("graphics.html".into()),
            )
            .title("Modo gráfico")
            .inner_size(f64::from(self.size.0), f64::from(self.size.1))
            .visible(false)
            .resizable(false)
            .build();
            let window = match window {
                Ok(window) => window,
                Err(error) => {
                    manager
                        .sessions
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                        .remove(&label);
                    return Err(error.to_string());
                }
            };
            let state = Arc::clone(&session);
            let app = self.app.clone();
            let id = self.id;
            window.on_window_event(move |event| match event {
                WindowEvent::CloseRequested { .. } => {
                    state.close();
                    app.state::<Runs>().with_current(id, |handle| handle.stop());
                }
                WindowEvent::Destroyed => state.close(),
                WindowEvent::Focused(false) => state.input.focus_lost(),
                _ => {}
            });
            self.window = Some((window, session));
        }
        Ok(self.window.as_ref().expect("janela criada"))
    }

    fn frame(&mut self, kind: u8, size: (i32, i32), frame: Vec<u8>) -> Result<Vec<u8>, String> {
        let mut request = Vec::with_capacity(9 + frame.len());
        request.push(kind);
        request.extend_from_slice(&size.0.to_le_bytes());
        request.extend_from_slice(&size.1.to_le_bytes());
        request.extend(frame);
        let session = Arc::clone(&self.ensure_window()?.1);
        session.submit(request, &self.interrupt)
    }
}

impl Host for DesktopHost {
    fn load_sound(&mut self, path: &Path) -> Result<Box<dyn Sound>, String> {
        self.audio.load(path)
    }

    fn select_file(
        &mut self,
        filters: &[FileFilter],
        accept_all: bool,
    ) -> Result<Option<String>, String> {
        Ok(self.observer.select_file(filters, accept_all))
    }

    fn graphics_open(&mut self, width: i32, height: i32) -> Result<(), String> {
        self.size = (width, height);
        let (window, session) = self.ensure_window()?;
        session.input.reopen();
        window.show().map_err(|error| error.to_string())?;
        window.set_focus().map_err(|error| error.to_string())
    }

    fn graphics_close(&mut self) {
        if let Some((window, session)) = self.window.take() {
            session.close();
            self.app
                .state::<GraphicsWindows>()
                .sessions
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .remove(window.label());
            let _ = window.destroy();
        }
        self.windowed_size = None;
    }

    fn graphics_present(&mut self, frame: Vec<u8>) -> Result<(), String> {
        self.frame(1, self.size, frame).map(|_| ())
    }

    fn graphics_snapshot(
        &mut self,
        frame: Vec<u8>,
        width: i32,
        height: i32,
    ) -> Result<Vec<u8>, String> {
        self.frame(2, (width, height), frame)
    }

    fn graphics_window(&mut self, command: WindowCommand) -> Result<(), String> {
        let window = self.ensure_window()?.0.clone();
        let result = match command {
            WindowCommand::Resize { width, height } => {
                if width <= 0 || height <= 0 {
                    return Err("As dimensões da janela precisam ser positivas.".into());
                }
                self.size = (width, height);
                window.set_size(LogicalSize::new(width, height))
            }
            WindowCommand::Title(title) => window.set_title(&title),
            WindowCommand::Minimize => window.minimize(),
            WindowCommand::Restore => window.unminimize(),
            WindowCommand::Decorations(decorations) => window.set_decorations(decorations),
            WindowCommand::FullScreen(fullscreen) => {
                if fullscreen && self.windowed_size.is_none() {
                    let screen = self.screen_size()?;
                    self.windowed_size = Some(self.size);
                    self.size = screen;
                } else if !fullscreen && let Some(size) = self.windowed_size.take() {
                    self.size = size;
                }
                window.set_fullscreen(fullscreen)
            }
            WindowCommand::Icon {
                width,
                height,
                rgba,
            } => window.set_icon(Image::new_owned(rgba, width, height)),
            WindowCommand::CursorVisible(visible) => window.set_cursor_visible(visible),
        };
        result.map_err(|error| error.to_string())
    }

    fn screen_size(&mut self) -> Result<(i32, i32), String> {
        let monitor = self
            .app
            .primary_monitor()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Nenhuma tela encontrada.".to_string())?;
        let size = monitor.size().to_logical::<i32>(monitor.scale_factor());
        Ok((size.width, size.height))
    }

    fn input(&mut self) -> Option<Arc<InputState>> {
        self.window
            .as_ref()
            .map(|(_, session)| Arc::clone(&session.input))
    }
}

impl Drop for DesktopHost {
    fn drop(&mut self) {
        self.graphics_close();
    }
}
