#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::ApplicationIdentity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
/// Stable application-local identifier for a native window.
pub struct WindowKey(String);

impl WindowKey {
    /// String value reserved for the primary application window.
    pub const MAIN_VALUE: &'static str = "main";

    /// Creates a key identifying a window within one application.
    /// `value` is the stable key used by the runtime to address the window.
    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    /// Creates the key reserved for the primary window.
    #[must_use]
    pub fn main() -> Self {
        Self::new(Self::MAIN_VALUE)
    }

    /// Returns this key's string value.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Policy applied when a user or platform requests window closure.
pub enum CloseBehavior {
    /// Quit the application when the main window closes.
    #[default]
    Quit,
    /// Close only the requested window.
    CloseWindow,
    /// Hide the window instead of closing it.
    Hide,
    /// Report the close request to the application.
    NotifyApp,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Requested stacking level for a native window.
pub enum WindowLevel {
    /// Keep the window below ordinary windows.
    AlwaysOnBottom,
    /// Use the platform's normal stacking level.
    #[default]
    Normal,
    /// Keep the window above ordinary windows.
    AlwaysOnTop,
}

impl From<WindowLevel> for winit::window::WindowLevel {
    fn from(level: WindowLevel) -> Self {
        match level {
            WindowLevel::AlwaysOnBottom => Self::AlwaysOnBottom,
            WindowLevel::Normal => Self::Normal,
            WindowLevel::AlwaysOnTop => Self::AlwaysOnTop,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Native window backend detected for the current target.
pub enum WindowBackend {
    /// Microsoft Windows backend.
    Windows,
    /// Apple macOS backend.
    MacOs,
    /// Android native activity backend.
    Android,
    /// Apple iOS backend.
    Ios,
    /// X11 display server backend.
    X11,
    /// Wayland display server backend.
    Wayland,
    /// Browser canvas backend.
    Web,
    /// Backend whose capabilities are not specifically known.
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
/// Operations supported by a window backend.
pub struct WindowCapabilities {
    /// Backend on which the window is running.
    pub backend: WindowBackend,
    /// Whether the platform supports moving a window through native drag.
    pub native_drag: bool,
    /// Whether the platform supports a native shadow on undecorated windows.
    pub native_shadow: bool,
    /// Whether the platform supports minimizing windows.
    pub minimize: bool,
    /// Whether the platform supports maximizing windows.
    pub maximize: bool,
    /// Whether the platform supports non-normal window levels.
    pub window_level: bool,
    /// Whether the platform supports pointer passthrough.
    pub mouse_passthrough: bool,
}

impl WindowBackend {
    /// Returns the window operations implemented by this backend.
    #[must_use]
    pub const fn capabilities(self) -> WindowCapabilities {
        let desktop = matches!(
            self,
            Self::Windows | Self::MacOs | Self::X11 | Self::Wayland
        );
        WindowCapabilities {
            backend: self,
            native_drag: desktop,
            native_shadow: matches!(self, Self::Windows | Self::MacOs),
            minimize: desktop,
            maximize: desktop,
            window_level: matches!(self, Self::Windows | Self::MacOs | Self::X11),
            mouse_passthrough: desktop,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Configuration for creating one application window.
pub struct WindowSpec {
    /// Key used to identify this window.
    pub key: WindowKey,
    /// Platform configuration for this window.
    pub window: WindowConfig,
    /// Whether the window should be shown after creation.
    pub visible: bool,
}

impl WindowSpec {
    /// Creates a visible window specification with the supplied key and configuration.
    /// `key` identifies the window and `window` contains its native settings.
    #[must_use]
    pub fn new(key: WindowKey, window: WindowConfig) -> Self {
        Self {
            key,
            window,
            visible: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
/// Portable window settings translated to native window attributes.
pub struct WindowConfig {
    /// Text shown by the native title bar and browser document.
    pub title: String,
    /// Requested native window width in logical pixels.
    ///
    /// A Web canvas takes its size from CSS instead.
    pub width: f64,
    /// Requested native window height in logical pixels.
    ///
    /// A Web canvas takes its size from CSS instead.
    pub height: f64,
    /// Whether the operating system draws its standard title bar and borders.
    pub decorations: bool,
    /// Whether the user can resize a native window.
    pub resizable: bool,
    /// Whether the surface is composited with transparency.
    pub transparent: bool,
    /// Opt in to desktop effects. Regions are supplied by Element::desktop_backdrop.
    pub desktop_backdrop: Option<argui_core::BackdropMaterial>,
    /// Whether an undecorated window requests a platform-drawn shadow.
    ///
    /// This is supported on Windows and macOS. Other platforms ignore it.
    pub native_shadow: bool,
    /// The native stacking level of the window.
    pub level: WindowLevel,
    /// Whether Winit appends a newly created Web canvas to the document body.
    ///
    /// Set this to `false` when the host attaches the canvas itself. Native
    /// targets ignore this setting.
    pub append_to_document: bool,
    /// ID of the Web element that receives the canvas after it is created.
    ///
    /// `None` leaves the canvas under the parent selected by Winit. Native
    /// targets ignore this setting.
    pub web_parent_id: Option<String>,
    /// Action taken when the user asks the operating system to close the window.
    pub close_behavior: CloseBehavior,
    /// Pointer timing and gesture thresholds used by this window.
    pub pointer: argui_core::PointerSettings,
    /// Whether the window or Web canvas requests keyboard focus when launched.
    ///
    /// The default is `true` on native targets. It is `false` on WebAssembly so
    /// an embedded canvas cannot steal focus and scroll its host page while it
    /// loads. Pointer interaction still focuses a Web canvas. A full-page Web
    /// application can opt in with [`WindowConfig::with_focus_on_launch`].
    pub focus_on_launch: bool,
    /// Explicit safe-area override in logical pixels. `None` uses native
    /// insets where Winit exposes them and zero elsewhere.
    pub safe_area_insets: Option<argui_core::Insets>,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            title: "Argui".into(),
            width: 960.0,
            height: 640.0,
            decorations: true,
            resizable: true,
            transparent: false,
            desktop_backdrop: None,
            native_shadow: false,
            level: WindowLevel::Normal,
            append_to_document: true,
            web_parent_id: None,
            close_behavior: CloseBehavior::Quit,
            pointer: argui_core::PointerSettings::default(),
            focus_on_launch: cfg!(not(target_arch = "wasm32")),
            safe_area_insets: None,
        }
    }
}

impl WindowConfig {
    /// Overrides the safe-area insets reported to the root view.
    /// `insets` is the explicit inset value in logical pixels.
    #[must_use]
    pub fn with_safe_area_insets(mut self, insets: argui_core::Insets) -> Self {
        self.safe_area_insets = Some(insets);
        self
    }

    /// Chooses whether this window requests keyboard focus as it launches.
    ///
    /// Web applications embedded in another page should leave this disabled to
    /// preserve the page's focus and scroll position. Full-page applications
    /// may enable it when keyboard input should be ready immediately.
    /// `focus` selects whether keyboard focus is requested at launch.
    #[must_use]
    pub fn with_focus_on_launch(mut self, focus: bool) -> Self {
        self.focus_on_launch = focus;
        self
    }

    /// Converts this portable configuration into Winit window attributes.
    #[must_use]
    pub fn into_attributes(self) -> WindowAttributes {
        let attributes = WindowAttributes::default()
            .with_title(self.title)
            .with_decorations(self.decorations)
            .with_resizable(self.resizable)
            .with_transparent(self.transparent || self.desktop_backdrop.is_some())
            .with_window_level(self.level.into())
            .with_active(self.focus_on_launch);

        #[cfg(target_os = "windows")]
        let attributes = {
            use winit::platform::windows::WindowAttributesExtWindows;

            attributes
                .with_undecorated_shadow(self.native_shadow)
                .with_no_redirection_bitmap(self.transparent || self.desktop_backdrop.is_some())
        };

        #[cfg(target_os = "macos")]
        let attributes = {
            use winit::platform::macos::WindowAttributesExtMacOS;

            if self.decorations {
                attributes
            } else {
                attributes.with_has_shadow(self.native_shadow)
            }
        };

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowAttributesExtWebSys;

            attributes.with_append(self.append_to_document)
        }

        #[cfg(not(target_arch = "wasm32"))]
        attributes.with_inner_size(LogicalSize::new(self.width, self.height))
    }

    /// Converts this configuration and applies the shared application identity.
    /// `identity` supplies the fallback title, application IDs, and icon.
    #[must_use]
    pub fn into_attributes_with_identity(self, identity: &ApplicationIdentity) -> WindowAttributes {
        let title = if self.title.is_empty() {
            identity.display_name.clone()
        } else {
            self.title.clone()
        };
        let mut attributes = self.into_attributes().with_title(&title);
        if let Some(icon) = identity.icons.best_square(32)
            && let Ok(icon) =
                winit::window::Icon::from_rgba(icon.rgba8.to_vec(), icon.width, icon.height)
        {
            attributes = attributes.with_window_icon(Some(icon));
        }

        #[cfg(target_os = "linux")]
        {
            use winit::platform::{
                wayland::WindowAttributesExtWayland, x11::WindowAttributesExtX11,
            };
            attributes = WindowAttributesExtWayland::with_name(
                attributes,
                identity.linux_application_id().to_owned(),
                title.clone(),
            );
            attributes = WindowAttributesExtX11::with_name(
                attributes,
                identity.linux_application_id().to_owned(),
                title,
            );
        }
        attributes
    }
}

/// Detects the native operations supported by `window`'s backend.
///
/// The result describes platform support, not current window state.
/// `window` is the native window whose backend is inspected.
#[must_use]
#[cfg_attr(coverage_nightly, coverage(off))]
pub fn window_capabilities(window: &Window) -> WindowCapabilities {
    window_backend(window).capabilities()
}

#[cfg(target_arch = "wasm32")]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Web
}

#[cfg(target_os = "windows")]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Windows
}

#[cfg(target_os = "macos")]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::MacOs
}

#[cfg(target_os = "android")]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Android
}

#[cfg(target_os = "ios")]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Ios
}

#[cfg(target_os = "linux")]
fn window_backend(window: &Window) -> WindowBackend {
    use winit::raw_window_handle::{HasWindowHandle, RawWindowHandle};

    match window.window_handle().map(|handle| handle.as_raw()) {
        Ok(RawWindowHandle::Xlib(_) | RawWindowHandle::Xcb(_)) => WindowBackend::X11,
        Ok(RawWindowHandle::Wayland(_)) => WindowBackend::Wayland,
        _ => WindowBackend::Other,
    }
}

#[cfg(not(any(
    target_arch = "wasm32",
    target_os = "windows",
    target_os = "macos",
    target_os = "android",
    target_os = "ios",
    target_os = "linux"
)))]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Other
}
