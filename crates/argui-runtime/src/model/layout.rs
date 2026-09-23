use argui_core::{Rect, Size};
use argui_ui::{NodeId, RetainedIdentity};

pub use argui_ui::ScrollRequest;

#[derive(Clone, Debug, PartialEq)]
pub struct LayoutBounds {
    /// Identifier of the UI node represented by these bounds.
    pub node: NodeId,
    /// Optional application key assigned to the element.
    pub key: Option<String>,
    /// Stable producer identity, independent of the public application key.
    #[doc(hidden)]
    pub retained_identity: Option<RetainedIdentity>,
    /// Bounds in logical window coordinates.
    pub bounds: Rect,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LayoutSnapshot {
    /// Bounds of the logical viewport.
    pub viewport: Rect,
    /// Bounds for the nodes included in the layout snapshot.
    pub nodes: Vec<LayoutBounds>,
}

impl LayoutSnapshot {
    /// Finds the bounds for an element with the given application key.
    ///
    /// `key` is the key assigned to the element. Returns `None` if no node has it.
    #[must_use]
    pub fn bounds(&self, key: &str) -> Option<Rect> {
        self.nodes
            .iter()
            .find(|node| node.key.as_deref() == Some(key))
            .map(|node| node.bounds)
    }

    /// Returns the logical size of the viewport.
    #[must_use]
    pub const fn viewport_size(&self) -> Size {
        self.viewport.size
    }
}
