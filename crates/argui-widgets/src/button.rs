use argui_paint::PaintStyle;
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Display, Element, FlexDirection, JustifyContent, LayoutStyle, StylePatch,
    StyleTransition, VisualState,
};

use crate::{ButtonBehavior, ButtonPart};

#[derive(Clone, Debug, PartialEq)]
pub struct ButtonStyle {
    pub layout: LayoutStyle,
    pub paint: PaintStyle,
    pub hovered: StylePatch,
    pub pressed: StylePatch,
    pub focused: Option<StylePatch>,
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
            hovered: StylePatch::from_quad(paint.quad.clone()),
            pressed: StylePatch::from_quad(paint.quad.clone()),
            focused: None,
            transition: StyleTransition::default(),
            paint,
            label,
        }
        .instant_hover()
    }

    #[must_use]
    pub fn layout(mut self, layout: LayoutStyle) -> Self {
        self.layout = layout;
        self
    }

    #[must_use]
    pub fn hovered(mut self, style: impl Into<StylePatch>) -> Self {
        self.hovered = style.into();
        self
    }

    #[must_use]
    pub fn pressed(mut self, style: impl Into<StylePatch>) -> Self {
        self.pressed = style.into();
        self
    }

    #[must_use]
    pub fn focused(mut self, style: impl Into<StylePatch>) -> Self {
        self.focused = Some(style.into());
        self
    }

    #[must_use]
    pub fn transition(mut self, transition: StyleTransition) -> Self {
        self.transition = transition;
        self
    }

    /// Restore immediate hover entry and exit after setting a custom transition.
    #[must_use]
    pub fn instant_hover(mut self) -> Self {
        self.transition = crate::theme::instant_hover(self.transition, VisualState::Hovered.into());
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
    content: Option<Element>,
    tooltip: Option<String>,
}

impl Button {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, style: ButtonStyle) -> Self {
        let label = label.into();
        Self {
            key: key.into(),
            tooltip: Some(label.clone()),
            label,
            style,
            leading: None,
            trailing: None,
            enabled: true,
            loading: None,
            content: None,
        }
    }

    #[must_use]
    pub fn leading(mut self, icon: Element) -> Self {
        self.leading = Some(icon);
        self
    }

    /// Replaces the visible label while retaining its accessible name.
    #[must_use]
    pub fn content(mut self, content: Element) -> Self {
        self.content = Some(content);
        self
    }

    #[must_use]
    pub fn icon(
        key: impl Into<String>,
        label: impl Into<String>,
        icon: Element,
        style: ButtonStyle,
    ) -> Self {
        Self::new(key, label, style).content(icon)
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

    /// Override the default label shown by `TooltipHost` on hover or keyboard focus.
    #[must_use]
    pub fn tooltip(mut self, description: impl Into<String>) -> Self {
        self.tooltip = Some(description.into());
        self
    }

    #[must_use]
    pub fn without_tooltip(mut self) -> Self {
        self.tooltip = None;
        self
    }

    #[must_use]
    pub fn build(self) -> Element {
        let loading = self.loading.is_some();
        let behavior = ButtonBehavior::new(self.key, self.label.clone())
            .enabled(self.enabled)
            .busy(loading);
        let mut children = Vec::with_capacity(3);
        children.extend(self.loading.or(self.leading));
        children.push(
            behavior.decorate(
                ButtonPart::Content,
                self.content
                    .unwrap_or_else(|| Element::text(self.label).text_style(self.style.label)),
            ),
        );
        children.extend(self.trailing);
        let mut element = behavior.decorate(
            ButtonPart::Root,
            Element::row(children)
                .layout_style(self.style.layout)
                .paint_style(self.style.paint)
                .when(VisualState::Hovered, self.style.hovered)
                .when(VisualState::Pressed, self.style.pressed)
                .transition(self.style.transition)
                .gap(8.0),
        );
        if let Some(focused) = self.style.focused {
            element = element.when(VisualState::FocusVisible, focused);
        }
        element.tooltip = self.tooltip.filter(|_| self.enabled && !loading);
        element
    }
}
