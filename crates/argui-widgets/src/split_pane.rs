use argui_core::{Key, KeyState};
use argui_ui::{
    AlignItems, CursorIcon, Element, EventType, GestureCapture, GestureDelivery, GestureKind,
    GesturePhase, GestureSet, HitTestStyle, Interaction, JustifyContent, Orientation, PanAxis,
    PanGesture, Role, SemanticAction, SemanticValue, Semantics, SplitHandlerValue, StateScopeId,
    StylePatch, UiEvent, UiEventKind, UserSelect, ValueHandler, VisualState, length, percent,
    property,
};

use crate::WidgetTheme;

const SPLIT_PANE_SCOPE: StateScopeId = StateScopeId::new("split-pane");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

/// Controlled sizing shared by application panes and docked tools.
#[derive(Clone, Debug)]
pub struct SplitPane {
    key: String,
    pub axis: SplitAxis,
    pub size: f32,
    minimum: f32,
    maximum: f32,
    reset: f32,
    start: Option<f32>,
    trailing: bool,
    change_handlers: Vec<ValueHandler<f32>>,
}

impl SplitPane {
    /// Creates a resizable pane with the requested axis, size and inclusive size limits.
    /// `key` identifies the separator; `size`, `minimum` and `maximum` are logical pixels.
    ///
    /// # Panics
    ///
    /// Panics if bounds are non-finite, `minimum` is negative, `maximum` is below it,
    /// or `size` is non-finite.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        axis: SplitAxis,
        size: f32,
        minimum: f32,
        maximum: f32,
    ) -> Self {
        assert!(minimum.is_finite() && maximum.is_finite() && minimum >= 0.0 && maximum >= minimum);
        assert!(size.is_finite());
        let size = size.clamp(minimum, maximum);
        Self {
            key: key.into(),
            axis,
            size,
            minimum,
            maximum,
            reset: size,
            start: None,
            trailing: false,
            change_handlers: Vec::new(),
        }
    }

    /// Adds a handler receiving the next controlled pane size in logical pixels.
    #[must_use]
    pub fn on_change(mut self, handler: ValueHandler<f32>) -> Self {
        self.change_handlers.push(handler);
        self
    }

    #[must_use]
    /// Places the resizable pane after the separator when `trailing` is true.
    pub fn trailing(mut self, trailing: bool) -> Self {
        self.trailing = trailing;
        self
    }

    /// Applies an event to the desired size. Layout clamping never erases that preference.
    pub fn update(&mut self, event: &UiEvent) -> bool {
        if event.target_key() != Some(self.key.as_str()) {
            return false;
        }
        let mut value = self.size;
        match &event.kind {
            UiEventKind::Gesture(gesture) => {
                let GestureKind::Pan { total, .. } = gesture.kind else {
                    return false;
                };
                if gesture.phase == GesturePhase::Started {
                    self.start = Some(self.size);
                }
                if let Some(start) = self.start {
                    if gesture.phase != GesturePhase::Cancelled {
                        let delta = match self.axis {
                            SplitAxis::Horizontal => total.x,
                            SplitAxis::Vertical => total.y,
                        };
                        value = start + delta * if self.trailing { -1.0 } else { 1.0 };
                    }
                    if matches!(gesture.phase, GesturePhase::Ended | GesturePhase::Cancelled) {
                        self.start = None;
                    }
                }
            }
            UiEventKind::Click(click) if click.count >= 2 => value = self.reset,
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                let step = if input.modifiers.shift { 1.0 } else { 10.0 };
                let direction = if self.trailing { -1.0 } else { 1.0 };
                value = match (&input.key, self.axis) {
                    (Key::ArrowLeft, SplitAxis::Horizontal)
                    | (Key::ArrowUp, SplitAxis::Vertical) => self.size - step * direction,
                    (Key::ArrowRight, SplitAxis::Horizontal)
                    | (Key::ArrowDown, SplitAxis::Vertical) => self.size + step * direction,
                    (Key::Home, _) => self.minimum,
                    (Key::End, _) => self.maximum,
                    _ => return false,
                };
            }
            _ => return false,
        }
        if !value.is_finite() {
            return false;
        }
        value = value.clamp(self.minimum, self.maximum);
        let changed = value != self.size;
        self.size = value;
        changed
    }

    #[must_use]
    /// Returns the pane size for `available` space after preserving `other_minimum` and separator space.
    pub fn effective_size(&self, available: f32, other_minimum: f32) -> f32 {
        self.size.min((available - other_minimum - 6.0).max(0.0))
    }

    #[must_use]
    /// Builds the keyboard- and pointer-resizable separator using `theme`.
    pub fn separator(&self, theme: &WidgetTheme) -> Element {
        let horizontal = self.axis == SplitAxis::Horizontal;
        let line = Element::container([])
            .width(if horizontal {
                length(2.0)
            } else {
                percent(1.0)
            })
            .height(if horizontal {
                percent(1.0)
            } else {
                length(2.0)
            })
            .background(theme.border)
            .when(
                argui_ui::StateSelector::scope(SPLIT_PANE_SCOPE, VisualState::Hovered),
                StylePatch::new().set(property::BackgroundColor, theme.primary),
            )
            .when(
                argui_ui::StateSelector::scope(SPLIT_PANE_SCOPE, VisualState::Pressed),
                StylePatch::new().set(property::BackgroundColor, theme.primary),
            )
            .when(
                argui_ui::StateSelector::scope(SPLIT_PANE_SCOPE, VisualState::FocusVisible),
                StylePatch::new().set(property::BackgroundColor, theme.primary),
            );
        let mut separator = Element::container([line])
            .keyed(self.key.clone())
            .width(if horizontal {
                length(6.0)
            } else {
                percent(1.0)
            })
            .height(if horizontal {
                percent(1.0)
            } else {
                length(6.0)
            })
            .shrink(0.0)
            .align_items(AlignItems::CENTER)
            .justify_content(JustifyContent::CENTER)
            .state_scope(SPLIT_PANE_SCOPE)
            .user_select(UserSelect::None)
            .hit_test(HitTestStyle::default().slop(argui_ui::Sides::length(9.0)))
            .interaction(
                Interaction::default()
                    .focus_policy(argui_ui::FocusPolicy::TabStop)
                    .cursor(if horizontal {
                        CursorIcon::EwResize
                    } else {
                        CursorIcon::NsResize
                    })
                    .gestures(
                        GestureSet::default().pan(
                            PanGesture::default()
                                .axis(if horizontal {
                                    PanAxis::Horizontal
                                } else {
                                    PanAxis::Vertical
                                })
                                .immediate()
                                .capture(GestureCapture::OnPress)
                                .delivery(GestureDelivery::FrameCoalesced),
                        ),
                    ),
            )
            .semantics(
                Semantics::new(Role::Separator)
                    .label("Resize panels")
                    .orientation(if horizontal {
                        Orientation::Vertical
                    } else {
                        Orientation::Horizontal
                    })
                    .value(SemanticValue::Number {
                        value: self.size as f64,
                        minimum: Some(self.minimum as f64),
                        maximum: Some(self.maximum as f64),
                        step: Some(10.0),
                    })
                    .action(SemanticAction::Focus),
            );
        let source = SplitHandlerValue::new(
            self.size,
            self.minimum,
            self.maximum,
            self.reset,
            horizontal,
            self.trailing,
        );
        for handler in &self.change_handlers {
            separator = separator
                .on(handler
                    .direct_listener(EventType::Gesture)
                    .split_handler_value(source))
                .on(handler
                    .direct_listener(EventType::Key)
                    .split_handler_value(source))
                .on(handler
                    .direct_listener(EventType::Click)
                    .split_handler_value(source));
        }
        separator
    }

    #[must_use]
    /// Builds the two panes and separator within the available extent.
    ///
    /// `available` is the size along the split axis; `other_minimum` reserves space for
    /// the flexible pane. `first` and `second` provide the two pane contents.
    pub fn build(
        &self,
        first: Element,
        separator: Element,
        second: Element,
        available: f32,
        other_minimum: f32,
    ) -> Element {
        let size = self.effective_size(available, other_minimum);
        let fixed = |pane: Element| match self.axis {
            SplitAxis::Horizontal => pane.width(length(size)).min_width(length(0.0)).shrink(0.0),
            SplitAxis::Vertical => pane
                .height(length(size))
                .min_height(length(0.0))
                .shrink(0.0),
        };
        let flexible = |pane: Element| {
            pane.grow(1.0)
                .min_width(length(0.0))
                .min_height(length(0.0))
        };
        let children = if self.trailing {
            [flexible(first), separator, fixed(second)]
        } else {
            [fixed(first), separator, flexible(second)]
        };
        match self.axis {
            SplitAxis::Horizontal => Element::row(children),
            SplitAxis::Vertical => Element::column(children),
        }
        .width(percent(1.0))
        .height(percent(1.0))
    }
}
