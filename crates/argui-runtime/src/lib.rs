#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod app;
mod error;
mod event;

pub use app::{run, run_ui, run_ui_with_text_engine, run_with_text, run_with_text_engine};
pub use error::RuntimeError;
pub use event::RuntimeEvent;
