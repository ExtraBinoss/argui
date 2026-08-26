#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod app;
mod error;
mod event;

pub use app::run;
pub use error::RuntimeError;
pub use event::RuntimeEvent;
