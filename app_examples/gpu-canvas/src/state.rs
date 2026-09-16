use std::sync::{
    RwLock,
    atomic::{AtomicU64, Ordering},
};

/// Interactive scene state shared by the Argui model and GPU-canvas renderer.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LabState {
    pan: [f32; 2],
    zoom: f32,
    elapsed: f32,
    revision: u64,
    paused: bool,
    force_error: bool,
}

impl Default for LabState {
    /// Creates a running scene at its neutral pan and zoom.
    fn default() -> Self {
        Self {
            pan: [0.0, 0.0],
            zoom: 1.0,
            elapsed: 0.0,
            revision: 1,
            paused: false,
            force_error: false,
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

    /// Pans the scene by logical-pixel deltas and advances its revision.
    pub fn pan_by(&mut self, x: f32, y: f32) {
        self.pan[0] += x;
        self.pan[1] += y;
        self.bump();
    }

    /// Multiplies the zoom by `factor`, clamps it to 0.2–8.0, and advances the revision.
    pub fn zoom_by(&mut self, factor: f32) {
        if factor.is_finite() && factor > 0.0 {
            self.zoom = (self.zoom * factor).clamp(0.2, 8.0);
            self.bump();
        }
    }

    /// Restores pan, zoom and animation time while preserving pause state.
    pub fn reset_view(&mut self) {
        self.pan = [0.0, 0.0];
        self.zoom = 1.0;
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

    /// Advances animation by finite non-negative `seconds` only while running.
    ///
    /// Returns whether the scene and its revision changed.
    pub fn advance(&mut self, seconds: f32) -> bool {
        if self.paused || !seconds.is_finite() || seconds < 0.0 {
            return false;
        }
        self.elapsed += seconds.min(0.1);
        self.bump();
        true
    }

    /// Advances the explicit content revision after a visible scene mutation.
    fn bump(&mut self) {
        self.revision = self.revision.wrapping_add(1);
    }
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
