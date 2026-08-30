#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Native and web window/event integration.

mod application;
mod clipboard;
mod error;
mod event;
mod identity;
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
mod native_tray;
mod preferences;
mod tray;
#[cfg(target_arch = "wasm32")]
mod web_identity;
mod window;

pub use application::{ApplicationConfig, ApplicationConfigError};
pub use argui_core::{PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase};
pub use clipboard::{Clipboard, ClipboardError};
pub use error::PlatformError;
pub use event::{
    ButtonState, ImeInput, Key, KeyInput, KeyState, Modifiers, PlatformEvent, ScrollDelta,
};
pub use identity::{
    AppIcon, AppIconError, ApplicationId, ApplicationIdError, ApplicationIdentity, IconSet,
};
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
pub use native_tray::{NativeTray, TrayEventHandler};
pub use preferences::{
    PreferenceOverrides, PreferenceSource, ResolvedPreference, SystemPreferences,
};
pub use tray::{
    TrayAction, TrayConfig, TrayConfigError, TrayEvent, TrayItemId, TrayMenuItem, TrayPointerButton,
};
#[cfg(target_arch = "wasm32")]
pub use web_identity::{apply_web_identity, attach_web_canvas};
pub use window::{CloseBehavior, WindowConfig, WindowKey, WindowSpec};
