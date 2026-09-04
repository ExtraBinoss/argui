use argui_core::{Rect, Size};
use argui_ui::NodeId;

pub use argui_ui::ScrollRequest;

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
