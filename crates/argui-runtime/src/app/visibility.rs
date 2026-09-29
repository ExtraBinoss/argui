use super::Application;
use crate::{RuntimeEvent, ViewUpdate};
use argui_platform::{PlatformEvent, WindowBackend};

#[cfg(test)]
#[path = "../../tests/app/presentation_visibility.rs"]
mod tests;

impl Application {
    /// Returns the requested open policy independently of minimization and occlusion.
    /// Retained hidden windows are excluded from grouped application activation.
    pub(crate) fn is_requested_visible(&self) -> bool {
        self.initial_visible
    }

    /// Recovers stale host occlusion when the pointer returns to its actual surface.
    /// Some compositors deliver pointer entry without another focus event.
    pub(super) fn host_pointer_entered(&mut self) {
        let backend = self
            .window
            .as_ref()
            .map(|window| window.capabilities().backend);
        self.host_pointer_entered_on_backend(backend);
    }

    /// Resumes a Wayland surface restored by the compositor with unchanged focus.
    /// `backend` identifies whether explicit hiding uses native minimization.
    fn host_pointer_entered_on_backend(&mut self, backend: Option<WindowBackend>) {
        if self.occluded || (!self.initial_visible && backend == Some(WindowBackend::Wayland)) {
            self.host_focus_changed_on_backend(true, backend);
        }
    }

    /// Applies platform occlusion and requests a frame when presentation resumes.
    /// Clearing occlusion must wake an idle event loop without waiting for input.
    pub(super) fn host_occlusion_changed(&mut self, occluded: bool) {
        self.occluded = occluded;
        self.sync_host_visibility();
        if !occluded && self.presentation_visible {
            self.pending_ui_frame.request_paint();
            self.sync_animations();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    /// Restores painting when the OS focuses a mapped surface after occlusion or background idle.
    pub(super) fn host_focus_changed(&mut self, focused: bool) {
        let backend = self
            .window
            .as_ref()
            .map(|window| window.capabilities().backend);
        self.host_focus_changed_on_backend(focused, backend);
    }

    /// Wayland implements hiding by minimizing. An authorized compositor activation
    /// can restore that retained window without passing through `ShowWindow`.
    fn host_focus_changed_on_backend(&mut self, focused: bool, backend: Option<WindowBackend>) {
        if focused {
            if backend == Some(WindowBackend::Wayland) {
                self.initial_visible = true;
            }
            // Some backends omit Occluded(false) when restoring a covered surface.
            // A real focus event proves the window is available for presentation again.
            self.occluded = false;
        }
        self.sync_host_visibility();
        if focused && self.presentation_visible {
            self.pending_ui_frame.request_paint();
            self.sync_animations();
            if let Some(window) = &self.window {
                window.request_redraw();
            }
        }
    }

    pub(crate) fn set_window_visible(&mut self, visible: bool) {
        if visible
            && !self.initial_visible
            && let Some(window) = self.window.as_ref().and_then(|host| host.winit())
            && let Err(error) =
                argui_platform::prepare_window_presentation(window, &self.window_config)
        {
            (self.on_event)(RuntimeEvent::CommandFailed(error));
            return;
        }
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
            if let super::RendererState::Ready(renderer) = &mut *self.renderer.borrow_mut() {
                renderer.reconfigure_surface();
            }
            // A remapped native surface needs fresh layout, paint, and hit
            // regions even when its retained JavaScript graph has not changed.
            self.pending_ui_frame.request_layout();
            self.invalidate(ViewUpdate::Rebuild);
        }
        (self.on_event)(RuntimeEvent::Platform(PlatformEvent::VisibilityChanged(
            visible,
        )));
    }
}
