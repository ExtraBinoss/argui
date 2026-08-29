fn gradient_palette(value: f32) -> vec3<f32> {
    let cyan = vec3<f32>(0.12, 0.82, 1.0);
    let violet = vec3<f32>(0.58, 0.25, 1.0);
    let coral = vec3<f32>(1.0, 0.24, 0.42);
    let first = mix(cyan, violet, smoothstep(0.0, 0.55, value));
    return mix(first, coral, smoothstep(0.48, 1.0, value));
}

fn argui_effect(
    uv: vec2<f32>,
    source: vec4<f32>,
    backdrop: vec4<f32>,
) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let local = (pixel - params.bounds.xy) / max(params.bounds.zw, vec2<f32>(1.0));
    let wave = 0.5 + 0.5 * sin(
        (local.x + local.y * 0.35) * 6.2831853 * max(argui_param_f32(1u), 0.01)
            - argui_param_f32(0u)
    );
    let original = source.rgb / max(source.a, 0.0001);
    let color = mix(original, gradient_palette(wave), clamp(argui_param_f32(2u), 0.0, 1.0));
    return vec4<f32>(color * source.a, source.a);
}
