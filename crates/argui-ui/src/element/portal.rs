use argui_core::Rect;
use argui_paint::{Filter, LayerStyle};

use crate::{
    AnchorPortal, DismissPolicy, Element, FloatingPlacement, LengthPercentageAuto, Portal,
    PortalTarget, Position, Sides, ViewportPlacement, WindowLayer,
};

impl Element {
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
