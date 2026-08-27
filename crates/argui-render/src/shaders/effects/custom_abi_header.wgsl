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

fn global_pixel(uv: vec2<f32>) -> vec2<f32> {
    return params.target_region.xy + uv * params.target_region.zw;
}

fn effect_region_uv(pixel: vec2<f32>, region: vec4<f32>) -> vec2<f32> {
    return (pixel - region.xy) / region.zw;
}

fn effect_allocated_uv(pixel: vec2<f32>, region: vec4<f32>, allocation: vec4<f32>) -> vec2<f32> {
    return allocation.xy + effect_region_uv(pixel, region) * allocation.zw;
}

fn effect_in_region(pixel: vec2<f32>, region: vec4<f32>) -> bool {
    return all(pixel >= region.xy) && all(pixel < region.xy + region.zw);
}

fn source_at(pixel: vec2<f32>) -> vec4<f32> {
    if !effect_in_region(pixel, params.source) { return vec4<f32>(0.0); }
    return textureSample(source_texture, linear_sampler, effect_allocated_uv(pixel, params.source, params.source_uv));
}

fn backdrop_at(pixel: vec2<f32>) -> vec4<f32> {
    if !effect_in_region(pixel, params.backdrop) { return vec4<f32>(0.0); }
    return textureSample(backdrop_texture, linear_sampler, effect_allocated_uv(pixel, params.backdrop, params.backdrop_uv));
}

fn layer_rounded_distance(pixel: vec2<f32>) -> f32 {
    let center = params.bounds.xy + params.bounds.zw * 0.5;
    let local = pixel - center;
    let radius = select(
        select(params.radii.x, params.radii.y, local.x > 0.0),
        select(params.radii.w, params.radii.z, local.x > 0.0),
        local.y > 0.0
    );
    let q = abs(local) - params.bounds.zw * 0.5 + vec2<f32>(radius);
    return min(max(q.x, q.y), 0.0) + length(max(q, vec2<f32>(0.0))) - radius;
}
