#![cfg_attr(coverage_nightly, feature(coverage_attribute))]
//! Native and web window/event integration.

mod application;
mod clipboard;
#[cfg(all(feature = "desktop-backdrop", not(target_arch = "wasm32")))]
pub mod desktop_backdrop;
mod error;
mod event;
#[cfg(feature = "file-picker")]
pub mod file_picker;
#[cfg(all(feature = "gtk-host", target_os = "linux"))]
pub mod gtk_host;
mod identity;
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
mod native_tray;
#[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
pub mod popup;
mod preferences;
mod tray;
#[cfg(target_arch = "wasm32")]
mod web_identity;
mod window;

pub use application::{ApplicationConfig, ApplicationConfigError};
pub use argui_core::{
    Insets, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase, PointerSettings,
};
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
pub use window::{
    CloseBehavior, WindowBackend, WindowCapabilities, WindowConfig, WindowKey, WindowLevel,
    WindowSpec, window_capabilities,
};
