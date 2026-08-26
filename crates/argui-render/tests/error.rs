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
        RendererError::GlyphAtlasFull.to_string(),
        "the bounded glyph atlas is full"
    );
    assert_eq!(
        RendererError::Validation.to_string(),
        "surface validation failed"
    );
    assert_eq!(
        RendererError::InvalidDisplayList("bad nesting".into()).to_string(),
        "invalid display list: bad nesting"
    );
    assert_eq!(
        RendererError::InvalidShader("bad WGSL".into()).to_string(),
        "invalid effect shader: bad WGSL"
    );
    assert_eq!(
        RendererError::MissingShader(42).to_string(),
        "effect shader 42 is not registered"
    );
    assert_eq!(
        RendererError::TooManyEffectParameters {
            provided: 25,
            maximum: 24,
        }
        .to_string(),
        "custom effect has 25 parameters but the ABI supports 24"
    );
}
