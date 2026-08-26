struct Viewport {
    size: vec2<f32>,
    padding: vec2<f32>,
}

@group(0) @binding(0) var<uniform> viewport: Viewport;

struct VertexInput {
    @builtin(vertex_index) vertex_index: u32,
    @location(0) rect: vec4<f32>,
    @location(1) background: vec4<f32>,
    @location(2) border_color: vec4<f32>,
    @location(3) radii: vec4<f32>,
    @location(4) border_widths: vec4<f32>,
    @location(5) clip: vec4<f32>,
    @location(6) params: vec4<f32>,
}

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) local: vec2<f32>,
    @location(1) @interpolate(flat) size: vec2<f32>,
    @location(2) @interpolate(flat) background: vec4<f32>,
    @location(3) @interpolate(flat) border_color: vec4<f32>,
    @location(4) @interpolate(flat) radii: vec4<f32>,
    @location(5) @interpolate(flat) border_widths: vec4<f32>,
    @location(6) @interpolate(flat) clip: vec4<f32>,
    @location(7) @interpolate(flat) params: vec4<f32>,
}

const CORNERS = array<vec2<f32>, 6>(
    vec2(0.0, 0.0), vec2(1.0, 0.0), vec2(0.0, 1.0),
    vec2(0.0, 1.0), vec2(1.0, 0.0), vec2(1.0, 1.0),
);

@vertex
fn vertex(input: VertexInput) -> VertexOutput {
    let corner = CORNERS[input.vertex_index];
    let pixel = input.rect.xy + corner * input.rect.zw;
    let ndc = vec2(
        pixel.x / viewport.size.x * 2.0 - 1.0,
        1.0 - pixel.y / viewport.size.y * 2.0,
    );
    var output: VertexOutput;
    output.position = vec4(ndc, 0.0, 1.0);
    output.local = corner * input.rect.zw;
    output.size = input.rect.zw;
    output.background = input.background;
    output.border_color = input.border_color;
    output.radii = input.radii;
    output.border_widths = input.border_widths;
    output.clip = input.clip;
    output.params = input.params;
    return output;
}

fn corner_radius(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    if point.y < size.y * 0.5 {
        return select(radii.y, radii.x, point.x < size.x * 0.5);
    }
    return select(radii.z, radii.w, point.x < size.x * 0.5);
}

fn rounded_distance(point: vec2<f32>, size: vec2<f32>, radii: vec4<f32>) -> f32 {
    let radius = clamp(corner_radius(point, size, radii), 0.0, min(size.x, size.y) * 0.5);
    let centered = point - size * 0.5;
    let offset = abs(centered) - max(size * 0.5 - vec2(radius), vec2(0.0));
    return length(max(offset, vec2(0.0))) + min(max(offset.x, offset.y), 0.0) - radius;
}

@fragment
fn fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let pixel = input.position.xy;
    if pixel.x < input.clip.x || pixel.y < input.clip.y ||
       pixel.x >= input.clip.z || pixel.y >= input.clip.w {
        discard;
    }

    let outer_distance = rounded_distance(input.local, input.size, input.radii);
    let outer_aa = max(fwidth(outer_distance), 0.75);
    let outer_coverage = 1.0 - smoothstep(-outer_aa, outer_aa, outer_distance);

    let left = input.border_widths.x;
    let right = input.border_widths.y;
    let top = input.border_widths.z;
    let bottom = input.border_widths.w;
    let inner_origin = vec2(left, top);
    let inner_size = max(input.size - vec2(left + right, top + bottom), vec2(0.0));
    let inner_radii = max(input.radii - vec4(
        max(left, top), max(right, top), max(right, bottom), max(left, bottom)
    ), vec4(0.0));
    let inner_distance = rounded_distance(input.local - inner_origin, inner_size, inner_radii);
    let inner_aa = max(fwidth(inner_distance), 0.75);
    let inner_coverage = 1.0 - smoothstep(-inner_aa, inner_aa, inner_distance);

    var color = mix(input.border_color, input.background, inner_coverage);
    color.a *= outer_coverage * input.params.x;
    return color;
}
