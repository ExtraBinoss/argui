// Separable Gaussian, sampled over +/- 3 sigma. Keep alpha premultiplied so
// transparent source edges cannot produce dark fringes under the lens.
fn glass_blur_axis(uv: vec2<f32>, source: vec4<f32>, axis: vec2<f32>) -> vec4<f32> {
    let sigma = argui_param_f32(2u);
    if sigma <= 0.0 { return source; }
    let pixel = global_pixel(uv);
    var color = vec4<f32>(0.0);
    var weight = 0.0;
    for (var i = -4; i <= 4; i += 1) {
        let distance = f32(i) * 0.75;
        let w = exp(-0.5 * distance * distance);
        let p = clamp(pixel + axis * distance * sigma,
            params.source.xy + vec2<f32>(0.5),
            params.source.xy + params.source.zw - vec2<f32>(0.5));
        color += argui_premultiply(source_at(p)) * w;
        weight += w;
    }
    return argui_unpremultiply(color / weight);
}
