fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let strength = clamp(argui_param_f32(0u), 0.0, 1.0);
    let frequency = clamp(argui_param_f32(1u), 4.0, 32.0);
    let wave = 0.5 + 0.5 * sin((uv.x * 2.0 + uv.y) * frequency);
    let tint = mix(
        argui_srgb_to_linear(vec3<f32>(0.12, 0.53, 0.98)),
        argui_srgb_to_linear(vec3<f32>(0.96, 0.27, 0.54)),
        wave,
    );
    let colored = mix(backdrop.rgb, tint, strength * 0.72);
    return vec4<f32>(mix(colored, source.rgb, source.a * 0.18), backdrop.a);
}
