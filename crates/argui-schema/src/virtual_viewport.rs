//! Bounded measurements for virtual windows that use their laid-out height.
use argui_ui::{RetainedIdentity, VirtualList};
use std::collections::HashMap;

/// Retains measured heights only for virtual windows mounted in the current tree.
#[derive(Default)]
pub struct VirtualViewportStore {
    entries: HashMap<RetainedIdentity, Measurement>,
    lists: HashMap<RetainedIdentity, RowMeasurement>,
    generation: u64,
}

struct Measurement {
    height: f32,
    generation: u64,
}

struct RowMeasurement {
    list: VirtualList,
    row_ids: Vec<u64>,
    estimate: f32,
    generation: u64,
}

impl VirtualViewportStore {
    /// Creates an empty store; ordinary UI trees require no layout subscriptions.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Starts a render generation; call `end_render` after a successful traversal.
    pub fn begin_render(&mut self) {
        self.generation = self.generation.wrapping_add(1);
    }

    /// Marks `identity` as mounted and returns its last measured viewport height.
    /// A newly mounted virtual window returns zero until its first layout arrives.
    pub fn height(&mut self, identity: &RetainedIdentity) -> f32 {
        let entry = self.entries.entry(identity.clone()).or_insert(Measurement {
            height: 0.0,
            generation: self.generation,
        });
        entry.generation = self.generation;
        entry.height
    }

    /// Returns a retained variable-height list for the current model row identities.
    ///
    /// `identity` selects one mounted window; `row_ids` preserves measurements
    /// through edits and reordering; `estimate`, `viewport`, and `overscan`
    /// configure its current logical geometry. Returns a shared measurement
    /// handle that must also be passed to the native adapter for layout feedback.
    pub fn list(
        &mut self,
        identity: &RetainedIdentity,
        row_ids: &[u64],
        estimate: f32,
        viewport: f32,
        overscan: usize,
    ) -> VirtualList {
        let entry = self
            .lists
            .entry(identity.clone())
            .or_insert_with(|| RowMeasurement {
                list: VirtualList::variable(row_ids.len(), estimate, viewport),
                row_ids: row_ids.to_vec(),
                estimate,
                generation: self.generation,
            });
        if entry.estimate != estimate {
            entry.list = VirtualList::variable(row_ids.len(), estimate, viewport);
            entry.row_ids = row_ids.to_vec();
            entry.estimate = estimate;
        } else if entry.row_ids != row_ids {
            let old = entry
                .row_ids
                .iter()
                .copied()
                .enumerate()
                .map(|(index, id)| (id, index))
                .collect::<HashMap<_, _>>();
            entry
                .list
                .remap(row_ids.iter().map(|id| old.get(id).copied()));
            entry.row_ids = row_ids.to_vec();
        }
        entry.list = entry
            .list
            .clone()
            .with_viewport(viewport)
            .overscan(overscan);
        entry.generation = self.generation;
        entry.list.clone()
    }

    /// Removes measurements belonging to windows absent from the completed render.
    pub fn end_render(&mut self) {
        self.entries
            .retain(|_, entry| entry.generation == self.generation);
        self.lists
            .retain(|_, entry| entry.generation == self.generation);
    }

    /// Applies `(identity, height)` pairs from `bounds` for subscribed windows.
    /// Returns whether an effective height changed. Unrelated geometry and
    /// invalid heights are ignored; an unsubscribed tree does not iterate `bounds`.
    pub fn update<'a>(
        &mut self,
        bounds: impl IntoIterator<Item = (&'a RetainedIdentity, f32)>,
    ) -> bool {
        if self.entries.is_empty() {
            return false;
        }
        let mut changed = false;
        for (identity, height) in bounds {
            if let Some(entry) = self.entries.get_mut(identity)
                && height.is_finite()
                && height >= 0.0
                && entry.height != height
            {
                entry.height = height;
                changed = true;
            }
        }
        changed
    }
}
