use std::{collections::HashMap, sync::Arc};

use argui_core::Name;
use argui_reactive::{Property, transaction};

use crate::{ThemeError, ThemeImpact, ThemeSchema, ThemeTokenId, ThemeValue};

/// Tokens and strongest invalidation produced by one atomic theme change.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ThemeChange {
    tokens: Vec<ThemeTokenId>,
    impact: Option<ThemeImpact>,
}

impl ThemeChange {
    /// Returns changed tokens in schema order.
    #[must_use]
    pub fn tokens(&self) -> &[ThemeTokenId] {
        &self.tokens
    }

    /// Returns the strongest invalidation required by changed tokens.
    #[must_use]
    pub const fn impact(&self) -> Option<ThemeImpact> {
        self.impact
    }

    /// Returns whether no resolved token value changed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

/// Runtime token storage with variants, typed overrides, and observable reads.
#[derive(Clone)]
pub struct ThemeRuntime {
    schema: Arc<ThemeSchema>,
    variants: HashMap<Name, HashMap<ThemeTokenId, ThemeValue>>,
    active_variant: Option<Name>,
    overrides: HashMap<ThemeTokenId, ThemeValue>,
    values: Vec<Property<ThemeValue>>,
}

impl ThemeRuntime {
    /// Creates runtime storage initialized from a validated schema.
    ///
    /// * `schema` — immutable token definitions shared with compiler/runtime users.
    #[must_use]
    pub fn new(schema: Arc<ThemeSchema>) -> Self {
        let values = (0..schema.len())
            .map(|index| {
                Property::new(
                    schema
                        .default_value(ThemeTokenId::from_index(index))
                        .expect("schema defaults cover every token")
                        .clone(),
                )
            })
            .collect();
        Self {
            schema,
            variants: HashMap::new(),
            active_variant: None,
            overrides: HashMap::new(),
            values,
        }
    }

    /// Returns the immutable schema used by this runtime.
    #[must_use]
    pub fn schema(&self) -> &Arc<ThemeSchema> {
        &self.schema
    }

    /// Reads one resolved token and registers a reactive dependency.
    ///
    /// * `id` — dense token identity from this runtime's schema.
    #[must_use]
    pub fn value(&self, id: ThemeTokenId) -> Option<ThemeValue> {
        self.values.get(id.index()).map(Property::get)
    }

    /// Returns the accepted-change revision for one token.
    ///
    /// * `id` — dense token identity from this runtime's schema.
    #[must_use]
    pub fn token_revision(&self, id: ThemeTokenId) -> Option<u64> {
        self.values.get(id.index()).map(Property::revision)
    }

    /// Defines or atomically replaces a named theme variant.
    ///
    /// # Errors
    ///
    /// Returns an error when an ID is unknown or a value has the wrong type.
    ///
    /// * `name` — variant name such as `light`, `dark`, or a user-defined mode.
    /// * `values` — sparse token overrides for this variant.
    pub fn define_variant(
        &mut self,
        name: impl Into<Name>,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<ThemeChange, ThemeError> {
        let name = name.into();
        let mut variant = HashMap::new();
        for (id, value) in values {
            self.schema.validate(id, &value)?;
            variant.insert(id, value);
        }
        self.variants.insert(name.clone(), variant);
        if self.active_variant.as_ref() == Some(&name) {
            Ok(self.refresh())
        } else {
            Ok(ThemeChange::default())
        }
    }

    /// Activates a previously defined variant and returns its selective invalidation.
    ///
    /// # Errors
    ///
    /// Returns [`ThemeError::UnknownVariant`] when `name` is not defined.
    ///
    /// * `name` — variant to activate.
    pub fn activate(&mut self, name: impl Into<Name>) -> Result<ThemeChange, ThemeError> {
        let name = name.into();
        if !self.variants.contains_key(&name) {
            return Err(ThemeError::UnknownVariant(name.as_str().to_owned()));
        }
        if self.active_variant.as_ref() == Some(&name) {
            return Ok(ThemeChange::default());
        }
        self.active_variant = Some(name);
        Ok(self.refresh())
    }

    /// Clears the active variant and returns to schema defaults plus runtime overrides.
    pub fn clear_variant(&mut self) -> ThemeChange {
        if self.active_variant.take().is_none() {
            return ThemeChange::default();
        }
        self.refresh()
    }

    /// Sets a typed runtime override and returns its selective invalidation.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    ///
    /// * `id` — token to override.
    /// * `value` — typed override value.
    pub fn set_override(
        &mut self,
        id: ThemeTokenId,
        value: ThemeValue,
    ) -> Result<ThemeChange, ThemeError> {
        self.schema.validate(id, &value)?;
        if self.overrides.get(&id) == Some(&value) {
            return Ok(ThemeChange::default());
        }
        self.overrides.insert(id, value);
        Ok(self.refresh())
    }

    /// Removes a runtime override and returns its selective invalidation.
    ///
    /// * `id` — token whose override should be removed.
    pub fn remove_override(&mut self, id: ThemeTokenId) -> ThemeChange {
        if self.overrides.remove(&id).is_none() {
            return ThemeChange::default();
        }
        self.refresh()
    }

    fn refresh(&self) -> ThemeChange {
        let resolved = (0..self.schema.len())
            .map(|index| self.resolve(ThemeTokenId::from_index(index)))
            .collect::<Vec<_>>();
        let mut changed = Vec::new();
        transaction(|| {
            for (index, value) in resolved.into_iter().enumerate() {
                if self.values[index].set(value) {
                    changed.push(ThemeTokenId::from_index(index));
                }
            }
        });
        let impact = changed
            .iter()
            .filter_map(|id| self.schema.definition(*id))
            .map(|definition| definition.impact_class())
            .max();
        ThemeChange {
            tokens: changed,
            impact,
        }
    }

    fn resolve(&self, id: ThemeTokenId) -> ThemeValue {
        if let Some(value) = self.overrides.get(&id) {
            return value.clone();
        }
        if let Some(value) = self
            .active_variant
            .as_ref()
            .and_then(|variant| self.variants.get(variant))
            .and_then(|variant| variant.get(&id))
        {
            return value.clone();
        }
        self.schema.reference(id).map_or_else(
            || {
                self.schema
                    .default_value(id)
                    .expect("validated token has a default")
                    .clone()
            },
            |source| self.resolve(source),
        )
    }
}
