use argui_core::Rect;
use argui_paint::{Filter, LayerStyle};

use crate::{
    AnchorPortal, DismissPolicy, Element, FloatingPlacement, LengthPercentageAuto, Portal,
    PortalTarget, Position, Sides, ViewportPlacement, WindowLayer,
};

impl Element {
    /// Overrides the presentation preference inherited by this portal and its descendants.
    #[must_use]
    pub fn portal_surface(mut self, surface: crate::OverlaySurface) -> Self {
        if let Some(portal) = &mut self.portal {
            portal.surface = Some(surface);
        }
        self
    }

    /// Clips content to the element's bounds and sets its rounded surface corners.
    /// Geometric clipping does not require an offscreen compositing layer.
    #[must_use]
    pub fn clip(self, radii: argui_paint::CornerRadii) -> Self {
        self.radius(radii).overflow(crate::Axes {
            x: crate::Overflow::Hidden,
            y: crate::Overflow::Hidden,
        })
    }

    /// Appends a content filter, preserving previously configured layer properties.
    #[must_use]
    pub fn filter(mut self, filter: Filter) -> Self {
        self.layer
            .get_or_insert_with(|| LayerStyle::new(Rect::default()))
            .filters
            .push(filter);
        self
    }

    #[must_use]
    pub fn mask(mut self, mask: argui_paint::LayerMask) -> Self {
        self.layer
            .get_or_insert_with(|| LayerStyle::new(Rect::default()))
            .mask = mask;
        self
    }

    /// Sets group opacity; this is not per-primitive alpha.
    #[must_use]
    pub fn opacity(mut self, opacity: f32) -> Self {
        self.layer
            .get_or_insert_with(|| LayerStyle::new(Rect::default()))
            .opacity = if opacity.is_finite() {
            opacity.clamp(0.0, 1.0)
        } else {
            1.0
        };
        self
    }

    #[must_use]
    pub fn anchored_portal(
        mut self,
        layer: WindowLayer,
        key: impl Into<String>,
        placement: FloatingPlacement,
    ) -> Self {
        self.prepare_portal();
        self.portal = Some(Portal::new(
            layer,
            PortalTarget::Anchor(AnchorPortal::new(key, placement)),
        ));
        self
    }

    /// Anchor to viewport coordinates, for context menus, carets or selection bounds.
    #[must_use]
    pub fn rect_portal(
        mut self,
        layer: WindowLayer,
        bounds: Rect,
        placement: FloatingPlacement,
    ) -> Self {
        self.prepare_portal();
        self.portal = Some(Portal::new(layer, PortalTarget::Rect { bounds, placement }));
        self
    }

    #[must_use]
    pub fn viewport_portal(mut self, layer: WindowLayer, placement: ViewportPlacement) -> Self {
        self.prepare_portal();
        self.portal = Some(Portal::new(layer, PortalTarget::Viewport(placement)));
        self
    }

    #[must_use]
    pub fn portal(mut self, layer: WindowLayer) -> Self {
        self.prepare_portal();
        match &mut self.portal {
            Some(portal) => portal.layer = layer,
            None => self.portal = Some(Portal::new(layer, PortalTarget::Layout)),
        }
        self
    }

    #[must_use]
    pub fn portal_dismiss(mut self, dismiss: DismissPolicy) -> Self {
        if let Some(portal) = &mut self.portal {
            portal.dismiss = dismiss;
        }
        self
    }

    #[must_use]
    pub fn backdrop_filter(mut self, filter: Filter) -> Self {
        self.layer
            .get_or_insert_with(|| LayerStyle::new(Rect::default()))
            .backdrop_filters
            .push(filter);
        self
    }

    fn prepare_portal(&mut self) {
        self.style.position = Position::Absolute;
        self.style.inset = Sides {
            left: LengthPercentageAuto::auto(),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::auto(),
            bottom: LengthPercentageAuto::auto(),
        };
    }
}
