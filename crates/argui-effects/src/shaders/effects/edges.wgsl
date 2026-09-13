fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let normalized = vec3<f32>((pixel - params.bounds.xy) / max(params.bounds.zw, vec2<f32>(0.0001)), 1.0);
    let local = vec2<f32>(
        dot(normalized, vec3<f32>(argui_param_f32(6u), argui_param_f32(7u), argui_param_f32(8u))),
        dot(normalized, vec3<f32>(argui_param_f32(9u), argui_param_f32(10u), argui_param_f32(11u)))
    );
    let authored_size = vec2<f32>(argui_param_f32(12u), argui_param_f32(13u));
    let size = select(params.bounds.zw, authored_size, authored_size > vec2<f32>(0.0));
    let width = min(vec2<f32>(max(argui_param_f32(0u), 0.0)), size * 0.5);
    let distance = vec4<f32>(local * size, (vec2<f32>(1.0) - local) * size);
    let coverage = 1.0 - smoothstep(vec4<f32>(0.0), max(width.xyxy, vec4<f32>(0.0001)), distance);
    let strengths = vec4<f32>(argui_param_f32(2u), argui_param_f32(3u), argui_param_f32(4u), argui_param_f32(5u));
    let edges = clamp(coverage * strengths * argui_param_f32(1u), vec4<f32>(0.0), vec4<f32>(1.0));
    let amount = 1.0 - (1.0 - edges.x) * (1.0 - edges.y) * (1.0 - edges.z) * (1.0 - edges.w);
    if argui_param_f32(0u) <= 0.0 || any(local < vec2<f32>(0.0)) || any(local > vec2<f32>(1.0)) {
        return source;
    }
    if argui_param_bool(18u) {
        let color = vec4<f32>(argui_param_f32(14u), argui_param_f32(15u), argui_param_f32(16u), argui_param_f32(17u));
        // Composite the shadow over the entire viewport, including transparent gaps.
        let shadow_alpha = clamp(amount * color.a, 0.0, 1.0);
        let alpha = shadow_alpha + source.a * (1.0 - shadow_alpha);
        let rgb = color.rgb * shadow_alpha + source.rgb * source.a * (1.0 - shadow_alpha);
        return vec4<f32>(rgb / max(alpha, 0.000001), alpha);
    }
    // The custom-effect ABI handles premultiplication on input/output.
    return vec4<f32>(source.rgb, source.a * (1.0 - amount));
}
