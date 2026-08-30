use argui_core::{Key, KeyState};
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, CursorIcon, Element, FocusScope, GestureSet, InitialFocus, Interaction,
    JustifyContent, KeyboardActivation, Overflow, OverlayAlign, OverlayPlacement, PlacementSide,
    Role, ScrollConfig, SemanticAction, SemanticState, Semantics, StateStyle, StyleTransition,
    UiEvent, UiEventKind, VisualState, length, percent,
};

use crate::WidgetTheme;

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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SelectAction {
    Toggle,
    Close,
    Highlight(usize),
    Select(usize),
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
    pub fn option_key(key: &str, index: usize) -> String {
        format!("{key}::option::{index}")
    }

    #[must_use]
    pub fn list_key(key: &str) -> String {
        format!("{key}::list")
    }

    #[must_use]
    pub fn action(
        key: &str,
        options: &[SelectOption],
        highlighted: usize,
        event: &UiEvent,
    ) -> Option<SelectAction> {
        let event_key = event.key.as_deref()?;
        if matches!(event.kind, UiEventKind::Clicked) {
            if event_key == key {
                return Some(SelectAction::Toggle);
            }
            let index = event_key
                .strip_prefix(&format!("{key}::option::"))?
                .parse::<usize>()
                .ok()?;
            return options
                .get(index)
                .is_some_and(|option| option.enabled)
                .then_some(SelectAction::Select(index));
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        let option_prefix = format!("{key}::option::");
        if event_key != key
            && event_key != Self::list_key(key)
            && !event_key.starts_with(&option_prefix)
        {
            return None;
        }
        if input.state != KeyState::Pressed {
            return None;
        }
        match &input.key {
            Key::Escape => Some(SelectAction::Close),
            Key::Enter => Some(SelectAction::Select(highlighted)),
            Key::ArrowDown => next_enabled(options, highlighted, true).map(SelectAction::Highlight),
            Key::ArrowUp => next_enabled(options, highlighted, false).map(SelectAction::Highlight),
            Key::Home => first_enabled(options).map(SelectAction::Highlight),
            Key::End => last_enabled(options).map(SelectAction::Highlight),
            Key::Character(query) => options
                .iter()
                .enumerate()
                .find(|(_, option)| {
                    option.enabled
                        && option
                            .label
                            .to_lowercase()
                            .starts_with(&query.to_lowercase())
                })
                .map(|(index, _)| SelectAction::Highlight(index)),
            _ => None,
        }
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let selected_label = self
            .selected
            .and_then(|index| self.options.get(index))
            .map_or(self.label.as_str(), |option| option.label.as_str());
        let trigger_state = SemanticState {
            expanded: Some(self.open),
            ..SemanticState::default()
        };
        let mut trigger_children = vec![
            Element::text(selected_label)
                .text_style(label_style(theme.foreground))
                .semantic_hidden(true),
        ];
        trigger_children.extend(self.trailing.clone());
        let trigger = Element::row(trigger_children)
            .keyed(self.key.clone())
            .width(percent(1.0))
            .padding(argui_ui::sides(12.0, 9.0))
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .paint_style(self_theme_input(theme))
            .interaction(
                Interaction::default()
                    .focusable(true)
                    .cursor(CursorIcon::Pointer)
                    .gestures(GestureSet::NONE.tap())
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .state(VisualState::Hovered, theme.input.hovered.clone())
            .state(VisualState::Focused, theme.input.focused.clone())
            .transition(theme.input.transition.clone())
            .semantics(
                Semantics::new(Role::Button)
                    .label(self.label.clone())
                    .state(trigger_state)
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Expand)
                    .action(SemanticAction::Collapse),
            );
        let overlay = self.open.then(|| self.overlay(theme));
        Element::container(std::iter::once(trigger).chain(overlay))
    }

    fn overlay(&self, theme: &WidgetTheme) -> Element {
        let option_count = self.options.len() as u32;
        let options = self.options.iter().enumerate().map(|(index, option)| {
            let selected = self.selected == Some(index);
            let highlighted = self.highlighted == index;
            let state = SemanticState {
                selected,
                disabled: !option.enabled,
                ..SemanticState::default()
            };
            Element::container([Element::text(option.label.clone())
                .text_style(label_style(if option.enabled {
                    theme.foreground
                } else {
                    theme.muted_foreground
                }))
                .semantic_hidden(true)])
            .keyed(Self::option_key(&self.key, index))
            .width(percent(1.0))
            .padding(argui_ui::sides(10.0, 8.0))
            .background(if highlighted {
                theme.muted
            } else {
                argui_core::Color::TRANSPARENT
            })
            .radius(CornerRadii::all(5.0))
            .interaction(
                Interaction::default()
                    .enabled(option.enabled)
                    .focusable(option.enabled)
                    .cursor(CursorIcon::Pointer)
                    .gestures(GestureSet::NONE.tap())
                    .keyboard_activation(KeyboardActivation::EnterOrSpace),
            )
            .state(
                VisualState::Hovered,
                StateStyle::from_quad(QuadStyle::solid(theme.muted).radius(CornerRadii::all(5.0))),
            )
            .transition(StyleTransition::default())
            .semantics(
                Semantics::new(Role::Option)
                    .label(option.label.clone())
                    .state(state)
                    .position_in_set((index + 1) as u32, option_count)
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            )
        });
        Element::column(options)
            .keyed(Self::list_key(&self.key))
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
            .anchored_to(
                self.key.clone(),
                OverlayPlacement::new(PlacementSide::Bottom)
                    .align(OverlayAlign::Start)
                    .gap(6.0)
                    .margin(10.0),
            )
            .focus_scope(FocusScope::trapped(InitialFocus::Target(
                Self::option_key(&self.key, self.highlighted).into(),
            )))
            .z_index(1_000)
            .semantics(
                Semantics::new(Role::ListBox)
                    .label(self.label.clone())
                    .orientation(argui_ui::Orientation::Vertical),
            )
    }
}

fn self_theme_input(theme: &WidgetTheme) -> PaintStyle {
    theme.input.paint.clone()
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

fn first_enabled(options: &[SelectOption]) -> Option<usize> {
    options.iter().position(|option| option.enabled)
}

fn last_enabled(options: &[SelectOption]) -> Option<usize> {
    options.iter().rposition(|option| option.enabled)
}

fn next_enabled(options: &[SelectOption], current: usize, forward: bool) -> Option<usize> {
    if options.is_empty() {
        return None;
    }
    (1..=options.len()).find_map(|offset| {
        let index = if forward {
            (current + offset) % options.len()
        } else {
            (current + options.len() - offset % options.len()) % options.len()
        };
        options[index].enabled.then_some(index)
    })
}
