#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
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
    pub append_to_document: bool,
    pub web_parent_id: Option<String>,
    pub close_behavior: CloseBehavior,
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
            append_to_document: true,
            web_parent_id: None,
            close_behavior: CloseBehavior::Quit,
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
            .with_transparent(self.transparent);

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
