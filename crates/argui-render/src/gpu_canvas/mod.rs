mod cache;
mod pipeline;
mod registry;
mod target;

pub(crate) use pipeline::CanvasGpu;

use std::{fmt, time::Duration};

use argui_core::Rect;
use argui_paint::{GpuCanvasId, RenderObjectId};

pub use registry::{
    GpuCanvasRegistration, GpuCanvasRegistry, GpuCanvasRegistryError, GpuCanvasRequirements,
};

/// Error explicitly returned by an application GPU-canvas factory or renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuCanvasError {
    message: String,
}

impl GpuCanvasError {
    /// Creates an application error with a concise actionable English message.
    #[must_use]
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the application-provided error message.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for GpuCanvasError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for GpuCanvasError {}

/// Creates one application-owned renderer for each surface renderer that uses a registration.
pub trait GpuCanvasFactory: Send + Sync + 'static {
    /// Declares required and optional device capabilities before device creation.
    fn requirements(&self) -> GpuCanvasRequirements {
        GpuCanvasRequirements::default()
    }

    /// Creates the retained renderer using Argui's selected device and queue.
    ///
    /// Device resources created here may be retained by the returned renderer.
    ///
    /// # Errors
    ///
    /// Returns an application error when renderer resources cannot be created.
    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError>;
}

/// Encodes application GPU work for one retained canvas texture.
///
/// Native renderers are `Send + Sync` so [`crate::SurfaceRenderer`] preserves
/// its native thread traits. WebGPU handles are intentionally thread-local on
/// `wasm32`, so this trait has no thread bound there.
#[cfg(not(target_arch = "wasm32"))]
pub trait GpuCanvasRenderer: Send + Sync + 'static {
    /// Encodes all compute, render and copy work for a dirty canvas.
    ///
    /// # Errors
    ///
    /// Returns an application error to discard this canvas encoder and display
    /// a recoverable placeholder without aborting the surrounding UI frame.
    fn render(&mut self, context: &mut GpuCanvasRenderContext<'_>) -> Result<(), GpuCanvasError>;
}

/// Encodes application GPU work for one retained canvas texture.
///
/// WebGPU handles are thread-local in browsers. The methods and behavior are
/// otherwise identical to the native `Send + Sync` trait.
#[cfg(target_arch = "wasm32")]
pub trait GpuCanvasRenderer: 'static {
    /// Encodes all compute, render and copy work for a dirty canvas.
    ///
    /// # Errors
    ///
    /// Returns an application error to discard this canvas encoder and display
    /// a recoverable placeholder without aborting the surrounding UI frame.
    fn render(&mut self, context: &mut GpuCanvasRenderContext<'_>) -> Result<(), GpuCanvasError>;
}

/// Read-only device information supplied while a canvas renderer is created.
pub struct GpuCanvasDeviceContext<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) features: wgpu::Features,
    pub(crate) limits: wgpu::Limits,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) generation: u64,
}

impl GpuCanvasDeviceContext<'_> {
    /// Returns Argui's selected WGPU device.
    ///
    /// Application-created resources may retain their normal owned WGPU handles.
    #[must_use]
    pub const fn device(&self) -> &wgpu::Device {
        self.device
    }

    /// Returns Argui's selected WGPU queue.
    ///
    /// `write_buffer` and `write_texture` are supported. Calling
    /// [`wgpu::Queue::submit`] is outside the GPU-canvas contract.
    #[must_use]
    pub const fn queue(&self) -> &wgpu::Queue {
        self.queue
    }

    /// Returns the feature set enabled on the selected device.
    #[must_use]
    pub const fn features(&self) -> wgpu::Features {
        self.features
    }

    /// Returns the effective limits enabled on the selected device.
    #[must_use]
    pub const fn limits(&self) -> &wgpu::Limits {
        &self.limits
    }

    /// Returns the sRGB texture format used by this surface renderer.
    #[must_use]
    pub const fn target_format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Returns an opaque identity shared by surfaces using the same device.
    #[must_use]
    pub const fn device_generation(&self) -> u64 {
        self.generation
    }
}

/// Borrowed WGPU state supplied while a dirty retained canvas is encoded.
pub struct GpuCanvasRenderContext<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) queue: &'a wgpu::Queue,
    pub(crate) encoder: &'a mut wgpu::CommandEncoder,
    pub(crate) target: &'a wgpu::TextureView,
    pub(crate) format: wgpu::TextureFormat,
    pub(crate) extent: [u32; 2],
    pub(crate) logical_bounds: Rect,
    pub(crate) scale_factor: f32,
    pub(crate) resolution_scale: f32,
    pub(crate) canvas: GpuCanvasId,
    pub(crate) object: RenderObjectId,
    pub(crate) slot: u32,
    pub(crate) frame: u64,
}

