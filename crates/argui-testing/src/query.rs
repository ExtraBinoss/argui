use std::fmt;

use argui_accessibility::{Role, SemanticState};

/// Semantic-state predicate used by [`Selector::State`].
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SemanticMatcher {
    Enabled,
    Disabled,
    Checked,
    Selected,
    Expanded,
    Busy,
    Invalid,
}

impl SemanticMatcher {
    pub(crate) fn matches(self, state: &SemanticState) -> bool {
        match self {
            Self::Enabled => !state.disabled,
            Self::Disabled => state.disabled,
            Self::Checked => state
                .checked
                .is_some_and(|value| value != argui_accessibility::CheckedState::Unchecked),
            Self::Selected => state.selected,
            Self::Expanded => state.expanded == Some(true),
            Self::Busy => state.busy,
            Self::Invalid => state.invalid,
        }
    }
}

/// Stable selector accepted by headless queries and diagnostics.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Selector {
    Key(String),
    RoleName { role: Role, name: String },
    Text(String),
    Label(String),
    State(SemanticMatcher),
    Focused,
}

impl Selector {
    /// Selects an element by its application key.
    #[must_use]
    pub fn key(key: impl Into<String>) -> Self {
        Self::Key(key.into())
    }

    /// Selects an accessible node by exact role and name.
    #[must_use]
    pub fn role(role: Role, name: impl Into<String>) -> Self {
        Self::RoleName {
            role,
            name: name.into(),
        }
    }

    /// Selects a visible text semantic by exact content.
    #[must_use]
    pub fn text(text: impl Into<String>) -> Self {
        Self::Text(text.into())
    }

    /// Selects an accessible node by exact label.
    #[must_use]
    pub fn label(label: impl Into<String>) -> Self {
        Self::Label(label.into())
    }

    /// Selects accessible nodes matching `state`.
    #[must_use]
    pub const fn state(state: SemanticMatcher) -> Self {
        Self::State(state)
    }
}

impl fmt::Display for Selector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Key(key) => write!(formatter, "key({key:?})"),
            Self::RoleName { role, name } => write!(formatter, "role({role:?}, {name:?})"),
            Self::Text(text) => write!(formatter, "text({text:?})"),
            Self::Label(label) => write!(formatter, "label({label:?})"),
            Self::State(state) => write!(formatter, "state({state:?})"),
            Self::Focused => formatter.write_str("focused()"),
        }
    }
}
