//! Development-only live runtime for typed, pre-resolved Argui DSL packages.

mod bytecode;
#[cfg(not(target_arch = "wasm32"))]
mod client;
mod error;
mod event;
mod inspection;
mod instance;
mod live_value;
mod native;
mod observation;
mod package;
mod render;
mod runtime;
mod transport;
mod value;
#[cfg(target_arch = "wasm32")]
mod web_client;

pub use argui_dsl_ir as ir;
pub use bytecode::{EvaluationContext, Instruction, Program};
#[cfg(not(target_arch = "wasm32"))]
pub use client::LiveClient;
pub use error::RuntimeError;
pub use inspection::{InstanceInspection, RuntimeInspection};
pub use instance::{ComponentInstance, DynamicProperty, InstanceId};
pub use live_value::LiveValue;
pub use package::{AssetPayload, LivePackage};
pub use runtime::{LiveRuntime, PreparedReload, ReloadOutcome};
pub use transport::ClientEvent;
pub use value::DslValue;
