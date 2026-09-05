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
        RendererError::MissingEffect("test.effect").to_string(),
        "effect 'test.effect' is not registered"
    );
    assert_eq!(
        RendererError::EffectParametersTooLarge {
            provided: 25,
            maximum: 24,
        }
        .to_string(),
        "effect parameters need 25 bytes but this adapter allows 24"
    );
}

#[test]
fn resource_and_effect_validation_errors_are_human_readable() {
    assert_eq!(
        RendererError::UnsupportedSurfaceTransparency.to_string(),
        "the surface does not support premultiplied transparent composition"
    );
    assert_eq!(
        RendererError::DuplicateEffect("blur").to_string(),
        "effect 'blur' is registered twice"
    );
    assert_eq!(
        RendererError::InvalidEffectDefinition("missing pass".into()).to_string(),
        "invalid effect definition: missing pass"
    );
    assert_eq!(
        RendererError::InvalidEffectParameters {
            effect: "blur",
            message: "negative radius".into(),
        }
        .to_string(),
        "invalid parameters for effect 'blur': negative radius"
    );
    assert_eq!(
        RendererError::MissingImage(9).to_string(),
        "image 9 is not registered"
    );
    assert_eq!(
        RendererError::MissingVector(11).to_string(),
        "vector 11 is not registered"
    );
    assert_eq!(
        RendererError::VectorAtlasFull.to_string(),
        "the bounded vector atlas is full"
    );
    assert_eq!(
        RendererError::VectorRasterization("invalid path".into()).to_string(),
        "SVG rasterization failed: invalid path"
    );
    assert_eq!(
        RendererError::ImageCacheFull {
            requested: 12,
            capacity: 8,
        }
        .to_string(),
        "image needs 12 bytes but the GPU image cache allows 8"
    );
    assert_eq!(
        RendererError::TooManyGradientStops {
            provided: 10,
            capacity: 4,
        }
        .to_string(),
        "display list has 10 gradient stops but the frame budget allows 4"
    );
}
