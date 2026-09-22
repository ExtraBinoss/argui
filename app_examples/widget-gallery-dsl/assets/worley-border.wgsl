fn border_hash(cell: vec2<f32>) -> vec2<f32> {
    let x = sin(dot(cell, vec2<f32>(127.1, 311.7))) * 43758.5453;
    let y = sin(dot(cell, vec2<f32>(269.5, 183.3))) * 43758.5453;
    return fract(vec2<f32>(x, y));
}

fn border_worley(point: vec2<f32>) -> f32 {
    let cell = floor(point);
    let local = fract(point);
    var nearest = 2.0;
    for (var y = -1; y <= 1; y += 1) {
        for (var x = -1; x <= 1; x += 1) {
            let offset = vec2<f32>(f32(x), f32(y));
            let feature = offset + border_hash(cell + offset);
            nearest = min(nearest, length(feature - local));
        }
    }
    return nearest;
}

fn argui_effect(_uv: vec2<f32>, source: vec4<f32>, _backdrop: vec4<f32>) -> vec4<f32> {
    let pixel = global_pixel(_uv) - params.bounds.xy;
    let scale = max(argui_param_f32(1u), 0.01);
    let cells = border_worley(pixel * scale);
    let heat = 1.0 - smoothstep(0.12, 0.8, cells);
    let ember = vec3<f32>(0.96, 0.12, 0.02);
    let flame = vec3<f32>(1.0, 0.77, 0.18);
    let color = mix(ember, flame, heat);
    return vec4<f32>(mix(source.rgb, color, clamp(argui_param_f32(0u), 0.0, 1.0)), source.a);
}
