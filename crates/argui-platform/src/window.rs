#[cfg(not(target_arch = "wasm32"))]
use winit::dpi::LogicalSize;
use winit::window::WindowAttributes;

#[derive(Clone, Debug, PartialEq)]
pub struct WindowConfig {
    pub title: String,
    pub width: f64,
    pub height: f64,
    pub decorations: bool,
    pub resizable: bool,
    pub transparent: bool,
    pub append_to_document: bool,
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
}
