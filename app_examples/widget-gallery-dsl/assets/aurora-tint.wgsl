fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let strength = clamp(argui_param_f32(0u), 0.0, 1.0);
    let glow = vec3<f32>(0.33, 0.86, 0.96);
    return vec4<f32>(mix(source.rgb, glow, strength), source.a);
}
