#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod animation;
mod app;
mod application;
mod clipboard;
mod environment;
mod error;
mod event;
mod host;
mod input;
mod launch;
mod model;
mod multi;
mod translate;

pub use app::{Inspection, InspectionCache};
pub use application::{
    AppCommand, AppEvent, AppModel, AppUpdate, SingleWindowModel, WindowInvalidation,
};
pub use environment::{ThemeRequest, WindowEnvironment};
pub use error::RuntimeError;
pub use event::{AnimationProfile, RuntimeEvent, WindowRuntimeEvent};
pub use launch::{
    run, run_app, run_app_with_text_engine, run_application, run_application_with_text_engine,
    run_ui, run_ui_with_text_engine, run_with_text, run_with_text_engine,
};
pub use model::{
    AnyEntity, Context, Entity, LayoutBounds, LayoutSnapshot, Render, ScrollRequest, ViewUpdate,
    WeakEntity,
};
