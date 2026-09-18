use super::Application;
use crate::{RuntimeEvent, ViewUpdate};
use argui_platform::{PlatformEvent, WindowBackend};

impl Application {
    pub(crate) fn set_window_visible(&mut self, visible: bool) {
        self.initial_visible = visible;
        if let Some(window) = &self.window {
            if !visible
                && window.winit().is_some()
                && window.capabilities().backend == WindowBackend::Wayland
            {
                // Winit's Wayland `set_visible(false)` is intentionally a no-op.
                // Minimizing immediately removes the surface; a later authorized
                // XDG activation restores it for launcher-style applications.
                window.set_minimized(true);
            } else {
                window.set_visible(visible);
            }
        }
        self.sync_host_visibility();
    }

    pub(crate) fn sync_host_visibility(&mut self) {
        let visible = self.initial_visible
            && !self.occluded
            && !self
                .window
                .as_ref()
                .and_then(|window| window.is_minimized())
                .unwrap_or(false);
        if self.presentation_visible == visible {
            return;
        }
        self.presentation_visible = visible;
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        for popup in &self.popups.entries {
            popup.native.window().set_visible(visible && popup.shown);
        }
        if let Some(model) = &self.model {
            model.set_host_visible(visible);
        }
        self.sync_animations();
        if visible {
            self.invalidate(ViewUpdate::Rebuild);
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::VisibilityChanged(
            visible,
        )));
    }
}
