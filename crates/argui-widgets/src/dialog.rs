use argui_core::Color;
use argui_paint::{Border, CornerRadii, Filter, PaintStyle, QuadStyle};
use argui_ui::{
    AlignItems, Display, Element, JustifyContent, Sides, ViewportPlacement, WindowLayer, length,
    percent,
};

use crate::{DialogBehavior, DialogPart, WidgetTheme};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DialogPlacement {
    #[default]
    Center,
    Left,
    Right,
    Top,
    Bottom,
}

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
    placement: DialogPlacement,
    initial_focus: Option<argui_ui::InitialFocus>,
    alert: bool,
}

impl Dialog {
    /// Creates a controlled dialog around `content`, with `trigger` as its opener.
    ///
    /// `key` identifies the dialog, `label` names it accessibly, and `open` supplies its state.
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
            placement: DialogPlacement::Center,
            initial_focus: None,
            alert: false,
        }
    }

    #[must_use]
    /// Sets the panel's placement relative to the viewport.
    pub const fn placement(mut self, placement: DialogPlacement) -> Self {
        self.placement = placement;
        self
    }

    #[must_use]
    /// Sets the focus target selected when the dialog opens.
    pub fn initial_focus(mut self, focus: argui_ui::InitialFocus) -> Self {
        self.initial_focus = Some(focus);
        self
    }

    #[must_use]
    /// Enables alert-dialog semantics when `alert` is true.
    pub const fn alert(mut self, alert: bool) -> Self {
        self.alert = alert;
        self
    }

    #[must_use]
    /// Overrides the backdrop color.
    pub const fn backdrop(mut self, color: Color) -> Self {
        self.backdrop = Some(color);
        self
    }

    #[must_use]
    /// Sets the backdrop blur radius; non-positive values disable blur.
    pub const fn backdrop_blur(mut self, radius: f32) -> Self {
        self.backdrop_blur = Some(radius);
        self
    }

    #[must_use]
    /// Replaces the panel's paint style.
    pub fn panel_paint(mut self, paint: PaintStyle) -> Self {
        self.panel_paint = Some(paint);
        self
    }

    #[must_use]
    /// Sets the preferred panel width in logical pixels.
    pub const fn panel_width(mut self, width: f32) -> Self {
        self.panel_width = width;
        self
    }

    #[must_use]
    /// Sets the minimum margin between the panel and viewport edges.
    pub const fn viewport_margin(mut self, margin: f32) -> Self {
        self.viewport_margin = margin;
        self
    }

    #[must_use]
    /// Builds the dialog using `theme` for its default backdrop and panel styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let mut behavior = DialogBehavior::new(&self.key, &self.label, self.open).alert(self.alert);
        if let Some(focus) = self.initial_focus {
            behavior = behavior.initial_focus(focus);
        }
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
                        .border(Border::all(1.0, theme.popover_border))
                        .radius(CornerRadii::all(12.0)),
                )
            });
            let mut panel = Element::column([self.content])
                .width(length(self.panel_width.max(0.0)))
                .max_width(percent(0.90))
                .padding(argui_ui::Sides::length(24.0))
                .gap(18.0)
                .paint_style(panel_paint)
                .z_index(1)
                .layer(theme.overlay_layer(12.0, theme.overlay_blur));
            panel = panel
                .max_height(percent(1.0))
                .overflow(argui_ui::Axes {
                    x: argui_ui::Overflow::Hidden,
                    y: argui_ui::Overflow::Auto,
                })
                .scroll_config(
                    argui_ui::ScrollConfig::default()
                        .propagation(argui_ui::ScrollPropagation::Contain)
                        .scrollbar(theme.scrollbar.clone()),
                );
            let (align, justify) = match self.placement {
                DialogPlacement::Center => (AlignItems::CENTER, JustifyContent::CENTER),
                DialogPlacement::Left => {
                    panel = panel.height(percent(1.0));
                    (AlignItems::STRETCH, JustifyContent::START)
                }
                DialogPlacement::Right => {
                    panel = panel.height(percent(1.0));
                    (AlignItems::STRETCH, JustifyContent::END)
                }
                DialogPlacement::Top => {
                    panel = panel.width(percent(1.0)).max_width(percent(1.0));
                    (AlignItems::START, JustifyContent::CENTER)
                }
                DialogPlacement::Bottom => {
                    panel = panel.width(percent(1.0)).max_width(percent(1.0));
                    (AlignItems::END, JustifyContent::CENTER)
                }
            };
            let panel = behavior.decorate(DialogPart::Panel, panel);
            behavior.decorate(
                DialogPart::Overlay,
                Element::container([backdrop, panel])
                    .width(percent(1.0))
                    .height(percent(1.0))
                    .padding(Sides::length(self.viewport_margin.max(0.0)))
                    .display(Display::Flex)
                    .align_items(align)
                    .justify_content(justify)
                    .viewport_portal(WindowLayer::Modal, ViewportPlacement::fill()),
            )
        });
        behavior.decorate(
            DialogPart::Root,
            Element::container(std::iter::once(trigger).chain(overlay)),
        )
    }
}
