use argui_paint::{Border, CornerRadii, LayerStyle, PaintStyle, QuadStyle};
use argui_ui::{
    AnchorWidth, Axes, DismissPolicy, Element, EventFilter, EventHandler, EventType,
    FloatingPlacement, FocusScope, InitialFocus, Overflow, Placement, ScrollConfig, Sides,
    ValueHandler, WindowLayer, length,
};

use crate::{PopoverBehavior, PopoverPart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Popover {
    key: String,
    surface: Option<argui_ui::OverlaySurface>,
    label: String,
    open: bool,
    trigger: Element,
    content: Element,
    placement: FloatingPlacement,
    paint: Option<PaintStyle>,
    width: f32,
    max_height: f32,
    padding: f32,
    radius: f32,
    backdrop_blur: Option<f32>,
    layer: Option<LayerStyle>,
    trap_focus: bool,
    presence: Option<crate::Presence>,
    open_handlers: Vec<ValueHandler<bool>>,
    dismiss_handlers: Vec<EventHandler>,
}

impl Popover {
    /// Creates a controlled popover with a trigger and panel content.
    ///
    /// `key` identifies the popover, `label` names it accessibly, and `open` supplies visibility.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        trigger: Element,
        content: Element,
    ) -> Self {
        Self {
            key: key.into(),
            surface: None,
            label: label.into(),
            open,
            trigger,
            content,
            placement: FloatingPlacement::new(Placement::BottomStart)
                .anchor_width(AnchorWidth::AtLeastAnchor),
            paint: None,
            width: 288.0,
            max_height: 320.0,
            padding: 12.0,
            radius: 8.0,
            backdrop_blur: None,
            layer: None,
            trap_focus: false,
            presence: None,
            open_handlers: Vec::new(),
            dismiss_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets the panel placement relative to its trigger.
    pub const fn placement(mut self, placement: FloatingPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[must_use]
    /// Replaces the panel paint style.
    pub fn paint(mut self, paint: PaintStyle) -> Self {
        self.paint = Some(paint);
        self
    }

    #[must_use]
    /// Sets the preferred width and maximum height in logical pixels; `max_height` bounds the panel vertically.
    pub const fn size(mut self, width: f32, max_height: f32) -> Self {
        self.width = width;
        self.max_height = max_height;
        self
    }

    #[must_use]
    /// Sets the panel's inner padding in logical pixels.
    pub const fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    #[must_use]
    /// Sets the panel corner radius.
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    #[must_use]
    /// Sets the backdrop blur radius; non-positive values disable blur.
    pub const fn backdrop_blur(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius);
        self
    }

    /// Replace the panel layer, including its filters, backdrop effects, mask and shadows.
    #[must_use]
    pub fn layer(mut self, layer: LayerStyle) -> Self {
        self.layer = Some(layer);
        self
    }

    #[must_use]
    /// Sets whether focus is trapped within the open panel; `trap` enables focus containment.
    pub const fn trap_focus(mut self, trap: bool) -> Self {
        self.trap_focus = trap;
        self
    }

    #[must_use]
    /// Uses retained presence state for visibility and entry/exit motion.
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.open = presence.is_open();
        self.presence = Some(presence.clone());
        self
    }

    #[must_use]
    /// Sets the overlay surface policy.
    pub const fn surface(mut self, surface: argui_ui::OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Adds a callback receiving the requested controlled open state.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    /// Adds a callback for pointer-outside, native, or Escape dismissal.
    #[must_use]
    pub fn on_dismiss(mut self, handler: EventHandler) -> Self {
        self.dismiss_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the popover using `theme` for default panel and backdrop styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = PopoverBehavior::new(&self.key, &self.label, self.open);
        let mut trigger = behavior.decorate(PopoverPart::Trigger, self.trigger);
        for handler in &self.open_handlers {
            trigger = trigger.on(handler.direct_listener_value(EventType::Click, !self.open));
        }
        let content = (self.open || self.presence.as_ref().is_some_and(crate::Presence::visible))
            .then(|| {
                let radius = self.radius.max(0.0);
                let paint = self.paint.unwrap_or_else(|| {
                    PaintStyle::new(
                        QuadStyle::solid(theme.popover)
                            .border(Border::all(1.0, theme.popover_border))
                            .radius(CornerRadii::all(radius)),
                    )
                });
                let mut content = Element::column([self.content])
                    .width(length(self.width.max(0.0)))
                    .max_height(length(self.max_height.max(0.0)))
                    .padding(Sides::length(self.padding.max(0.0)))
                    .paint_style(paint)
                    .overflow(Axes {
                        x: Overflow::Auto,
                        y: Overflow::Auto,
                    })
                    .scroll_config(
                        ScrollConfig::default()
                            .propagation(argui_ui::ScrollPropagation::Contain)
                            .scrollbar(theme.scrollbar.clone()),
                    )
                    .anchored_portal(WindowLayer::Popover, self.key.clone(), self.placement)
                    .portal_dismiss(DismissPolicy::OutsidePointer);
                if let Some(surface) = self.surface {
                    content = content.portal_surface(surface);
                }
                let blur = self.backdrop_blur.unwrap_or(theme.overlay_blur).max(0.0);
                content = content.layer(
                    self.layer
                        .unwrap_or_else(|| theme.overlay_layer(radius, blur)),
                );
                content = if self.trap_focus {
                    content.focus_scope(FocusScope::trapped(InitialFocus::First))
                } else {
                    content.focus_scope(FocusScope::restoring())
                };
                let mut content = behavior.decorate(PopoverPart::Content, content);
                for handler in &self.open_handlers {
                    content = content
                        .on(handler.direct_listener_value(EventType::PointerOutside, false))
                        .on(handler.direct_listener_value(EventType::Dismiss, false))
                        .on(handler
                            .listener_value(EventType::Key, false)
                            .filter(EventFilter::EscapePressed));
                }
                for handler in &self.dismiss_handlers {
                    content = content
                        .on(handler.direct_listener(EventType::PointerOutside))
                        .on(handler.direct_listener(EventType::Dismiss))
                        .on(handler
                            .listener(EventType::Key)
                            .filter(EventFilter::EscapePressed));
                }
                match &self.presence {
                    Some(presence) => presence.decorate(content),
                    None => content,
                }
            });
        behavior.decorate(
            PopoverPart::Root,
            Element::container(std::iter::once(trigger).chain(content)),
        )
    }
}
