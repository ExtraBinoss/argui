use argui_render::shader::{
    ShaderError, ShaderParameterMetadata, validate_effect_source, wrap_effect_source,
};

const VALID: &str = r#"
fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    return source * argui_param_f32(0u);
}
"#;

#[test]
fn valid_source_is_wrapped_hashed_and_validated_without_wgpu() {
    let parameters = [ShaderParameterMetadata::new("amount", 1)];
    let first = validate_effect_source("shaders/glow.wgsl", VALID, &parameters).unwrap();
    let second = validate_effect_source("shaders/glow.wgsl", VALID, &parameters).unwrap();
    assert_eq!(first.hash, second.hash);
    assert!(first.source.contains("@fragment"));
    assert!(first.source.contains(VALID));
}

#[test]
fn syntax_diagnostics_map_to_the_user_file() {
    let source = "\nfn argui_effect( -> vec4<f32> {\n    return vec4<f32>(1.0);\n}";
    let error = validate_effect_source("shaders/broken.wgsl", source, &[]).unwrap_err();
    let ShaderError::Diagnostic(diagnostic) = error else {
        panic!("expected source diagnostic");
    };
    assert_eq!(diagnostic.position.source_name, "shaders/broken.wgsl");
    assert_eq!(diagnostic.position.line, 2);
    assert!(diagnostic.position.column > 1);
}

#[test]
fn source_map_excludes_generated_abi_and_tracks_unicode_bytes() {
    let source = "// é\nfn argui_effect( -> vec4<f32> { return vec4<f32>(1.0); }";
    let wrapped = wrap_effect_source("unicode.wgsl", source);
    let declaration = wrapped.source.find("fn argui_effect( ->").unwrap();
    let offset = declaration + "fn argui_effect( ".len();
    let position = wrapped
        .source_map
        .map_span(naga::Span::new(offset as u32, offset as u32 + 2), source)
        .unwrap();
    assert_eq!((position.line, position.column), (2, 18));
}

#[test]
fn parameter_metadata_rejects_duplicates_and_zero_widths() {
    assert!(matches!(
        validate_effect_source(
            "effect.wgsl",
            VALID,
            &[
                ShaderParameterMetadata::new("same", 1),
                ShaderParameterMetadata::new("same", 1),
            ],
        ),
        Err(ShaderError::InvalidParameter(_))
    ));
    assert!(matches!(
        validate_effect_source(
            "effect.wgsl",
            VALID,
            &[ShaderParameterMetadata::new("empty", 0)],
        ),
        Err(ShaderError::InvalidParameter(_))
    ));
    assert!(matches!(
        validate_effect_source("effect.wgsl", VALID, &[ShaderParameterMetadata::new("", 1)],),
        Err(ShaderError::InvalidParameter(_))
    ));
}

#[test]
fn semantic_validation_failures_preserve_a_source_location() {
    let source = "fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source[9]; }";
    let error = validate_effect_source("semantic.wgsl", source, &[]).unwrap_err();
    let ShaderError::Diagnostic(diagnostic) = error else {
        panic!("expected Naga validation diagnostic");
    };
    assert!(diagnostic.position.line > 0);
    assert!(!diagnostic.message.is_empty());
}
