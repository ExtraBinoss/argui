use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, AnchorWidth, Axes, DismissPolicy, Element, EventFilter, EventType,
    FloatingPlacement, FocusScope, InitialFocus, JustifyContent, Overflow, Placement, ScrollConfig,
    StylePatch, StyleTransition, ValueHandler, VisualState, WindowLayer, length, percent,
};

use crate::{SelectBehavior, SelectPart, WidgetTheme};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SelectOption {
    pub label: String,
    pub enabled: bool,
}

impl SelectOption {
    /// Creates an enabled option with the supplied display label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    #[must_use]
    /// Sets whether this option can be highlighted or selected.
    /// `enabled` controls whether it participates in interaction.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Select {
    key: String,
    surface: Option<argui_ui::OverlaySurface>,
    label: String,
    options: Vec<SelectOption>,
    selected: Option<usize>,
    highlighted: usize,
    open: bool,
    trailing: Option<Element>,
    presence: Option<crate::Presence>,
    select_handlers: Vec<ValueHandler<usize>>,
    open_handlers: Vec<ValueHandler<bool>>,
}

impl Select {
    /// Creates a controlled select from options in source order.
    /// `key` identifies the control, `label` names it accessibly, and `selected` is the initial option index.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = SelectOption>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            surface: None,
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            highlighted: selected.unwrap_or(0),
            open: false,
            trailing: None,
            presence: None,
            select_handlers: Vec::new(),
            open_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets whether the option list is initially open.
    pub const fn open(mut self, open: bool) -> Self {
        self.open = open;
        self
    }

    #[must_use]
    /// Sets the source index of the currently highlighted option.
    pub const fn highlighted(mut self, highlighted: usize) -> Self {
        self.highlighted = highlighted;
        self
    }

    #[must_use]
    /// Adds a trailing element to the trigger.
    pub fn trailing(mut self, trailing: Element) -> Self {
        self.trailing = Some(trailing);
        self
    }

    #[must_use]
    /// Uses retained presence state for popup visibility and motion.
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.open = presence.is_open();
        self.presence = Some(presence.clone());
        self
    }

    #[must_use]
    /// Sets the overlay surface policy.
    pub const fn surface(mut self, surface: argui_ui::OverlaySurface) -> Self {
        self.surface = Some(surface);
        self
    }

    /// Adds a callback for selection of an enabled option's source index.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<usize>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    /// Adds a callback for a requested popup open-state change.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the select trigger and popup using `theme` for their styling.
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
        let mut trigger = behavior.decorate(
            SelectPart::Trigger,
            Element::row(trigger_children)
                .width(percent(1.0))
                .height(length(36.0))
                .padding(argui_ui::sides(12.0, 8.0))
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .paint_style(self_theme_input(theme))
                .when(VisualState::Hovered, theme.input().hovered.clone())
                .when(VisualState::FocusVisible, theme.input().focused.clone())
                .transition(crate::theme::instant_hover(
                    StyleTransition::default(),
                    VisualState::Hovered.into(),
                )),
        );
        for handler in &self.open_handlers {
            trigger = trigger.on(handler.direct_listener_value(EventType::Click, !self.open));
        }
        let overlay = (self.open || self.presence.as_ref().is_some_and(crate::Presence::visible))
            .then(|| {
                let overlay = self.overlay(theme);
                match &self.presence {
                    Some(presence) => presence.decorate(overlay),
                    None => overlay,
                }
            });
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
            let mut row = behavior.decorate(
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
                .transition(crate::theme::instant_hover(
                    StyleTransition::default(),
                    VisualState::Hovered.into(),
                )),
            );
            if option.enabled {
                for handler in &self.select_handlers {
                    row = row.on(handler.direct_listener_value(EventType::Click, index));
                }
            }
            row
        });
        let mut list = Element::column(options)
            .width(length(260.0))
            .max_height(length(280.0))
            .padding(argui_ui::Sides::length(5.0))
            .gap(2.0)
            .paint_style(PaintStyle::new(
                QuadStyle::solid(theme.popover)
                    .border(Border::all(1.0, theme.popover_border))
                    .radius(CornerRadii::all(8.0)),
            ))
            .overflow(Axes {
                x: Overflow::Auto,
                y: Overflow::Auto,
            })
            .scroll_config(
                ScrollConfig::default()
                    .propagation(argui_ui::ScrollPropagation::Contain)
                    .scrollbar(theme.scrollbar.clone()),
            )
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
            .z_index(1_000)
            .layer(theme.overlay_layer(8.0, theme.overlay_blur));
        for handler in &self.open_handlers {
            list = list
                .on(handler.direct_listener_value(EventType::PointerOutside, false))
                .on(handler.direct_listener_value(EventType::Dismiss, false))
                .on(handler
                    .listener_value(EventType::Key, false)
                    .filter(EventFilter::EscapePressed));
        }
        behavior.decorate(
            SelectPart::List,
            if let Some(surface) = self.surface {
                list.portal_surface(surface)
            } else {
                list
            },
        )
    }
}

fn self_theme_input(theme: &WidgetTheme) -> PaintStyle {
    theme.input().paint.clone()
}

fn label_style(color: argui_core::Color) -> TextStyle {
    TextStyle {
        font_size: 15.0,
        line_height: 20.0,
        color,
        wrap: TextWrap::None,
        ..TextStyle::default()
    }
}
