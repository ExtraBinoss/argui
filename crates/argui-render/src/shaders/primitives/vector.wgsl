struct Viewport { size: vec2<f32>, origin: vec2<f32> }
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32>, radii: vec4<f32> }
@group(0) @binding(0) var<uniform> viewport: Viewport;
@group(0) @binding(1) var<storage, read> clips: array<Clip>;

struct Output {
    @builtin(position) position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) @interpolate(flat) clip_meta: vec2<u32>,
    @location(2) barycentric: vec3<f32>,
    @location(3) @interpolate(flat) boundary: vec3<f32>,
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
    @location(0) start_position: vec2<f32>, @location(1) end_position: vec2<f32>,
    @location(2) color_from: vec4<f32>, @location(3) color_to: vec4<f32>,
    @location(4) barycentric: vec3<f32>, @location(5) boundary: vec3<f32>,
    @location(6) rect: vec4<f32>, @location(7) source_size: vec2<f32>,
    @location(8) params: vec2<f32>, @location(9) transform_a: vec4<f32>,
    @location(10) transform_b: vec4<f32>, @location(11) clip_meta: vec4<u32>,
) -> Output {
    let source = mix(start_position, end_position, params.x);
    let point = rect.xy + source / max(source_size, vec2(0.0001)) * rect.zw;
    let world = vec2(
        transform_a.x * point.x + transform_a.z * point.y + transform_b.x,
        transform_a.y * point.x + transform_a.w * point.y + transform_b.y,
    );
    let pixel = world - viewport.origin;
    var output: Output;
    output.position = vec4(pixel / viewport.size * vec2(2.0, -2.0) + vec2(-1.0, 1.0), 0.0, 1.0);
    output.color = mix(color_from, color_to, params.x) * vec4(1.0, 1.0, 1.0, params.y);
    output.clip_meta = clip_meta.xy;
    output.barycentric = barycentric;
    output.boundary = boundary;
    return output;
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
        let clip_distance = rounded_distance(local - clip.bounds.xy, clip.bounds.zw, clip.radii);
        let clip_width = max(fwidth(clip_distance), 0.75);
        clip_coverage *= 1.0 - smoothstep(-clip_width, clip_width, clip_distance);
    }
    let width = max(fwidth(input.barycentric), vec3(0.0001));
    let edge_distance = input.barycentric / width;
    let outside = vec3(100000.0);
    let boundary_distance = select(outside, edge_distance, input.boundary > vec3(0.5));
    let distance = min(boundary_distance.x, min(boundary_distance.y, boundary_distance.z));
    let coverage = clamp(distance + 0.5, 0.0, 1.0);
    return vec4(input.color.rgb, input.color.a * coverage * clip_coverage);
}
