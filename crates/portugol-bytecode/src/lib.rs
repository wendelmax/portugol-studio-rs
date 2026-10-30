//! Bytecode do Portugol 2.0: conjunto de instruções ([`op`]) e compilador a
//! partir da AST ([`compiler`]).
//!
//! ```no_run
//! let fonte = "programa { funcao inicio() { escreva(\"oi\") } }";
//! let ast = portugol_syntax::parse(fonte).program;
//! let modulo = portugol_bytecode::compile(&ast).unwrap();
//! assert!(modulo.function("inicio").is_some());
//! ```

pub mod compiler;
pub mod op;

pub use compiler::{CompileError, ENTRY_NAME, compile};
pub use op::{Function, GlobalInfo, LocalInfo, Module, NativeRef, Op, ValueType};
