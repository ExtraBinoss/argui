//! External incremental compiler and transport host used by `argui dev`.

mod change;
mod commands;
mod project;
mod scaffold;
mod service;
mod transport;

pub use change::{ChangeTracker, SourceChange};
pub use commands::{complete, format, schema, symbols};
pub use project::{ProjectFiles, canonical_relative};
pub use scaffold::new_project;
pub use service::{
    CompileAttempt, DevCompilerService, ServiceError, is_relevant_event, is_relevant_path,
};
pub use transport::WebSocketHub;
#[cfg(not(target_arch = "wasm32"))]
pub mod dashboard;
