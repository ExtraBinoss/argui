use std::{
    collections::HashMap,
    sync::{Arc, Mutex, Weak, mpsc},
};

use argui_core::{ColorScheme, Name};

use crate::state::{ThemeChange, ThemeSelection, ThemeSnapshot};
use crate::{ThemeError, ThemeMode, ThemeSchema, ThemeTokenId, ThemeValue};

/// Private candidate layers edited by one atomic theme update.
#[derive(Clone, Debug)]
pub struct ThemeDraft {
    schema: Arc<ThemeSchema>,
    variants: HashMap<Name, HashMap<ThemeTokenId, ThemeValue>>,
    selection: ThemeSelection,
    overrides: HashMap<ThemeTokenId, ThemeValue>,
    system_scheme: ColorScheme,
    system_variants: [Name; 2],
}

impl ThemeDraft {
    /// Defines or replaces name with validated sparse values.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn define_variant(
        &mut self,
        name: impl Into<Name>,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<(), ThemeError> {
        let mut variant = HashMap::new();
        for (id, value) in values {
            self.schema.validate(id, &value)?;
            variant.insert(id, value);
        }
        self.variants.insert(name.into(), variant);
        Ok(())
    }

    /// Selects name, which must already be a defined variant.
    ///
    /// # Errors
    ///
    /// Returns an error when name is not defined.
    pub fn activate(&mut self, name: impl Into<Name>) -> Result<(), ThemeError> {
        let name = name.into();
        if !self.variants.contains_key(&name) {
            return Err(ThemeError::UnknownVariant(name.as_str().to_owned()));
        }
        self.selection = ThemeSelection::Variant(name);
        Ok(())
    }

    /// Selects schema defaults without a variant.
    pub fn clear_variant(&mut self) {
        self.selection = ThemeSelection::Defaults;
    }

    /// Selects light, dark, or system-following mode.
    ///
    /// An undefined light or dark variant falls back to schema defaults.
    pub fn set_mode(&mut self, mode: ThemeMode) {
        self.selection = match mode {
            ThemeMode::Light => ThemeSelection::Variant(self.system_variants[0].clone()),
            ThemeMode::Dark => ThemeSelection::Variant(self.system_variants[1].clone()),
            ThemeMode::System => ThemeSelection::System,
        };
    }

    /// Changes the variant names used for light and dark system modes.
    pub fn set_system_variants(&mut self, light: impl Into<Name>, dark: impl Into<Name>) {
        let [light, dark] = [light.into(), dark.into()];
        if let ThemeSelection::Variant(name) = &self.selection {
            if name == &self.system_variants[0] {
                self.selection = ThemeSelection::Variant(light.clone());
            } else if name == &self.system_variants[1] {
                self.selection = ThemeSelection::Variant(dark.clone());
            }
        }
        self.system_variants = [light, dark];
    }

    /// Records scheme for future system-following resolution.
    pub fn set_system_scheme(&mut self, scheme: ColorScheme) {
        self.system_scheme = scheme;
    }

