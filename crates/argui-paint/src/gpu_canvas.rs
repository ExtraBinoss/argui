use std::sync::atomic::{AtomicU64, Ordering};

use argui_core::{Affine2D, Rect};

use crate::{ClipChain, CornerRadii, ImageSampling, RenderObjectId};

/// Process-local identity of a registered GPU-canvas factory.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct GpuCanvasId(u64);

static NEXT_GPU_CANVAS_ID: AtomicU64 = AtomicU64::new(1);

impl GpuCanvasId {
    /// Converts a nonzero process-local numeric identity received over the native host wire.
    ///
    /// `raw` must come from a registration in this process. A caller must still
    /// check membership in its renderer's canvas registry before using the ID.
    #[must_use]
    pub const fn from_raw(raw: u64) -> Option<Self> {
        if raw == 0 { None } else { Some(Self(raw)) }
    }

    /// Allocates a process-local opaque GPU-canvas identity.
    ///
    /// Registrations normally call this on behalf of applications.
    ///
    /// # Panics
    ///
    /// Panics if the process-local identity space is exhausted.
    #[must_use]
    pub fn fresh() -> Self {
        let id = NEXT_GPU_CANVAS_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "GPU-canvas identity space exhausted");
        Self(id)
    }

    /// Returns the process-local numeric identity for diagnostics.
    #[must_use]
    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Renderer-neutral description of one retained GPU-canvas viewport.
#[derive(Clone, Debug, PartialEq)]
pub struct GpuCanvasPrimitive {
    /// Registration selecting the application-owned renderer factory.
    pub canvas: GpuCanvasId,
    /// Retained UI object identity used to own cached GPU resources.
    pub object: RenderObjectId,
    /// Application-selected sub-identity within one retained object.
    pub slot: u32,
    /// Logical viewport bounds before `transform` is applied.
    pub bounds: Rect,
    /// Application-authored revision of the canvas pixels.
    pub content_revision: u64,
    /// Explicit multiplier applied after the window scale factor.
    pub resolution_scale: f32,
    /// Sampling used while composing the retained canvas texture.
    pub sampling: ImageSampling,
    /// Straight-alpha opacity applied during composition.
    pub opacity: f32,
    /// Rounded corners applied during composition.
    pub radii: CornerRadii,
    /// Transform from logical canvas coordinates to the target surface.
    pub transform: Affine2D,
    /// Ancestor clipping regions in target-surface coordinates.
    pub clips: ClipChain,
}
