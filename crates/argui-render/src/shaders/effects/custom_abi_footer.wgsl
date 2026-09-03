@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let pixel = params.target_region.xy + input.uv * params.target_region.zw;
    let source_uv = effect_allocated_uv(pixel, params.source, params.source_uv);
    let backdrop_uv = effect_allocated_uv(pixel, params.backdrop, params.backdrop_uv);
    let source = argui_unpremultiply(textureSampleLevel(source_texture, linear_sampler, source_uv, 0.0));
    let backdrop = argui_unpremultiply(textureSampleLevel(backdrop_texture, linear_sampler, backdrop_uv, 0.0));
    return argui_premultiply(argui_effect(input.uv, source, backdrop));
}
