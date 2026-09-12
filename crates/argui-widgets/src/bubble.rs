use crate::WidgetTheme;
use argui_paint::{Border, CornerRadii};
use argui_ui::{AlignSelf, Element, Sides, percent};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BubbleVariant {
    Primary,
    #[default]
    Secondary,
    Muted,
    Tinted,
    Outline,
    Ghost,
    Destructive,
}

/// Conversation surface; reactions remain ordinary interactive children.
#[derive(Clone, Debug)]
pub struct Bubble {
    pub key: String,
    pub content: Element,
    pub reactions: Option<Element>,
    pub variant: BubbleVariant,
    pub end: bool,
}

impl Bubble {
    #[must_use]
    pub fn new(key: impl Into<String>, content: Element) -> Self {
        Self {
            key: key.into(),
            content,
            reactions: None,
            variant: BubbleVariant::Secondary,
            end: false,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let color = match self.variant {
            BubbleVariant::Primary => theme.primary,
            BubbleVariant::Secondary => theme.secondary,
            BubbleVariant::Muted => theme.muted,
            BubbleVariant::Tinted => theme.primary.with_alpha(0.14),
            BubbleVariant::Outline | BubbleVariant::Ghost => argui_core::Color::TRANSPARENT,
            BubbleVariant::Destructive => theme.destructive,
        };
        let mut surface = Element::column([self.content])
            .padding(Sides::length(12.0))
            .background(color)
            .radius(CornerRadii::all(16.0));
        if self.variant == BubbleVariant::Outline {
            surface = surface.border(Border::all(1.0, theme.border));
        }
        Element::column(std::iter::once(surface).chain(self.reactions))
            .keyed(self.key)
            .gap(4.0)
            .max_width(percent(if self.variant == BubbleVariant::Ghost {
                1.0
            } else {
                0.8
            }))
            .align_self(if self.end {
                AlignSelf::END
            } else {
                AlignSelf::START
            })
    }
}
