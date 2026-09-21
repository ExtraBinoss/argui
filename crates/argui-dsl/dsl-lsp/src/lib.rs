//! Thin JSON-RPC/LSP adapter over `argui-dsl-semantic` queries.

mod codec;
mod convert;
mod protocol;
mod server;
mod workspace;

pub use codec::{MessageReader, write_message};
pub use server::{LanguageServer, ServerError, run_stdio};
