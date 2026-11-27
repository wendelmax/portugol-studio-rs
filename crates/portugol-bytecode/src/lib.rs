//! Bytecode do Portugol-RS: conjunto de instruções ([`op`]) e compilador a
//! partir da HIR tipada ([`pors`]).

pub mod op;
pub mod pors;

pub use op::{
    Function, GlobalInfo, InstructionMapping, LocalInfo, Module, NativeRef, Op, PorsDebugField,
    PorsDebugShape, PorsDebugType, PorsDebugVariant, ValueType,
};
pub use pors::{PorsCompileError, compile_pors};
