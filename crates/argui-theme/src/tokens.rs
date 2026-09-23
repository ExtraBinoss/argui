use argui_core::Color;
use std::collections::BTreeMap;

pub use crate::value::ThemeValue;

/// Sparse named overrides accepted by existing Rust theme factories.
///
/// [`crate::ThemeRuntime`] uses numeric token IDs for typed themes. This
/// string-keyed adapter remains at the Rust theme-provider boundary.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemeOverrides(BTreeMap<String, ThemeValue>);

impl ThemeOverrides {
    /// Inserts or replaces a valid token, returning whether the snapshot changed.
    ///
    /// * `name` — token name to set.
    /// * `value` — typed token value to store.
    pub fn set(&mut self, name: impl Into<String>, value: ThemeValue) -> bool {
        if !value.is_valid() {
            return false;
        }
        let name = name.into();
        if self.0.get(&name) == Some(&value) {
            return false;
        }
        self.0.insert(name, value);
        true
    }

    /// Removes the named token, returning whether it existed.
    ///
    /// * `name` — token name to remove.
    pub fn remove(&mut self, name: &str) -> bool {
        self.0.remove(name).is_some()
    }

    /// Returns a clone of the named token value, if present.
    ///
    /// * `name` — token name to look up.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<ThemeValue> {
        self.0.get(name).cloned()
    }

    /// Returns whether this snapshot contains no token overrides.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over token names and cloned values in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, ThemeValue)> {
        self.0
            .iter()
            .map(|(name, value)| (name.as_str(), value.clone()))
    }
}

/// Input to a Rust theme factory.
pub trait ThemeSource {
    /// Returns the primary color used to construct theme tokens.
    fn primary_color(&self) -> Color;

    /// Returns optional sparse overrides supplied by this theme source.
    fn theme_overrides(&self) -> Option<&ThemeOverrides> {
        None
    }
}

impl ThemeSource for Color {
    fn primary_color(&self) -> Color {
        *self
    }
}

impl<T: ThemeSource + ?Sized> ThemeSource for &T {
    fn primary_color(&self) -> Color {
        T::primary_color(self)
    }

    fn theme_overrides(&self) -> Option<&ThemeOverrides> {
        T::theme_overrides(self)
    }
}
