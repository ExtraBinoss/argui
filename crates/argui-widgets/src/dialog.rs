use argui_core::Color;
use argui_paint::{Border, CornerRadii, Filter, LayerMask, LayerStyle, PaintStyle, QuadStyle};
use argui_ui::{
    AlignItems, Display, Element, JustifyContent, Sides, ViewportPlacement, WindowLayer, length,
    percent,
};

use crate::{DialogBehavior, DialogPart, WidgetTheme};

#[derive(Clone, Debug)]
pub struct Dialog {
    key: String,
    label: String,
    open: bool,
    trigger: Element,
    content: Element,
    backdrop: Option<Color>,
    backdrop_blur: Option<f32>,
    panel_paint: Option<PaintStyle>,
    panel_width: f32,
    viewport_margin: f32,
}

impl Dialog {
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
            backdrop: None,
            backdrop_blur: None,
            panel_paint: None,
            panel_width: 480.0,
            viewport_margin: 24.0,
        }
    }

    #[must_use]
    pub const fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = Some(color);
        self
    }

    #[must_use]
    pub const fn backdrop_blur(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius);
        self
    }

    #[must_use]
    pub fn panel_paint(mut self, paint: PaintStyle) -> Self {
        self.panel_paint = Some(paint);
        self
    }

    #[must_use]
    pub const fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    #[must_use]
    pub const fn viewport_margin(mut self, margin: f32) -> Self {
        self.viewport_margin = margin;
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = DialogBehavior::new(&self.key, &self.label, self.open);
        let trigger = behavior.decorate(DialogPart::Trigger, self.trigger);
        let overlay = self.open.then(|| {
            let backdrop_blur = self
                .backdrop_blur
                .unwrap_or(theme.dialog_backdrop_blur)
                .max(0.0);
            let mut backdrop = Element::container([])
                .absolute(Sides::length(0.0))
                .width(percent(1.0))
                .height(percent(1.0))
                .background(self.backdrop.unwrap_or(theme.dialog_backdrop))
                .z_index(0);
            if backdrop_blur > 0.0 {
                backdrop = backdrop.backdrop_filter(Filter::Blur(backdrop_blur));
            }
            let backdrop = behavior.decorate(DialogPart::Backdrop, backdrop);
            let panel_paint = self.panel_paint.unwrap_or_else(|| {
                PaintStyle::new(
                    QuadStyle::solid(theme.popover)
                        .border(Border::all(1.0, theme.border))
                        .radius(CornerRadii::all(12.0)),
                )
            });
            let mut panel = Element::column([self.content])
                .width(length(self.panel_width.max(0.0)))
                .max_width(percent(0.90))
                .padding(argui_ui::Sides::length(24.0))
                .gap(18.0)
                .paint_style(panel_paint)
                .z_index(1);
            if theme.overlay_blur > 0.0 {
                panel = panel.layer(
                    LayerStyle::new(Default::default())
                        .backdrop(Filter::Blur(theme.overlay_blur))
                        .mask(LayerMask::Rounded(CornerRadii::all(12.0))),
                );
            }
            let panel = behavior.decorate(DialogPart::Panel, panel);
            behavior.decorate(
                DialogPart::Overlay,
                Element::container([backdrop, panel])
                    .width(percent(1.0))
                    .height(percent(1.0))
                    .padding(Sides::length(self.viewport_margin.max(0.0)))
                    .display(Display::Flex)
                    .align_items(AlignItems::CENTER)
                    .justify_content(JustifyContent::CENTER)
                    .viewport_portal(WindowLayer::Modal, ViewportPlacement::fill()),
            )
        });
        behavior.decorate(
            DialogPart::Root,
            Element::container(std::iter::once(trigger).chain(overlay)),
        )
    }
}
