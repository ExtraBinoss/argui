use argui_render::{RenderStatus, RendererError};
use wgpu::CurrentSurfaceTexture;

#[path = "../../src/surface/acquisition.rs"]
mod policy;

#[test]
fn timeouts_retry_while_occluded_surfaces_wait_for_visibility() {
    assert!(matches!(
        policy::resolve(CurrentSurfaceTexture::Timeout),
        Ok(policy::Acquisition::Deferred(RenderStatus::Retry))
    ));
    assert!(matches!(
        policy::resolve(CurrentSurfaceTexture::Occluded),
        Ok(policy::Acquisition::Deferred(RenderStatus::Skipped))
    ));
}

#[test]
fn invalid_surfaces_keep_their_distinct_recovery_actions() {
    assert!(matches!(
        policy::resolve(CurrentSurfaceTexture::Outdated),
        Ok(policy::Acquisition::Deferred(RenderStatus::Reconfigure))
    ));
    assert!(matches!(
        policy::resolve(CurrentSurfaceTexture::Lost),
        Ok(policy::Acquisition::Deferred(RenderStatus::RecreateSurface))
    ));
    assert!(matches!(
        policy::resolve(CurrentSurfaceTexture::Validation),
        Err(RendererError::Validation)
    ));
}
