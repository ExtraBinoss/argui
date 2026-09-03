struct Params {
    viewport: vec2<f32>,
    mode: u32,
    blend: u32,
    target_region: vec4<f32>,
    source: vec4<f32>,
    backdrop: vec4<f32>,
    source_uv: vec4<f32>,
    backdrop_uv: vec4<f32>,
    bounds: vec4<f32>,
    radii: vec4<f32>,
    color: vec4<f32>,
    data: vec4<f32>,
    matrix: array<vec4<f32>, 5>,
};

@group(0) @binding(0) var source_texture: texture_2d<f32>;
@group(0) @binding(1) var backdrop_texture: texture_2d<f32>;
@group(0) @binding(2) var linear_sampler: sampler;
@group(0) @binding(3) var<uniform> params: Params;

struct VertexOut {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
};

@vertex
fn vs_main(@builtin(vertex_index) vertex: u32) -> VertexOut {
    var positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0)
    );
    var output: VertexOut;
    output.position = vec4<f32>(positions[vertex], 0.0, 1.0);
    output.uv = positions[vertex] * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5);
    return output;
}

fn blend_rgb(source: vec3<f32>, backdrop: vec3<f32>, mode: u32) -> vec3<f32> {
    if mode == 1u { return source * backdrop; }
    if mode == 2u { return source + backdrop - source * backdrop; }
    if mode == 3u {
        return select(
            2.0 * source * backdrop,
            1.0 - 2.0 * (1.0 - source) * (1.0 - backdrop),
            backdrop >= vec3<f32>(0.5)
        );
    }
    if mode == 4u { return min(source, backdrop); }
    if mode == 5u { return max(source, backdrop); }
    if mode == 6u { return abs(backdrop - source); }
    if mode == 7u { return backdrop + source - 2.0 * backdrop * source; }
    if mode == 8u { return min(backdrop + source, vec3<f32>(1.0)); }
    return source;
}

fn global_pixel(uv: vec2<f32>) -> vec2<f32> {
    return params.target_region.xy + uv * params.target_region.zw;
}

fn region_uv(pixel: vec2<f32>, region: vec4<f32>) -> vec2<f32> {
    return (pixel - region.xy) / region.zw;
}

fn allocated_uv(pixel: vec2<f32>, region: vec4<f32>, allocation: vec4<f32>) -> vec2<f32> {
    return allocation.xy + region_uv(pixel, region) * allocation.zw;
}

fn in_region(pixel: vec2<f32>, region: vec4<f32>) -> bool {
    return all(pixel >= region.xy) && all(pixel < region.xy + region.zw);
}

fn sample_source(pixel: vec2<f32>) -> vec4<f32> {
    if !in_region(pixel, params.source) { return vec4<f32>(0.0); }
    return textureSampleLevel(source_texture, linear_sampler, allocated_uv(pixel, params.source, params.source_uv), 0.0);
}

fn sample_backdrop(pixel: vec2<f32>) -> vec4<f32> {
    if !in_region(pixel, params.backdrop) { return vec4<f32>(0.0); }
    return textureSampleLevel(backdrop_texture, linear_sampler, allocated_uv(pixel, params.backdrop, params.backdrop_uv), 0.0);
}

fn mask_coverage(pixel: vec2<f32>) -> f32 {
    let center = params.bounds.xy + params.bounds.zw * 0.5;
    let local = pixel - center;
    let radius = select(
        select(params.radii.x, params.radii.y, local.x > 0.0),
        select(params.radii.w, params.radii.z, local.x > 0.0),
        local.y > 0.0
    );
    let half_size = params.bounds.zw * 0.5;
    let q = abs(local) - half_size + vec2<f32>(radius);
    let distance = min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
    return clamp(0.5 - distance / max(fwidth(distance), 0.001), 0.0, 1.0);
}

fn composite(source: vec4<f32>, backdrop: vec4<f32>, pixel: vec2<f32>) -> vec4<f32> {
    let alpha = source.a * params.data.x * mask_coverage(pixel);
    let straight_source = source.rgb / max(source.a, 0.00001);
    let straight_backdrop = backdrop.rgb / max(backdrop.a, 0.00001);
    let blended = blend_rgb(straight_source, straight_backdrop, params.blend);
    let composited_source = mix(straight_source, blended, backdrop.a);
    return vec4<f32>(
        composited_source * alpha + backdrop.rgb * (1.0 - alpha),
        alpha + backdrop.a * (1.0 - alpha),
    );
}

fn sample_blur(pixel: vec2<f32>, axis: vec2<f32>) -> vec4<f32> {
    let source_size = vec2<f32>(textureDimensions(source_texture));
    let step = axis * max(params.data.x, 0.0) / source_size;
    let uv = allocated_uv(pixel, params.source, params.source_uv);
    let weights = array<f32, 7>(
        0.137023, 0.129618, 0.109719, 0.083108, 0.056331, 0.034167, 0.018544
    );
    var color = textureSampleLevel(source_texture, linear_sampler, uv, 0.0) * weights[0];
    for (var tap = 1u; tap < 7u; tap += 1u) {
        let offset = step * f32(tap);
        color += textureSampleLevel(source_texture, linear_sampler, uv + offset, 0.0) * weights[tap];
        color += textureSampleLevel(source_texture, linear_sampler, uv - offset, 0.0) * weights[tap];
    }
    return color;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let pixel = global_pixel(input.uv);
    let original = sample_source(pixel);
    var source = original;
    let backdrop = sample_backdrop(pixel);
    if params.mode == 0u { return composite(source, backdrop, pixel); }
    if params.mode == 1u { source = sample_blur(pixel, vec2<f32>(1.0, 0.0)); }
    if params.mode == 2u { source = sample_blur(pixel, vec2<f32>(0.0, 1.0)); }
    if params.mode == 8u {
        let straight = vec4<f32>(source.rgb / max(source.a, 0.00001), source.a);
        let transformed = vec4<f32>(
            dot(straight, params.matrix[0]) + params.matrix[4].x,
            dot(straight, params.matrix[1]) + params.matrix[4].y,
            dot(straight, params.matrix[2]) + params.matrix[4].z,
            dot(straight, params.matrix[3]) + params.matrix[4].w
        );
        source = vec4<f32>(transformed.rgb * transformed.a, transformed.a);
    }
    if params.mode == 9u {
        source = refracted_backdrop(pixel);
    }
    if params.mode == 10u || params.mode == 11u {
        let offset_pixel = pixel - params.data.yz;
        var alpha = sample_source(offset_pixel).a;
        if params.mode == 11u { alpha = 1.0 - alpha; }
        let spread_alpha = clamp(alpha * (1.0 + max(params.data.w, 0.0) * 0.08), 0.0, 1.0);
        let box_coverage = mask_coverage(pixel);
        let placement = select(1.0 - box_coverage, box_coverage, params.mode == 11u);
        let shadow_alpha = params.color.a * spread_alpha * placement;
        let shadow = vec4<f32>(params.color.rgb * shadow_alpha, shadow_alpha);
        return vec4<f32>(
            shadow.rgb + backdrop.rgb * (1.0 - shadow.a),
            shadow.a + backdrop.a * (1.0 - shadow.a)
        );
    }
    if params.mode == 12u {
        return mix(backdrop, source, mask_coverage(pixel) * clamp(params.data.x, 0.0, 1.0));
    }
    if params.mode == 99u { return source; }
    return mix(original, source, mask_coverage(pixel));
}
