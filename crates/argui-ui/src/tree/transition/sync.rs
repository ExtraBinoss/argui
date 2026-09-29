use crate::{NodeId, TreeUpdate, UiEventKind, update::strongest_update};

use super::TransitionSync;
use crate::tree::UiTree;

impl UiTree {
    /// Resolves every transition after authored tree, scroll, or layout changes.
    pub(in crate::tree) fn sync_transitions(&mut self) -> TreeUpdate {
        self.sync_transitions_with(self.reduced_motion)
    }

    /// Resolves every transition using `reduced_motion` for target presentation.
    pub(in crate::tree) fn sync_transitions_with(&mut self, reduced_motion: bool) -> TreeUpdate {
        self.sync_transition_subtree(0, reduced_motion)
    }

    /// Resolves only subtrees whose visual state may depend on `events`.
    ///
    /// A state scope, container scope, styled ancestor, or scrollbar can make a
    /// descendant's state observable elsewhere in that ancestor's subtree.
    /// Unknown event targets fall back to a complete resolution.
    pub(in crate::tree) fn sync_transitions_for_events(
        &mut self,
        events: &[(NodeId, UiEventKind)],
    ) -> TreeUpdate {
        if events.is_empty() {
            return self.sync_transitions();
        }
        let mut roots = Vec::with_capacity(events.len());
        for (node, _) in events {
            let Some(mut root) = self.index.position(*node) else {
                return self.sync_transitions();
            };
            let mut ancestor = self.index.parent(root);
            while let Some(index) = ancestor {
                let Some(element) = self.index.at(index) else {
                    return self.sync_transitions();
                };
                if element.state_scope.is_some()
                    || element.container_scope.is_some()
                    || !element.conditional_styles.rules().is_empty()
                    || element.scroll.is_some()
                {
                    root = index;
                }
                ancestor = self.index.parent(index);
            }
            roots.push(root);
        }
        roots.sort_unstable();
        roots.dedup();
        let mut update = TreeUpdate::None;
        let mut covered_until = 0;
        for root in roots {
            if root < covered_until {
                continue;
            }
            update = strongest_update(
                update,
                self.sync_transition_subtree(root, self.reduced_motion),
            );
            covered_until = self.index.subtree_ends()[root] as usize;
        }
        update
    }

    /// Resolves the subtree at `root` and returns its strongest invalidation.
    /// Full-tree calls also discard transitions whose targets were removed.
    fn sync_transition_subtree(&mut self, root: usize, reduced_motion: bool) -> TreeUpdate {
        if !self.index.has_transitions() {
            return self.transitions.clear();
        }
        let Some(element) = self.index.at(root) else {
            return TreeUpdate::None;
        };
        let end = self.index.subtree_ends()[root] as usize;
        let interaction = &self.interaction;
        let scroll = &self.scroll;
        let container_sizes = &self.container_sizes;
        let states_for = |node| interaction.visual_states(node);
        let scrollbar_states_for = |node, part, enabled| scroll.visual_states(node, part, enabled);
        let scroll_for = |node| scroll.offset(node);
        let container_size = |node| {
            container_sizes
                .iter()
                .find(|(id, _)| *id == node)
                .map(|(_, size)| *size)
        };
        self.transitions.sync(TransitionSync {
            root: element,
            node_ids: &self.node_ids[root..end],
            complete: root == 0,
            states_for: &states_for,
            scrollbar_states_for: &scrollbar_states_for,
            scroll_for: &scroll_for,
            container_size: &container_size,
            reduced_motion,
        })
    }

    /// Returns the visual revision associated with the retained `node` ID.
    #[must_use]
    pub fn visual_revision(&self, node: NodeId) -> u64 {
        self.transitions.revision(node)
    }
}
