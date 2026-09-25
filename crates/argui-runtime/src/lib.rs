#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Composition root connecting platform events to rendering and, later, the UI tree.

mod animation;
mod app;
mod application;
mod clipboard;
mod effect;
mod environment;
mod error;
mod event;
mod host;
mod input;
mod launch;
pub mod mobile;
mod model;
mod multi;
mod native_host;
#[cfg(feature = "tasks")]
pub mod tasks;
mod theme_bridge;
mod translate;

#[cfg(feature = "inspect")]
pub use app::{Inspection, InspectionCache};
pub use application::{
    AppCommand, AppEvent, AppModel, AppUpdate, SingleWindowModel, WindowInvalidation,
};
pub use argui_core::Color;
pub use argui_host::{
    CallbackDelivery, CallbackId, CommitResult as HostCommitResult, Host as NativeHost, HostError,
    HostId, Operation as HostOperation,
};
pub use argui_platform::{ApplicationConfig, ApplicationIdentity, WindowConfig};
pub use argui_render::RendererConfig;
pub use argui_theme::{
    ThemeChange, ThemeDimension, ThemeError, ThemeImpact, ThemeMode, ThemeRuntime, ThemeSchema,
    ThemeSnapshot, ThemeTokenDefinition, ThemeTokenId, ThemeValue, ThemeValueType,
};
pub use argui_ui::{Element, EventType, UiTree};
pub use effect::{VisualEffectTarget, apply_visual_effect, apply_visual_effect_scoped};
pub use environment::{ThemeRequest, WindowEnvironment};
pub use error::RuntimeError;
pub use event::{AnimationProfile, RuntimeEvent, WindowRuntimeEvent};
#[cfg(target_arch = "wasm32")]
pub use launch::WebHostHandle;
#[cfg(not(target_arch = "wasm32"))]
pub use launch::run_native_host;
#[cfg(not(target_arch = "wasm32"))]
pub use launch::{NativeHostApplicationChannels, run_native_host_application};
pub use launch::{
    run, run_app, run_app_with_text_engine, run_application, run_application_with_text_engine,
    run_ui, run_ui_with_text_engine, run_with_text, run_with_text_engine,
};
#[cfg(all(feature = "android", target_os = "android"))]
#[doc(hidden)]
pub use launch::{
    run_android_application, run_android_application_with_text_engine,
    run_android_native_host_with_text_engine,
};
pub use model::{
    AnyEntity, Context, Entity, EntityId, EventEmitter, EventError, LayoutBounds, LayoutSnapshot,
    ModelContext, ModelRuntime, Mount, MountEvent, MountId, MountTransition, ObservationReader,
    ObservedInteraction, ObservedScroll, Render, ResourceLease, ResourceScope, ScopeClosed,
    ScrollRequest, ServiceAlreadyRegistered, ServiceRegistration, SourceIdentityIndex,
    Subscription, ViewUpdate, WeakEntity, WeakMount,
};
pub use native_host::{
    NativeHostApplicationRequest, NativeHostAssets, NativeHostBatch, NativeHostCommit,
    NativeHostControl, NativeHostDelivery, NativePointerPosition, NativeWindowInfo, WireHostId,
    WireOperation, WireValue, validate_native_host_assets, validate_native_host_canvases,
};
pub use theme_bridge::ThemeBridge;

pub use model::shutdown_presentations;
