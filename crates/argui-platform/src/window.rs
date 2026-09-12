#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
use winit::window::Window;
use winit::window::WindowAttributes;

use crate::ApplicationIdentity;

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct WindowKey(String);

impl WindowKey {
    pub const MAIN_VALUE: &'static str = "main";

    #[must_use]
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    #[must_use]
    pub fn main() -> Self {
        Self::new(Self::MAIN_VALUE)
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum CloseBehavior {
    #[default]
    Quit,
    CloseWindow,
    Hide,
    NotifyApp,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum WindowLevel {
    AlwaysOnBottom,
    #[default]
    Normal,
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
pub enum WindowBackend {
    Windows,
    MacOs,
    X11,
    Wayland,
    Web,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct WindowCapabilities {
    pub backend: WindowBackend,
    pub native_drag: bool,
    pub native_shadow: bool,
    pub minimize: bool,
    pub maximize: bool,
    pub window_level: bool,
    pub mouse_passthrough: bool,
}

impl WindowBackend {
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
pub struct WindowSpec {
    pub key: WindowKey,
    pub window: WindowConfig,
    pub visible: bool,
}

impl WindowSpec {
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
pub struct WindowConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub decorations: bool,
    pub resizable: bool,
    pub transparent: bool,
    /// Opt in to desktop effects. Regions are supplied by Element::desktop_backdrop.
    pub desktop_backdrop: Option<argui_core::BackdropMaterial>,
    pub native_shadow: bool,
    pub level: WindowLevel,
    pub append_to_document: bool,
    pub web_parent_id: Option<String>,
    pub close_behavior: CloseBehavior,
    pub pointer: argui_core::PointerSettings,
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
        }
    }
}

impl WindowConfig {
    #[must_use]
    pub fn into_attributes(self) -> WindowAttributes {
        let attributes = WindowAttributes::default()
            .with_title(self.title)
            .with_decorations(self.decorations)
            .with_resizable(self.resizable)
            .with_transparent(self.transparent || self.desktop_backdrop.is_some())
            .with_window_level(self.level.into());

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
    target_os = "linux"
)))]
fn window_backend(_window: &Window) -> WindowBackend {
    WindowBackend::Other
}
