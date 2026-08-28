use argui_paint::{PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};

use crate::{Align, CursorIcon, Edges, Element, Interaction, LayoutStyle, Length};

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: QuadStyle,
    pub pressed: QuadStyle,
    pub focused: QuadStyle,
    pub label: TextStyle,
}

impl ButtonStyle {
    #[must_use]
    pub fn new(paint: PaintStyle, mut label: TextStyle) -> Self {
        label.wrap = TextWrap::None;
        Self {
            layout: LayoutStyle {
                width: Length::Auto,
                padding: Edges::symmetric(18.0, 11.0),
                align: Align::Center,
                shrink: 0.0,
                ..LayoutStyle::default()
            },
            hovered: paint.quad.clone(),
            pressed: paint.quad.clone(),
            focused: paint.quad.clone(),
            paint,
            label,
        }
    }

    #[must_use]
    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn hovered(mut self, style: QuadStyle) -> Self {
        self.hovered = style;
        self
    }

    #[must_use]
    pub fn pressed(mut self, style: QuadStyle) -> Self {
        self.pressed = style;
        self
    }

    #[must_use]
    pub fn focused(mut self, style: QuadStyle) -> Self {
        self.focused = style;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct Button {
    key: String,
    label: String,
    style: ButtonStyle,
}

impl Button {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, style: ButtonStyle) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            style,
        }
    }

    #[must_use]
    pub fn build(self) -> Element {
        let interaction = Interaction::default()
            .focusable(true)
            .cursor(CursorIcon::Pointer)
            .hovered(self.style.hovered)
            .pressed(self.style.pressed)
            .focused(self.style.focused);
        Element::container([Element::text(self.label).text_style(self.style.label)])
            .keyed(self.key)
            .layout_style(self.style.layout)
            .paint_style(self.style.paint)
            .interaction(interaction)
    }
}
