//! Retained, copy-on-write model rows with stable identities and explicit edits.

use std::{cmp::Ordering, collections::HashSet, ops::Deref, sync::Arc};

/// Compares two source rows while building a sorted projection.
type RowComparator<T> = dyn FnMut(&T, &T) -> Ordering;

/// Last structural or row edit committed by a model.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModelChange {
    /// A row with `id` was inserted at `index`.
    Insert { id: u64, index: usize },
    /// A row with `id` was removed from `index`.
    Remove { id: u64, index: usize },
    /// A row with `id` moved from `from` to `to`.
    Move { id: u64, from: usize, to: usize },
    /// A row with `id` changed at `index`.
    Update { id: u64, index: usize },
}

/// A sequence with stable row identities, cheap snapshots, and in-place edits.
///
/// A `Property<Model<T>>` supplies graph notifications; call `Property::mutate`
/// around edits to invalidate bindings without cloning the entire collection.
#[derive(Clone, Debug)]
pub struct Model<T: Clone> {
    rows: Arc<Vec<T>>,
    ids: Arc<Vec<u64>>,
    next_id: u64,
    revision: u64,
    last_change: Option<ModelChange>,
}

impl<T: Clone> Model<T> {
    /// Creates rows in source order with stable, nonzero identities.
    ///
    /// `rows` is the initial sequence. Returns an owned model at revision zero.
    #[must_use]
    pub fn new(rows: impl IntoIterator<Item = T>) -> Self {
        let rows: Vec<T> = rows.into_iter().collect();
        let ids = (1..=rows.len() as u64).collect();
        Self {
            next_id: rows.len() as u64 + 1,
            rows: Arc::new(rows),
            ids: Arc::new(ids),
            revision: 0,
            last_change: None,
        }
    }

    /// Restores rows with stable identities from another backend.
    ///
    /// `rows` pairs each nonzero identity with its value; `revision` is the
    /// source edit count. Returns `None` for duplicate, zero, or exhausted IDs.
    #[must_use]
    pub fn from_identified_rows(
        rows: impl IntoIterator<Item = (u64, T)>,
        revision: u64,
    ) -> Option<Self> {
        let (ids, rows): (Vec<_>, Vec<_>) = rows.into_iter().unzip();
        let mut seen = HashSet::new();
        if ids.iter().any(|id| *id == 0 || !seen.insert(*id)) {
            return None;
        }
        let next_id = ids.iter().copied().max().unwrap_or(0).checked_add(1)?;
        Some(Self {
            rows: Arc::new(rows),
            ids: Arc::new(ids),
            next_id,
            revision,
            last_change: None,
        })
    }

    /// Returns the stable identity of the row at `index`, if it exists.
    ///
    /// `index` is zero based; the result is absent outside the model.
    #[must_use]
    pub fn row_id(&self, index: usize) -> Option<u64> {
        self.ids.get(index).copied()
    }

    /// Returns stable row identities in current order without copying them.
    #[must_use]
    pub fn row_ids(&self) -> &[u64] {
        &self.ids
    }

    /// Returns the accepted edit count.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns the last accepted edit, if any.
    #[must_use]
    pub const fn last_change(&self) -> Option<ModelChange> {
        self.last_change
    }

    /// Inserts `row` at `index` and returns its stable identity.
    ///
    /// # Panics
    ///
    /// Panics if `index` exceeds the row count.
    pub fn insert(&mut self, index: usize, row: T) -> u64 {
        let id = self.next_id.max(1);
        self.next_id = id.checked_add(1).expect("model row identity exhausted");
        Arc::make_mut(&mut self.rows).insert(index, row);
        Arc::make_mut(&mut self.ids).insert(index, id);
        self.changed(ModelChange::Insert { id, index });
        id
    }

    /// Removes and returns the row at `index` and its identity, if present.
    ///
    /// `index` is zero based; an out-of-range index leaves the model unchanged.
    pub fn remove(&mut self, index: usize) -> Option<(u64, T)> {
        let id = self.row_id(index)?;
        let row = Arc::make_mut(&mut self.rows).remove(index);
        Arc::make_mut(&mut self.ids).remove(index);
        self.changed(ModelChange::Remove { id, index });
        Some((id, row))
    }

    /// Moves a row from `from` to the final index `to` and returns whether it moved.
    ///
    /// Invalid or equal positions leave the model unchanged.
    pub fn move_row(&mut self, from: usize, to: usize) -> bool {
        if from >= self.rows.len() || to >= self.rows.len() || from == to {
            return false;
        }
        let id = self.ids[from];
        let row = Arc::make_mut(&mut self.rows).remove(from);
        Arc::make_mut(&mut self.rows).insert(to, row);
        Arc::make_mut(&mut self.ids).remove(from);
        Arc::make_mut(&mut self.ids).insert(to, id);
        self.changed(ModelChange::Move { id, from, to });
        true
    }

    /// Mutates the row at `index` while preserving its identity.
    ///
    /// `edit` returns whether it changed the row; a false result must leave it
    /// unchanged. Returns false for an invalid index or unchanged row.
    pub fn update(&mut self, index: usize, edit: impl FnOnce(&mut T) -> bool) -> bool {
        let Some(id) = self.row_id(index) else {
            return false;
        };
        if !edit(&mut Arc::make_mut(&mut self.rows)[index]) {
            return false;
        }
        self.changed(ModelChange::Update { id, index });
        true
    }

    /// Returns source indices that pass `keep`, optionally sorted by `compare`.
    ///
    /// `keep` selects rows; `compare` orders selected rows. A projection owns only
    /// indices and leaves the source model and stable row identities intact.
    #[must_use]
    pub fn project(
        &self,
        mut keep: impl FnMut(&T) -> bool,
        mut compare: Option<&mut RowComparator<T>>,
    ) -> Vec<usize> {
        let mut indices: Vec<_> = self
            .rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| keep(row).then_some(index))
            .collect();
        if let Some(compare) = compare.as_mut() {
            indices.sort_by(|left, right| compare(&self.rows[*left], &self.rows[*right]));
        }
        indices
    }

    /// Records a model edit and increments its revision.
    ///
    /// `change` describes the accepted row operation.
    fn changed(&mut self, change: ModelChange) {
        self.revision = self.revision.wrapping_add(1);
        self.last_change = Some(change);
    }
}

impl<T: Clone> Default for Model<T> {
    fn default() -> Self {
        Self::new([])
    }
}

impl<T: Clone> Deref for Model<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        &self.rows
    }
}

impl<T: Clone> IntoIterator for Model<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        Arc::try_unwrap(self.rows)
            .unwrap_or_else(|rows| (*rows).clone())
            .into_iter()
    }
}

impl<T: Clone> PartialEq for Model<T> {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.rows, &other.rows) && self.revision == other.revision
    }
}
