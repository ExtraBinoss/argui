use std::{
    collections::{BTreeSet, HashMap},
    sync::Arc,
};

use argui_core::Modifiers;

/// An item's identity is independent of its position and displayed label.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CollectionItem {
    pub id: String,
    pub label: String,
    pub enabled: bool,
}

impl CollectionItem {
    /// Creates an enabled item with the supplied stable identity and display label.
    /// `id` is the stable item identity; `label` is the displayed search text.
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            enabled: true,
        }
    }
}

/// Immutable ordered snapshot. Build it when data changes, then borrow it during rendering.
#[derive(Clone, Debug, Default)]
pub struct Collection(Arc<CollectionData>);

#[derive(Debug, Default)]
struct CollectionData {
    items: Vec<CollectionItem>,
    indices: HashMap<String, usize>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DuplicateItemId(pub String);

impl Collection {
    /// Creates an immutable ordered snapshot, rejecting duplicate item identities.
    ///
    /// # Errors
    ///
    /// Returns the repeated identity when two items share an `id`.
    pub fn new(items: impl IntoIterator<Item = CollectionItem>) -> Result<Self, DuplicateItemId> {
        let items: Vec<_> = items.into_iter().collect();
        let mut indices = HashMap::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            if indices.insert(item.id.clone(), index).is_some() {
                return Err(DuplicateItemId(item.id.clone()));
            }
        }
        Ok(Self(Arc::new(CollectionData { items, indices })))
    }

    /// Returns all items in display order.
    pub fn items(&self) -> &[CollectionItem] {
        &self.0.items
    }
    /// Returns the number of items in the snapshot.
    pub fn len(&self) -> usize {
        self.0.items.len()
    }
    /// Returns whether the snapshot contains no items.
    pub fn is_empty(&self) -> bool {
        self.0.items.is_empty()
    }
    /// Returns the item at `index`, if present.
    pub fn get(&self, index: usize) -> Option<&CollectionItem> {
        self.0.items.get(index)
    }
    /// Returns the current index of the item identified by `id`, if present.
    pub fn index_of(&self, id: &str) -> Option<usize> {
        self.0.indices.get(id).copied()
    }

    /// Preserve measured heights by identity after sorting, insertion or filtering.
    /// A clone retains this immutable snapshot in constant time.
    ///
    /// # Panics
    ///
    /// Panics if `heights` does not contain one measurement for every item in `previous`.
    pub fn remap_heights(&self, previous: &Self, heights: &mut argui_ui::VirtualList) {
        assert_eq!(
            previous.len(),
            heights.item_count(),
            "previous collection must match retained heights"
        );
        if !Arc::ptr_eq(&self.0, &previous.0) {
            heights.remap(self.0.items.iter().map(|item| previous.index_of(&item.id)));
        }
    }

    pub(crate) fn enabled_from(&self, start: usize, backwards: bool) -> Option<usize> {
        if backwards {
            (0..=start.min(self.len().checked_sub(1)?))
                .rev()
                .find(|&i| self.0.items[i].enabled)
        } else {
            (start..self.len()).find(|&i| self.0.items[i].enabled)
        }
    }
}

/// Controlled selection shared by lists and tables, retained across changes in order.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ListState {
    pub selected: BTreeSet<String>,
    pub active: Option<String>,
    anchor: Option<String>,
}

impl ListState {
    /// Remove deleted identities using the full source collection, before filtering it.
    /// `collection` must be the source snapshot, not a filtered view.
    pub fn reconcile(&mut self, collection: &Collection) {
        self.selected.retain(|id| collection.index_of(id).is_some());
        if self
            .active
            .as_ref()
            .is_some_and(|id| collection.index_of(id).is_none())
        {
            self.active = collection
                .enabled_from(0, false)
                .map(|i| collection.0.items[i].id.clone());
        }
        if self
            .anchor
            .as_ref()
            .is_some_and(|id| collection.index_of(id).is_none())
        {
            self.anchor = None;
        }
    }

    /// Selects the enabled item at `index` using the configured multiplicity and modifiers.
    /// `collection` supplies the ordered items, `multiple` selects multi-select behavior,
    /// and `modifiers` controls range/toggle selection semantics.
    pub fn select(
        &mut self,
        index: usize,
        collection: &Collection,
        multiple: bool,
        modifiers: Modifiers,
    ) {
        let Some(item) = collection.get(index).filter(|item| item.enabled) else {
            return;
        };
        if multiple && modifiers.shift {
            let anchor = self
                .anchor
                .as_ref()
                .or(self.active.as_ref())
                .and_then(|id| collection.index_of(id))
                .unwrap_or(index);
            if !modifiers.command() {
                self.selected.clear();
            }
            self.selected.extend(
                collection.0.items[anchor.min(index)..=anchor.max(index)]
                    .iter()
                    .filter(|item| item.enabled)
                    .map(|item| item.id.clone()),
            );
            self.anchor = Some(collection.0.items[anchor].id.clone());
        } else {
            if !multiple || !modifiers.command() {
                self.selected.clear();
            }
            if !self.selected.insert(item.id.clone()) {
                self.selected.remove(&item.id);
            }
            self.anchor = Some(item.id.clone());
        }
        self.active = Some(item.id.clone());
    }
}
