use super::Application;
use crate::{RuntimeEvent, ViewUpdate};
use argui_platform::PlatformEvent;

impl Application {
    pub(crate) fn set_window_visible(&mut self, visible: bool) {
        self.initial_visible = visible;
        if let Some(window) = &self.window {
            window.set_visible(visible);
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
