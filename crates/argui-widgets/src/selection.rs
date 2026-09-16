#[cfg(feature = "switch")]
use argui_core::Transform2D;
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::{TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Dimensions, Display, Element, EventType, FlexDirection, JustifyContent,
    LayoutStyle, StylePatch, StyleTransition, ValueHandler, VisualState, auto, length,
};

use crate::WidgetTheme;
#[cfg(feature = "radio-group")]
use crate::{RadioGroupBehavior, RadioGroupPart};
#[cfg(feature = "switch")]
use crate::{TOGGLE_CHECKED, TOGGLE_SCOPE};
#[cfg(any(feature = "checkbox", feature = "switch"))]
use crate::{ToggleBehavior, TogglePart};
#[cfg(feature = "radio-group")]
use argui_ui::Orientation;
#[cfg(any(feature = "checkbox", feature = "switch"))]
use argui_ui::Role;
#[cfg(feature = "switch")]
use argui_ui::{StateSelector, property};

#[cfg(feature = "checkbox")]
#[derive(Clone, Debug)]
pub struct Checkbox {
    key: String,
    label: String,
    checked: argui_ui::CheckedState,
    enabled: bool,
    indicator: Option<Element>,
    change_handlers: Vec<ValueHandler<argui_ui::CheckedState>>,
}

#[cfg(feature = "checkbox")]
impl Checkbox {
    /// Creates a checkbox with controlled `checked` state and accessible `label`.
    /// `key` identifies the control.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        checked: argui_ui::CheckedState,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            checked,
            enabled: true,
            indicator: None,
            change_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets whether the checkbox can be changed.
    /// `enabled` controls its interaction availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    /// Replaces the default checked/mixed mark with `indicator`.
    pub fn indicator(mut self, indicator: Element) -> Self {
        self.indicator = Some(indicator);
        self
    }

    /// Adds a callback receiving the next checked state.
    ///
    /// `handler` is normally created with `Context::value_callback`.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<argui_ui::CheckedState>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the checkbox using `theme` for its track and mark colors.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let box_color = if self.checked != argui_ui::CheckedState::Unchecked {
            theme.primary
        } else {
            theme.card
        };
        let mark = self.indicator.unwrap_or_else(|| {
            Element::container([])
                .width(length(if self.checked == argui_ui::CheckedState::Mixed {
                    10.0
                } else {
                    7.0
                }))
                .height(length(if self.checked == argui_ui::CheckedState::Mixed {
                    2.0
                } else {
                    7.0
                }))
                .background(theme.primary_foreground)
                .radius(CornerRadii::all(2.0))
        });
        let box_element =
            Element::container((self.checked != argui_ui::CheckedState::Unchecked).then_some(mark))
                .width(length(18.0))
                .height(length(18.0))
                .display(Display::Flex)
                .align_items(AlignItems::CENTER)
                .justify_content(JustifyContent::CENTER)
                .background(box_color)
                .border(Border::all(
                    1.0,
                    if self.checked != argui_ui::CheckedState::Unchecked {
                        theme.primary
                    } else {
                        theme.border
                    },
                ))
                .radius(CornerRadii::all(5.0))
                .semantic_hidden(true);
        let behavior = ToggleBehavior::new(&self.key, &self.label, Role::CheckBox, self.checked)
            .enabled(self.enabled);
        let next = self.checked.toggled();
        let mut root = control_row(behavior, self.label, box_element, theme);
        for handler in self.change_handlers {
            root = root.on(handler.direct_listener_value(EventType::Click, next));
        }
        root
    }
}

#[cfg(feature = "switch")]
#[derive(Clone, Debug)]
pub struct Switch {
    key: String,
    label: String,
    checked: bool,
    enabled: bool,
    change_handlers: Vec<ValueHandler<bool>>,
}

