//! Native focus-transfer state while the window manager minimizes an app.

use argui_platform::WindowKey;
use std::collections::HashSet;
use web_time::Instant;

#[derive(Default)]
pub(super) struct WindowActivationState {
    pub(super) minimized: HashSet<WindowKey>,
    pub(super) restored: HashSet<WindowKey>,
    pub(super) minimize_focus_until: Option<Instant>,
}
