use std::sync::{
    RwLock,
    atomic::{AtomicU64, Ordering},
};

/// Interactive scene state shared by the Argui model and GPU-canvas renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabState {
    pan: [f32; 2],
    target_pan: [f32; 2],
    zoom: f32,
    target_zoom: f32,
    elapsed: f32,
    revision: u64,
    paused: bool,
    force_error: bool,
    natural_vertical_drag: bool,
}

impl Default for LabState {
    /// Creates a running scene at its neutral pan and zoom.
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            target_pan: [0.0, 0.0],
            zoom: 1.0,
            target_zoom: 1.0,
            elapsed: 0.0,
            revision: 1,
            paused: false,
            force_error: false,
            natural_vertical_drag: true,
        }
    }
}

impl LabState {
    /// Returns the scene's logical pan offset.
    #[must_use]
    pub const fn pan_offset(&self) -> [f32; 2] {
        self.pan
    }

    /// Returns the clamped scene zoom factor.
    #[must_use]
    pub const fn zoom_factor(&self) -> f32 {
        self.zoom
    }

    /// Returns the destination zoom used by the smoothed camera animation.
    #[must_use]
    pub const fn target_zoom_factor(&self) -> f32 {
        self.target_zoom
    }

    /// Returns elapsed animation time in seconds.
    #[must_use]
    pub const fn elapsed_seconds(&self) -> f32 {
        self.elapsed
    }

    /// Returns the explicit revision supplied to `GpuCanvasSpec`.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.revision
    }

    /// Returns whether animation is paused.
    #[must_use]
    pub const fn paused(&self) -> bool {
        self.paused
    }

    /// Returns whether the demo renderer should emit a recoverable error.
    #[must_use]
    pub const fn force_error(&self) -> bool {
        self.force_error
    }

    /// Returns whether vertical pointer drags move the scene with the pointer.
    #[must_use]
    pub const fn natural_vertical_drag(&self) -> bool {
        self.natural_vertical_drag
    }

    /// Moves the camera destination by logical-pixel deltas.
    pub fn pan_by(&mut self, x: f32, y: f32) {
        if x.is_finite() && y.is_finite() {
            self.target_pan[0] += x;
            self.target_pan[1] += y;
        }
    }

    /// Multiplies the camera destination zoom by `factor` and clamps it to 0.2–8.0.
    pub fn zoom_by(&mut self, factor: f32) {
        if factor.is_finite() && factor > 0.0 {
            self.target_zoom = (self.target_zoom * factor).clamp(0.2, 8.0);
        }
    }

    /// Smoothly restores pan and zoom and immediately resets animation time.
    pub fn reset_view(&mut self) {
        self.target_pan = [0.0, 0.0];
        self.target_zoom = 1.0;
        self.elapsed = 0.0;
        self.bump();
    }

    /// Toggles pause state and returns the new value.
    pub fn toggle_paused(&mut self) -> bool {
        self.paused = !self.paused;
        self.bump();
        self.paused
    }

    /// Toggles the recoverable demo failure and returns the new value.
    pub fn toggle_error(&mut self) -> bool {
        self.force_error = !self.force_error;
        self.bump();
        self.force_error
    }

    /// Toggles vertical drag direction and returns whether natural mode is active.
    pub fn toggle_vertical_drag(&mut self) -> bool {
        self.natural_vertical_drag = !self.natural_vertical_drag;
        self.natural_vertical_drag
    }

    /// Returns whether the camera is still moving toward a requested view.
    #[must_use]
    pub fn view_is_settling(&self) -> bool {
        self.pan != self.target_pan || self.zoom != self.target_zoom
    }

    /// Advances animation and camera smoothing by finite non-negative `seconds`.
    ///
    /// Camera motion continues while particle animation is paused. Returns whether
    /// the visible scene and its revision changed.
    pub fn advance(&mut self, seconds: f32) -> bool {
        if !seconds.is_finite() || seconds <= 0.0 {
            return false;
        }
        let seconds = seconds.min(0.1);
        let blend = 1.0 - (-14.0 * seconds).exp();
        let mut changed = false;
        for axis in 0..2 {
            changed |= smooth_value(&mut self.pan[axis], self.target_pan[axis], blend, 0.01);
        }
        changed |= smooth_value(&mut self.zoom, self.target_zoom, blend, 0.0005);
        if !self.paused {
            self.elapsed += seconds;
            changed = true;
        }
        if changed {
            self.bump();
        }
        changed
    }

    /// Advances the explicit content revision after a visible scene mutation.
    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
}

/// Interpolates `current` toward `target` and snaps values within `epsilon`.
fn smooth_value(current: &mut f32, target: f32, blend: f32, epsilon: f32) -> bool {
    let difference = target - *current;
    if difference.abs() <= epsilon {
        if *current == target {
            return false;
        }
        *current = target;
        return true;
    }
    *current += difference * blend;
    true
}

pub(crate) struct SharedLab {
    state: RwLock<LabState>,
    diagnostic: RwLock<String>,
    capability: RwLock<String>,
    callback_count: AtomicU64,
}

impl Default for SharedLab {
    /// Creates synchronized scene, capability and diagnostic state.
    fn default() -> Self {
        Self {
            state: RwLock::new(LabState::default()),
            diagnostic: RwLock::new("No GPU canvas errors".into()),
            capability: RwLock::new("Waiting for renderer…".into()),
            callback_count: AtomicU64::new(0),
        }
    }
}

impl SharedLab {
    /// Copies and returns the scene under a short-lived read lock.
    pub(crate) fn state(&self) -> LabState {
        *self.state.read().unwrap_or_else(|error| error.into_inner())
    }

    /// Applies `update` under the scene write lock and releases it immediately.
    pub(crate) fn update(&self, update: impl FnOnce(&mut LabState)) {
        update(
            &mut self
                .state
                .write()
                .unwrap_or_else(|error| error.into_inner()),
        );
    }

    /// Returns cloned capability/diagnostic text and the callback counter.
    pub(crate) fn status(&self) -> (String, String, u64) {
        (
            self.capability
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            self.diagnostic
                .read()
                .unwrap_or_else(|error| error.into_inner())
                .clone(),
            self.callback_count.load(Ordering::Relaxed),
        )
    }

    /// Replaces the latest runtime diagnostic with `message`.
    pub(crate) fn set_diagnostic(&self, message: String) {
        *self
            .diagnostic
            .write()
            .unwrap_or_else(|error| error.into_inner()) = message;
    }

    /// Replaces the selected renderer capability summary with `message`.
    pub(crate) fn set_capability(&self, message: String) {
        *self
            .capability
            .write()
            .unwrap_or_else(|error| error.into_inner()) = message;
    }

    /// Increments the observable GPU-canvas callback count.
    pub(crate) fn record_callback(&self) {
        self.callback_count.fetch_add(1, Ordering::Relaxed);
    }
}
