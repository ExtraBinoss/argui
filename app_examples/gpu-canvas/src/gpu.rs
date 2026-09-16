use std::sync::Arc;

use argui::render::{
    GpuCanvasDeviceContext, GpuCanvasError, GpuCanvasFactory, GpuCanvasRenderContext,
    GpuCanvasRenderer, wgpu,
};

use crate::state::SharedLab;

const PARTICLES: u32 = 256;
const COMPUTE_SHADER: &str = r#"
struct Params { scene: vec4<f32>, viewport: vec4<f32> };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read_write> particles: array<vec4<f32>, 256>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) invocation: vec3<u32>) {
    let index = invocation.x;
    if index >= 256u { return; }
    let phase = f32(index) / 256.0;
    let ring = f32(index % 23u) / 23.0;
    let angle = phase * 6.2831853 + params.scene.w * (0.16 + ring * 0.32);
    let radius = 0.12 + ring * 0.72;
    particles[index] = vec4<f32>(cos(angle) * radius, sin(angle) * radius, phase, 1.0);
}
"#;
const RENDER_SHADER: &str = r#"
struct Params { scene: vec4<f32>, viewport: vec4<f32> };
@group(0) @binding(0) var<uniform> params: Params;
@group(0) @binding(1) var<storage, read> particles: array<vec4<f32>, 256>;

struct VertexOutput {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) color: vec3<f32>,
};

@vertex
fn grid_vertex(@builtin(vertex_index) index: u32) -> VertexOutput {
    let positions = array<vec2<f32>, 3>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(3.0, -1.0), vec2<f32>(-1.0, 3.0));
    let position = positions[index];
    var output: VertexOutput;
    output.position = vec4<f32>(position, 0.0, 1.0);
    output.uv = position * 0.5 + 0.5;
    output.color = vec3<f32>(0.0);
    return output;
}

@fragment
fn grid_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let aspect = params.viewport.x / max(params.viewport.y, 1.0);
    let pan = params.scene.xy / max(params.viewport.xy, vec2<f32>(1.0));
    let world = (input.uv - 0.5 - pan) * vec2<f32>(aspect, 1.0) / params.scene.z;
    let cell = abs(fract(world * 18.0 + 0.5) - 0.5) / fwidth(world * 18.0);
    let line = 1.0 - min(min(cell.x, cell.y), 1.0);
    let base = vec3<f32>(0.025, 0.04, 0.075);
    return vec4<f32>(base + line * vec3<f32>(0.045, 0.12, 0.18), 1.0);
}

@vertex
fn particle_vertex(
    @builtin(vertex_index) vertex: u32,
    @builtin(instance_index) instance: u32,
) -> VertexOutput {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0));
    let particle = particles[instance];
    let pan = params.scene.xy / max(params.viewport.xy, vec2<f32>(1.0)) * 2.0;
    let center = particle.xy * params.scene.z + pan;
    let size = vec2<f32>(5.0) / max(params.viewport.xy, vec2<f32>(1.0)) * 2.0;
    var output: VertexOutput;
    output.position = vec4<f32>(center + corners[vertex] * size, 0.0, 1.0);
    output.uv = corners[vertex];
    output.color = vec3<f32>(0.25 + particle.z * 0.55, 0.72, 1.0 - particle.z * 0.35);
    return output;
}

@fragment
fn particle_fragment(input: VertexOutput) -> @location(0) vec4<f32> {
    let distance = length(input.uv);
    let alpha = 1.0 - smoothstep(0.55, 1.0, distance);
    return vec4<f32>(input.color, alpha * 0.92);
}
"#;

/// Creates the example's per-surface compute and render pipelines.
pub(crate) struct LabCanvasFactory {
    shared: Arc<SharedLab>,
}

impl LabCanvasFactory {
    /// Creates a factory that reads short-lived scene snapshots from `shared`.
    pub(crate) fn new(shared: Arc<SharedLab>) -> Self {
        Self { shared }
    }
}

impl GpuCanvasFactory for LabCanvasFactory {
    /// Creates one surface-local renderer from Argui's selected device context.
    fn create(
        &self,
        context: &GpuCanvasDeviceContext<'_>,
    ) -> Result<Box<dyn GpuCanvasRenderer>, GpuCanvasError> {
        self.shared.set_capability(format!(
            "WebGPU baseline · {:?} · device generation {} · max texture {}",
            context.target_format(),
            context.device_generation(),
            context.limits().max_texture_dimension_2d
        ));
        Ok(Box::new(LabCanvasRenderer::new(
            context,
            Arc::clone(&self.shared),
        )))
    }
}

struct LabCanvasRenderer {
    shared: Arc<SharedLab>,
    uniform: wgpu::Buffer,
    _particles: wgpu::Buffer,
    compute_group: wgpu::BindGroup,
    render_group: wgpu::BindGroup,
    compute: wgpu::ComputePipeline,
    grid: wgpu::RenderPipeline,
    particles: wgpu::RenderPipeline,
}

