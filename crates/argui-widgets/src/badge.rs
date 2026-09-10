use argui_core::Color;
use argui_paint::{Border, CornerRadii};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{AlignItems, AlignSelf, Element, Role, Semantics, length, sides};

use crate::WidgetTheme;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum BadgeVariant {
    #[default]
    Primary,
    Secondary,
    Destructive,
    Outline,
    Ghost,
}

/// A compact, non-interactive label. Wrap it in a button or link for actions.
#[derive(Clone, Debug)]
pub struct Badge {
    key: String,
    label: String,
    variant: BadgeVariant,
    leading: Option<Element>,
    trailing: Option<Element>,
}

impl Badge {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            variant: BadgeVariant::default(),
            leading: None,
            trailing: None,
        }
    }

    #[must_use]
    pub const fn variant(mut self, variant: BadgeVariant) -> Self {
        self.variant = variant;
        self
    }

    #[must_use]
    pub fn leading(mut self, icon: Element) -> Self {
        self.leading = Some(icon);
        self
    }

    #[must_use]
    pub fn trailing(mut self, icon: Element) -> Self {
        self.trailing = Some(icon);
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let (background, foreground, border) = match self.variant {
            BadgeVariant::Primary => (theme.primary, theme.primary_foreground, Color::TRANSPARENT),
            BadgeVariant::Secondary => (theme.secondary, theme.foreground, Color::TRANSPARENT),
            BadgeVariant::Destructive => (
                theme.destructive,
                theme.destructive_foreground,
                Color::TRANSPARENT,
            ),
            BadgeVariant::Outline => (Color::TRANSPARENT, theme.foreground, theme.border),
            BadgeVariant::Ghost => (Color::TRANSPARENT, theme.foreground, Color::TRANSPARENT),
        };
        let label = Element::text(self.label.clone()).text_style(TextStyle {
            font_size: 12.0,
            line_height: 16.0,
            weight: 600,
            color: foreground,
            wrap: TextWrap::None,
            ..TextStyle::default()
        });
        Element::row(
            self.leading
                .into_iter()
                .chain([label])
                .chain(self.trailing)
                .map(|child| child.semantic_hidden(true)),
        )
        .keyed(self.key)
        .semantics(Semantics::new(Role::Text).label(self.label))
        .align_items(AlignItems::CENTER)
        .align_self(AlignSelf::START)
        .shrink(0.0)
        .min_height(length(22.0))
        .padding(sides(8.0, 2.0))
        .gap(4.0)
        .radius(CornerRadii::all(999.0))
        .background(background)
        .border(Border::all(1.0, border))
    }
}
