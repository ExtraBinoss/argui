// Copyright 2025 Kyant. Licensed under the Apache License, Version 2.0.
// https://www.apache.org/licenses/LICENSE-2.0
// Distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND.
// See ../../../LICENSE-android-liquid-glass for the full license.
//
// Modified for Argui: WGSL port of Backdrop 2.0.0's rounded lens, seven-channel
// dispersion and directional highlight; linear premultiplied sampling, safe
// zero normals, bounded controls and color/tint composition added by Argui.
// https://github.com/Kyant0/AndroidLiquidGlass/blob/bebb11a91bd97bf1dabde479f3b332ad9898731f/backdrop/src/commonMain/kotlin/com/kyant/backdrop/internal/Shaders.kt

fn glass_normalize(v: vec2<f32>) -> vec2<f32> {
    return v / max(length(v), 0.00001);
}

fn glass_gradient(coord: vec2<f32>, half_size: vec2<f32>, radius: f32) -> vec2<f32> {
    let corner = abs(coord) - (half_size - vec2<f32>(radius));
    if corner.x >= 0.0 || corner.y >= 0.0 {
        return sign(coord) * glass_normalize(max(corner, vec2<f32>(0.0)));
    }
    let grad_x = step(corner.y, corner.x);
    return sign(coord) * vec2<f32>(grad_x, 1.0 - grad_x);
}

fn glass_sample(pixel: vec2<f32>) -> vec4<f32> {
    let p = clamp(pixel, params.source.xy + vec2<f32>(0.5),
        params.source.xy + params.source.zw - vec2<f32>(0.5));
    return argui_premultiply(source_at(p));
}

fn glass_spectrum(pixel: vec2<f32>, spread: vec2<f32>) -> vec4<f32> {
    let red = glass_sample(pixel + spread);
    let orange = glass_sample(pixel + spread * (2.0 / 3.0));
    let yellow = glass_sample(pixel + spread * (1.0 / 3.0));
    let green = glass_sample(pixel);
    let cyan = glass_sample(pixel - spread * (1.0 / 3.0));
    let blue = glass_sample(pixel - spread * (2.0 / 3.0));
    let purple = glass_sample(pixel - spread);
    return vec4<f32>(
        (red.r + orange.r + yellow.r) / 3.5 + purple.r / 7.0,
        (yellow.g + green.g + cyan.g) / 3.5 + orange.g / 7.0,
        (cyan.b + blue.b + purple.b) / 3.0,
        (red.a + orange.a + yellow.a + green.a + cyan.a + blue.a + purple.a) / 7.0);
}

fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    let pixel = global_pixel(uv);
    let half_size = params.bounds.zw * 0.5;
    let centered = pixel - params.bounds.xy - half_size;
    let sd = layer_rounded_distance(pixel);
    if sd > 0.0 { return source; }
    let radius = select(
        select(params.radii.x, params.radii.y, centered.x > 0.0),
        select(params.radii.w, params.radii.z, centered.x > 0.0), centered.y > 0.0);
    let grad_radius = min(radius * 1.5, min(half_size.x, half_size.y));
    let gradient = glass_gradient(centered, half_size, grad_radius);
    let height = argui_param_f32(4u);
    var color = source;
    if height > 0.0 && -sd < height && argui_param_f32(0u) > 0.0 {
        let x = clamp(1.0 + sd / height, 0.0, 1.0);
        // Lens.kt passes the negative amount: the lens samples inward.
        let distance = -(1.0 - sqrt(max(1.0 - x * x, 0.0))) * argui_param_f32(0u);
        let depth = select(0.0, 1.0, argui_param_bool(8u));
        let normal = glass_normalize(gradient + depth * glass_normalize(centered));
        let displacement = distance * normal;
        let refracted = pixel + displacement;
        let dispersion = argui_param_f32(1u)
            * (centered.x * centered.y) / max(half_size.x * half_size.y, 0.00001);
        if dispersion != 0.0 {
            color = argui_unpremultiply(glass_spectrum(refracted, displacement * dispersion));
        } else {
            color = argui_unpremultiply(glass_sample(refracted));
        }
    }
    let luma = dot(color.rgb, vec3<f32>(0.213, 0.715, 0.072));
    var rgb = mix(vec3<f32>(luma), color.rgb, argui_param_f32(5u));
    rgb = (rgb - vec3<f32>(0.5)) * argui_param_f32(7u)
        + vec3<f32>(0.5 + argui_param_f32(6u));
    let tint = vec3<f32>(argui_param_f32(9u), argui_param_f32(10u), argui_param_f32(11u));
    rgb = mix(rgb, tint, argui_param_f32(12u));
    // Backdrop's default highlight at 45 degrees, clipped to a narrow inner rim.
    let light = abs(dot(gradient, vec2<f32>(0.7071068)));
    let rim = 1.0 - smoothstep(0.0, 0.75, -sd);
    rgb += vec3<f32>(light * rim * argui_param_f32(3u));
    return vec4<f32>(clamp(rgb, vec3<f32>(0.0), vec3<f32>(1.0)), color.a);
}
