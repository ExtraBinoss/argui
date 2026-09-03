use argui_core::{Key, KeyState, Point, Rect};
use argui_runtime::LayoutSnapshot;
use argui_ui::{
    CursorIcon, Element, GestureKind, GesturePhase, GestureSet, Interaction, Orientation, Role,
    SemanticAction, SemanticState, SemanticValue, Semantics, StateScopeId, UiEvent, UiEventKind,
    UserSelect,
};

pub const RANGE_SCOPE: StateScopeId = StateScopeId::new("range");

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangeDirection {
    Forward,
    Reverse,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeConfig {
    pub minimum: f32,
    pub maximum: f32,
    pub step: f32,
    pub axis: RangeAxis,
    pub direction: RangeDirection,
    pub detents: Option<RangeDetents>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RangeDetents {
    pub interval: f32,
    pub tolerance: f32,
    pub maximum_velocity: f32,
}

impl RangeDetents {
    #[must_use]
    pub const fn new(interval: f32, tolerance: f32, maximum_velocity: f32) -> Self {
        Self {
            interval,
            tolerance,
            maximum_velocity,
        }
    }

    fn normalized(self) -> Self {
        Self {
            interval: self.interval.abs().max(f32::EPSILON),
            tolerance: self.tolerance.abs(),
            maximum_velocity: self.maximum_velocity.abs(),
        }
    }
}

impl RangeConfig {
    #[must_use]
    pub const fn new(minimum: f32, maximum: f32, step: f32) -> Self {
        Self {
            minimum,
            maximum,
            step,
            axis: RangeAxis::Horizontal,
            direction: RangeDirection::Forward,
            detents: None,
        }
    }

    #[must_use]
    pub const fn axis(mut self, axis: RangeAxis) -> Self {
        self.axis = axis;
        self
    }

    #[must_use]
    pub const fn direction(mut self, direction: RangeDirection) -> Self {
        self.direction = direction;
        self
    }

    #[must_use]
    pub const fn detents(mut self, detents: RangeDetents) -> Self {
        self.detents = Some(detents);
        self
    }

    #[must_use]
    pub fn clamp(self, value: f32) -> f32 {
        let config = self.normalized();
        let stepped =
            ((value - config.minimum) / config.step).round() * config.step + config.minimum;
        stepped.clamp(config.minimum, config.maximum)
    }

    #[must_use]
    pub fn ratio(self, value: f32) -> f32 {
        let config = self.normalized();
        let range = config.maximum - config.minimum;
        if range <= f32::EPSILON {
            return 0.0;
        }
        let ratio = ((config.clamp(value) - config.minimum) / range).clamp(0.0, 1.0);
        match config.direction {
            RangeDirection::Forward => ratio,
            RangeDirection::Reverse => 1.0 - ratio,
        }
    }

    fn normalized(self) -> Self {
        Self {
            minimum: self.minimum.min(self.maximum),
            maximum: self.maximum.max(self.minimum),
            step: self.step.abs().max(f32::EPSILON),
            detents: self.detents.map(RangeDetents::normalized),
            ..self
        }
    }
}

impl Default for RangeConfig {
    fn default() -> Self {
        Self::new(0.0, 100.0, 1.0)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RangePart {
    Root,
    Control,
    Track,
}

#[derive(Clone, Debug)]
pub struct RangeBehavior {
    key: String,
    label: String,
    value: f32,
    config: RangeConfig,
    enabled: bool,
}

impl RangeBehavior {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        value: f32,
        config: RangeConfig,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            value,
            config: config.normalized(),
            enabled: true,
        }
    }

    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }

    #[must_use]
    pub fn ratio(&self) -> f32 {
        self.config.ratio(self.value)
    }

    #[must_use]
    pub const fn config(&self) -> RangeConfig {
        self.config
    }

    #[must_use]
    pub fn track_key(&self) -> String {
        format!("{}::track", self.key)
    }

    #[must_use]
    pub fn decorate(&self, part: RangePart, element: Element) -> Element {
        match part {
            RangePart::Track => element.keyed(self.track_key()).semantic_hidden(true),
            RangePart::Root => element.state_scope(RANGE_SCOPE),
            RangePart::Control => {
                let cursor = match self.config.axis {
                    RangeAxis::Horizontal => CursorIcon::EwResize,
                    RangeAxis::Vertical => CursorIcon::NsResize,
                };
                element
                    .keyed(self.key.clone())
                    .user_select(UserSelect::None)
                    .interaction(
                        Interaction::default()
                            .enabled(self.enabled)
                            .focusable(self.enabled)
                            .cursor(cursor)
                            .gestures(GestureSet::NONE.pan_immediate()),
                    )
                    .semantics(
                        Semantics::new(Role::Slider)
                            .label(self.label.clone())
                            .orientation(match self.config.axis {
                                RangeAxis::Horizontal => Orientation::Horizontal,
                                RangeAxis::Vertical => Orientation::Vertical,
                            })
                            .value(SemanticValue::Number {
                                value: f64::from(self.config.clamp(self.value)),
                                minimum: Some(f64::from(self.config.minimum)),
                                maximum: Some(f64::from(self.config.maximum)),
                                step: Some(f64::from(self.config.step)),
                            })
                            .state(SemanticState {
                                disabled: !self.enabled,
                                ..SemanticState::default()
                            })
                            .action(SemanticAction::Focus)
                            .action(SemanticAction::Increment)
                            .action(SemanticAction::Decrement)
                            .action(SemanticAction::SetValue),
                    )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RangeAction {
    Begin(f32),
    Update(f32),
    Commit(f32),
    Cancel(f32),
}

impl RangeAction {
    #[must_use]
    pub const fn value(self) -> f32 {
        match self {
            Self::Begin(value)
            | Self::Update(value)
            | Self::Commit(value)
            | Self::Cancel(value) => value,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct RangeState {
    track: Option<Rect>,
    drag_start: Option<f32>,
    keyboard_active: bool,
}

impl RangeState {
    pub fn layout_changed(&mut self, layout: &LayoutSnapshot, behavior: &RangeBehavior) {
        self.track = layout.bounds(&behavior.track_key());
    }

    #[must_use]
    pub fn update(&mut self, event: &UiEvent, behavior: &RangeBehavior) -> Option<RangeAction> {
        if !behavior.enabled || event.key.as_deref() != Some(behavior.key.as_str()) {
            return None;
        }
        let config = behavior.config;
        let value = config.clamp(behavior.value);
        match &event.kind {
            UiEventKind::KeyInput(input) => self.key(input, value, config),
            UiEventKind::SemanticAction { action, value: set } => {
                self.semantic(*action, set.as_ref(), value, config)
            }
            UiEventKind::Gesture(gesture) => match gesture.kind {
                GestureKind::Tap { position } => self
                    .value_at(position, 0.0, config)
                    .map(RangeAction::Commit),
                GestureKind::Pan {
                    position, velocity, ..
                } => {
                    let axis_velocity = match config.axis {
                        RangeAxis::Horizontal => velocity.x.abs(),
                        RangeAxis::Vertical => velocity.y.abs(),
                    };
                    self.pan(gesture.phase, position, axis_velocity, value, config)
                }
                _ => None,
            },
            _ => None,
        }
    }

    fn key(
        &mut self,
        input: &argui_core::KeyInput,
        value: f32,
        config: RangeConfig,
    ) -> Option<RangeAction> {
        let delta = match input.key {
            Key::ArrowLeft | Key::ArrowDown => -config.step,
            Key::ArrowRight | Key::ArrowUp => config.step,
            Key::Home => config.minimum - value,
            Key::End => config.maximum - value,
            _ => return None,
        };
        match input.state {
            KeyState::Pressed => {
                let next = config.clamp(value + delta);
                if self.keyboard_active || input.repeat {
                    Some(RangeAction::Update(next))
                } else {
                    self.keyboard_active = true;
                    Some(RangeAction::Begin(next))
                }
            }
            KeyState::Released if self.keyboard_active => {
                self.keyboard_active = false;
                Some(RangeAction::Commit(value))
            }
            KeyState::Released => None,
        }
    }

    fn semantic(
        &self,
        action: SemanticAction,
        set: Option<&SemanticValue>,
        value: f32,
        config: RangeConfig,
    ) -> Option<RangeAction> {
        let next = match action {
            SemanticAction::Increment => config.clamp(value + config.step),
            SemanticAction::Decrement => config.clamp(value - config.step),
            SemanticAction::SetValue => match set {
                Some(SemanticValue::Number { value, .. }) => config.clamp(*value as f32),
                _ => return None,
            },
            _ => return None,
        };
        Some(RangeAction::Commit(next))
    }

    fn pan(
        &mut self,
        phase: GesturePhase,
        position: Point,
        velocity: f32,
        value: f32,
        config: RangeConfig,
    ) -> Option<RangeAction> {
        match phase {
            GesturePhase::Started => {
                self.drag_start = Some(value);
                self.value_at(position, velocity, config)
                    .map(RangeAction::Begin)
            }
            GesturePhase::Changed => self
                .value_at(position, velocity, config)
                .map(RangeAction::Update),
            GesturePhase::Ended => {
                let next = self.value_at(position, velocity, config)?;
                self.drag_start = None;
                Some(RangeAction::Commit(next))
            }
            GesturePhase::Cancelled => {
                let start = self.drag_start?;
                self.drag_start = None;
                Some(RangeAction::Cancel(start))
            }
        }
    }

    fn value_at(self, point: Point, velocity: f32, config: RangeConfig) -> Option<f32> {
        let track = self.track?;
        let ratio = match config.axis {
            RangeAxis::Horizontal if track.size.width > f32::EPSILON => {
                (point.x - track.origin.x) / track.size.width
            }
            RangeAxis::Vertical if track.size.height > f32::EPSILON => {
                (point.y - track.origin.y) / track.size.height
            }
            _ => return None,
        };
        Some(value_from_ratio(ratio, velocity, config))
    }
}

fn value_from_ratio(ratio: f32, velocity: f32, config: RangeConfig) -> f32 {
    let ratio = ratio.clamp(0.0, 1.0);
    let ratio = match config.direction {
        RangeDirection::Forward => ratio,
        RangeDirection::Reverse => 1.0 - ratio,
    };
    let value = config.clamp(config.minimum + ratio * (config.maximum - config.minimum));
    let Some(detents) = config.detents else {
        return value;
    };
    if velocity > detents.maximum_velocity {
        return value;
    }
    let detent =
        ((value - config.minimum) / detents.interval).round() * detents.interval + config.minimum;
    if (value - detent).abs() <= detents.tolerance {
        config.clamp(detent)
    } else {
        value
    }
}
