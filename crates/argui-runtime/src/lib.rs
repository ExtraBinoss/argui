#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod app;
mod error;
mod event;
mod launch;
mod model;

pub use error::RuntimeError;
pub use event::RuntimeEvent;
pub use launch::{
    run, run_app, run_app_with_text_engine, run_ui, run_ui_with_text_engine, run_with_text,
    run_with_text_engine,
};
pub use model::{UiApp, ViewUpdate};
