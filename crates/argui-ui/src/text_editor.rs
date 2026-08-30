use argui_paint::{ClipBehavior, Color, PaintStyle};
use argui_text::{TextColor, TextStyle, TextWrap};

use crate::{
    Align, CursorIcon, Edges, Element, ElementKind, GestureSet, Interaction, LayoutStyle, Length,
    Role, ScrollChaining, ScrollConfig, ScrollbarStyle, SemanticAction, SemanticValue, Semantics,
    StateStyle, StyleTransition, VisualState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct TextInputStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: StateStyle,
    pub focused: StateStyle,
    pub transition: StyleTransition,
    pub text: TextStyle,
    pub placeholder: TextStyle,
    pub selection: Color,
    pub caret: Color,
}

impl TextInputStyle {
    #[must_use]
    pub fn new(paint: PaintStyle, mut text: TextStyle) -> Self {
        text.wrap = TextWrap::None;
        let mut placeholder = text.clone();
        placeholder.color = TextColor::rgba(0.55, 0.60, 0.68, 1.0);
        Self {
            layout: LayoutStyle {
                width: Length::Percent(1.0),
                padding: Edges::symmetric(13.0, 10.0),
                align: Align::Center,
                shrink: 0.0,
                ..LayoutStyle::default()
            },
            hovered: StateStyle::from_quad(paint.quad.clone()),
            focused: StateStyle::from_quad(paint.quad.clone()),
            transition: StyleTransition::default(),
            paint,
            text,
            placeholder,
            selection: Color::rgba(0.20, 0.68, 0.94, 0.38),
            caret: Color::WHITE,
        }
    }

    #[must_use]
    pub fn hovered(mut self, style: impl Into<StateStyle>) -> Self {
        self.hovered = style.into();
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

    #[must_use]
    pub const fn selection(mut self, color: Color) -> Self {
        self.selection = color;
        self
    }

    #[must_use]
    pub const fn caret(mut self, color: Color) -> Self {
        self.caret = color;
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TextInput {
    key: String,
    value: String,
    placeholder: String,
    style: TextInputStyle,
}

impl TextInput {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        style: TextInputStyle,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            placeholder: placeholder.into(),
            style,
        }
    }

    #[must_use]
    pub fn build(self) -> Element {
        editor(self.key, self.value, self.placeholder, self.style, false)
    }
}

/// Controlled multiline text editor with soft wrapping.
#[derive(Clone, Debug, PartialEq)]
pub struct TextArea {
    key: String,
    value: String,
    placeholder: String,
    style: TextInputStyle,
    scroll: ScrollConfig,
}

impl TextArea {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        value: impl Into<String>,
        placeholder: impl Into<String>,
        mut style: TextInputStyle,
    ) -> Self {
        style.text.wrap = TextWrap::WordOrGlyph;
        style.placeholder.wrap = TextWrap::WordOrGlyph;
        style.layout.align = Align::Start;
        Self {
            key: key.into(),
            value: value.into(),
            placeholder: placeholder.into(),
            style,
            scroll: ScrollConfig::default().chaining(ScrollChaining::Contain),
        }
    }

    #[must_use]
    pub fn scroll_config(mut self, scroll: ScrollConfig) -> Self {
        self.scroll = scroll;
        self
    }

    #[must_use]
    pub fn scrollbar(mut self, scrollbar: ScrollbarStyle) -> Self {
        self.scroll = self.scroll.scrollbar(scrollbar);
        self
    }

    #[must_use]
    pub fn build(self) -> Element {
        let mut element = editor(self.key, self.value, self.placeholder, self.style, true);
        element.paint.clip = ClipBehavior::Bounds;
        element.scroll = Some(self.scroll);
        element
    }
}

fn editor(
    key: String,
    value: String,
    placeholder: String,
    style: TextInputStyle,
    multiline: bool,
) -> Element {
    let interaction = Interaction::default()
        .focusable(true)
        .cursor(CursorIcon::Text)
        .gestures(GestureSet::NONE.tap().pan());
    let semantics = Semantics::new(if multiline {
        Role::TextArea
    } else {
        Role::TextInput
    })
    .label(placeholder.clone())
    .value(SemanticValue::Text(value.clone()))
    .action(SemanticAction::Focus)
    .action(SemanticAction::SetValue);
    let mut element = Element::container([]).semantics(semantics);
    element.key = Some(key);
    element.kind = ElementKind::TextEditor {
        value,
        placeholder,
        multiline,
        text: style.text,
        placeholder_text: style.placeholder,
        selection: style.selection,
        caret: style.caret,
    };
    element.style = style.layout;
    element.paint = style.paint;
    element.interaction = Some(interaction);
    element
        .state_styles
        .set(VisualState::Hovered, style.hovered);
    element
        .state_styles
        .set(VisualState::Focused, style.focused);
    element.style_transition = Some(style.transition);
    element
}
