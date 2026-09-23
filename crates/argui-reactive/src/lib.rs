//! Typed, language-independent reactive properties for Argui.
//!
//! The graph is deliberately single-threaded: UI properties are mutated on the
//! owning application thread. This crate defines typed properties, computed
//! values, transactions,
//! observers, and two-way links.

mod computed;
mod error;
mod graph;
mod link;
mod model;
mod property;
mod retained;
mod subscription;
mod transaction;

pub use computed::Computed;
pub use error::ReactiveError;
pub use link::TwoWayLink;
pub use model::{Model, ModelChange};
pub use property::Property;
pub use retained::RetainedPropertyStore;
pub use subscription::Subscription;
pub use transaction::transaction;
