//! Entrada e saída do programa, abstraídas para a VM rodar tanto no terminal
//! quanto na IDE (ou em testes, com [`BufferIo`]).

use std::collections::VecDeque;
use std::io::{BufRead, Write};

pub trait Io {
    fn write(&mut self, text: &str);
    /// Uma linha digitada, sem o terminador. `None` quando a entrada acabou.
    fn read_line(&mut self) -> Option<String>;
    fn clear(&mut self) {}
}

/// Terminal: stdout/stdin do processo.
#[derive(Debug, Default)]
pub struct StdIo;

impl Io for StdIo {
    fn write(&mut self, text: &str) {
        let mut out = std::io::stdout().lock();
        let _ = out.write_all(text.as_bytes());
        let _ = out.flush();
    }

    fn read_line(&mut self) -> Option<String> {
        let mut line = String::new();
        match std::io::stdin().lock().read_line(&mut line) {
            Ok(0) | Err(_) => None,
            Ok(_) => Some(strip_newline(line)),
        }
    }

    fn clear(&mut self) {
        self.write("\x1b[2J\x1b[H");
    }
}

/// Entrada pré-definida e saída acumulada em memória.
#[derive(Debug, Default)]
pub struct BufferIo {
    pub input: VecDeque<String>,
    pub output: String,
}

impl BufferIo {
    pub fn new(input: &str) -> Self {
        BufferIo {
            input: input.lines().map(str::to_string).collect(),
            output: String::new(),
        }
    }
}

impl Io for BufferIo {
    fn write(&mut self, text: &str) {
        self.output.push_str(text);
    }

    fn read_line(&mut self) -> Option<String> {
        self.input.pop_front()
    }

    fn clear(&mut self) {
        self.output.clear();
    }
}

fn strip_newline(mut line: String) -> String {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
    line
}
