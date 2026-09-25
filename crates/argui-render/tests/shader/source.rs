use argui_render::shader::wrap_effect_source;

#[test]
fn source_mapping_rejects_prefix_footer_and_undefined_spans() {
    let source = "fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> { return source; }";
    let wrapped = wrap_effect_source("source.wgsl", source);
    assert_eq!(wrapped.source_map.source_name(), "source.wgsl");
    assert!(
        wrapped
            .source_map
            .map_span(naga::Span::UNDEFINED, source)
            .is_none()
    );
    assert!(
        wrapped
            .source_map
            .map_span(naga::Span::new(0, 1), source)
            .is_none()
    );
    let footer = wrapped.source.len() - 1;
    assert!(
        wrapped
            .source_map
            .map_span(naga::Span::new(footer as u32, footer as u32 + 1), source)
            .is_none()
    );
    let start = wrapped.source.find(source).unwrap();
    assert!(
        wrapped
            .source_map
            .map_span(naga::Span::new(start as u32 + 1, start as u32 + 2), "")
            .is_none()
    );
    assert_ne!(
        wrapped.hash,
        wrap_effect_source("source.wgsl", "different").hash
    );
}
