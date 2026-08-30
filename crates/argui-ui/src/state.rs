use argui_animation::Transition;
use argui_core::{Color, Point, Transform2D};
use argui_paint::{Border, EffectId, Fill, QuadStyle};

use crate::BindingImpact;
use crate::binding::{GradientPointTarget, LayoutTarget};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VisualState {
    Focused,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VisualStates(u8);

impl VisualStates {
    pub const NONE: Self = Self(0);

    #[must_use]
    pub const fn contains(self, state: VisualState) -> bool {
        self.0 & state.bit() != 0
    }

    pub(crate) fn insert(&mut self, state: VisualState) {
        self.0 |= state.bit();
    }
}

impl VisualState {
    const fn bit(self) -> u8 {
        match self {
            Self::Focused => 1,
            Self::Hovered => 2,
            Self::Pressed => 4,
            Self::Disabled => 8,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct EffectPropertyKey {
    pub effect: EffectId,
    pub parameter: &'static str,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum PropertyKey {
    Transform,
    Background,
    BackgroundColor,
    Border,
    BorderColor,
    BorderWidths,
    CornerRadii,
    Opacity,
    GradientPoint(GradientPointTarget),
    GradientStopOffset(usize),
    GradientStopColor(usize),
    LayerOpacity,
    LayerMaskRadii,
    ShadowOffset(usize),
    ShadowBlur(usize),
    ShadowSpread(usize),
    ShadowColor(usize),
    Layout(LayoutTarget),
    Scroll,
    EffectF32(EffectPropertyKey),
    EffectLogicalPixels(EffectPropertyKey),
    EffectVec2(EffectPropertyKey),
    EffectVec3(EffectPropertyKey),
    EffectVec4(EffectPropertyKey),
    EffectMat3(EffectPropertyKey),
    EffectMat4(EffectPropertyKey),
    EffectColor(EffectPropertyKey),
}

impl PropertyKey {
    #[must_use]
    pub const fn impact(self) -> BindingImpact {
        match self {
            Self::Layout(_) => BindingImpact::Layout,
            Self::Scroll => BindingImpact::Scroll,
            _ => BindingImpact::Paint,
        }
    }

    pub(crate) const fn is_quad(self) -> bool {
        matches!(
            self,
            Self::Background
                | Self::BackgroundColor
                | Self::Border
                | Self::BorderColor
                | Self::BorderWidths
                | Self::CornerRadii
                | Self::Opacity
                | Self::GradientPoint(_)
                | Self::GradientStopOffset(_)
                | Self::GradientStopColor(_)
        )
    }
}

#[derive(Clone, Debug, PartialEq)]
#[doc(hidden)]
pub enum StateValue {
    Transform(Transform2D),
    Background(Option<Fill>),
    BackgroundColor(Color),
    Border(Option<Border>),
    BorderColor(Color),
    BorderWidths([f32; 4]),
    CornerRadii([f32; 4]),
    Opacity(f32),
    Point(Point),
    F32(f32),
    Color(Color),
    Vec2([f32; 2]),
    Vec3([f32; 3]),
    Vec4([f32; 4]),
    Mat3([f32; 9]),
    Mat4([f32; 16]),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StatePropertyValue {
    pub key: PropertyKey,
    #[doc(hidden)]
    pub value: StateValue,
}

pub trait StateProperty: crate::binding::private::Sealed {
    type Value;

    #[doc(hidden)]
    fn into_state_value(self, value: Self::Value) -> StatePropertyValue;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StateStyle {
    values: Vec<StatePropertyValue>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ElementStateStyles {
    focused: Option<StateStyle>,
    hovered: Option<StateStyle>,
    pressed: Option<StateStyle>,
    disabled: Option<StateStyle>,
}

impl ElementStateStyles {
    pub(crate) const fn new() -> Self {
        Self {
            focused: None,
            hovered: None,
            pressed: None,
            disabled: None,
        }
    }

    pub(crate) fn set(&mut self, state: VisualState, style: StateStyle) {
        *match state {
            VisualState::Focused => &mut self.focused,
            VisualState::Hovered => &mut self.hovered,
            VisualState::Pressed => &mut self.pressed,
            VisualState::Disabled => &mut self.disabled,
        } = Some(style);
    }

    pub(crate) fn get(&self, state: VisualState) -> Option<&StateStyle> {
        match state {
            VisualState::Focused => self.focused.as_ref(),
            VisualState::Hovered => self.hovered.as_ref(),
            VisualState::Pressed => self.pressed.as_ref(),
            VisualState::Disabled => self.disabled.as_ref(),
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.focused.is_none()
            && self.hovered.is_none()
            && self.pressed.is_none()
            && self.disabled.is_none()
    }

    pub(crate) fn impact(&self) -> Option<BindingImpact> {
        [
            self.focused.as_ref(),
            self.hovered.as_ref(),
            self.pressed.as_ref(),
            self.disabled.as_ref(),
        ]
        .into_iter()
        .flatten()
        .flat_map(StateStyle::values)
        .map(|value| value.key.impact())
        .max_by_key(|impact| match impact {
            BindingImpact::Paint => 0,
            BindingImpact::Scroll => 1,
            BindingImpact::Layout => 2,
        })
    }

    pub(crate) fn contains(&self, key: PropertyKey) -> bool {
        [
            self.focused.as_ref(),
            self.hovered.as_ref(),
            self.pressed.as_ref(),
            self.disabled.as_ref(),
        ]
        .into_iter()
        .flatten()
        .flat_map(StateStyle::values)
        .any(|value| value.key == key)
    }
}

impl StateStyle {
    #[must_use]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    #[must_use]
    pub fn set<P: StateProperty>(mut self, property: P, value: P::Value) -> Self {
        let value = property.into_state_value(value);
        if let Some(existing) = self.values.iter_mut().find(|item| item.key == value.key) {
            *existing = value;
        } else {
            self.values.push(value);
        }
        self
    }

    #[must_use]
    pub fn from_quad(style: QuadStyle) -> Self {
        let mut state = Self::new()
            .set(crate::property::CornerRadii, style.radii.as_array())
            .set(crate::property::Opacity, style.opacity);
        state = match style.background {
            Some(Fill::Solid(color)) => state.set(crate::property::BackgroundColor, color),
            background => state.set(crate::property::Background, background),
        };
        match style.border {
            Some(border) => state
                .set(crate::property::BorderColor, border.color)
                .set(crate::property::BorderWidths, border.widths.as_array()),
            None => state.set(crate::property::Border, None),
        }
    }

    pub(crate) fn values(&self) -> &[StatePropertyValue] {
        &self.values
    }
}

impl From<QuadStyle> for StateStyle {
    fn from(value: QuadStyle) -> Self {
        Self::from_quad(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransitionDirection {
    Enter(VisualState),
    Exit(VisualState),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionRule {
    pub property: Option<PropertyKey>,
    pub direction: Option<TransitionDirection>,
    pub transition: Transition,
}

impl TransitionRule {
    #[must_use]
    pub const fn new(transition: Transition) -> Self {
        Self {
            property: None,
            direction: None,
            transition,
        }
    }

    #[must_use]
    pub const fn property(mut self, property: PropertyKey) -> Self {
        self.property = Some(property);
        self
    }

    #[must_use]
    pub const fn direction(mut self, direction: TransitionDirection) -> Self {
        self.direction = Some(direction);
        self
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleTransition {
    pub default: Transition,
    rules: Vec<TransitionRule>,
}

impl StyleTransition {
    #[must_use]
    pub const fn new(default: Transition) -> Self {
        Self {
            default,
            rules: Vec::new(),
        }
    }

    #[must_use]
    pub fn rule(mut self, rule: TransitionRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub(crate) fn resolve(
        &self,
        property: PropertyKey,
        direction: Option<TransitionDirection>,
    ) -> &Transition {
        self.rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| {
                rule.property.is_none_or(|candidate| candidate == property)
                    && rule
                        .direction
                        .is_none_or(|candidate| Some(candidate) == direction)
            })
            .max_by_key(|(index, rule)| {
                (
                    u8::from(rule.property.is_some()) + u8::from(rule.direction.is_some()),
                    *index,
                )
            })
            .map_or(&self.default, |(_, rule)| &rule.transition)
    }
}

impl Default for StyleTransition {
    fn default() -> Self {
        Self::new(Transition::tween(argui_animation::Tween::new(
            argui_animation::Duration::from_millis(120),
        )))
    }
}
