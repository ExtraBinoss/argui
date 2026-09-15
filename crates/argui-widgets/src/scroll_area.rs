use crate::WidgetTheme;
use argui_ui::{
    Axes, Element, FocusPolicy, Interaction, Orientation, Overflow, Role, ScrollConfig,
    ScrollPropagation, Semantics, length, percent,
};

/// Focusable scrolling viewport using the engine's wheel, keyboard, touch and scrollbar paths.
#[derive(Clone, Debug)]
pub struct ScrollArea {
    pub key: String,
    pub label: String,
    pub content: Element,
    pub orientation: Orientation,
    pub extent: f32,
    pub config: Option<ScrollConfig>,
}

impl ScrollArea {
    /// Creates a scroll area identified by `key` around `content`.
    /// `label` is its accessible name; `extent` sets the viewport length along its scroll axis.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        extent: f32,
        content: Element,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            content,
            orientation: Orientation::Vertical,
            extent,
            config: None,
        }
    }

    #[must_use]
    /// Builds the scroll area using `theme` for its scrollbar.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let vertical = self.orientation == Orientation::Vertical;
        Element::layout_boundary(self.content.shrink(0.0))
            .keyed(self.key)
            .width(percent(1.0))
            .min_width(length(0.0))
            .height(length(self.extent.max(0.0)))
            .overflow(Axes {
                x: if vertical {
                    Overflow::Hidden
                } else {
                    Overflow::Auto
                },
                y: if vertical {
                    Overflow::Auto
                } else {
                    Overflow::Hidden
                },
            })
            .scroll_config(self.config.unwrap_or_else(|| {
                ScrollConfig::default()
                    .propagation(ScrollPropagation::Contain)
                    .scrollbar(theme.scrollbar.clone())
            }))
            .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
            .semantics(Semantics::new(Role::Group).label(self.label))
    }
}
