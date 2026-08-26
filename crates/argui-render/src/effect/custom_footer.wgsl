@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let source = textureSample(source_texture, linear_sampler, input.uv);
    let backdrop = textureSample(backdrop_texture, linear_sampler, input.uv);
    return argui_effect(input.uv, source, backdrop);
}
