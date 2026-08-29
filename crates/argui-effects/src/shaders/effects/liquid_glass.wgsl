fn glass_normal(pixel: vec2<f32>) -> vec2<f32> {
    let dx = layer_rounded_distance(pixel + vec2<f32>(1.0, 0.0))
        - layer_rounded_distance(pixel - vec2<f32>(1.0, 0.0));
    let dy = layer_rounded_distance(pixel + vec2<f32>(0.0, 1.0))
        - layer_rounded_distance(pixel - vec2<f32>(0.0, 1.0));
    let gradient = vec2<f32>(dx, dy);
    return gradient / max(length(gradient), 0.0001);
}

fn glass_blur(pixel: vec2<f32>, radius: f32) -> vec4<f32> {
    let step = radius * 0.5;
    return (
        source_at(pixel) * 4.0
        + source_at(pixel + vec2<f32>(step, 0.0))
        + source_at(pixel - vec2<f32>(step, 0.0))
        + source_at(pixel + vec2<f32>(0.0, step))
        + source_at(pixel - vec2<f32>(0.0, step))
    ) / 8.0;
}

fn argui_effect(
    uv: vec2<f32>,
    source: vec4<f32>,
    backdrop: vec4<f32>,
) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let distance = layer_rounded_distance(pixel);
    let edge_width = max(argui_param_f32(4u), 1.0);

    // The layer target can include an outer-effect expansion (for example a
    // glow or border fire). Liquid glass is clipped to the actual layer, so do
    // not run its texture taps in that invisible outer area.
    if distance > 1.0 {
        return source;
    }

    let edge = 1.0 - smoothstep(-edge_width, 0.0, distance);
    let normal = glass_normal(pixel);
    let warped = pixel - normal * argui_param_f32(0u) * edge * edge;
    let blurred = glass_blur(warped, max(argui_param_f32(2u), 0.0));
    let chroma = normal * argui_param_f32(1u) * edge;
    let glass = vec4<f32>(
        source_at(warped + chroma).r,
        blurred.g,
        source_at(warped - chroma).b,
        blurred.a
    );
    let local = (pixel - params.bounds.xy) / params.bounds.zw;
    let light = edge * smoothstep(0.9, 0.1, local.y) * max(argui_param_f32(3u), 0.0);
    let luminance = dot(glass.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
    let saturated = mix(vec3<f32>(luminance), glass.rgb, max(argui_param_f32(5u), 0.0));
    let lit = clamp(saturated + vec3<f32>(light), vec3<f32>(0.0), vec3<f32>(1.0));
    return vec4<f32>(lit, glass.a);
}
