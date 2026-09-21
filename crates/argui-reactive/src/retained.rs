//! Typed properties retained for keyed component subtrees across render passes.

use std::{
    any::Any,
    collections::{HashMap, HashSet},
    hash::Hash,
};

use crate::Property;

struct Retained<T: Clone + PartialEq + 'static> {
    property: Property<T>,
    synced_revision: u64,
}

/// Heterogeneous keyed property store owned by one rendered root.
///
/// The owner controls render-pass boundaries so properties from unmounted
/// subtrees are dropped at the end of each completed pass.
pub struct RetainedPropertyStore<K: Clone + Eq + Hash> {
    entries: HashMap<(K, u64), Box<dyn Any>>,
    seen: HashSet<(K, u64)>,
}

impl<K: Clone + Eq + Hash> Default for RetainedPropertyStore<K> {
    fn default() -> Self {
        Self {
            entries: HashMap::new(),
            seen: HashSet::new(),
        }
    }
}

impl<K: Clone + Eq + Hash> RetainedPropertyStore<K> {
    /// Creates an empty keyed property store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Begins a render pass and clears the mounted-property mark set.
    pub fn begin_render(&mut self) {
        self.seen.clear();
    }

    /// Reuses an unbound property, updating its default only while unmodified.
    ///
    /// * `key` — stable component-site identity, including any repeater key.
    /// * `property` — stable ID of a property within that component.
    /// * `default` — newly evaluated default for this render pass.
    ///
    /// Returns a clone of the retained reactive handle. Local writes survive
    /// later renders even if the evaluated default changes.
    ///
    /// # Panics
    ///
    /// Panics if one key and property ID are reused for a different Rust type.
    #[must_use]
    pub fn defaulted<T: Clone + PartialEq + 'static>(
        &mut self,
        key: K,
        property: u64,
        default: T,
    ) -> Property<T> {
        let retained = self.entry(key, property, default.clone());
        if retained.property.revision() == retained.synced_revision {
            retained.property.set(default);
            retained.synced_revision = retained.property.revision();
        }
        retained.property.clone()
    }

    /// Reuses a one-way bound property and applies the parent's current value.
    ///
    /// * `key` — stable component-site identity, including any repeater key.
    /// * `property` — stable ID of a property within that component.
    /// * `value` — controlled value supplied by the parent on this render.
    ///
    /// Returns a clone of the retained reactive handle. Parent input wins over
    /// a child-local write on the next render; use a two-way alias for shared state.
    ///
    /// # Panics
    ///
    /// Panics if one key and property ID are reused for a different Rust type.
    #[must_use]
    pub fn controlled<T: Clone + PartialEq + 'static>(
        &mut self,
        key: K,
        property: u64,
        value: T,
    ) -> Property<T> {
        let retained = self.entry(key, property, value.clone());
        retained.property.set(value);
        retained.synced_revision = retained.property.revision();
        retained.property.clone()
    }

    /// Drops properties not visited in the completed render pass.
    pub fn end_render(&mut self) {
        self.entries.retain(|key, _| self.seen.contains(key));
    }

    /// Returns the number of currently retained typed properties.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns whether no typed properties are retained.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Finds or creates one typed slot and marks it mounted in this pass.
    ///
    /// * `key` — stable component identity.
    /// * `property` — stable property ID under that component.
    /// * `initial` — value used only when the slot is first created.
    ///
    /// Returns the mutable retained record for the typed property.
    ///
    /// # Panics
    ///
    /// Panics if a stable property ID changes Rust type within the same store.
    fn entry<T: Clone + PartialEq + 'static>(
        &mut self,
        key: K,
        property: u64,
        initial: T,
    ) -> &mut Retained<T> {
        let id = (key, property);
        self.seen.insert(id.clone());
        self.entries
            .entry(id)
            .or_insert_with(|| {
                Box::new(Retained {
                    property: Property::new(initial),
                    synced_revision: 0,
                })
            })
            .downcast_mut::<Retained<T>>()
            .expect("stable property ID must retain its Rust type")
    }
}
