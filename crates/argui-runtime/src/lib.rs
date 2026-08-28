#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod animation;
mod app;
mod application;
mod clipboard;
mod error;
mod event;
mod input;
mod launch;
mod model;
mod multi;
mod translate;

pub use application::{AppCommand, AppEvent, AppModel, AppUpdate, WindowInvalidation};
pub use argui_render::EffectShader;
pub use error::RuntimeError;
pub use event::{AnimationProfile, RuntimeEvent, WindowRuntimeEvent};
pub use launch::{
    run, run_app, run_app_with_text_engine, run_application, run_application_with_text_engine,
    run_ui, run_ui_with_text_engine, run_with_text, run_with_text_engine,
};
pub use model::{LayoutBounds, LayoutSnapshot, ScrollRequest, UiApp, ViewUpdate};
