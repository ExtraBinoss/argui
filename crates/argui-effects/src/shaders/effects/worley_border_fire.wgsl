fn pcg2d(point: vec2<i32>) -> vec2<u32> {
    var value = bitcast<vec2<u32>>(point) * 1664525u + 1013904223u;
    value.x += value.y * 1664525u;
    value.y += value.x * 1664525u;
    value ^= value >> vec2<u32>(16u);
    value.x += value.y * 1664525u;
    value.y += value.x * 1664525u;
    value ^= value >> vec2<u32>(16u);
    return value;
}

fn hash_float(value: u32) -> f32 {
    return f32(value) * (1.0 / 4294967295.0);
}

fn worley_f1(position: vec2<f32>) -> f32 {
    let base = vec2<i32>(floor(position));
    var nearest = 1e10;
    for (var y = -1; y <= 1; y += 1) {
        for (var x = -1; x <= 1; x += 1) {
            let cell = base + vec2<i32>(x, y);
            let hash = pcg2d(cell);
            let feature = vec2<f32>(cell)
                + vec2<f32>(hash_float(hash.x), hash_float(hash.y));
            nearest = min(nearest, distance(position, feature));
        }
    }
    return nearest;
}

fn worley_fbm(position: vec2<f32>, phase: f32) -> f32 {
    var total = 0.0;
    var weight = 0.58;
    var frequency = 1.0;
    // Two octaves keep the cellular motion while bounding the cost of a
    // continuously animated UI decoration.
    for (var octave = 0; octave < 2; octave += 1) {
        let speed = 0.34 + 0.13 * f32(octave);
        let drift = vec2<f32>(f32(octave) * 7.13, -phase * speed);
        let cells = worley_f1(position * frequency + drift);
        total += (1.0 - smoothstep(0.08, 0.82, cells)) * weight;
        frequency *= 2.03;
        weight *= 0.5;
    }
    return total;
}

fn fire_palette(heat: f32) -> vec3<f32> {
    let ember = argui_srgb_to_linear(vec3<f32>(0.24, 0.004, 0.015));
    let red = argui_srgb_to_linear(vec3<f32>(0.95, 0.035, 0.005));
    let orange = argui_srgb_to_linear(vec3<f32>(1.0, 0.34, 0.015));
    let yellow = argui_srgb_to_linear(vec3<f32>(1.0, 0.94, 0.42));
    let low = mix(ember, red, smoothstep(0.0, 0.38, heat));
    let high = mix(orange, yellow, smoothstep(0.68, 1.0, heat));
    return mix(low, high, smoothstep(0.34, 0.76, heat));
}

fn argui_effect(
    uv: vec2<f32>,
    source: vec4<f32>,
    backdrop: vec4<f32>,
) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let signed_distance = layer_rounded_distance(pixel);
    let expansion = max(argui_param_f32(3u), 1.0);

    // The fire is an outer-border effect. Most fragments belong either to the
    // layer content or to the transparent area beyond its expansion. Avoid the
    // expensive Worley search for both; this is especially important for large
    // popovers, where the useful band is only a small fraction of the target.
    if signed_distance < -1.0 || signed_distance > expansion {
        return source;
    }

    let outside = smoothstep(-0.8, 0.8, signed_distance);
    let edge_fade = 1.0 - smoothstep(1.0, expansion, signed_distance);
    let metric = (pixel - params.bounds.xy) / min(params.bounds.z, params.bounds.w);
    let flow = metric * max(argui_param_f32(2u), 0.1);
    let turbulence = clamp(worley_fbm(flow, argui_param_f32(0u)) * 1.14, 0.0, 1.0);
    let radial_heat = pow(max(edge_fade, 0.0), 0.7);
    let heat = smoothstep(0.18, 0.92, turbulence * radial_heat + radial_heat * 0.28);
    let alpha = outside * edge_fade * smoothstep(0.08, 0.46, heat)
        * clamp(argui_param_f32(1u), 0.0, 1.0);
    let output_alpha = alpha + source.a * (1.0 - alpha);
    let premultiplied = fire_palette(heat) * alpha
        + source.rgb * source.a * (1.0 - alpha);
    let output_rgb = select(
        vec3<f32>(0.0),
        premultiplied / max(output_alpha, 0.000001),
        output_alpha > 0.000001,
    );
    return vec4<f32>(output_rgb, output_alpha);
}
