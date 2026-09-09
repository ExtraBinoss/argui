//! Renderer-independent accessibility semantics and incremental tree updates.

mod schema;
mod tree;

#[cfg(not(target_arch = "wasm32"))]
mod native;

#[cfg(not(target_arch = "wasm32"))]
pub use native::AccessKitTree;
pub use schema::{
    CheckedState, FocusPolicy, LiveRegion, Orientation, Role, SemanticAction, SemanticRequest,
    SemanticState, SemanticValue, Semantics,
};
pub use tree::{SemanticNode, SemanticNodeId, SemanticPatch, SemanticTree};

#[cfg(target_arch = "wasm32")]
mod web;
#[cfg(target_arch = "wasm32")]
pub use web::DomTree;

mod relations;
pub use relations::{GridPosition, PopupKind, SemanticRelations, SortDirection};
