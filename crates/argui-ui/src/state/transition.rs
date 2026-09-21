use argui_animation::Transition;

use super::{PropertyKey, StyleCondition};

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
    pub fn property(mut self, property: PropertyKey) -> Self {
        self.property = Some(property);
        self
    }

    /// Restricts this rule to entering or exiting a condition.
    ///
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
        self.rules.retain(|rule| {
            !rule
                .property
                .as_ref()
                .is_some_and(|key| properties.contains(key))
        });
        for property in properties {
            self.rules.push(
                TransitionRule::new(Transition::tween(argui_animation::Tween::new(
                    argui_animation::Duration::ZERO,
                )))
                .property(property.clone()),
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
        property: &PropertyKey,
        direction: Option<TransitionDirection>,
    ) -> &Transition {
        self.rules
            .iter()
            .enumerate()
            .filter(|(_, rule)| {
                rule.property
                    .as_ref()
                    .is_none_or(|candidate| candidate == property)
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
