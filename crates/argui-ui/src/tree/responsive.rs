use argui_core::Size;

use crate::TreeUpdate;

use super::UiTree;

impl UiTree {
    #[must_use]
    pub const fn has_container_queries(&self) -> bool {
        self.has_container_queries
    }

    pub fn resolve_container_queries(&mut self, sizes: &[Size]) -> TreeUpdate {
        let next = self
            .container_indices
            .iter()
            .filter_map(|(node, index)| sizes.get(*index).copied().map(|size| (*node, size)))
            .collect::<Vec<_>>();
        if self.container_sizes == next {
            return TreeUpdate::None;
        }
        let first_resolution = self.container_sizes.is_empty();
        self.container_sizes = next;
        let update = self.sync_transitions_with(self.reduced_motion || first_resolution);
        if update == TreeUpdate::Layout {
            self.layout_dirty = true;
        }
        update
    }

    pub(super) fn sync_responsive_registry(&mut self) {
        let elements = crate::traversal::flattened(&self.root);
        self.has_container_queries = elements
            .iter()
            .any(|element| element.has_container_queries());
        self.container_indices = elements
            .into_iter()
            .enumerate()
            .filter_map(|(index, element)| {
                element
                    .container_scope
                    .map(|_| (self.node_ids[index], index))
            })
            .collect();
        self.container_sizes
            .retain(|(node, _)| self.container_indices.iter().any(|(id, _)| id == node));
    }
}
