use std::collections::HashSet;

use crate::model::InteractionSnapshot;
use argui_ui::RetainedIdentity;

use super::Application;

impl Application {
    /// Samples `watched` source nodes against the latest hit regions.
    /// Returns an empty snapshot without indexing when none are watched.
    pub(super) fn current_interaction_snapshot(
        &self,
        watched: &HashSet<RetainedIdentity>,
    ) -> InteractionSnapshot {
        InteractionSnapshot::capture(
            self.ui_tree.as_ref(),
            self.ui_layout
                .as_ref()
                .map_or(&[], |layout| layout.hit_regions.as_slice()),
            self.ui_layout
                .as_ref()
                .map_or(&[], |layout| layout.scroll_regions.as_slice()),
            self.primary_touch.or(self.last_touch),
            watched,
            &mut self.source_index.borrow_mut(),
        )
    }

    /// Schedules a rebuild when a state read during rendering has changed.
    /// Publishes the latest snapshot to model event handlers before dispatch.
    pub(super) fn refresh_observed_interactions(&mut self) {
        let mut watched = HashSet::<RetainedIdentity>::new();
        if let Some(model) = &self.model {
            model.collect_observed(&mut watched);
        }
        let next = self.current_interaction_snapshot(&watched);
        let previous = std::mem::replace(&mut self.interaction_snapshot, next.clone());
        let Some(model) = &self.model else {
            return;
        };
        model.set_interaction_snapshot(&next);
        let changed = previous.changed(&next, &watched);
        if !changed.is_empty() && model.invalidate_observed(&changed) {
            self.pending_ui_frame.request_rebuild();
        }
    }
}
