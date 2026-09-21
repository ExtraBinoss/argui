struct Viewport { size: vec2<f32>, origin: vec2<f32> }
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32>, radii: vec4<f32> }
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
    let scale = vec2(
        length(vec2(quad.transform_a.x, quad.transform_a.y)),
        length(vec2(quad.transform_a.z, quad.transform_a.w)),
    );
    let fringe = vec2(1.5) / max(scale, vec2(0.00001));
    let local = CORNERS[vertex_index] * (quad.rect.zw + fringe * 2.0) - fringe;
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

fn edge_coverage(distance: f32) -> f32 {
    let gradient = length(vec2(dpdx(distance), dpdy(distance)));
    let pixel_width = max(gradient, 0.75) * 1.5;
    return clamp(0.5 - distance / pixel_width, 0.0, 1.0);
}

fn srgb_to_linear(value: vec3<f32>) -> vec3<f32> {
    let magnitude = abs(value);
    let converted = select(
        magnitude / 12.92,
        pow((magnitude + vec3<f32>(0.055)) / 1.055, vec3<f32>(2.4)),
        magnitude > vec3<f32>(0.04045),
    );
    return sign(value) * converted;
}

fn oklab_to_linear_srgb(value: vec3<f32>) -> vec3<f32> {
    let lms = vec3<f32>(
        value.x + 0.39633778 * value.y + 0.21580376 * value.z,
        value.x - 0.105561346 * value.y - 0.06385417 * value.z,
        value.x - 0.08948418 * value.y - 1.2914855 * value.z,
    );
    let cubed = lms * lms * lms;
    return vec3<f32>(
        4.0767417 * cubed.x - 3.3077116 * cubed.y + 0.23096994 * cubed.z,
        -1.268438 * cubed.x + 2.6097574 * cubed.y - 0.34131938 * cubed.z,
        -0.0041960863 * cubed.x - 0.7034186 * cubed.y + 1.7076147 * cubed.z,
    );
}

fn gradient_to_linear(color: vec4<f32>, mode: u32) -> vec4<f32> {
    if color.a <= 0.000001 { return vec4<f32>(0.0); }
    let straight = color.rgb / color.a;
    if mode == 0u { return vec4<f32>(oklab_to_linear_srgb(straight), color.a); }
    if mode == 2u { return vec4<f32>(srgb_to_linear(straight), color.a); }
    return vec4<f32>(straight, color.a);
}

fn gradient_color(quad: Quad, value: f32) -> vec4<f32> {
    let start = quad.gradient_meta.x;
    let count = quad.gradient_meta.y;
    if value <= gradient_stops[start].offset.x {
        return gradient_to_linear(gradient_stops[start].color, quad.gradient_meta.z);
    }
    if value >= gradient_stops[start + count - 1u].offset.x {
        return gradient_to_linear(gradient_stops[start + count - 1u].color, quad.gradient_meta.z);
    }
    var lower_index = 0u;
    var upper_index = count - 1u;
    loop {
        if upper_index - lower_index <= 1u { break; }
        let middle = lower_index + (upper_index - lower_index) / 2u;
        if value <= gradient_stops[start + middle].offset.x {
            upper_index = middle;
        } else {
            lower_index = middle;
        }
    }
    let lower = gradient_stops[start + lower_index].offset.x;
    let upper = gradient_stops[start + upper_index].offset.x;
    let progress = clamp((value - lower) / max(upper - lower, 0.00001), 0.0, 1.0);
    let color = mix(gradient_stops[start + lower_index].color, gradient_stops[start + upper_index].color, progress);
    return gradient_to_linear(color, quad.gradient_meta.z);
}

fn fill_color(quad: Quad, local: vec2<f32>) -> vec4<f32> {
    let uv = local / max(quad.rect.zw, vec2(0.00001));
    if quad.params.y < 0.5 { return quad.background; }
    if quad.params.y < 1.5 {
        let axis = quad.fill_geometry.zw - quad.fill_geometry.xy;
        return gradient_color(quad, dot(uv - quad.fill_geometry.xy, axis) / max(dot(axis, axis), 0.00001));
    }
    if quad.params.y < 2.5 {
        return gradient_color(quad, length((uv - quad.fill_geometry.xy) / max(quad.fill_geometry.zw, vec2(0.00001))));
    }
    if quad.params.y > 3.5 {
        let direction = uv - quad.fill_geometry.xy;
        let angle = atan2(direction.y, direction.x) - quad.fill_geometry.z;
        let turn = fract(angle / 6.283185307179586 + 1.0);
        return gradient_color(quad, turn);
    }
    let start = quad.gradient_meta.x;
    let position = clamp(uv, vec2(0.0), vec2(1.0));
    let top = mix(gradient_stops[start].color, gradient_stops[start + 1u].color, position.x);
    let bottom = mix(gradient_stops[start + 2u].color, gradient_stops[start + 3u].color, position.x);
    return gradient_to_linear(mix(top, bottom, position.y), quad.gradient_meta.z);
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
    let quad = quads[input.instance];
    let pixel = input.position.xy + viewport.origin;
    var clip_coverage = 1.0;
    for (var offset = 0u; offset < quad.clip_meta.y; offset++) {
        let clip = clips[quad.clip_meta.x + offset];
        let local = transformed(clip.inverse_a, clip.inverse_b, pixel);
        let distance = rounded_distance(local - clip.bounds.xy, clip.bounds.zw, clip.radii);
        let pixel_width = max(length(clip_gradient(clip, local, distance)), 0.75) * 1.5;
        clip_coverage *= clamp(0.5 - distance / pixel_width, 0.0, 1.0);
    }
    let outer_distance = rounded_distance(input.local, quad.rect.zw, quad.radii);
    let outer_coverage = edge_coverage(outer_distance);
    let origin = vec2(quad.border_widths.x, quad.border_widths.z);
    let size = max(quad.rect.zw - vec2(quad.border_widths.x + quad.border_widths.y, quad.border_widths.z + quad.border_widths.w), vec2(0.0));
    let radii = max(quad.radii - vec4(
        max(quad.border_widths.x, quad.border_widths.z), max(quad.border_widths.y, quad.border_widths.z),
        max(quad.border_widths.y, quad.border_widths.w), max(quad.border_widths.x, quad.border_widths.w)
    ), vec4(0.0));
    let inner_distance = rounded_distance(input.local - origin, size, radii);
    let inner_coverage = edge_coverage(inner_distance);
    let border_coverage = max(outer_coverage - inner_coverage, 0.0);
    let fill = fill_color(quad, input.local);
    let border_alpha = quad.border_color.a * border_coverage;
    let fill_alpha = fill.a * inner_coverage;
    let alpha = border_alpha + fill_alpha;
    let color = (
        quad.border_color.rgb * border_alpha + fill.rgb * fill_alpha
    ) / max(alpha, 0.00001);
    return vec4(color, alpha * quad.params.x * clip_coverage);
}
