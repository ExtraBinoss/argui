use argui_render::RendererError;

#[test]
fn renderer_errors_keep_actionable_context() {
    assert_eq!(
        RendererError::SurfaceCreation("bad handle".into()).to_string(),
        "surface creation failed: bad handle"
    );
    assert_eq!(
        RendererError::AdapterRequest("none".into()).to_string(),
        "adapter request failed: none"
    );
    assert_eq!(
        RendererError::DeviceRequest("denied".into()).to_string(),
        "device request failed: denied"
    );
    assert_eq!(
        RendererError::UnsupportedSurface.to_string(),
        "the adapter cannot present to this surface"
    );
    assert_eq!(
        RendererError::Validation.to_string(),
        "surface validation failed"
    );
}
