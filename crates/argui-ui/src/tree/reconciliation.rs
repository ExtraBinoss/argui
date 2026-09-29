use super::{TreeUpdate, TreeUpdateStats, UiTree};
use crate::update::{classify_update, strongest_update};
use crate::{Element, UiEvent, UiEventKind, identity};

impl UiTree {
    /// Reconciles a new element hierarchy with this tree's retained state.
    ///
    /// * `root` — new element hierarchy.
    ///
    /// Returns the strongest update required by the reconciliation.
    pub fn update(&mut self, mut root: Element) -> TreeUpdate {
        let mut stats = TreeUpdateStats::default();
        let update = classify_update(&self.root, &mut root, &mut stats);
        self.update_stats = stats;
        let focused_before = self.interaction.focused();
        let focus_visible_before = focused_before.is_some_and(|node| {
            self.interaction
                .visual_states(node)
                .contains(crate::VisualState::FocusVisible)
        });
        let focused_key = focused_before.and_then(|node| self.key_for(node).map(ToOwned::to_owned));
        match update {
            TreeUpdate::None => return update,
            TreeUpdate::Semantics => {
                self.root = root;
                self.sync_animation_registry(false);
                self.focus.sync(
                    &self.root,
                    &self.node_ids,
                    focused_before,
                    focus_visible_before,
                    None,
                );
            }
            TreeUpdate::Composite | TreeUpdate::Paint | TreeUpdate::Scroll => {
                self.root = root;
                self.sync_animation_registry(false);
            }
            TreeUpdate::Layout => {
                let node_ids = identity::reconcile_ids(
                    &self.root,
                    &self.node_ids,
                    self.index.subtree_ends(),
                    &root,
                    &mut self.next_node_id,
                );
                let structure_changed = self.node_ids != node_ids;
                self.node_ids = node_ids;
                self.root = root;
                self.sync_animation_registry(structure_changed);
                self.interaction.retain(&self.node_ids);
                self.interaction_bounds
                    .retain(|node, _| self.node_ids.contains(node));
                self.pressed_positions
                    .retain(|node, _| self.node_ids.contains(node));
                self.pending_gestures
                    .retain(|gesture| self.node_ids.contains(&gesture.target));
                self.scroll.retain(&self.node_ids);
                self.sync_text_inputs();
                if self.document_selection().is_some_and(|selection| {
                    !self.node_ids.contains(&selection.anchor.node)
                        || !self.node_ids.contains(&selection.focus.node)
                }) {
                    self.document_selection =
                        crate::text_selection::DocumentSelectionState::default();
                }
                self.revision = self.revision.wrapping_add(1);
                self.layout_dirty = true;
                let removed_focus = focused_before
                    .filter(|node| !self.node_ids.contains(node))
                    .map(|target| UiEvent::new(target, focused_key, UiEventKind::Blurred));
                self.focus.sync(
                    &self.root,
                    &self.node_ids,
                    focused_before,
                    focus_visible_before,
                    removed_focus,
                );
            }
        }
        self.press_bounces.retain(|node, _| {
            self.index
                .element(*node)
                .and_then(|element| element.interaction.as_ref())
                .is_some_and(|interaction| {
                    interaction.enabled && interaction.press_bounce_scale.is_some()
                })
        });
        self.events.sync(&self.index);
        self.sync_resize_handles();
        if update == TreeUpdate::Layout {
            self.sync_responsive_registry();
        }
        let transition_update = self.sync_transitions();
        if transition_update == TreeUpdate::Layout && update != TreeUpdate::Layout {
            self.revision = self.revision.wrapping_add(1);
        }
        let scroll_update = if self.sync_declared_scroll_offsets() {
            TreeUpdate::Scroll
        } else {
            TreeUpdate::None
        };
        let resize_update = if self.resize.restored.is_empty() {
            TreeUpdate::None
        } else {
            TreeUpdate::Layout
        };
        let update = strongest_update(
            update,
            strongest_update(
                resize_update,
                strongest_update(transition_update, scroll_update),
            ),
        );
        self.layout_dirty |= update == TreeUpdate::Layout;
        update
    }
}
