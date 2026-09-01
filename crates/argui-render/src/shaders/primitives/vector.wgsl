struct Viewport { size: vec2<f32>, origin: vec2<f32> }
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32>, radii: vec4<f32> }
@group(0) @binding(0) var<uniform> viewport: Viewport;
@group(0) @binding(1) var<storage, read> clips: array<Clip>;
@group(1) @binding(0) var atlas: texture_2d<f32>;
@group(1) @binding(1) var atlas_sampler: sampler;

const CORNERS = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
);

struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) @interpolate(flat) color: vec4<f32>,
    @location(2) @interpolate(flat) params: vec2<f32>,
    @location(3) @interpolate(flat) clip_meta: vec2<u32>,
}

@vertex
fn vertex(
    @builtin(vertex_index) index: u32,
    @location(0) rect: vec4<f32>, @location(1) uv_rect: vec4<f32>,
    @location(2) color: vec4<f32>, @location(3) transform_a: vec4<f32>,
    @location(4) transform_b: vec4<f32>, @location(5) params: vec4<f32>,
    @location(6) clip_meta: vec4<u32>,
) -> Output {
    let corner = CORNERS[index];
    let point = rect.xy + corner * rect.zw;
    let world = vec2(
        transform_a.x * point.x + transform_a.z * point.y + transform_b.x,
        transform_a.y * point.x + transform_a.w * point.y + transform_b.y,
    );
    let pixel = world - viewport.origin;
    var output: Output;
    output.position = vec4(pixel / viewport.size * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    output.uv = mix(uv_rect.xy, uv_rect.zw, corner);
    output.color = color;
    output.params = params.xy;
    output.clip_meta = clip_meta.xy;
    return output;
}

fn rounded_distance(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var radius = select(radii.z, radii.w, point.x < size.x * 0.5);
    if point.y < size.y * 0.5 { radius = select(radii.y, radii.x, point.x < size.x * 0.5); }
    radius = clamp(radius, 0.0, min(size.x, size.y) * 0.5);
    let offset = abs(point - size * 0.5) - max(size * 0.5 - vec2(radius), vec2(0.0));
    return length(max(offset, vec2(0.0))) + min(max(offset.x, offset.y), 0.0) - radius;
}

@fragment
fn fragment(input: Output) -> @location(0) vec4<f32> {
    let pixel = input.position.xy + viewport.origin;
    var clip_coverage = 1.0;
    for (var offset = 0u; offset < input.clip_meta.y; offset++) {
        let clip = clips[input.clip_meta.x + offset];
        let local = vec2(
            clip.inverse_a.x * pixel.x + clip.inverse_a.z * pixel.y + clip.inverse_b.x,
            clip.inverse_a.y * pixel.x + clip.inverse_a.w * pixel.y + clip.inverse_b.y,
        );
        let distance = rounded_distance(local - clip.bounds.xy, clip.bounds.zw, clip.radii);
        let width = max(fwidth(distance), 0.75);
        clip_coverage *= 1.0 - smoothstep(-width, width, distance);
    }
    let sampled = textureSample(atlas, atlas_sampler, input.uv);
    let tintable = input.params.y > 0.5;
    let rgb = select(sampled.rgb, input.color.rgb, tintable);
    let tint_alpha = select(1.0, input.color.a, tintable);
    return vec4(rgb, sampled.a * tint_alpha * input.params.x * clip_coverage);
}
