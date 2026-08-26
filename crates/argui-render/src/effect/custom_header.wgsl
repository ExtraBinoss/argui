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
