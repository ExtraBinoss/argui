struct Viewport { size: vec2<f32>, origin: vec2<f32> }
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32> }
struct Quad {
    rect: vec4<f32>, background: vec4<f32>, border_color: vec4<f32>, radii: vec4<f32>,
    border_widths: vec4<f32>, transform_a: vec4<f32>, transform_b: vec4<f32>,
    fill_geometry: vec4<f32>, params: vec4<f32>, clip_meta: vec4<u32>, gradient_meta: vec4<u32>,
}
struct GradientStop { offset: vec4<f32>, color: vec4<f32> }
@group(0) @binding(0) var<uniform> viewport: Viewport;
@group(0) @binding(1) var<storage, read> quads: array<Quad>;
@group(0) @binding(2) var<storage, read> clips: array<Clip>;
@group(0) @binding(3) var<storage, read> gradient_stops: array<GradientStop>;

const CORNERS = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
);
struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) instance: u32,
}

fn transformed(a: vec4<f32>, b: vec4<f32>, point: vec2<f32>) -> vec2<f32> {
    return vec2(a.x * point.x + a.z * point.y + b.x, a.y * point.x + a.w * point.y + b.y);
}

@vertex
fn vertex(@builtin(vertex_index) vertex_index: u32, @builtin(instance_index) instance: u32) -> VertexOutput {
    let quad = quads[instance];
    let local = CORNERS[vertex_index] * quad.rect.zw;
    let pixel = transformed(quad.transform_a, quad.transform_b, quad.rect.xy + local) - viewport.origin;
    let ndc = vec2(pixel.x / viewport.size.x * 2.0 - 1.0, 1.0 - pixel.y / viewport.size.y * 2.0);
    var output: VertexOutput;
    output.position = vec4(ndc, 0.0, 1.0);
    output.local = local;
    output.instance = instance;
    return output;
}

fn corner_radius(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    if point.y < size.y * 0.5 { return select(radii.y, radii.x, point.x < size.x * 0.5); }
    return select(radii.z, radii.w, point.x < size.x * 0.5);
}

fn rounded_distance(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let radius = clamp(corner_radius(point, size, radii), 0.0, min(size.x, size.y) * 0.5);
    let centered = point - size * 0.5;
    let offset = abs(centered) - max(size * 0.5 - vec2(radius), vec2(0.0));
    return length(max(offset, vec2(0.0))) + min(max(offset.x, offset.y), 0.0) - radius;
}

fn gradient_color(quad: Quad, value: f32) -> vec4<f32> {
    let start = quad.gradient_meta.x;
    let count = quad.gradient_meta.y;
    if value <= gradient_stops[start].offset.x { return gradient_stops[start].color; }
    for (var index = 1u; index < count; index++) {
        let upper = gradient_stops[start + index].offset.x;
        if value <= upper {
            let lower = gradient_stops[start + index - 1u].offset.x;
            let progress = clamp((value - lower) / max(upper - lower, 0.00001), 0.0, 1.0);
            return mix(gradient_stops[start + index - 1u].color, gradient_stops[start + index].color, progress);
        }
    }
    return gradient_stops[start + count - 1u].color;
}

fn fill_color(quad: Quad, local: vec2<f32>) -> vec4<f32> {
    let uv = local / max(quad.rect.zw, vec2(0.00001));
    if quad.params.y < 0.5 { return quad.background; }
    if quad.params.y < 1.5 {
        let axis = quad.fill_geometry.zw - quad.fill_geometry.xy;
        return gradient_color(quad, dot(uv - quad.fill_geometry.xy, axis) / max(dot(axis, axis), 0.00001));
    }
    return gradient_color(quad, length((uv - quad.fill_geometry.xy) / max(quad.fill_geometry.zw, vec2(0.00001))));
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let quad = quads[input.instance];
    let pixel = input.position.xy + viewport.origin;
    for (var offset = 0u; offset < quad.clip_meta.y; offset++) {
        let clip = clips[quad.clip_meta.x + offset];
        let local = transformed(clip.inverse_a, clip.inverse_b, pixel);
        if local.x < clip.bounds.x || local.y < clip.bounds.y ||
           local.x >= clip.bounds.x + clip.bounds.z || local.y >= clip.bounds.y + clip.bounds.w { discard; }
    }
    let outer_distance = rounded_distance(input.local, quad.rect.zw, quad.radii);
    let outer_coverage = 1.0 - smoothstep(-max(fwidth(outer_distance), 0.75), max(fwidth(outer_distance), 0.75), outer_distance);
    let origin = vec2(quad.border_widths.x, quad.border_widths.z);
    let size = max(quad.rect.zw - vec2(quad.border_widths.x + quad.border_widths.y, quad.border_widths.z + quad.border_widths.w), vec2(0.0));
    let radii = max(quad.radii - vec4(
        max(quad.border_widths.x, quad.border_widths.z), max(quad.border_widths.y, quad.border_widths.z),
        max(quad.border_widths.y, quad.border_widths.w), max(quad.border_widths.x, quad.border_widths.w)
    ), vec4(0.0));
    let inner_distance = rounded_distance(input.local - origin, size, radii);
    let inner_coverage = 1.0 - smoothstep(-max(fwidth(inner_distance), 0.75), max(fwidth(inner_distance), 0.75), inner_distance);
    var color = mix(quad.border_color, fill_color(quad, input.local), inner_coverage);
    color.a *= outer_coverage * quad.params.x;
    return color;
}
