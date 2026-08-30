use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, CursorIcon, Dimensions, Display, Element, FlexDirection, GestureSet, Interaction,
    JustifyContent, KeyboardActivation, LayoutStyle, Orientation, Role, SemanticAction,
    SemanticState, Semantics, StateStyle, StyleTransition, UiEvent, UiEventKind, VisualState, auto,
    length,
};

use crate::WidgetTheme;

#[derive(Clone, Debug)]
pub struct Checkbox {
    key: String,
    label: String,
    checked: bool,
    enabled: bool,
    indicator: Option<Element>,
}

impl Checkbox {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            checked,
            enabled: true,
            indicator: None,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn indicator(mut self, indicator: Element) -> Self {
        self.indicator = Some(indicator);
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let box_color = if self.checked {
            theme.primary
        } else {
            theme.card
        };
        let mark = self.indicator.unwrap_or_else(|| {
            Element::container([])
                .width(length(7.0))
                .height(length(7.0))
                .background(theme.primary_foreground)
                .radius(CornerRadii::all(2.0))
        });
        let box_element = Element::container(self.checked.then_some(mark))
            .width(length(18.0))
            .height(length(18.0))
            .display(Display::Flex)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .background(box_color)
            .border(Border::all(
                1.0,
                if self.checked {
                    theme.primary
                } else {
                    theme.border
                },
            ))
            .radius(CornerRadii::all(5.0))
            .semantic_hidden(true);
        let state = SemanticState {
            checked: Some(self.checked),
            disabled: !self.enabled,
            ..SemanticState::default()
        };
        control_row(
            self.key,
            self.label,
            box_element,
            self.enabled,
            Semantics::new(Role::CheckBox).state(state),
            theme,
        )
    }
}

#[derive(Clone, Debug)]
pub struct Switch {
    key: String,
    label: String,
    checked: bool,
    enabled: bool,
}

impl Switch {
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            checked,
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let thumb = Element::container([])
            .width(length(18.0))
            .height(length(18.0))
            .background(if self.checked {
                theme.primary_foreground
            } else {
                theme.foreground
            })
            .radius(CornerRadii::all(999.0));
        let track = Element::row([thumb])
            .width(length(38.0))
            .height(length(22.0))
            .padding(argui_ui::Sides::length(2.0))
            .justify_content(if self.checked {
                JustifyContent::END
            } else {
                JustifyContent::START
            })
            .background(if self.checked {
                theme.primary
            } else {
                theme.muted
            })
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(999.0))
            .semantic_hidden(true);
        let state = SemanticState {
            checked: Some(self.checked),
            disabled: !self.enabled,
            ..SemanticState::default()
        };
        control_row(
            self.key,
            self.label,
            track,
            self.enabled,
            Semantics::new(Role::Switch).state(state),
            theme,
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioOption {
    pub label: String,
    pub enabled: bool,
}

impl RadioOption {
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
pub struct RadioGroup {
    key: String,
    label: String,
    options: Vec<RadioOption>,
    selected: Option<usize>,
    orientation: Orientation,
}

impl RadioGroup {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        options: impl IntoIterator<Item = RadioOption>,
        selected: Option<usize>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            options: options.into_iter().collect(),
            selected,
            orientation: Orientation::Vertical,
        }
    }

    #[must_use]
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    #[must_use]
    pub fn option_key(key: &str, index: usize) -> String {
        format!("{key}::option::{index}")
    }

    #[must_use]
    pub fn selection(key: &str, event: &UiEvent) -> Option<usize> {
        if !matches!(event.kind, UiEventKind::Clicked) {
            return None;
        }
        event
            .key
            .as_deref()?
            .strip_prefix(&format!("{key}::option::"))?
            .parse()
            .ok()
    }

    #[must_use]
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let children = self.options.iter().enumerate().map(|(index, option)| {
            let selected = self.selected == Some(index);
            let dot = Element::container(selected.then(|| {
                Element::container([])
                    .width(length(8.0))
                    .height(length(8.0))
                    .background(theme.primary)
                    .radius(CornerRadii::all(999.0))
            }))
            .width(length(18.0))
            .height(length(18.0))
            .display(Display::Flex)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .border(Border::all(
                1.5,
                if selected {
                    theme.primary
                } else {
                    theme.border
                },
            ))
            .radius(CornerRadii::all(999.0));
            let state = SemanticState {
                checked: Some(selected),
                disabled: !option.enabled,
                ..SemanticState::default()
            };
            control_row(
                Self::option_key(&self.key, index),
                option.label.clone(),
                dot,
                option.enabled,
                Semantics::new(Role::RadioButton)
                    .state(state)
                    .position_in_set((index + 1) as u32, self.options.len() as u32),
                theme,
            )
        });
        let content = match self.orientation {
            Orientation::Horizontal => Element::row(children.collect::<Vec<_>>()),
            Orientation::Vertical => Element::column(children.collect::<Vec<_>>()),
        };
        content.gap(10.0).semantics(
            Semantics::new(Role::Group)
                .label(self.label)
                .orientation(self.orientation),
        )
    }
}

fn control_row(
    key: String,
    label: String,
    control: Element,
    enabled: bool,
    semantics: Semantics,
    theme: &WidgetTheme,
) -> Element {
    let interaction = Interaction::default()
        .enabled(enabled)
        .focusable(enabled)
        .cursor(CursorIcon::Pointer)
        .gestures(GestureSet::NONE.tap())
        .keyboard_activation(KeyboardActivation::EnterOrSpace);
    let label_style = TextStyle {
        font_size: 14.0,
        line_height: 20.0,
        color: if enabled {
            theme.foreground
        } else {
            theme.muted_foreground
        },
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let hovered = StateStyle::from_quad(QuadStyle::solid(theme.muted));
    Element::row([
        control,
        Element::text(label.clone())
            .text_style(label_style)
            .semantic_hidden(true),
    ])
    .keyed(key)
    .layout_style(LayoutStyle {
        display: Display::Flex,
        flex_direction: FlexDirection::Row,
        size: Dimensions {
            width: auto(),
            height: auto(),
        },
        padding: argui_ui::sides(6.0, 5.0),
        align_items: Some(AlignItems::CENTER),
        gap: Dimensions::length(9.0),
        ..LayoutStyle::default()
    })
    .paint_style(PaintStyle::new(
        QuadStyle::solid(argui_core::Color::TRANSPARENT).radius(CornerRadii::all(6.0)),
    ))
    .interaction(interaction)
    .state(VisualState::Hovered, hovered)
    .state(
        VisualState::Focused,
        QuadStyle::solid(argui_core::Color::TRANSPARENT)
            .border(Border::all(2.0, theme.ring))
            .radius(CornerRadii::all(6.0))
            .into(),
    )
    .transition(StyleTransition::default())
    .semantics(
        semantics
            .label(label)
            .action(SemanticAction::Click)
            .action(SemanticAction::Focus),
    )
}
