// Optical rim adapted from Liquid Glass Studio's WGSL, MIT (c) 2024 Charles Yin.
// https://github.com/iyinchao/liquid-glass-studio/blob/d13c3e53813ebc1d8b52d878071b63212a550ebc/src/shaders-wgsl/fragment-main.wgsl
// See ../../../LICENSE-liquid-glass-studio. Argui uses linear color, its own
// rounded SDF and bounded pixel offsets rather than viewport-relative offsets.
// Seeded gradient noise summed as fractal noise, analogous to feTurbulence.
fn glass_hash(cell: vec2<i32>, channel: u32) -> u32 {
    var h = bitcast<u32>(cell.x) * 374761393u + bitcast<u32>(cell.y) * 668265263u;
    h = h ^ (argui_param_u32(8u) * 2246822519u + channel * 3266489917u);
    h = (h ^ (h >> 13u)) * 1274126177u;
    return h ^ (h >> 16u);
}
fn glass_gradient(cell: vec2<i32>, channel: u32) -> vec2<f32> {
    let gradients = array<vec2<f32>, 8>(
        vec2<f32>(1.0, 0.0), vec2<f32>(-1.0, 0.0),
        vec2<f32>(0.0, 1.0), vec2<f32>(0.0, -1.0),
        vec2<f32>(0.7071068, 0.7071068), vec2<f32>(-0.7071068, 0.7071068),
        vec2<f32>(0.7071068, -0.7071068), vec2<f32>(-0.7071068, -0.7071068));
    return gradients[glass_hash(cell, channel) & 7u];
}
fn glass_noise(p: vec2<f32>, channel: u32) -> f32 {
    let cell = vec2<i32>(floor(p));
    let local = fract(p);
    let ease = local * local * local * (local * (local * 6.0 - 15.0) + 10.0);
    let a = dot(glass_gradient(cell, channel), local);
    let b = dot(glass_gradient(cell + vec2<i32>(1, 0), channel), local - vec2<f32>(1.0, 0.0));
    let c = dot(glass_gradient(cell + vec2<i32>(0, 1), channel), local - vec2<f32>(0.0, 1.0));
    let d = dot(glass_gradient(cell + vec2<i32>(1, 1), channel), local - vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, ease.x), mix(c, d, ease.x), ease.y) * 1.4142136;
}
fn glass_fractal(p: vec2<f32>) -> vec2<f32> {
    var point = p;
    var amplitude = 1.0;
    var total = vec2<f32>(0.0);
    var weight = 0.0;
    for (var octave = 0u; octave < min(argui_param_u32(7u), 6u); octave++) {
        total += vec2<f32>(glass_noise(point, 0u), glass_noise(point, 1u)) * amplitude;
        weight += amplitude;
        amplitude *= 0.5;
        point *= 2.0;
    }
    return total / max(weight, 1.0);
}
fn glass_normal(pixel: vec2<f32>) -> vec2<f32> {
    let gradient = vec2<f32>(
        layer_rounded_distance(pixel + vec2<f32>(1.0, 0.0)) - layer_rounded_distance(pixel - vec2<f32>(1.0, 0.0)),
        layer_rounded_distance(pixel + vec2<f32>(0.0, 1.0)) - layer_rounded_distance(pixel - vec2<f32>(0.0, 1.0)));
    return gradient / max(length(gradient), 0.0001);
}
fn glass_sample(pixel: vec2<f32>) -> vec4<f32> {
    return source_at(clamp(pixel, params.source.xy + vec2<f32>(0.5), params.source.xy + params.source.zw - vec2<f32>(0.5)));
}
fn glass_blur(pixel: vec2<f32>, radius: f32) -> vec4<f32> {
    if radius <= 0.0 { return glass_sample(pixel); }
    let step = radius * 0.5;
    let sum = argui_premultiply(glass_sample(pixel)) * 4.0
        + argui_premultiply(glass_sample(pixel + vec2<f32>(step, 0.0)))
        + argui_premultiply(glass_sample(pixel - vec2<f32>(step, 0.0)))
        + argui_premultiply(glass_sample(pixel + vec2<f32>(0.0, step)))
        + argui_premultiply(glass_sample(pixel - vec2<f32>(0.0, step)));
    return argui_unpremultiply(sum / 8.0);
}
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let distance = layer_rounded_distance(pixel);
    if distance > 0.0 { return source; }
    let width = max(1.0, min(argui_param_f32(4u), min(params.bounds.z, params.bounds.w) * 0.25));
    let edge = clamp(1.0 + distance / width, 0.0, 1.0);
    let normal = glass_normal(pixel);
    // Studio's curved-bezel incidence and Snell transmission. A flat center has
    // zero displacement, unlike a noise field that deforms the entire window.
    let theta_i = asin(clamp(edge * edge, 0.0, 0.9999));
    let theta_t = asin(sin(theta_i) / argui_param_f32(14u));
    let edge_factor = min(max(-tan(theta_t - theta_i), 0.0), 1.5);
    var displacement = -normal * edge_factor * argui_param_f32(0u);
    if argui_param_f32(0u) > 0.0 && argui_param_f32(9u) > 0.0 {
        displacement += glass_fractal((pixel - params.bounds.xy) / max(argui_param_f32(6u), 1.0))
            * (0.5 * argui_param_f32(9u) * argui_param_f32(0u));
    }
    let warped = pixel + displacement;
    let radius = argui_param_f32(2u);
    let center = glass_blur(warped, radius);
    let chroma = normal * argui_param_f32(1u) * edge_factor;
    let glass = vec3<f32>(glass_blur(warped + chroma, radius).r, center.g, glass_blur(warped - chroma, radius).b);
    let luminance = dot(glass, vec3<f32>(0.2126, 0.7152, 0.0722));
    let saturated = mix(vec3<f32>(luminance), glass, argui_param_f32(5u));
    let tint = vec3<f32>(argui_param_f32(10u), argui_param_f32(11u), argui_param_f32(12u));
    var color = mix(saturated, tint, argui_param_f32(13u));
    // Studio's narrow fifth-power Fresnel rim and angular/opposite-side glare,
    // applied in Argui's linear color space instead of Studio's LCH conversions.
    let rim = pow(edge, 5.0);
    color = mix(color, vec3<f32>(1.0), rim * argui_param_f32(15u));
    let angle = atan2(normal.y, normal.x) - 0.785398163;
    let angular = pow(clamp(0.5 + 0.5 * sin(angle * 2.0), 0.0, 1.0), 2.0);
    let side = select(0.35, 1.0, dot(normal, vec2<f32>(-0.7071068, -0.7071068)) > 0.0);
    color += vec3<f32>(rim * angular * side * argui_param_f32(3u));
    return vec4<f32>(clamp(color, vec3<f32>(0.0), vec3<f32>(1.0)), center.a);
}
