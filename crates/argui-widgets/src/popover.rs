use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_ui::{
    AnchorWidth, Axes, DismissPolicy, Element, FloatingPlacement, FocusScope, InitialFocus,
    Overflow, Placement, ScrollConfig, Sides, WindowLayer, length,
};

use crate::{PopoverBehavior, PopoverPart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Popover {
    key: String,
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
    trap_focus: bool,
    presence: Option<crate::Presence>,
}

impl Popover {
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
            trap_focus: false,
            presence: None,
        }
    }

    #[must_use]
    pub const fn placement(mut self, placement: FloatingPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[must_use]
    pub fn paint(mut self, paint: PaintStyle) -> Self {
        self.paint = Some(paint);
        self
    }

    #[must_use]
    pub const fn size(mut self, width: f32, max_height: f32) -> Self {
        self.width = width;
        self.max_height = max_height;
        self
    }

    #[must_use]
    pub const fn padding(mut self, padding: f32) -> Self {
        self.padding = padding;
        self
    }

    #[must_use]
    pub const fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    #[must_use]
    pub const fn backdrop_blur(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius);
        self
    }

    #[must_use]
    pub const fn trap_focus(mut self, trap: bool) -> Self {
        self.trap_focus = trap;
        self
    }

    #[must_use]
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.open = presence.is_open();
        self.presence = Some(presence.clone());
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = PopoverBehavior::new(&self.key, &self.label, self.open);
        let trigger = behavior.decorate(PopoverPart::Trigger, self.trigger);
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
                        x: Overflow::Hidden,
                        y: Overflow::Auto,
                    })
                    .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
                    .anchored_portal(WindowLayer::Popover, self.key.clone(), self.placement)
                    .portal_dismiss(DismissPolicy::OutsidePointer);
                let blur = self.backdrop_blur.unwrap_or(theme.overlay_blur).max(0.0);
                content = content.layer(theme.overlay_layer(radius, blur));
                content = if self.trap_focus {
                    content.focus_scope(FocusScope::trapped(InitialFocus::First))
                } else {
                    content.focus_scope(FocusScope::restoring())
                };
                let content = behavior.decorate(PopoverPart::Content, content);
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
