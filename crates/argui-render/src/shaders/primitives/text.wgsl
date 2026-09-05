struct Viewport {
    size: vec2<f32>,
    origin: vec2<f32>,
}

@group(0) @binding(0) var glyph_atlas: texture_2d<f32>;
@group(0) @binding(1) var glyph_sampler: sampler;
@group(0) @binding(2) var<uniform> viewport: Viewport;
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32>, radii: vec4<f32> }
@group(0) @binding(3) var<storage, read> clips: array<Clip>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: f32,
    @location(3) @interpolate(flat) clip_meta: vec2<u32>,
}

fn rounded_distance(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    var radius = select(radii.z, radii.w, point.x < size.x * 0.5);
    if point.y < size.y * 0.5 { radius = select(radii.y, radii.x, point.x < size.x * 0.5); }
    radius = clamp(radius, 0.0, min(size.x, size.y) * 0.5);
    let offset = abs(point - size * 0.5) - max(size * 0.5 - vec2(radius), vec2(0.0));
    return length(max(offset, vec2(0.0))) + min(max(offset.x, offset.y), 0.0) - radius;
}

@vertex
fn vertex(
    @builtin(vertex_index) vertex_index: u32,
    @location(0) rect: vec4<f32>,
    @location(1) uv_rect: vec4<f32>,
    @location(2) color: vec4<f32>,
    @location(3) mode: vec4<f32>,
    @location(4) transform_a: vec4<f32>,
    @location(5) transform_b: vec4<f32>,
    @location(6) clip_meta: vec4<u32>,
) -> VertexOutput {
    var corners = array<vec2<f32>, 6>(
        vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
        vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
    );
    let corner = corners[vertex_index];
    let point = rect.xy + corner * rect.zw;
    let world = vec2(
        transform_a.x * point.x + transform_a.z * point.y + transform_b.x,
        transform_a.y * point.x + transform_a.w * point.y + transform_b.y,
    );
    let pixel = world - viewport.origin;
    let ndc = pixel / viewport.size * vec2(2.0, -2.0) + vec2(-1.0, 1.0);

    var output: VertexOutput;
    output.position = vec4(ndc, 0.0, 1.0);
    output.uv = mix(uv_rect.xy, uv_rect.zw, corner);
    output.color = color;
    output.mode = mode.x;
    output.clip_meta = clip_meta.xy;
    return output;
}

// Evaluate the SDF one framebuffer pixel away using the inverse affine basis.
// Unlike fragment derivatives, these differences are valid in variable clip loops.
fn clip_gradient(clip: Clip, local: vec2<f32>, distance: f32) -> vec2<f32> {
    let point = local - clip.bounds.xy;
    return vec2(
        rounded_distance(point + clip.inverse_a.xy, clip.bounds.zw, clip.radii) - distance,
        rounded_distance(point + clip.inverse_a.zw, clip.bounds.zw, clip.radii) - distance,
    );
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = input.position.xy + viewport.origin;
    var clip_coverage = 1.0;
    for (var offset = 0u; offset < input.clip_meta.y; offset++) {
        let clip = clips[input.clip_meta.x + offset];
        let local = vec2(
            clip.inverse_a.x * pixel.x + clip.inverse_a.z * pixel.y + clip.inverse_b.x,
            clip.inverse_a.y * pixel.x + clip.inverse_a.w * pixel.y + clip.inverse_b.y,
        );
        let clip_distance = rounded_distance(local - clip.bounds.xy, clip.bounds.zw, clip.radii);
        let gradient = abs(clip_gradient(clip, local, clip_distance));
        let clip_width = max(gradient.x + gradient.y, 0.75);
        clip_coverage *= 1.0 - smoothstep(-clip_width, clip_width, clip_distance);
    }
    let sampled = textureSample(glyph_atlas, glyph_sampler, input.uv);
    if input.mode > 1.5 {
        return input.color * vec4(1.0, 1.0, 1.0, clip_coverage);
    }
    if input.mode > 0.5 {
        return sampled * vec4(1.0, 1.0, 1.0, input.color.a * clip_coverage);
    }
    return sampled * input.color * vec4(1.0, 1.0, 1.0, clip_coverage);
}
