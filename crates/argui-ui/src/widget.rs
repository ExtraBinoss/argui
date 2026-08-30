use argui_paint::PaintStyle;
use argui_text::{TextStyle, TextWrap};

use crate::{
    Align, CursorIcon, Edges, Element, GestureSet, Interaction, KeyboardActivation, LayoutStyle,
    Length, Role, SemanticAction, Semantics, StateStyle, StyleTransition, VisualState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: StateStyle,
    pub pressed: StateStyle,
    pub focused: StateStyle,
    pub transition: StyleTransition,
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
            hovered: StateStyle::from_quad(paint.quad.clone()),
            pressed: StateStyle::from_quad(paint.quad.clone()),
            focused: StateStyle::from_quad(paint.quad.clone()),
            transition: StyleTransition::default(),
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
    pub fn hovered(mut self, style: impl Into<StateStyle>) -> Self {
        self.hovered = style.into();
        self
    }

    #[must_use]
    pub fn pressed(mut self, style: impl Into<StateStyle>) -> Self {
        self.pressed = style.into();
        self
    }

    #[must_use]
    pub fn focused(mut self, style: impl Into<StateStyle>) -> Self {
        self.focused = style.into();
        self
    }

    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = transition;
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
            .gestures(GestureSet::NONE.tap())
            .keyboard_activation(KeyboardActivation::EnterOrSpace);
        let semantics = Semantics::new(Role::Button)
            .label(self.label.clone())
            .action(SemanticAction::Click)
            .action(SemanticAction::Focus);
        Element::container([Element::text(self.label)
            .text_style(self.style.label)
            .semantic_hidden(true)])
        .keyed(self.key)
        .layout_style(self.style.layout)
        .paint_style(self.style.paint)
        .interaction(interaction)
        .state(VisualState::Hovered, self.style.hovered)
        .state(VisualState::Pressed, self.style.pressed)
        .state(VisualState::Focused, self.style.focused)
        .transition(self.style.transition)
        .semantics(semantics)
    }
}
