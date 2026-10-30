//! Máquina virtual do Portugol 2.0.
//!
//! Executa um [`portugol_bytecode::Module`] com a semântica do Java gerado
//! pelo Portugol Studio original (`int` de 32 bits, `double`,
//! `Double.toString`, divisão inteira por zero como erro etc.).
//!
//! ```no_run
//! use portugol_vm::{BufferIo, run};
//!
//! let ast = portugol_syntax::parse("programa { funcao inicio() { escreva(1 + 1) } }").program;
//! let module = portugol_bytecode::compile(&ast).unwrap();
//! let mut io = BufferIo::default();
//! run(&module, &mut io).unwrap();
//! assert_eq!(io.output, "2");
//! ```

pub mod host;
pub mod io;
pub mod stdlib;
pub mod value;
pub mod vm;

pub use host::{FileFilter, Host, NoHost};
pub use io::{BufferIo, Io, StdIo};
pub use value::Value;
pub use vm::{RuntimeError, Status, Vm, run};
