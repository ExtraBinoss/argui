use super::{RenderStatus, RendererError};
use wgpu::{CurrentSurfaceTexture, SurfaceTexture};

/// A drawable frame or the host action needed before presentation can continue.
pub(super) enum Acquisition {
    Ready {
        frame: SurfaceTexture,
        suboptimal: bool,
    },
    Deferred(RenderStatus),
}

/// Converts `outcome` into a drawable frame or a distinct recovery policy.
/// Timeouts retry without waiting for input; occluded surfaces remain idle.
///
/// # Errors
/// Returns a validation error when the GPU rejects frame acquisition.
pub(super) fn resolve(outcome: CurrentSurfaceTexture) -> Result<Acquisition, RendererError> {
    Ok(match outcome {
        CurrentSurfaceTexture::Success(frame) => Acquisition::Ready {
            frame,
            suboptimal: false,
        },
        CurrentSurfaceTexture::Suboptimal(frame) => Acquisition::Ready {
            frame,
            suboptimal: true,
        },
        CurrentSurfaceTexture::Timeout => Acquisition::Deferred(RenderStatus::Retry),
        CurrentSurfaceTexture::Occluded => Acquisition::Deferred(RenderStatus::Skipped),
        CurrentSurfaceTexture::Outdated => Acquisition::Deferred(RenderStatus::Reconfigure),
        CurrentSurfaceTexture::Lost => Acquisition::Deferred(RenderStatus::RecreateSurface),
        CurrentSurfaceTexture::Validation => return Err(RendererError::Validation),
    })
}
