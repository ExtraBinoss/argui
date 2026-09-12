use argui_core::Rect;

use crate::{NodeId, OverlaySurface, UiTree};

impl UiTree {
    /// The requested policy is independent of whether the current host supports native popups.
    #[must_use]
    pub fn portal_surface_preference(&self, node: NodeId) -> OverlaySurface {
        let mut cursor = self.node_ids.iter().position(|id| *id == node);
        while let Some(index) = cursor {
            if let Some(surface) = self
                .element_at(index)
                .and_then(|element| element.portal.as_ref())
                .and_then(|portal| portal.surface)
            {
                return surface;
            }
            cursor = self.events.parent(index);
        }
        OverlaySurface::InWindow
    }

    /// Install geometry accepted by the native host, in the owning tree's logical coordinates.
    pub fn set_native_portal(&mut self, node: NodeId, bounds: Option<Rect>) -> bool {
        self.native_portals
            .retain(|id, _| self.node_ids.contains(id));
        let bounds = bounds.filter(|bounds| {
            self.element_for(node)
                .is_some_and(|element| element.portal.is_some())
                && bounds.origin.x.is_finite()
                && bounds.origin.y.is_finite()
                && bounds.size.width.is_finite()
                && bounds.size.height.is_finite()
                && bounds.size.width > 0.0
                && bounds.size.height > 0.0
        });
        if self.native_portals.get(&node).copied() == bounds {
            return false;
        }
        match bounds {
            Some(bounds) => {
                self.native_portals.insert(node, bounds);
            }
            None => {
                self.native_portals.remove(&node);
            }
        }
        self.layout_dirty = true;
        true
    }

    #[must_use]
    pub fn native_portal_bounds(&self, node: NodeId) -> Option<Rect> {
        self.native_portals.get(&node).copied()
    }

    /// The physical surface containing this element; None is the application's window.
    #[must_use]
    pub fn native_portal_owner(&self, node: NodeId) -> Option<NodeId> {
        let mut cursor = self.node_ids.iter().position(|id| *id == node);
        while let Some(index) = cursor {
            let node = self.node_ids[index];
            if self.native_portals.contains_key(&node) {
                return Some(node);
            }
            cursor = self.events.parent(index);
        }
        None
    }
}
