use crate::{NodeId, UiEventKind};

use super::{InteractionState, RawUpdate};

impl InteractionState {
    /// Applies pointer focus to a resolved target or handles a background press.
    ///
    /// * `target` — focusable target selected from the pressed node or its ancestors.
    /// * `preserve_on_background` — whether an unhandled press leaves focus unchanged.
    ///
    /// Returns focus and blur deliveries caused by the change.
    pub(crate) fn focus_pressed(
        &mut self,
        target: Option<NodeId>,
        preserve_on_background: bool,
    ) -> RawUpdate {
        let Some(target) = target else {
            return if preserve_on_background {
                RawUpdate::default()
            } else {
                self.clear_focus()
            };
        };
        let mut update = RawUpdate::default();
        let focus_changed = self.focused != Some(target);
        self.release_keyboard(&mut update, None);
        if focus_changed {
            if let Some(previous) = self.focused.replace(target) {
                update.push(previous, UiEventKind::Blurred);
            }
            update.push(target, UiEventKind::Focused);
        }
        self.focus_visible = false;
        update
    }
}
