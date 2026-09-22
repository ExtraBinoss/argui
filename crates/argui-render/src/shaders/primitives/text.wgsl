struct Viewport {
    size: vec2<f32>,
    origin: vec2<f32>,
}

@group(0) @binding(0) var glyph_atlas: texture_2d_array<f32>;
@group(0) @binding(4) var color_atlas: texture_2d_array<f32>;
@group(0) @binding(1) var glyph_sampler: sampler;
@group(0) @binding(2) var<uniform> viewport: Viewport;
struct Clip { inverse_a: vec4<f32>, inverse_b: vec4<f32>, bounds: vec4<f32>, radii: vec4<f32> }
@group(0) @binding(3) var<storage, read> clips: array<Clip>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec4<f32>,
    @location(2) mode: vec4<f32>,
    @location(3) @interpolate(flat) clip_meta: vec2<u32>,
    @location(4) @interpolate(flat) atlas_page: u32,
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
    output.mode = mode;
    output.clip_meta = clip_meta.xy;
    output.atlas_page = clip_meta.z;
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

fn srgb_to_linear_channel(value: f32) -> f32 {
    return select(
        value / 12.92,
        pow((value + 0.055) / 1.055, 2.4),
        value > 0.04045,
    );
}

fn linear_to_srgb_channel(value: f32) -> f32 {
    return select(
        value * 12.92,
        1.055 * pow(max(value, 0.0), 1.0 / 2.4) - 0.055,
        value > 0.0031308,
    );
}

fn srgb_to_linear(color: vec3<f32>) -> vec3<f32> {
    return vec3(
        srgb_to_linear_channel(color.r),
        srgb_to_linear_channel(color.g),
        srgb_to_linear_channel(color.b),
    );
}

fn linear_to_srgb(color: vec3<f32>) -> vec3<f32> {
    return vec3(
        linear_to_srgb_channel(color.r),
        linear_to_srgb_channel(color.g),
        linear_to_srgb_channel(color.b),
    );
}

// Glyph masks encode geometric coverage. Correct that coverage with the sRGB
// transfer curve while colors remain in the renderer's linear working space.
// The two branches are complementary, so dark-on-light and light-on-dark text
// retain the same apparent stroke weight.
fn text_coverage(mask: f32, color: vec3<f32>) -> f32 {
    let lightness = dot(color, vec3(0.2126, 0.7152, 0.0722));
    let light_on_dark = lightness > 0.21404114;
    let light_coverage = srgb_to_linear_channel(mask);
    let dark_coverage = 1.0 - srgb_to_linear_channel(1.0 - mask);
    return select(dark_coverage, light_coverage, light_on_dark);
}

// Reconstruct the straight-alpha source that makes linear GPU blending produce
// the same edge color as an sRGB coverage blend over a known opaque backdrop.
// Raising alpha when needed keeps every reconstructed source channel in gamut.
fn backdrop_aware_text(
    coverage: f32,
    foreground: vec3<f32>,
    backdrop: vec3<f32>,
) -> vec4<f32> {
    if coverage <= 0.00001 {
        return vec4(foreground, 0.0);
    }
    let target_color = srgb_to_linear(mix(
        linear_to_srgb(backdrop),
        linear_to_srgb(foreground),
        coverage,
    ));
    let delta = target_color - backdrop;
    let toward_white = delta / max(vec3(1.0) - backdrop, vec3(0.00001));
    let toward_black = -delta / max(backdrop, vec3(0.00001));
    let required = select(toward_black, toward_white, delta >= vec3(0.0));
    let alpha = clamp(max(coverage, max(required.r, max(required.g, required.b))), 0.0, 1.0);
    let source = clamp(backdrop + delta / max(alpha, 0.00001), vec3(0.0), vec3(1.0));
    return vec4(source, alpha);
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
    if input.mode.x > 1.5 && input.mode.x < 2.5 {
        return input.color * vec4(1.0, 1.0, 1.0, clip_coverage);
    }
    if input.mode.x > 0.5 && input.mode.x < 1.5 {
        let sampled = textureSampleLevel(color_atlas, glyph_sampler, input.uv, i32(input.atlas_page), 0.0);
        return sampled * vec4(1.0, 1.0, 1.0, input.color.a * clip_coverage);
    }
    let mask = textureSampleLevel(glyph_atlas, glyph_sampler, input.uv, i32(input.atlas_page), 0.0).r;
    if input.mode.x > 2.5 {
        let coverage = mask * input.color.a * clip_coverage;
        return backdrop_aware_text(coverage, input.color.rgb, input.mode.yzw);
    }
    let coverage = text_coverage(mask, input.color.rgb);
    return vec4(input.color.rgb, input.color.a * coverage * clip_coverage);
}
