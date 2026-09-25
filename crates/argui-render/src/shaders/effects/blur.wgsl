// Separable Gaussian and SIGGRAPH 2015 dual-filter blur share one isolated pipeline.
// The five-tap downsample and eight-tap upsample stencils follow Marius Bjorge,
// "Bandwidth-Efficient Rendering", SIGGRAPH 2015 (Arm course notes, pp. 20-21).
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

fn source_uv(pixel: vec2<f32>) -> vec2<f32> {
    let region_uv = (pixel - params.source.xy) / params.source.zw;
    return params.source_uv.xy + region_uv * params.source_uv.zw;
}

fn sample_source(uv: vec2<f32>) -> vec4<f32> {
    let half_texel = vec2<f32>(0.5) / vec2<f32>(textureDimensions(source_texture));
    let low = params.source_uv.xy + half_texel;
    let high = params.source_uv.xy + params.source_uv.zw - half_texel;
    return textureSampleLevel(source_texture, linear_sampler, clamp(uv, low, high), 0.0);
}

fn gaussian(uv: vec2<f32>, axis: vec2<f32>) -> vec4<f32> {
    let sigma = max(params.data.x, 0.01);
    if sigma < 0.25 { return sample_source(uv); }
    let texel = axis / vec2<f32>(textureDimensions(source_texture));
    let taps = min(u32(ceil(sigma * 3.0)), 12u);
    var color = sample_source(uv);
    var total = 1.0;
    // Bilinear interpolation combines each pair of adjacent Gaussian taps.
    for (var tap = 1u; tap <= taps; tap += 2u) {
        let first = exp(-0.5 * pow(f32(tap) / sigma, 2.0));
        var second = 0.0;
        if tap + 1u <= taps {
            second = exp(-0.5 * pow(f32(tap + 1u) / sigma, 2.0));
        }
        let weight = first + second;
        let distance = (f32(tap) * first + f32(tap + 1u) * second) / weight;
        let offset = texel * distance;
        color += (sample_source(uv + offset) + sample_source(uv - offset)) * weight;
        total += 2.0 * weight;
    }
    return color / total;
}

fn downsample(uv: vec2<f32>) -> vec4<f32> {
    let half_texel = vec2<f32>(0.5 * params.data.x) / vec2<f32>(textureDimensions(source_texture));
    return (
        sample_source(uv) * 4.0
        + sample_source(uv + vec2<f32>(-half_texel.x, -half_texel.y))
        + sample_source(uv + vec2<f32>(half_texel.x, -half_texel.y))
        + sample_source(uv + vec2<f32>(-half_texel.x, half_texel.y))
        + sample_source(uv + vec2<f32>(half_texel.x, half_texel.y))
    ) / 8.0;
}

fn upsample(uv: vec2<f32>) -> vec4<f32> {
    let half_texel = vec2<f32>(0.5 * params.data.x) / vec2<f32>(textureDimensions(source_texture));
    return (
        sample_source(uv + vec2<f32>(-2.0 * half_texel.x, 0.0))
        + sample_source(uv + vec2<f32>(-half_texel.x, half_texel.y)) * 2.0
        + sample_source(uv + vec2<f32>(0.0, 2.0 * half_texel.y))
        + sample_source(uv + vec2<f32>(half_texel.x, half_texel.y)) * 2.0
        + sample_source(uv + vec2<f32>(2.0 * half_texel.x, 0.0))
        + sample_source(uv + vec2<f32>(half_texel.x, -half_texel.y)) * 2.0
        + sample_source(uv + vec2<f32>(0.0, -2.0 * half_texel.y))
        + sample_source(uv + vec2<f32>(-half_texel.x, -half_texel.y)) * 2.0
    ) / 12.0;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let pixel = params.target_region.xy + input.uv * params.target_region.zw;
    let uv = source_uv(pixel);
    if params.mode == 1u { return gaussian(uv, vec2<f32>(1.0, 0.0)); }
    if params.mode == 2u { return gaussian(uv, vec2<f32>(0.0, 1.0)); }
    if params.mode == 3u { return downsample(uv); }
    return upsample(uv);
}
