//! Máquina virtual do Ferro.
//!
//! Executa um [`ferro_bytecode::Module`] compilado a partir da HIR tipada
//! com suporte a depuração, bibliotecas padrão, multimídia e UI declarativa MVU.
//!
//! ```no_run
//! use ferro_vm::{BufferIo, run};
//!
//! let hir = ferro_frontend::lower("funcao inicio() { escreva(1 + 1); }").unwrap();
//! let module = ferro_bytecode::compile_ferro(&hir).unwrap();
//! let mut io = BufferIo::default();
//! run(&module, &mut io).unwrap();
//! assert_eq!(io.output, "2");
//! ```

pub mod debug;
mod ferro_libs;
pub mod graphics;
pub mod host;
pub mod io;
#[cfg(feature = "stdlib")]
pub mod stdlib;
pub mod ui;
pub mod value;
pub mod vm;

pub use host::{FileFilter, Host, NoHost, Sound};
pub use io::{BufferIo, Io, StdIo};
pub use value::Value;
pub use vm::{HostNativeHandler, RuntimeError, RuntimeLimitReason, Status, Vm, run};
