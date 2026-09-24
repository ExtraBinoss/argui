//! Virtual-list metadata on retained elements.

use super::Element;

impl Element {
    /// Returns this element's virtual-list item metadata, if present.
    #[must_use]
    pub fn virtual_item(&self) -> Option<&crate::VirtualItem> {
        self.virtual_item.as_ref()
    }

    /// Returns native virtual-window metadata on a scroll container, if present.
    #[must_use]
    pub fn virtual_viewport(&self) -> Option<&crate::VirtualViewport> {
        self.virtual_viewport.as_ref()
    }
}
