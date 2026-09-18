use argui_core::{KeyInput, PinchUpdate, Point, PointerId, PointerPhase, ScrollDelta};

use super::{
    Application,
    zoom::{ZoomCommand, magnify_zoom, wheel_zoom},
};

#[cfg_attr(coverage_nightly, coverage(off))]
impl Application {
    /// Configures runtime UI zoom and installs the current application-wide factor.
    ///
    /// `enabled` controls built-in keyboard and touch gestures; `factor` is the
    /// factor already shared by other windows. Returns the configured runtime.
    pub(crate) fn ui_zoom(mut self, enabled: bool, factor: f32) -> Self {
        self.ui_zoom_enabled = enabled;
        self.install_ui_zoom(factor, false);
        self
    }

    /// Handles a normalized keyboard shortcut owned by the runtime.
    ///
    /// `input` is the current keyboard event. Returns whether the event was
    /// consumed as a UI zoom command.
    pub(super) fn handle_ui_zoom_key(&mut self, input: &KeyInput) -> bool {
        if !self.ui_zoom_enabled {
            return false;
        }
        let Some(command) = ZoomCommand::from_key_input(input) else {
            return false;
        };
        self.install_ui_zoom(command.apply(self.ui_zoom_factor), true);
        true
    }

    /// Handles a Control/Command wheel or browser trackpad pinch sample.
    ///
    /// `delta` follows Argui's scroll convention. Returns whether the runtime
    /// consumed the gesture as application-wide UI zoom.
    pub(super) fn handle_ui_zoom_wheel(&mut self, delta: ScrollDelta) -> bool {
        if !self.ui_zoom_enabled || !self.modifiers.command() || self.modifiers.alt {
            return false;
        }
        if let Some(factor) = wheel_zoom(self.ui_zoom_factor, delta) {
            self.install_ui_zoom(factor, true);
        }
        true
    }

    /// Handles one incremental native trackpad magnification sample.
    ///
    /// `delta` is positive when magnifying. Returns whether the runtime consumed
    /// the gesture as application-wide UI zoom.
    pub(super) fn handle_ui_zoom_magnify(&mut self, delta: f64) -> bool {
        if !self.ui_zoom_enabled {
            return false;
        }
        if let Some(factor) = magnify_zoom(self.ui_zoom_factor, delta) {
            self.install_ui_zoom(factor, true);
        }
        true
    }

    /// Feeds a physical touch sample to the runtime's global pinch recognizer.
    ///
    /// `id`, `phase`, and `position` identify the contact in physical pixels.
    /// The return value tells input dispatch whether this sample belongs to the
    /// global zoom gesture and whether existing UI touch state must be cancelled.
    pub(super) fn handle_ui_zoom_touch(
        &mut self,
        id: PointerId,
        phase: PointerPhase,
        position: Point,
    ) -> PinchUpdate {
        if !self.ui_zoom_enabled {
            return PinchUpdate::default();
        }
        let update = self.touch_zoom.observe(id, phase, position);
        if let Some(scale) = update.scale {
            let factor = self.ui_zoom_factor * scale;
            if (factor - self.ui_zoom_factor).abs() >= 0.005 {
                self.install_ui_zoom(factor, true);
            }
        }
        update
    }

    /// Removes and returns a zoom factor requested by local input.
    ///
    /// The returned factor is consumed by the multi-window host so it can
    /// synchronize every open window.
    pub(crate) fn take_ui_zoom_request(&mut self) -> Option<f32> {
        self.pending_ui_zoom.take()
    }

    /// Installs a UI zoom factor and invalidates every coordinate-dependent frame resource.
    ///
    /// `factor` is clamped to the supported accessibility range. `announce`
    /// records the change for the multi-window host to propagate.
    pub(crate) fn install_ui_zoom(&mut self, factor: f32, announce: bool) {
        let factor = factor.clamp(0.5, 3.0);
        if (factor - self.ui_zoom_factor).abs() < f32::EPSILON {
            return;
        }
        self.ui_zoom_factor = factor;
        self.environment.ui_zoom = factor;
        self.scale_factor = self.native_scale_factor * factor;
        if announce {
            self.pending_ui_zoom = Some(factor);
        }
        if let Some(window) = self.window.clone() {
            let size = window.drawable_size();
            self.update_viewport(size.width, size.height);
            self.set_safe_area_insets(self.window_config.safe_area_insets);
            self.update_ime(window.as_ref());
            self.pending_ui_frame.request_rebuild();
            self.pending_ui_frame.request_layout();
            #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
            self.invalidate_popup_environment();
            window.request_redraw();
        }
    }

    /// Converts a platform-logical point into the zoomed UI coordinate space.
    ///
    /// `point` is expressed in host logical pixels. The return value is suitable
    /// for layout hit testing.
    #[cfg(all(feature = "webview", target_os = "linux"))]
    pub(super) fn platform_to_ui_point(&self, point: Point) -> Point {
        Point::new(point.x / self.ui_zoom_factor, point.y / self.ui_zoom_factor)
    }
}