impl GpuCanvasRenderContext<'_> {
    /// Returns Argui's selected device; owned resources created from it may be retained.
    #[must_use]
    pub const fn device(&self) -> &wgpu::Device {
        self.device
    }

    /// Returns Argui's queue for resource writes.
    ///
    /// Calling `submit` or blocking device polling is unsupported. Encode frame
    /// work through [`Self::encoder`] so Argui preserves submission ordering.
    #[must_use]
    pub const fn queue(&self) -> &wgpu::Queue {
        self.queue
    }

    /// Returns the borrowed encoder for this dirty canvas.
    ///
    /// The reference must not be retained after [`GpuCanvasRenderer::render`].
    pub fn encoder(&mut self) -> &mut wgpu::CommandEncoder {
        self.encoder
    }

    /// Returns the Argui-owned offscreen target view.
    ///
    /// The reference must not be retained after [`GpuCanvasRenderer::render`].
    #[must_use]
    pub const fn target_view(&self) -> &wgpu::TextureView {
        self.target
    }

    /// Borrows the encoder and target together for beginning a render pass.
    ///
    /// Neither reference may be retained after [`GpuCanvasRenderer::render`].
    pub fn encoder_and_target(&mut self) -> (&mut wgpu::CommandEncoder, &wgpu::TextureView) {
        (self.encoder, self.target)
    }

    /// Returns the sRGB target texture format.
    #[must_use]
    pub const fn target_format(&self) -> wgpu::TextureFormat {
        self.format
    }

    /// Returns the exact physical target width and height.
    #[must_use]
    pub const fn physical_extent(&self) -> [u32; 2] {
        self.extent
    }

    /// Returns the logical canvas viewport bounds.
    #[must_use]
    pub const fn logical_bounds(&self) -> Rect {
        self.logical_bounds
    }

    /// Returns the window logical-to-physical scale factor.
    #[must_use]
    pub const fn scale_factor(&self) -> f32 {
        self.scale_factor
    }

    /// Returns the explicit application resolution multiplier.
    #[must_use]
    pub const fn resolution_scale(&self) -> f32 {
        self.resolution_scale
    }

    /// Returns the canvas registration identity.
    #[must_use]
    pub const fn canvas_id(&self) -> GpuCanvasId {
        self.canvas
    }

    /// Returns the retained UI object identity.
    #[must_use]
    pub const fn object_id(&self) -> RenderObjectId {
        self.object
    }

    /// Returns the local slot within the retained UI object.
    #[must_use]
    pub const fn slot(&self) -> u32 {
        self.slot
    }

    /// Returns the monotonically wrapping surface-renderer frame number.
    #[must_use]
    pub const fn frame_number(&self) -> u64 {
        self.frame
    }
}

/// Stage at which a recoverable GPU-canvas failure occurred.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuCanvasFailureStage {
    /// Display-list registration lookup or descriptor validation.
    Validation,
    /// Retained texture allocation or memory-budget reservation.
    Allocation,
    /// Application factory creation.
    Creation,
    /// Application command encoding.
    Render,
}

/// Change in the recoverable state of one retained GPU canvas.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GpuCanvasDiagnosticKind {
    /// A new or changed failure was observed.
    Failed,
    /// A previously failing canvas rendered successfully.
    Recovered,
}

/// Window-local renderer diagnostic for one retained GPU-canvas instance.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GpuCanvasDiagnostic {
    /// Whether this record reports failure or recovery.
    pub kind: GpuCanvasDiagnosticKind,
    /// Stage associated with the failure or recovery.
    pub stage: GpuCanvasFailureStage,
    /// Stable registration label used in logs and UI.
    pub label: String,
    /// Registration identity referenced by the display list.
    pub canvas: GpuCanvasId,
    /// Retained UI object identity.
    pub object: RenderObjectId,
    /// Local slot within the retained object.
    pub slot: u32,
    /// Concise actionable English message.
    pub message: String,
}

/// Per-frame and retained-cache GPU-canvas statistics.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct GpuCanvasStats {
    /// Number of retained canvas textures.
    pub entries: usize,
    /// Exact retained texture bytes, excluding allocator overhead.
    pub allocated_bytes: u64,
    /// Dirty callbacks encoded during the frame.
    pub renders_this_frame: usize,
    /// Retained textures reused without invoking callbacks.
    pub hits_this_frame: usize,
    /// Canvas instances displaying a placeholder this frame.
    pub failures_this_frame: usize,
    /// CPU time spent preparing and invoking canvas callbacks.
    pub encode_time: Duration,
}
