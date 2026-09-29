//! Focus-preserving restoration of an application's currently open windows.

use super::MultiApplication;
use super::activation_types::WindowActivationState;
use crate::RuntimeEvent;
use argui_platform::WindowKey;
use std::time::Duration;
use web_time::Instant;

const MINIMIZE_FOCUS_GRACE: Duration = Duration::from_millis(150);

impl MultiApplication {
    /// Restores open presentations and raises `source` last so it retains focus
    /// and its position above overlapping peers. Minimized windows are open;
    /// explicitly hidden retained scenes are not. Unsupported hosts stay unchanged.
    pub(super) fn raise_open_windows(&mut self, source: &WindowKey) {
        if !self.activation.may_restore(source, Instant::now()) {
            return;
        }
        let Some(activated) = self
            .windows
            .get(source)
            .and_then(|entry| entry.runtime.window())
        else {
            return;
        };
        for key in self.open_activation_order(source) {
            if let Some(window) = self
                .windows
                .get(&key)
                .and_then(|entry| entry.runtime.window())
                && let Err(error) = window.restore_and_raise(activated)
            {
                self.emit(RuntimeEvent::CommandFailed(format!(
                    "window restoration failed for '{}': {error}",
                    key.as_str()
                )));
            }
        }
    }

    /// Observes real minimization before focus callbacks change visibility.
    /// Restoring a minimized source proves activation; a newly minimized peer
    /// can instead transfer focus automatically inside the application.
    pub(super) fn observe_window_minimization(&mut self) {
        let now = Instant::now();
        for (key, entry) in &self.windows {
            if !entry.runtime.is_requested_visible() {
                self.activation.minimized.remove(key);
                self.activation.restored.remove(key);
            } else if let Some(minimized) = entry
                .runtime
                .window()
                .and_then(|window| window.is_minimized())
            {
                self.activation.observe(key, minimized, now);
            }
        }
    }

    /// Returns only requested-open windows in a stable stacking order ending
    /// with `source`. Stale focus from a hidden or removed source does nothing.
    fn open_activation_order(&self, source: &WindowKey) -> Vec<WindowKey> {
        if !self
            .windows
            .get(source)
            .is_some_and(|entry| entry.runtime.is_requested_visible())
        {
            return Vec::new();
        }
        let mut keys: Vec<_> = self
            .windows
            .iter()
            .filter(|(key, entry)| *key != source && entry.runtime.is_requested_visible())
            .map(|(key, _)| key.clone())
            .collect();
        keys.sort_by(|left, right| left.as_str().cmp(right.as_str()));
        if !keys.is_empty() {
            keys.push(source.clone());
        }
        keys
    }
}

impl WindowActivationState {
    /// Defers automatic group restoration during the WM's focus transfer.
    /// `now` is the native request time; no polling timer is installed.
    pub(super) fn minimize_requested(&mut self, now: Instant) {
        self.minimize_focus_until = Some(now + MINIMIZE_FOCUS_GRACE);
    }

    /// Records `window`'s actual minimization and restoration at `now`.
    fn observe(&mut self, window: &WindowKey, minimized: bool, now: Instant) {
        if minimized {
            if self.minimized.insert(window.clone()) {
                self.restored.remove(window);
                self.minimize_requested(now);
            }
        } else if self.minimized.remove(window) {
            self.restored.insert(window.clone());
        }
    }

    /// Allows OS restoration immediately, otherwise excludes focus fallback
    /// during a native minimization. Later activation is allowed normally.
    fn may_restore(&mut self, source: &WindowKey, now: Instant) -> bool {
        self.restored.remove(source) || self.minimize_focus_until.is_none_or(|until| now >= until)
    }
}

#[cfg(test)]
#[path = "../../tests/multi/activation.rs"]
mod tests;
