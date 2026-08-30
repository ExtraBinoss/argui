use argui_paint::PaintStyle;
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, CursorIcon, Display, Element, FlexDirection, GestureSet, Interaction,
    JustifyContent, KeyboardActivation, LayoutStyle, Role, SemanticAction, Semantics, StateStyle,
    StyleTransition, VisualState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: StateStyle,
    pub pressed: StateStyle,
    pub focused: Option<StateStyle>,
    pub transition: StyleTransition,
    pub label: TextStyle,
}

impl ButtonStyle {
    #[must_use]
    pub fn new(paint: PaintStyle, mut label: TextStyle) -> Self {
        label.wrap = TextWrap::None;
        Self {
            layout: LayoutStyle {
                display: Display::Flex,
                flex_direction: FlexDirection::Row,
                padding: argui_ui::sides(18.0, 11.0),
                align_items: Some(AlignItems::CENTER),
                justify_content: Some(JustifyContent::CENTER),
                flex_shrink: 0.0,
                ..LayoutStyle::default()
            },
            hovered: StateStyle::from_quad(paint.quad.clone()),
            pressed: StateStyle::from_quad(paint.quad.clone()),
            focused: None,
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
        self.focused = Some(style.into());
        self
    }

    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = transition;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Button {
    key: String,
    label: String,
    style: ButtonStyle,
    leading: Option<Element>,
    trailing: Option<Element>,
    enabled: bool,
    loading: Option<Element>,
}

impl Button {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, style: ButtonStyle) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            style,
            leading: None,
            trailing: None,
            enabled: true,
            loading: None,
        }
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
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn loading(mut self, indicator: Element) -> Self {
        self.loading = Some(indicator);
        self
    }

    #[must_use]
    pub fn build(self) -> Element {
        let loading = self.loading.is_some();
        let enabled = self.enabled && !loading;
        let cursor = if !self.enabled {
            CursorIcon::NotAllowed
        } else if loading {
            CursorIcon::Progress
        } else {
            CursorIcon::Pointer
        };
        let interaction = Interaction::default()
            .enabled(enabled)
            .focusable(enabled)
            .cursor(cursor)
            .gestures(GestureSet::NONE.tap())
            .keyboard_activation(KeyboardActivation::EnterOrSpace);
        let state = argui_ui::SemanticState {
            disabled: !enabled,
            busy: loading,
            ..argui_ui::SemanticState::default()
        };
        let semantics = Semantics::new(Role::Button)
            .label(self.label.clone())
            .state(state)
            .action(SemanticAction::Click)
            .action(SemanticAction::Focus);
        let mut children = Vec::with_capacity(3);
        children.extend(self.loading.or(self.leading));
        children.push(
            Element::text(self.label)
                .text_style(self.style.label)
                .semantic_hidden(true),
        );
        children.extend(self.trailing);
        let mut element = Element::row(children)
            .keyed(self.key)
            .layout_style(self.style.layout)
            .paint_style(self.style.paint)
            .interaction(interaction)
            .state(VisualState::Hovered, self.style.hovered)
            .state(VisualState::Pressed, self.style.pressed)
            .transition(self.style.transition)
            .semantics(semantics)
            .gap(8.0);
        if let Some(focused) = self.style.focused {
            element = element.state(VisualState::Focused, focused);
        }
        element
    }
}
