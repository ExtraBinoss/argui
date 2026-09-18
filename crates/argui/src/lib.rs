//! The small public facade for Argui.

/// Common application-author imports.
///
/// The prelude intentionally keeps widgets behind the [`widgets`] module so
/// feature-gated widget names cannot collide with application model types.
/// Library crates may prefer explicit imports from the facade modules.
pub mod prelude {
    pub use crate::platform::{ApplicationConfig, UiZoomConfig, WindowConfig};
    pub use crate::runtime::{Context, Render, WindowEnvironment, run_app};
    pub use crate::theme::{Theme, ThemeMode};
    pub use crate::ui::{Element, EventHandler, TextEdit, ValueHandler, length, percent, sides};
    #[cfg(feature = "argui-widgets")]
    pub use crate::widgets::{self, WidgetTheme, default_theme};
}

pub use argui_accessibility as accessibility;
pub use argui_animation as animation;
pub use argui_core as core;
#[cfg(feature = "devtools")]
pub use argui_devtools as devtools;
#[cfg(feature = "i18n")]
pub use argui_i18n as i18n;
pub use argui_layout as layout;
pub use argui_paint as paint;
pub use argui_platform as platform;
pub use argui_render as render;
pub use argui_runtime as runtime;
pub use argui_text as text;
pub use argui_theme as theme;
pub use argui_ui as ui;
#[cfg(feature = "updater")]
pub use argui_updater as updater;
pub use argui_vector as vector;
#[cfg(feature = "webview")]
pub use argui_webview as webview;
#[cfg(feature = "argui-widgets")]
pub use argui_widgets as widgets;
