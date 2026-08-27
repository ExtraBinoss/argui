use argui_animation::Frame;
use argui_core::{Point, Rect, Size};
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_render::EffectShader;
use argui_ui::{ClipboardRequest, Element, NodeId, UiEvent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewUpdate {
    #[default]
    None,
    Rebuild,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBounds {
    pub node: NodeId,
    pub key: Option<String>,
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutSnapshot {
    pub viewport: Rect,
    pub nodes: Vec<LayoutBounds>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollRequest {
    pub key: String,
    pub offset: Point,
}

impl LayoutSnapshot {
    #[must_use]
    pub fn bounds(&self, key: &str) -> Option<Rect> {
        self.nodes
            .iter()
            .find(|node| node.key.as_deref() == Some(key))
            .map(|node| node.bounds)
    }

    #[must_use]
    pub const fn viewport_size(&self) -> Size {
        self.viewport.size
    }
}

pub trait UiApp: 'static {
    fn view(&self) -> Element;

    fn update(&mut self, event: &UiEvent) -> ViewUpdate;

    fn animation_frame(&mut self, _frame: Frame) -> ViewUpdate {
        ViewUpdate::None
    }

    fn wants_animation_frame(&self) -> bool {
        false
    }

    /// Receives stable logical-pixel bounds after layout. Returning `Rebuild`
    /// performs one bounded second layout pass, suitable for overlay placement.
    fn layout_changed(&mut self, _layout: &LayoutSnapshot) -> ViewUpdate {
        ViewUpdate::None
    }

    fn effect_shaders(&self) -> &'static [EffectShader] {
        &[]
    }

    /// Returns immutable assets uploaded once when the renderer starts.
    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    /// Returns immutable vector meshes uploaded once when the renderer starts.
    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    fn inspector(&self) -> Option<InspectorHandle> {
        None
    }

    /// Returns one application-originated clipboard operation, if pending.
    fn take_clipboard_request(&mut self) -> Option<ClipboardRequest> {
        None
    }

    /// Returns one application-originated programmatic scroll, if pending.
    fn take_scroll_request(&mut self) -> Option<ScrollRequest> {
        None
    }
}