impl LabCanvasRenderer {
    /// Allocates baseline WebGPU resources for one Argui surface renderer.
    fn new(context: &GpuCanvasDeviceContext<'_>, shared: Arc<SharedLab>) -> Self {
        let device = context.device();
        let uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-canvas-lab-uniform"),
            size: 32,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let particles = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("gpu-canvas-lab-particles"),
            size: u64::from(PARTICLES) * 16,
            usage: wgpu::BufferUsages::STORAGE,
            mapped_at_creation: false,
        });
        let compute_layout = bind_group_layout(device, false);
        let render_layout = bind_group_layout(device, true);
        let compute_group = bind_group(device, &compute_layout, &uniform, &particles, "compute");
        let render_group = bind_group(device, &render_layout, &uniform, &particles, "render");
        let compute_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu-canvas-lab-compute-shader"),
            source: wgpu::ShaderSource::Wgsl(COMPUTE_SHADER.into()),
        });
        let compute_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("gpu-canvas-lab-compute-pipeline-layout"),
                bind_group_layouts: &[Some(&compute_layout)],
                immediate_size: 0,
            });
        let compute = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: Some("gpu-canvas-lab-compute-pipeline"),
            layout: Some(&compute_pipeline_layout),
            module: &compute_module,
            entry_point: Some("main"),
            compilation_options: Default::default(),
            cache: None,
        });
        let render_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("gpu-canvas-lab-render-shader"),
            source: wgpu::ShaderSource::Wgsl(RENDER_SHADER.into()),
        });
        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("gpu-canvas-lab-render-pipeline-layout"),
                bind_group_layouts: &[Some(&render_layout)],
                immediate_size: 0,
            });
        let grid = render_pipeline(
            device,
            &render_module,
            &render_pipeline_layout,
            context.target_format(),
            "grid",
            "grid_vertex",
            "grid_fragment",
            None,
        );
        let particles_pipeline = render_pipeline(
            device,
            &render_module,
            &render_pipeline_layout,
            context.target_format(),
            "particles",
            "particle_vertex",
            "particle_fragment",
            Some(wgpu::BlendState::ALPHA_BLENDING),
        );
        Self {
            shared,
            uniform,
            _particles: particles,
            compute_group,
            render_group,
            compute,
            grid,
            particles: particles_pipeline,
        }
    }
}

impl GpuCanvasRenderer for LabCanvasRenderer {
    /// Encodes the bounded compute and render passes into the supplied context.
    fn render(&mut self, context: &mut GpuCanvasRenderContext<'_>) -> Result<(), GpuCanvasError> {
        let state = self.shared.state();
        self.shared.record_callback();
        if state.force_error() {
            return Err(GpuCanvasError::new(
                "simulated failure; choose Recover canvas to retry",
            ));
        }
        let [width, height] = context.physical_extent();
        let words = [
            state.pan_offset()[0],
            state.pan_offset()[1],
            state.zoom_factor(),
            state.elapsed_seconds(),
            width as f32,
            height as f32,
            context.scale_factor(),
            context.resolution_scale(),
        ];
        context
            .queue()
            .write_buffer(&self.uniform, 0, &float_bytes(words));
        let (encoder, target) = context.encoder_and_target();
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("gpu-canvas-lab-compute-pass"),
                ..Default::default()
            });
            pass.set_pipeline(&self.compute);
            pass.set_bind_group(0, &self.compute_group, &[]);
            pass.dispatch_workgroups(PARTICLES.div_ceil(64), 1, 1);
        }
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("gpu-canvas-lab-render-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_bind_group(0, &self.render_group, &[]);
        pass.set_pipeline(&self.grid);
        pass.draw(0..3, 0..1);
        pass.set_pipeline(&self.particles);
        pass.draw(0..6, 0..PARTICLES);
        Ok(())
    }
}

/// Creates a two-binding layout for the shared uniform and particle buffers.
fn bind_group_layout(device: &wgpu::Device, read_only: bool) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("gpu-canvas-lab-bind-group-layout"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: if read_only {
                    wgpu::ShaderStages::VERTEX_FRAGMENT
                } else {
                    wgpu::ShaderStages::COMPUTE
                },
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
            wgpu::BindGroupLayoutEntry {
                binding: 1,
                visibility: if read_only {
                    wgpu::ShaderStages::VERTEX
                } else {
                    wgpu::ShaderStages::COMPUTE
                },
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Storage { read_only },
                    has_dynamic_offset: false,
                    min_binding_size: None,
                },
                count: None,
            },
        ],
    })
}

/// Creates a bind group over the resources shared by compute and render pipelines.
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    uniform: &wgpu::Buffer,
    particles: &wgpu::Buffer,
    label: &str,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some(label),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: particles.as_entire_binding(),
            },
        ],
    })
}

/// Creates one fullscreen or instanced render pipeline for the canvas target.
#[allow(clippy::too_many_arguments)]
fn render_pipeline(
    device: &wgpu::Device,
    module: &wgpu::ShaderModule,
    layout: &wgpu::PipelineLayout,
    format: wgpu::TextureFormat,
    label: &str,
    vertex: &str,
    fragment: &str,
    blend: Option<wgpu::BlendState>,
) -> wgpu::RenderPipeline {
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some(label),
        layout: Some(layout),
        vertex: wgpu::VertexState {
            module,
            entry_point: Some(vertex),
            compilation_options: Default::default(),
            buffers: &[],
        },
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        fragment: Some(wgpu::FragmentState {
            module,
            entry_point: Some(fragment),
            compilation_options: Default::default(),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend,
                write_mask: wgpu::ColorWrites::ALL,
            })],
        }),
        multiview_mask: None,
        cache: None,
    })
}

/// Encodes eight `f32` values as native-endian bytes for a WGPU uniform write.
fn float_bytes<const N: usize>(values: [f32; N]) -> Vec<u8> {
    values
        .into_iter()
        .flat_map(f32::to_ne_bytes)
        .collect::<Vec<_>>()
}
