use argui_paint::{Border, CornerRadii, Filter, LayerMask, LayerStyle, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, AnchorWidth, Axes, DismissPolicy, Element, FloatingPlacement, FocusScope,
    InitialFocus, JustifyContent, Overflow, Placement, ScrollConfig, StylePatch, StyleTransition,
    VisualState, WindowLayer, length, percent,
};

use crate::{SelectBehavior, SelectPart, WidgetTheme};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    pub label: String,
    pub enabled: bool,
}

impl SelectOption {
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Select {
    key: String,
    label: String,
    options: Vec<SelectOption>,
    selected: Option<usize>,
    highlighted: usize,
    open: bool,
    trailing: Option<Element>,
}

impl Select {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            highlighted: selected.unwrap_or(0),
            open: false,
            trailing: None,
        }
    }

    #[must_use]
    pub const fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    pub const fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }

    #[must_use]
    pub fn trailing(mut self, trailing: Element) -> Self {
        self.trailing = Some(trailing);
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = SelectBehavior::new(
            &self.key,
            &self.label,
            self.options.iter().cloned(),
            self.selected,
        )
        .highlighted(self.highlighted)
        .open(self.open);
        let selected_label = self
            .selected
            .and_then(|index| self.options.get(index))
            .map_or(self.label.as_str(), |option| option.label.as_str());
        let mut trigger_children = vec![behavior.decorate(
            SelectPart::Value,
            Element::text(selected_label).text_style(label_style(theme.foreground)),
        )];
        trigger_children.extend(self.trailing.clone());
        let trigger = behavior.decorate(
            SelectPart::Trigger,
            Element::row(trigger_children)
                .width(percent(1.0))
                .padding(argui_ui::sides(12.0, 9.0))
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .paint_style(self_theme_input(theme))
                .when(VisualState::Hovered, theme.input().hovered.clone())
                .when(VisualState::FocusVisible, theme.input().focused.clone())
                .transition(theme.input().transition.clone()),
        );
        let overlay = self.open.then(|| self.overlay(theme));
        behavior.decorate(
            SelectPart::Root,
            Element::container(std::iter::once(trigger).chain(overlay)),
        )
    }

    fn overlay(&self, theme: &WidgetTheme) -> Element {
        let behavior = SelectBehavior::new(
            &self.key,
            &self.label,
            self.options.iter().cloned(),
            self.selected,
        )
        .highlighted(self.highlighted)
        .open(self.open);
        let options = self.options.iter().enumerate().map(|(index, option)| {
            let highlighted = self.highlighted == index;
            behavior.decorate(
                SelectPart::Option(index),
                Element::container([behavior.decorate(
                    SelectPart::Value,
                    Element::text(option.label.clone()).text_style(label_style(
                        if option.enabled {
                            theme.foreground
                        } else {
                            theme.muted_foreground
                        },
                    )),
                )])
                .width(percent(1.0))
                .padding(argui_ui::sides(10.0, 8.0))
                .background(if highlighted {
                    theme.primary.with_alpha(0.14)
                } else {
                    argui_core::Color::TRANSPARENT
                })
                .radius(CornerRadii::all(5.0))
                .when(
                    VisualState::Hovered,
                    StylePatch::from_quad(
                        QuadStyle::solid(theme.primary.with_alpha(0.20))
                            .radius(CornerRadii::all(5.0)),
                    ),
                )
                .transition(StyleTransition::default()),
            )
        });
        let mut list = Element::column(options)
            .width(length(260.0))
            .max_height(length(280.0))
            .padding(argui_ui::Sides::length(5.0))
            .gap(2.0)
            .paint_style(PaintStyle::new(
                QuadStyle::solid(theme.popover)
                    .border(Border::all(1.0, theme.border))
                    .radius(CornerRadii::all(8.0)),
            ))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            })
            .scroll_config(ScrollConfig::default().scrollbar(theme.scrollbar.clone()))
            .anchored_portal(
                WindowLayer::Popover,
                self.key.clone(),
                FloatingPlacement::new(Placement::BottomStart)
                    .anchor_width(AnchorWidth::AtLeastAnchor)
                    .offset(6.0)
                    .viewport_padding(10.0),
            )
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .focus_scope(FocusScope::trapped(InitialFocus::Target(
                behavior.option_key(self.highlighted).into(),
            )))
            .z_index(1_000);
        if theme.overlay_blur > 0.0 {
            list = list.layer(
                LayerStyle::new(Default::default())
                    .backdrop(Filter::Blur(theme.overlay_blur))
                    .mask(LayerMask::Rounded(CornerRadii::all(8.0))),
            );
        }
        behavior.decorate(SelectPart::List, list)
    }
}

fn self_theme_input(theme: &WidgetTheme) -> PaintStyle {
    theme.input().paint.clone()
}

fn label_style(color: argui_core::Color) -> TextStyle {
    TextStyle {
        font_size: 14.0,
        line_height: 20.0,
        color,
        wrap: TextWrap::None,
        ..TextStyle::default()
    }
}
