struct Params {
    viewport: vec2<f32>,
    mode: u32,
    blend: u32,
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

fn mask_coverage(uv: vec2<f32>) -> f32 {
    let pixel = params.viewport * uv;
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

fn composite(source: vec4<f32>, backdrop: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let alpha = source.a * params.data.x * mask_coverage(uv);
    let straight_source = source.rgb / max(source.a, 0.00001);
    let mixed = blend_rgb(straight_source, backdrop.rgb, params.blend);
    return vec4<f32>(mix(backdrop.rgb, mixed, alpha), alpha + backdrop.a * (1.0 - alpha));
}

fn sample_blur(uv: vec2<f32>, axis: vec2<f32>) -> vec4<f32> {
    let source_size = vec2<f32>(textureDimensions(source_texture));
    let step = axis * max(params.data.x / 3.0, 0.5) / source_size;
    let weights = array<f32, 7>(
        0.137023, 0.129618, 0.109719, 0.083108, 0.056331, 0.034167, 0.018544
    );
    var color = textureSample(source_texture, linear_sampler, uv) * weights[0];
    for (var tap = 1u; tap < 7u; tap += 1u) {
        let offset = step * f32(tap);
        color += textureSample(source_texture, linear_sampler, uv + offset) * weights[tap];
        color += textureSample(source_texture, linear_sampler, uv - offset) * weights[tap];
    }
    return color;
}

@fragment
fn fs_main(input: VertexOut) -> @location(0) vec4<f32> {
    let original = textureSample(source_texture, linear_sampler, input.uv);
    var source = original;
    let backdrop = textureSample(backdrop_texture, linear_sampler, input.uv);
    if params.mode == 0u { return composite(source, backdrop, input.uv); }
    if params.mode == 1u { source = sample_blur(input.uv, vec2<f32>(1.0, 0.0)); }
    if params.mode == 2u { source = sample_blur(input.uv, vec2<f32>(0.0, 1.0)); }
    if params.mode == 3u { source = vec4<f32>(source.rgb * params.data.x, source.a); }
    if params.mode == 4u {
        source = vec4<f32>((source.rgb - 0.5) * params.data.x + 0.5, source.a);
    }
    if params.mode == 5u {
        let luminance = dot(source.rgb, vec3<f32>(0.2126, 0.7152, 0.0722));
        source = vec4<f32>(mix(vec3<f32>(luminance), source.rgb, params.data.x), source.a);
    }
    if params.mode == 6u {
        let angle = params.data.x;
        let cosine = cos(angle);
        let sine = sin(angle);
        let weights = vec3<f32>(0.213, 0.715, 0.072);
        let rotated = source.rgb * mat3x3<f32>(
            weights.x + cosine * (1.0 - weights.x) + sine * -weights.x,
            weights.x + cosine * -weights.x + sine * 0.143,
            weights.x + cosine * -weights.x + sine * -(1.0 - weights.x),
            weights.y + cosine * -weights.y + sine * -weights.y,
            weights.y + cosine * (1.0 - weights.y) + sine * 0.140,
            weights.y + cosine * -weights.y + sine * weights.y,
            weights.z + cosine * -weights.z + sine * (1.0 - weights.z),
            weights.z + cosine * -weights.z + sine * -0.283,
            weights.z + cosine * (1.0 - weights.z) + sine * weights.z
        );
        source = vec4<f32>(rotated, source.a);
    }
    if params.mode == 7u { source *= params.data.x; }
    if params.mode == 8u {
        source = vec4<f32>(
            dot(source, params.matrix[0]) + params.matrix[4].x,
            dot(source, params.matrix[1]) + params.matrix[4].y,
            dot(source, params.matrix[2]) + params.matrix[4].z,
            dot(source, params.matrix[3]) + params.matrix[4].w
        );
    }
    if params.mode == 9u {
        let center = (params.bounds.xy + params.bounds.zw * 0.5) / params.viewport;
        let direction = input.uv - center;
        let warped = input.uv - direction * params.data.x * 0.08;
        let shift = direction * params.data.y * 0.01;
        source = vec4<f32>(
            textureSample(backdrop_texture, linear_sampler, warped + shift).r,
            textureSample(backdrop_texture, linear_sampler, warped).g,
            textureSample(backdrop_texture, linear_sampler, warped - shift).b,
            1.0
        );
    }
    if params.mode == 10u || params.mode == 11u {
        let offset_uv = input.uv - params.data.yz / params.viewport;
        var alpha = textureSample(source_texture, linear_sampler, offset_uv).a;
        if params.mode == 11u { alpha = 1.0 - alpha; }
        let spread_alpha = clamp(alpha * (1.0 + max(params.data.w, 0.0) * 0.08), 0.0, 1.0);
        let box_coverage = mask_coverage(input.uv);
        let placement = select(1.0 - box_coverage, box_coverage, params.mode == 11u);
        let shadow_alpha = params.color.a * spread_alpha * placement;
        let shadow = vec4<f32>(params.color.rgb * shadow_alpha, shadow_alpha);
        return vec4<f32>(
            shadow.rgb + backdrop.rgb * (1.0 - shadow.a),
            shadow.a + backdrop.a * (1.0 - shadow.a)
        );
    }
    if params.mode == 12u {
        return mix(backdrop, source, mask_coverage(input.uv));
    }
    if params.mode == 99u { return source; }
    return mix(original, source, mask_coverage(input.uv));
}
