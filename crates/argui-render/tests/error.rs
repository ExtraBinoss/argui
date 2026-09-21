use argui_paint::EffectId;
use argui_render::{RendererAttemptFailure, RendererError};

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
        RendererError::MissingEffect(EffectId::new("test.effect")).to_string(),
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
fn renderer_initialization_errors_list_every_attempt() {
    let attempts = vec![
        RendererAttemptFailure {
            renderer: "DirectX 12 with DirectComposition".into(),
            error: "device request failed: Parent device is lost".into(),
        },
        RendererAttemptFailure {
            renderer: "DirectX 12 with an opaque window surface".into(),
            error: "adapter request failed: no compatible adapter".into(),
        },
        RendererAttemptFailure {
            renderer: "Vulkan".into(),
            error: "surface creation failed: unsupported handle".into(),
        },
    ];
    let message = RendererError::Initialization {
        attempts,
        fallback_enabled: true,
    }
    .to_string();

    assert!(message.starts_with("renderer initialization failed after 3 attempts:"));
    assert!(message.contains(
        "- DirectX 12 with DirectComposition: device request failed: Parent device is lost"
    ));
    assert!(message.contains(
        "- DirectX 12 with an opaque window surface: adapter request failed: no compatible adapter"
    ));
    assert!(message.contains("- Vulkan: surface creation failed: unsupported handle"));
    assert!(message.ends_with(
        "No compatible GPU renderer could be started. Update the graphics driver or choose a supported renderer configuration."
    ));
}

#[test]
fn strict_renderer_initialization_error_explains_disabled_fallback() {
    let message = RendererError::Initialization {
        attempts: vec![RendererAttemptFailure {
            renderer: "DirectX 12 with DirectComposition".into(),
            error: "device request failed: Parent device is lost".into(),
        }],
        fallback_enabled: false,
    }
    .to_string();

    assert!(message.starts_with("renderer initialization failed after 1 attempt:"));
    assert!(message.ends_with(
        "Renderer fallback is disabled; only DirectX 12 with DirectComposition was attempted."
    ));
}

#[test]
fn resource_and_effect_validation_errors_are_human_readable() {
    assert_eq!(
        RendererError::UnsupportedSurfaceTransparency.to_string(),
        "the surface does not support premultiplied transparent composition"
    );
    assert_eq!(
        RendererError::DuplicateEffect(EffectId::new("blur")).to_string(),
        "effect 'blur' is registered twice"
    );
    assert_eq!(
        RendererError::InvalidEffectDefinition("missing pass".into()).to_string(),
        "invalid effect definition: missing pass"
    );
    assert_eq!(
        RendererError::InvalidEffectParameters {
            effect: EffectId::new("blur"),
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
