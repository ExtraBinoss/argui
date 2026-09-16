use argui_paint::{GpuCanvasId, ImageSampling};

/// Renderer-independent options for a retained GPU-canvas element.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GpuCanvasSpec {
    canvas: GpuCanvasId,
    content_revision: u64,
    resolution_scale: f32,
    sampling: ImageSampling,
}

impl GpuCanvasSpec {
    /// Smallest supported explicit canvas resolution multiplier.
    pub const MIN_RESOLUTION_SCALE: f32 = 0.125;
    /// Largest supported explicit canvas resolution multiplier.
    pub const MAX_RESOLUTION_SCALE: f32 = 4.0;

    /// Creates a canvas specification for `canvas` at revision zero.
    #[must_use]
    pub const fn new(canvas: GpuCanvasId) -> Self {
        Self {
            canvas,
            content_revision: 0,
            resolution_scale: 1.0,
            sampling: ImageSampling::Linear,
        }
    }

    /// Sets the application-authored pixel-content revision.
    ///
    /// Increment this value whenever application data used by the GPU callback
    /// changes. Geometry-only changes do not require a new revision.
    #[must_use]
    pub const fn content_revision(mut self, revision: u64) -> Self {
        self.content_revision = revision;
        self
    }

    /// Sets the sampling mode used to compose the retained texture.
    #[must_use]
    pub const fn sampling(mut self, sampling: ImageSampling) -> Self {
        self.sampling = sampling;
        self
    }

    /// Sets the texture-resolution multiplier after normal DPI scaling.
    ///
    /// Finite positive values are clamped to the supported range. Non-finite
    /// and non-positive values normalize to the default of `1.0`.
    #[must_use]
    pub fn resolution_scale(mut self, scale: f32) -> Self {
        self.resolution_scale = if scale.is_finite() && scale > 0.0 {
            scale.clamp(Self::MIN_RESOLUTION_SCALE, Self::MAX_RESOLUTION_SCALE)
        } else {
            1.0
        };
        self
    }

    /// Returns the registered canvas factory identity.
    #[must_use]
    pub const fn canvas(&self) -> GpuCanvasId {
        self.canvas
    }

    /// Returns the application-authored pixel-content revision.
    #[must_use]
    pub const fn revision(&self) -> u64 {
        self.content_revision
    }

    /// Returns the validated explicit texture-resolution multiplier.
    #[must_use]
    pub const fn scale(&self) -> f32 {
        self.resolution_scale
    }

    /// Returns the composition sampling mode.
    #[must_use]
    pub const fn image_sampling(&self) -> ImageSampling {
        self.sampling
    }
}

impl crate::Element {
    /// Creates a retained GPU-canvas leaf using renderer-independent `spec`.
    ///
    /// Layout, interaction, semantics, transforms, clipping, opacity, rounded
    /// corners and effects use the normal [`crate::Element`] APIs.
    #[must_use]
    pub fn gpu_canvas(spec: GpuCanvasSpec) -> Self {
        let mut element = Self::container([]);
        element.kind = crate::ElementKind::GpuCanvas(spec);
        element
    }
}
