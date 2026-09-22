//! Pending host effects from manually dispatched live events.

use crate::{LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Returns the most recent render or routed-event failure.
    #[must_use]
    pub fn last_error(&self) -> Option<&RuntimeError> {
        self.render_error.as_ref().or(self.event_error.as_ref())
    }

    /// Removes and returns a routed-event failure recorded by the UI host.
    pub fn take_event_error(&mut self) -> Option<RuntimeError> {
        self.event_error.take()
    }

    /// Takes the most recent scroll request produced by a manually dispatched DSL event.
    ///
    /// Returns the pending host request, if any, and clears it. Host-connected
    /// native handlers consume this request automatically after dispatch.
    #[must_use]
    pub fn take_scroll_request(&mut self) -> Option<argui_ui::ScrollRequest> {
        self.pending_scroll.take()
    }
}
