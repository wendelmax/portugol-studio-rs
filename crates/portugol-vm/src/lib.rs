//! Máquina virtual do Portugol-RS.
//!
//! Executa um [`portugol_bytecode::Module`] compilado a partir da HIR tipada
//! com suporte a depuração, bibliotecas padrão, multimídia e UI declarativa MVU.
//!
//! ```no_run
//! use portugol_vm::{BufferIo, run};
//!
//! let hir = portugol_rs_frontend::lower("funcao inicio() { escreva(1 + 1); }").unwrap();
//! let module = portugol_bytecode::compile_pors(&hir).unwrap();
//! let mut io = BufferIo::default();
//! run(&module, &mut io).unwrap();
//! assert_eq!(io.output, "2");
//! ```

pub mod debug;
pub mod graphics;
pub mod host;
pub mod io;
mod pors_libs;
#[cfg(feature = "stdlib")]
pub mod stdlib;
pub mod ui;
pub mod value;
pub mod vm;

pub use host::{FileFilter, Host, NoHost, Sound};
pub use io::{BufferIo, Io, StdIo};
pub use value::Value;
pub use vm::{HostNativeHandler, RuntimeError, RuntimeLimitReason, Status, Vm, run};