    /// Sets a validated application or window override for id.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn set_override(&mut self, id: ThemeTokenId, value: ThemeValue) -> Result<(), ThemeError> {
        self.schema.validate(id, &value)?;
        self.overrides.insert(id, value);
        Ok(())
    }

    /// Removes the override for id, returning whether one existed.
    pub fn remove_override(&mut self, id: ThemeTokenId) -> bool {
        self.overrides.remove(&id).is_some()
    }

    /// Replaces every override with validated values.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn replace_overrides(
        &mut self,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<(), ThemeError> {
        let mut overrides = HashMap::new();
        for (id, value) in values {
            self.schema.validate(id, &value)?;
            overrides.insert(id, value);
        }
        self.overrides = overrides;
        Ok(())
    }

    /// Returns the selected variant name only when that variant is defined.
    fn resolved_variant(&self) -> Option<&Name> {
        let name = match &self.selection {
            ThemeSelection::Defaults => return None,
            ThemeSelection::System => match self.system_scheme {
                ColorScheme::Light => self.system_variants[0].as_str(),
                ColorScheme::Dark => self.system_variants[1].as_str(),
            },
            ThemeSelection::Variant(name) => name.as_str(),
        };
        self.variants.get_key_value(name).map(|(name, _)| name)
    }

    /// Resolves id through overrides, the variant, references, and defaults.
    fn resolve(&self, id: ThemeTokenId) -> ThemeValue {
        if let Some(value) = self.overrides.get(&id) {
            return value.clone();
        }
        if let Some(value) = self
            .resolved_variant()
            .and_then(|name| self.variants.get(name))
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

struct Watcher {
    id: u64,
    callback: Arc<dyn Fn(ThemeChange) + Send + Sync>,
}

struct Channel {
    tokens: Option<Vec<ThemeTokenId>>,
    sender: mpsc::Sender<ThemeChange>,
}

struct ThemeState {
    draft: ThemeDraft,
    snapshot: ThemeSnapshot,
    watchers: Vec<Watcher>,
    channels: Vec<Channel>,
    next_watcher_id: u64,
}

/// A callback registration removed when its handle is dropped.
pub struct ThemeSubscription {
    state: Weak<Mutex<ThemeState>>,
    id: u64,
}

impl Drop for ThemeSubscription {
    /// Removes this callback from its theme runtime if the runtime still exists.
    fn drop(&mut self) {
        if let Some(state) = self.state.upgrade() {
            let mut state = state.lock().unwrap_or_else(|poison| poison.into_inner());
            state.watchers.retain(|watcher| watcher.id != self.id);
        }
    }
}

/// Shared theme state for one application or window, independent of other instances.
#[derive(Clone)]
pub struct ThemeRuntime {
    schema: Arc<ThemeSchema>,
    state: Arc<Mutex<ThemeState>>,
}

impl ThemeRuntime {
    /// Creates runtime storage initialized from schema and the light system scheme.
    #[must_use]
    pub fn new(schema: Arc<ThemeSchema>) -> Self {
        let values = (0..schema.len())
            .map(|index| {
                schema
                    .default_value(ThemeTokenId::from_index(index))
                    .expect("schema defaults cover every token")
                    .clone()
            })
            .collect::<Vec<_>>();
        let snapshot = ThemeSnapshot {
            revision: 0,
            token_revisions: vec![0; values.len()].into(),
            values: values.into(),
            selection: ThemeSelection::System,
            resolved_variant: None,
            system_scheme: ColorScheme::Light,
            system_variants: [Name::from_static("light"), Name::from_static("dark")],
        };
        let draft = ThemeDraft {
            schema: Arc::clone(&schema),
            variants: HashMap::new(),
            selection: ThemeSelection::System,
            overrides: HashMap::new(),
            system_scheme: ColorScheme::Light,
            system_variants: [Name::from_static("light"), Name::from_static("dark")],
        };
        Self {
            schema,
            state: Arc::new(Mutex::new(ThemeState {
                draft,
                snapshot,
                watchers: Vec::new(),
                channels: Vec::new(),
                next_watcher_id: 1,
            })),
        }
    }

    /// Copies the current theme into a new independent application or window state.
    ///
    /// Existing subscriptions remain attached only to the source runtime.
    #[must_use]
    pub fn fork(&self) -> Self {
        let state = self.lock();
        Self {
            schema: Arc::clone(&self.schema),
            state: Arc::new(Mutex::new(ThemeState {
                draft: state.draft.clone(),
                snapshot: state.snapshot.clone(),
                watchers: Vec::new(),
                channels: Vec::new(),
                next_watcher_id: 1,
            })),
        }
    }

    /// Returns the immutable schema used by this runtime.
    #[must_use]
    pub fn schema(&self) -> &Arc<ThemeSchema> {
        &self.schema
    }

    /// Returns a coherent, cheaply cloned resolved-value snapshot.
    #[must_use]
    pub fn snapshot(&self) -> ThemeSnapshot {
        self.lock().snapshot.clone()
    }

    /// Returns the revision of resolved values or theme metadata.
    #[must_use]
    pub fn revision(&self) -> u64 {
        self.snapshot().revision()
    }

    /// Returns a clone of the resolved value for id, if it exists.
    #[must_use]
    pub fn value(&self, id: ThemeTokenId) -> Option<ThemeValue> {
        self.snapshot().value(id).cloned()
    }

    /// Returns the resolved change count for id, if it exists.
    #[must_use]
    pub fn token_revision(&self, id: ThemeTokenId) -> Option<u64> {
        self.snapshot().token_revision(id)
    }

    /// Returns the selected variant source.
    #[must_use]
    pub fn selection(&self) -> ThemeSelection {
        self.snapshot().selection().clone()
    }

    /// Returns the selected light, dark, or system mode, if applicable.
    #[must_use]
    pub fn mode(&self) -> Option<ThemeMode> {
        self.snapshot().mode()
    }

    /// Returns the last system color scheme supplied to this runtime.
    #[must_use]
    pub fn system_scheme(&self) -> ColorScheme {
        self.snapshot().system_scheme()
    }

    /// Registers callback until its returned handle is dropped.
    ///
    /// Callbacks run after a coherent commit and outside the state lock.
    pub fn watch(
        &self,
        callback: impl Fn(ThemeChange) + Send + Sync + 'static,
    ) -> ThemeSubscription {
        let mut state = self.lock();
        let id = state.next_watcher_id;
        state.next_watcher_id = state.next_watcher_id.wrapping_add(1);
        state.watchers.push(Watcher {
            id,
            callback: Arc::new(callback),
        });
        ThemeSubscription {
            state: Arc::downgrade(&self.state),
            id,
        }
    }

    /// Subscribes to every resolved token change after this call.
    #[must_use]
    pub fn subscribe(&self) -> mpsc::Receiver<ThemeChange> {
        self.subscribe_filter(None)
    }

    /// Subscribes only to changes affecting at least one token in tokens.
    ///
    /// # Errors
    ///
    /// Returns an error if any token ID is outside this runtime's schema.
    pub fn subscribe_tokens(
        &self,
        tokens: &[ThemeTokenId],
    ) -> Result<mpsc::Receiver<ThemeChange>, ThemeError> {
        for id in tokens {
            if self.schema.definition(*id).is_none() {
                return Err(ThemeError::UnknownToken(format!("#{}", id.index())));
            }
        }
        Ok(self.subscribe_filter(Some(tokens.to_vec())))
    }

    /// Applies all edits atomically and publishes one coherent token change.
    ///
    /// The edit closure receives a private draft. Returning an error discards all edits.
    /// The closure must not call this runtime while its state lock is held.
    ///
    /// # Errors
    ///
    /// Returns the error supplied by edit, leaving runtime state unchanged.
    ///
    /// # Panics
    ///
    /// Panics if edit panics, without committing its draft, or if a watcher
    /// callback panics after the commit.
    pub fn update(
        &self,
        edit: impl FnOnce(&mut ThemeDraft) -> Result<(), ThemeError>,
    ) -> Result<ThemeChange, ThemeError> {
        let mut state = self.lock();
        let mut draft = state.draft.clone();
        edit(&mut draft)?;
        let values = (0..self.schema.len())
            .map(|index| draft.resolve(ThemeTokenId::from_index(index)))
            .collect::<Vec<_>>();
        let mut token_revisions = state.snapshot.token_revisions.to_vec();
        let mut tokens = Vec::new();
        for (index, value) in values.iter().enumerate() {
            if state.snapshot.values[index] != *value {
                tokens.push(ThemeTokenId::from_index(index));
                token_revisions[index] = token_revisions[index].wrapping_add(1);
            }
        }
        let metadata_changed = draft.selection != state.draft.selection
            || draft.system_scheme != state.draft.system_scheme
            || draft.system_variants != state.draft.system_variants
            || draft.variants != state.draft.variants
            || draft.overrides != state.draft.overrides;
        let revision = state
            .snapshot
            .revision
            .wrapping_add(u64::from(metadata_changed || !tokens.is_empty()));
        let impact = tokens
            .iter()
            .filter_map(|id| self.schema.definition(*id))
            .map(|definition| definition.impact_class())
            .max();
        let change = ThemeChange {
            tokens,
            impact,
            revision,
        };
        state.snapshot = ThemeSnapshot {
            revision,
            values: if change.is_empty() {
                Arc::clone(&state.snapshot.values)
            } else {
                values.into()
            },
            token_revisions: token_revisions.into(),
            selection: draft.selection.clone(),
            resolved_variant: draft.resolved_variant().cloned(),
            system_scheme: draft.system_scheme,
            system_variants: draft.system_variants.clone(),
        };
        state.draft = draft;
        let callbacks = if !metadata_changed && change.is_empty() {
            Vec::new()
        } else {
            state.channels.retain(|channel| {
                !interested(channel.tokens.as_deref(), &change)
                    || channel.sender.send(change.clone()).is_ok()
            });
            state
                .watchers
                .iter()
                .map(|watcher| Arc::clone(&watcher.callback))
                .collect::<Vec<_>>()
        };
        drop(state);
        for callback in callbacks {
            callback(change.clone());
        }
        Ok(change)
    }

    /// Defines or atomically replaces a named variant with sparse values.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn define_variant(
        &self,
        name: impl Into<Name>,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<ThemeChange, ThemeError> {
        let name = name.into();
        let values = values.into_iter().collect::<Vec<_>>();
        self.update(move |draft| draft.define_variant(name, values))
    }

    /// Activates a previously defined named variant.
    ///
    /// # Errors
    ///
    /// Returns an error when name is not defined.
    pub fn activate(&self, name: impl Into<Name>) -> Result<ThemeChange, ThemeError> {
        let name = name.into();
        self.update(move |draft| draft.activate(name))
    }

    /// Returns to schema defaults plus application or window overrides.
    pub fn clear_variant(&self) -> ThemeChange {
        self.update(|draft| {
            draft.clear_variant();
            Ok(())
        })
        .expect("clearing a variant cannot fail")
    }

    /// Selects a light, dark, or system-following variant mode.
    pub fn set_mode(&self, mode: ThemeMode) -> ThemeChange {
        self.update(|draft| {
            draft.set_mode(mode);
            Ok(())
        })
        .expect("selecting a theme mode cannot fail")
    }

    /// Applies a new system color scheme, affecting values only in system mode.
    pub fn set_system_scheme(&self, scheme: ColorScheme) -> ThemeChange {
        self.update(|draft| {
            draft.set_system_scheme(scheme);
            Ok(())
        })
        .expect("setting the system color scheme cannot fail")
    }

    /// Changes the names used for light and dark system variants.
    pub fn set_system_variants(
        &self,
        light: impl Into<Name>,
        dark: impl Into<Name>,
    ) -> ThemeChange {
        let light = light.into();
        let dark = dark.into();
        self.update(move |draft| {
            draft.set_system_variants(light, dark);
            Ok(())
        })
        .expect("setting system variant names cannot fail")
    }

    /// Sets a validated application or window override for id.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn set_override(
        &self,
        id: ThemeTokenId,
        value: ThemeValue,
    ) -> Result<ThemeChange, ThemeError> {
        self.update(move |draft| draft.set_override(id, value))
    }

    /// Sets all supplied overrides in one atomic change.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn set_overrides(
        &self,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<ThemeChange, ThemeError> {
        let values = values.into_iter().collect::<Vec<_>>();
        self.update(move |draft| {
            for (id, value) in values {
                draft.set_override(id, value)?;
            }
            Ok(())
        })
    }

    /// Replaces all overrides in one atomic change.
    ///
    /// # Errors
    ///
    /// Returns an error for an unknown token, invalid value, or type mismatch.
    pub fn replace_overrides(
        &self,
        values: impl IntoIterator<Item = (ThemeTokenId, ThemeValue)>,
    ) -> Result<ThemeChange, ThemeError> {
        let values = values.into_iter().collect::<Vec<_>>();
        self.update(move |draft| draft.replace_overrides(values))
    }

    /// Removes an override for id and returns the resulting token change.
    pub fn remove_override(&self, id: ThemeTokenId) -> ThemeChange {
        self.update(|draft| {
            draft.remove_override(id);
            Ok(())
        })
        .expect("removing an override cannot fail")
    }

    /// Registers a channel for all tokens or the supplied token subset.
    fn subscribe_filter(&self, tokens: Option<Vec<ThemeTokenId>>) -> mpsc::Receiver<ThemeChange> {
        let (sender, receiver) = mpsc::channel();
        self.lock().channels.push(Channel { tokens, sender });
        receiver
    }

    /// Acquires theme state, recovering an uncommitted draft after a panic.
    fn lock(&self) -> std::sync::MutexGuard<'_, ThemeState> {
        self.state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner())
    }
}

/// Returns whether a token filter intersects this change, or has no filter.
fn interested(filter: Option<&[ThemeTokenId]>, change: &ThemeChange) -> bool {
    filter.is_none_or(|filter| filter.iter().any(|id| change.tokens.contains(id)))
}
