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
#[cfg(all(feature = "hot-reload", debug_assertions, not(target_arch = "wasm32")))]
mod hot_reload;
mod input;
mod launch;
mod model;
mod multi;
#[cfg(feature = "tasks")]
pub mod tasks;
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
#[cfg(target_os = "android")]
#[doc(hidden)]
pub use launch::{run_android_application, run_android_application_with_text_engine};
pub use model::{
    AnyEntity, Context, Entity, EntityId, EventEmitter, EventError, LayoutBounds, LayoutSnapshot,
    ModelContext, ModelRuntime, Mount, MountEvent, MountId, MountTransition, Render, ResourceLease,
    ResourceScope, ScopeClosed, ScrollRequest, ServiceAlreadyRegistered, ServiceRegistration,
    Subscription, ViewUpdate, WeakEntity, WeakMount,
};

pub use model::shutdown_presentations;
