use argui_animation::Transition;
use argui_core::{Color, Point, Transform2D};
use argui_paint::{Border, EffectId, Fill, QuadStyle};

use crate::binding::{GradientPointTarget, LayoutTarget};
use crate::{BindingImpact, ContainerQuery, Element};

impl Element {
    /// Returns whether state or conditional styles have an active animation.
    #[must_use]
    pub fn has_state_animation(&self) -> bool {
        self.style_transition.is_some() || !self.conditional_styles.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum VisualState {
    Focused,
    FocusVisible,
    Hovered,
    Pressed,
    Disabled,
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StateName(&'static str);

impl StateName {
    /// Creates an application-defined visual state name.
    ///
    /// * `name` — stable name used by state selectors.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the stable name of this state.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StateScopeId(&'static str);

impl StateScopeId {
    /// Creates an identifier for a named state scope.
    ///
    /// * `name` — stable application-defined scope name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Returns the stable name of this scope.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum State {
    Visual(VisualState),
    Named(StateName),
}

impl From<VisualState> for State {
    fn from(value: VisualState) -> Self {
        Self::Visual(value)
    }
}

impl From<StateName> for State {
    fn from(value: StateName) -> Self {
        Self::Named(value)
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StateSelector {
    Own(State),
    Scope { scope: StateScopeId, state: State },
}

impl StateSelector {
    /// Selects a state on the element where the style condition is declared.
    ///
    /// * `state` — visual or application-defined state to match.
    #[must_use]
    pub const fn own(state: State) -> Self {
        Self::Own(state)
    }

    /// Selects a state on an element in the named scope.
    ///
    /// * `scope` — state scope to search.
    /// * `state` — visual or application-defined state to match.
    #[must_use]
    pub fn scope(scope: StateScopeId, state: impl Into<State>) -> Self {
        Self::Scope {
            scope,
            state: state.into(),
        }
    }
}

impl From<VisualState> for StateSelector {
    fn from(value: VisualState) -> Self {
        Self::Own(State::Visual(value))
    }
}

impl From<StateName> for StateSelector {
    fn from(value: StateName) -> Self {
        Self::Own(State::Named(value))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StyleCondition {
    State(StateSelector),
    Container(ContainerQuery),
    All(Vec<Self>),
    Any(Vec<Self>),
    Not(Box<Self>),
}

impl StyleCondition {
    /// Creates a condition that matches a state selector.
    ///
    /// * `selector` — state and scope to test.
    #[must_use]
    pub fn state(selector: impl Into<StateSelector>) -> Self {
        Self::State(selector.into())
    }

    /// Creates a condition that matches a container query.
    ///
    /// * `query` — container size or orientation condition to test.
    #[must_use]
    pub const fn container(query: ContainerQuery) -> Self {
        Self::Container(query)
    }

    /// Creates a condition that matches when every child condition matches.
    ///
    /// * `conditions` — conditions combined with logical AND.
    #[must_use]
    pub fn all(conditions: impl IntoIterator<Item = Self>) -> Self {
        Self::All(conditions.into_iter().collect())
    }

    /// Creates a condition that matches when any child condition matches.
    ///
    /// * `conditions` — conditions combined with logical OR.
    #[must_use]
    pub fn any(conditions: impl IntoIterator<Item = Self>) -> Self {
        Self::Any(conditions.into_iter().collect())
    }

    pub(crate) fn matches(
        &self,
        state: &impl Fn(StateSelector) -> bool,
        container: &impl Fn(ContainerQuery) -> bool,
    ) -> bool {
        match self {
            Self::State(selector) => state(*selector),
            Self::Container(query) => container(*query),
            Self::All(conditions) => conditions.iter().all(|item| item.matches(state, container)),
            Self::Any(conditions) => conditions.iter().any(|item| item.matches(state, container)),
            Self::Not(condition) => !condition.matches(state, container),
        }
    }

    pub(crate) fn has_container_query(&self) -> bool {
        match self {
            Self::Container(_) => true,
            Self::All(conditions) | Self::Any(conditions) => {
                conditions.iter().any(Self::has_container_query)
            }
            Self::Not(condition) => condition.has_container_query(),
            Self::State(_) => false,
        }
    }
}

impl From<StateSelector> for StyleCondition {
    fn from(value: StateSelector) -> Self {
        Self::State(value)
    }
}

impl From<VisualState> for StyleCondition {
    fn from(value: VisualState) -> Self {
        Self::state(value)
    }
}

impl From<StateName> for StyleCondition {
    fn from(value: StateName) -> Self {
        Self::state(value)
    }
}

impl From<ContainerQuery> for StyleCondition {
    fn from(value: ContainerQuery) -> Self {
        Self::Container(value)
    }
}

impl std::ops::Not for StyleCondition {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self::Not(Box::new(self))
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct VisualStates(u8);

impl VisualStates {
    /// A set containing no active visual states.
    pub const NONE: Self = Self(0);

    /// Returns whether the given visual state is present.
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
            Self::FocusVisible => 2,
            Self::Hovered => 4,
            Self::Pressed => 8,
            Self::Disabled => 16,
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
    TextColor,
    VectorColor,
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
    LayoutStyle,
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
    /// Returns the strongest invalidation required when this property changes.
    #[must_use]
    pub const fn impact(self) -> BindingImpact {
        match self {
            Self::Layout(_) | Self::LayoutStyle => BindingImpact::Layout,
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
    LayoutStyle(Box<crate::LayoutStyle>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct StylePropertyValue {
    pub key: PropertyKey,
    #[doc(hidden)]
    pub value: StateValue,
}

/// Sealed mapping from a public style-property marker to its value type.
pub trait StyleProperty: crate::binding::private::Sealed {
    /// Value accepted by this property marker.
    type Value;

    #[doc(hidden)]
    /// Converts the typed property value to the internal representation.
    fn into_state_value(self, value: Self::Value) -> StylePropertyValue;
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct StylePatch {
    values: Vec<StylePropertyValue>,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct StyleRule {
    pub condition: StyleCondition,
    pub style: StylePatch,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ConditionalStyles {
    rules: Vec<StyleRule>,
}

impl ConditionalStyles {
    pub(crate) fn remove(&mut self, properties: &[PropertyKey]) {
        for rule in &mut self.rules {
            rule.style
                .values
                .retain(|value| !properties.contains(&value.key));
        }
        self.rules.retain(|rule| !rule.style.values.is_empty());
    }

    pub(crate) const fn new() -> Self {
        Self { rules: Vec::new() }
    }

    pub(crate) fn set(&mut self, condition: StyleCondition, style: StylePatch) {
        if let Some(rule) = self
            .rules
            .iter_mut()
            .find(|rule| rule.condition == condition)
        {
            rule.style = style;
        } else {
            self.rules.push(StyleRule { condition, style });
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }

    pub(crate) fn rules(&self) -> &[StyleRule] {
        &self.rules
    }

    pub(crate) fn impact(&self) -> Option<BindingImpact> {
        self.rules
            .iter()
            .flat_map(|rule| rule.style.values())
            .map(|value| value.key.impact())
            .max_by_key(|impact| match impact {
                BindingImpact::Paint => 0,
                BindingImpact::Scroll => 1,
                BindingImpact::Layout => 2,
            })
    }

    pub(crate) fn contains(&self, key: PropertyKey) -> bool {
        self.rules
            .iter()
            .flat_map(|rule| rule.style.values())
            .any(|value| value.key == key)
    }

    pub(crate) fn has_container_queries(&self) -> bool {
        self.rules
            .iter()
            .any(|rule| rule.condition.has_container_query())
    }
}

impl StylePatch {
    /// Creates an empty conditional style patch.
    #[must_use]
    pub const fn new() -> Self {
        Self { values: Vec::new() }
    }

    /// Adds or replaces a style property using `property` and its new `value`.
    #[must_use]
    pub fn set<P: StyleProperty>(mut self, property: P, value: P::Value) -> Self {
        let value = property.into_state_value(value);
        if let Some(existing) = self.values.iter_mut().find(|item| item.key == value.key) {
            *existing = value;
        } else {
            self.values.push(value);
        }
        self
    }

    /// Sets the complete layout style in this patch.
    ///
    /// * `style` — layout style to apply while the condition matches.
    #[must_use]
    pub fn layout(self, style: crate::LayoutStyle) -> Self {
        self.set(crate::property::Layout, style)
    }

    /// Creates a patch containing the supported properties of `style`.
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

    pub(crate) fn values(&self) -> &[StylePropertyValue] {
        &self.values
    }
}

impl From<QuadStyle> for StylePatch {
    fn from(value: QuadStyle) -> Self {
        Self::from_quad(value)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TransitionDirection {
    Enter(StyleCondition),
    Exit(StyleCondition),
}

#[derive(Clone, Debug, PartialEq)]
pub struct TransitionRule {
    pub property: Option<PropertyKey>,
    pub direction: Option<TransitionDirection>,
    pub transition: Transition,
}

impl TransitionRule {
    /// Creates a transition rule that applies to all properties and directions.
    ///
    /// * `transition` — transition used when this rule matches.
    #[must_use]
    pub const fn new(transition: Transition) -> Self {
        Self {
            property: None,
            direction: None,
            transition,
        }
    }

    /// Restricts this rule to one property.
    #[must_use]
    pub const fn property(mut self, property: PropertyKey) -> Self {
        self.property = Some(property);
        self
    }

    /// Restricts this rule to entering or exiting a condition.
    /// * `direction` — transition direction to which this rule applies.
    #[must_use]
    pub fn direction(mut self, direction: TransitionDirection) -> Self {
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
    pub(crate) fn make_immediate(&mut self, properties: &[PropertyKey]) {
        self.rules
            .retain(|rule| !rule.property.is_some_and(|key| properties.contains(&key)));
        for &property in properties {
            self.rules.push(
                TransitionRule::new(Transition::tween(argui_animation::Tween::new(
                    argui_animation::Duration::ZERO,
                )))
                .property(property),
            );
        }
    }

    /// Creates a policy using `default` when no more specific rule matches.
    #[must_use]
    pub const fn new(default: Transition) -> Self {
        Self {
            default,
            rules: Vec::new(),
        }
    }

    /// Adds `rule`; later rules win when specificity ties.
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
                        .as_ref()
                        .is_none_or(|candidate| Some(candidate) == direction.as_ref())
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
