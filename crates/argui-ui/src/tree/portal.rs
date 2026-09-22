use argui_core::Rect;

use crate::{InteractionUpdate, NodeId, OverlaySurface, UiEventKind, UiTree};

impl UiTree {
    /// Requests dismissal when the topmost portal closes on Escape.
    ///
    /// Returns that portal's dismiss deliveries, or an empty update when the
    /// topmost portal is manual or no portal is mounted. Hosts call this after
    /// cancelable key delivery.
    pub(crate) fn dismiss_portal_on_escape(&mut self) -> InteractionUpdate {
        let target = crate::traversal::flattened(self.root())
            .into_iter()
            .enumerate()
            .filter_map(|(index, element)| {
                let portal = element.portal.as_ref()?;
                Some((
                    portal.layer,
                    element.z_index,
                    index,
                    self.node_ids[index],
                    portal.dismiss,
                ))
            })
            .max_by_key(|(layer, z_index, index, _, _)| (*layer, *z_index, *index))
            .filter(|(_, _, _, _, policy)| {
                matches!(
                    policy,
                    crate::DismissPolicy::Escape
                        | crate::DismissPolicy::OutsidePointerOrEscape
                        | crate::DismissPolicy::OutsideHoverOrEscape
                )
            })
            .map(|(_, _, _, node, _)| node);
        target.map_or_else(InteractionUpdate::default, |node| InteractionUpdate {
            events: self.event_deliveries(node, UiEventKind::DismissRequested),
            ..InteractionUpdate::default()
        })
    }

    /// The requested policy is independent of whether the current host supports native popups.
    ///
    /// * `node` — portal node whose requested surface is queried.
    ///
    /// Returns the requested surface, defaulting to the in-window surface.
    #[must_use]
    pub fn portal_surface_preference(&self, node: NodeId) -> OverlaySurface {
        let mut cursor = self.index.position(node);
        while let Some(index) = cursor {
            if let Some(surface) = self
                .element_at(index)
                .and_then(|element| element.portal.as_ref())
                .and_then(|portal| portal.surface)
            {
                return surface;
            }
            cursor = self.index.parent(index);
        }
        OverlaySurface::InWindow
    }

    /// Install geometry accepted by the native host, in the owning tree's logical coordinates.
    ///
    /// * `node` — portal node receiving native geometry.
    /// * `bounds` — logical bounds accepted by the host, or `None` to clear them.
    ///
    /// Returns whether the stored geometry changed; invalid or non-portal bounds are cleared.
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

    /// Returns the accepted native bounds for a portal node, if present.
    ///
    /// * `node` — portal node to query.
    #[must_use]
    pub fn native_portal_bounds(&self, node: NodeId) -> Option<Rect> {
        self.native_portals.get(&node).copied()
    }

    /// The physical surface containing this element; None is the application's window.
    ///
    /// * `node` — node whose owning native portal is queried.
    ///
    /// Returns the nearest native portal node containing `node`, if any.
    #[must_use]
    pub fn native_portal_owner(&self, node: NodeId) -> Option<NodeId> {
        let mut cursor = self.index.position(node);
        while let Some(index) = cursor {
            let node = self.node_ids[index];
            if self.native_portals.contains_key(&node) {
                return Some(node);
            }
            cursor = self.index.parent(index);
        }
        None
    }
}
