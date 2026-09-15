use argui_core::Color;
use std::collections::BTreeMap;

/// Renderer-independent theme values. Token names belong to the theme provider.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeValue {
    Color(Color),
    Number(f32),
}

/// Sparse overrides shared by a window's presentations. Mutating a copied
/// snapshot leaves the previously rendered environment intact for invalidation.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ThemeOverrides(BTreeMap<String, ThemeValue>);

impl ThemeOverrides {
    /// Inserts or replaces a token, returning whether the snapshot changed.
    ///
    /// Non-finite numeric values and values identical to the current value are rejected.
    ///
    /// * `name` — token name to set.
    /// * `value` — token value to store.
    pub fn set(&mut self, name: impl Into<String>, value: ThemeValue) -> bool {
        if matches!(value, ThemeValue::Number(number) if !number.is_finite()) {
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

    #[must_use]
    /// Returns a copy of the named token value, if present.
    ///
    /// * `name` — token name to look up.
    pub fn get(&self, name: &str) -> Option<ThemeValue> {
        self.0.get(name).copied()
    }

    #[must_use]
    /// Returns whether this snapshot contains no token overrides.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Iterates over token names and copied values in key order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, ThemeValue)> {
        self.0.iter().map(|(name, value)| (name.as_str(), *value))
    }
}

/// Input to a theme factory. A plain accent color needs no override storage;
/// a window environment can provide its own sparse token snapshot.
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
