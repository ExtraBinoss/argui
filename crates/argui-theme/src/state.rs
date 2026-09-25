use std::sync::Arc;

use argui_core::{ColorScheme, Name};

use crate::{ThemeImpact, ThemeMode, ThemeTokenId, ThemeValue};

/// Tokens and strongest invalidation produced by one atomic theme change.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThemeChange {
    pub(crate) tokens: Vec<ThemeTokenId>,
    pub(crate) impact: Option<ThemeImpact>,
    pub(crate) revision: u64,
}

impl ThemeChange {
    /// Returns changed tokens in schema order.
    #[must_use]
    pub fn tokens(&self) -> &[ThemeTokenId] {
        &self.tokens
    }

    /// Returns the strongest update phase required by changed tokens.
    #[must_use]
    pub const fn impact(&self) -> Option<ThemeImpact> {
        self.impact
    }

    /// Returns the coherent theme revision after this operation.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns whether no resolved token value changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Source used to select a token variant.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ThemeSelection {
    /// Resolve only schema defaults and overrides.
    Defaults,
    /// Follow the current system color scheme.
    System,
    /// Resolve the named light, dark, or custom variant.
    Variant(Name),
}

/// Resolved values and metadata captured together at one revision.
#[derive(Clone, Debug, PartialEq)]
pub struct ThemeSnapshot {
    pub(crate) revision: u64,
    pub(crate) values: Arc<[ThemeValue]>,
    pub(crate) token_revisions: Arc<[u64]>,
    pub(crate) selection: ThemeSelection,
    pub(crate) resolved_variant: Option<Name>,
    pub(crate) system_scheme: ColorScheme,
    pub(crate) system_variants: [Name; 2],
}

impl ThemeSnapshot {
    /// Returns the revision of values or theme metadata.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns all resolved values in schema order.
    #[must_use]
    pub fn values(&self) -> &[ThemeValue] {
        &self.values
    }

    /// Returns the resolved value for id, if id belongs to the schema.
    #[must_use]
    pub fn value(&self, id: ThemeTokenId) -> Option<&ThemeValue> {
        self.values.get(id.index())
    }

    /// Returns the change count for id, if id belongs to the schema.
    #[must_use]
    pub fn token_revision(&self, id: ThemeTokenId) -> Option<u64> {
        self.token_revisions.get(id.index()).copied()
    }

    /// Returns the selected variant source.
    #[must_use]
    pub fn selection(&self) -> &ThemeSelection {
        &self.selection
    }

    /// Returns the explicitly selected variant name, if one was selected.
    #[must_use]
    pub fn active_variant(&self) -> Option<&str> {
        match &self.selection {
            ThemeSelection::Variant(name) => Some(name.as_str()),
            ThemeSelection::Defaults | ThemeSelection::System => None,
        }
    }

    /// Returns the selected light, dark, or system mode, if applicable.
    #[must_use]
    pub fn mode(&self) -> Option<ThemeMode> {
        match &self.selection {
            ThemeSelection::System => Some(ThemeMode::System),
            ThemeSelection::Variant(name) if name == &self.system_variants[0] => {
                Some(ThemeMode::Light)
            }
            ThemeSelection::Variant(name) if name == &self.system_variants[1] => {
                Some(ThemeMode::Dark)
            }
            ThemeSelection::Defaults | ThemeSelection::Variant(_) => None,
        }
    }

    /// Returns the active named variant, if one is defined and selected.
    #[must_use]
    pub fn resolved_variant(&self) -> Option<&str> {
        self.resolved_variant.as_ref().map(Name::as_str)
    }

    /// Returns the last system color scheme supplied to this runtime.
    #[must_use]
    pub const fn system_scheme(&self) -> ColorScheme {
        self.system_scheme
    }
}
