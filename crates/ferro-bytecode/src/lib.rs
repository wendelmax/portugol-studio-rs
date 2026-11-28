//! Bytecode do Ferro: conjunto de instruções ([`op`]) e compilador a
//! partir da HIR tipada ([`ferro`]).

pub mod ferro;
pub mod op;

pub use ferro::{FerroCompileError, compile_ferro};
pub use op::{
    FerroDebugField, FerroDebugShape, FerroDebugType, FerroDebugVariant, Function, GlobalInfo,
    InstructionMapping, LocalInfo, Module, NativeRef, Op, ValueType,
};