#[cfg(feature = "switch")]
impl Switch {
    /// Creates a controlled switch with accessible `label` and initial `checked` state.
    /// `key` identifies the control.
    #[must_use]
    pub fn new(key: impl Into<String>, label: impl Into<String>, checked: bool) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            checked,
            enabled: true,
            change_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets whether the switch can be changed.
    /// `enabled` controls its interaction availability.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    /// Adds a callback receiving the next switch state.
    ///
    /// `handler` is normally created with `Context::value_callback`.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the switch using `theme` for its track and thumb colors.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let thumb_key = format!("{}::thumb", self.key);
        let thumb = Element::container([])
            .keyed(thumb_key)
            .width(length(18.0))
            .height(length(18.0))
            .background(if self.checked {
                theme.primary_foreground
            } else {
                theme.switch_thumb
            })
            .radius(CornerRadii::all(999.0))
            .when(
                StateSelector::scope(TOGGLE_SCOPE, TOGGLE_CHECKED),
                StylePatch::new().set(
                    property::Transform,
                    Transform2D::IDENTITY.translate(18.0, 0.0),
                ),
            )
            .transition(StyleTransition::default());
        let track = Element::row([thumb])
            .width(length(40.0))
            .height(length(22.0))
            .padding(argui_ui::Sides::length(2.0))
            .justify_content(JustifyContent::START)
            .background(if self.checked {
                theme.primary
            } else {
                theme.switch_unchecked
            })
            .radius(CornerRadii::all(999.0))
            .transition(StyleTransition::default())
            .semantic_hidden(true);
        let behavior = ToggleBehavior::new(
            &self.key,
            &self.label,
            Role::Switch,
            if self.checked {
                argui_ui::CheckedState::Checked
            } else {
                argui_ui::CheckedState::Unchecked
            },
        )
        .enabled(self.enabled);
        let mut root = control_row(behavior, self.label, track, theme);
        for handler in self.change_handlers {
            root = root.on(handler.direct_listener_value(EventType::Click, !self.checked));
        }
        root
    }
}

#[cfg(feature = "radio-group")]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RadioOption {
    pub label: String,
    pub enabled: bool,
}

#[cfg(feature = "radio-group")]
impl RadioOption {
    /// Creates an enabled radio option with the given label.
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            enabled: true,
        }
    }

    #[must_use]
    /// Sets whether this option can be selected.
    /// `enabled` controls whether it can be selected.
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

#[cfg(feature = "radio-group")]
#[derive(Clone, Debug)]
pub struct RadioGroup {
    key: String,
    label: String,
    options: Vec<RadioOption>,
    selected: Option<usize>,
    orientation: Orientation,
    select_handlers: Vec<ValueHandler<usize>>,
}

#[cfg(feature = "radio-group")]
impl RadioGroup {
    /// Creates a radio group with options in display order and an optional selected index.
    /// `key` identifies the group and `label` names it accessibly.
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
            select_handlers: Vec::new(),
        }
    }

    #[must_use]
    /// Sets the direction in which options are laid out.
    /// `orientation` determines the layout and keyboard navigation axis.
    pub const fn orientation(mut self, orientation: Orientation) -> Self {
        self.orientation = orientation;
        self
    }

    /// Adds a callback receiving the selected source option index.
    ///
    /// `handler` is normally created with `Context::value_callback`. Disabled
    /// options never deliver the handler.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<usize>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    #[must_use]
    /// Builds the radio group using `theme` for option styling.
    pub fn build(self, theme: &WidgetTheme) -> Element {
        let behavior = RadioGroupBehavior::new(
            &self.key,
            &self.label,
            self.options
                .iter()
                .map(|option| (option.label.clone(), option.enabled)),
            self.selected,
        )
        .orientation(self.orientation);
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
            let mut option = behavior.decorate(
                RadioGroupPart::Option(index),
                control_row_content(
                    option.label.clone(),
                    behavior.decorate(RadioGroupPart::Indicator, dot),
                    option.enabled,
                    theme,
                ),
            );
            for handler in self.select_handlers.iter().copied() {
                option = option.on(handler.direct_listener_value(EventType::Click, index));
            }
            option
        });
        let content = match self.orientation {
            Orientation::Horizontal => Element::row(children.collect::<Vec<_>>()),
            Orientation::Vertical => Element::column(children.collect::<Vec<_>>()),
        };
        behavior.decorate(RadioGroupPart::Root, content.gap(10.0))
    }
}

#[cfg(any(feature = "checkbox", feature = "switch"))]
fn control_row(
    behavior: ToggleBehavior,
    label: String,
    control: Element,
    theme: &WidgetTheme,
) -> Element {
    let enabled = behavior.is_enabled();
    let control = behavior.decorate(TogglePart::Indicator, control);
    behavior.decorate(
        TogglePart::Root,
        control_row_content(label, control, enabled, theme),
    )
}

fn control_row_content(
    label: String,
    control: Element,
    enabled: bool,
    theme: &WidgetTheme,
) -> Element {
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
    let hovered = StylePatch::from_quad(QuadStyle::solid(theme.muted));
    let label = Element::text(label)
        .text_style(label_style)
        .semantic_hidden(true);
    Element::row([control, label])
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
        .when(VisualState::Hovered, hovered)
        .when(
            VisualState::FocusVisible,
            QuadStyle::solid(argui_core::Color::TRANSPARENT)
                .border(Border::all(2.0, theme.ring))
                .radius(CornerRadii::all(6.0))
                .into(),
        )
        .transition(crate::theme::instant_hover(
            StyleTransition::default(),
            VisualState::Hovered.into(),
        ))
}
