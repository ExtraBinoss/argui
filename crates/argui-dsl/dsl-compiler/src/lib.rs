//! Incremental Argui DSL compiler with reachability and Rust AOT emission.

mod codegen;
mod error;
mod project;
mod reachability;

pub use error::CompilerError;
pub use project::{CompiledProject, Compiler, CompilerSession, SourceModule};
pub use reachability::Reachability;
