//! Compositor-specific presentation policy for retained native windows.

use crate::{app::Application, host::WindowHost};
use argui_platform::WindowBackend;
use argui_render::SurfaceRenderer;

impl Application {
    /// Configures `renderer` for the actual `window` backend. Linux surfaces prefer mailbox
    /// to avoid blocking other windows on occluded swapchains. Active animations
    /// have their own deadline even when mailbox is unavailable.
    pub(super) fn configure_native_presentation(
        &mut self,
        renderer: &mut SurfaceRenderer,
        window: &dyn WindowHost,
    ) {
        let paced = matches!(
            window.capabilities().backend,
            WindowBackend::Wayland | WindowBackend::X11
        );
        if paced {
            renderer.prefer_mailbox_presentation();
        }
        self.animations.set_native_pacing(paced);
    }
}
