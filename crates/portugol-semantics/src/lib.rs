//! Análise semântica do Portugol 2.0.
//!
//! O ponto de entrada é [`analyze`], que recebe uma AST já produzida por
//! `portugol-syntax` e devolve diagnósticos `sem.*`.
//!
//! ```no_run
//! let fonte = "programa { funcao inicio() { inteiro x = 1 } }";
//! let resultado = portugol_syntax::parse(fonte);
//! assert!(portugol_semantics::analyze(&resultado.program).is_empty());
//! ```

pub mod analyzer;
pub mod symbols;
pub mod types;

pub use analyzer::analyze;
