use std::{collections::HashMap, mem::size_of};

use argui_paint::ShaderEffectId;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::RendererError;

const UNIFORM_STRIDE: u64 = 256;
const UNIFORM_CAPACITY: u64 = 1024;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct EffectUniform {
    pub viewport: [f32; 2],
    pub mode: u32,
    pub blend: u32,
    pub bounds: [f32; 4],
    pub radii: [f32; 4],
    pub color: [f32; 4],
    pub data: [f32; 4],
    pub matrix: [[f32; 4]; 5],
}

impl Default for EffectUniform {
    fn default() -> Self {
        Self {
            viewport: [1.0; 2],
            mode: 0,
            blend: 0,
            bounds: [0.0, 0.0, 1.0, 1.0],
            radii: [0.0; 4],
            color: [1.0; 4],
            data: [0.0; 4],
            matrix: [
                [1.0, 0.0, 0.0, 0.0],
                [0.0, 1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0, 0.0],
                [0.0, 0.0, 0.0, 1.0],
                [0.0; 4],
            ],
        }
    }
}

pub(crate) struct EffectGpu {
    pipeline: wgpu::RenderPipeline,
    custom: HashMap<ShaderEffectId, wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    next_slot: u64,
}

pub(crate) struct EffectDraw<'a> {
    pub target: &'a wgpu::TextureView,
    pub source: &'a wgpu::TextureView,
    pub backdrop: &'a wgpu::TextureView,
    pub uniform: EffectUniform,
    pub shader: Option<ShaderEffectId>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl EffectGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-effect-layout"),
            entries: &[
                texture_entry(0),
                texture_entry(1),
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(size_of::<EffectUniform>() as u64),
                    },
                    count: None,
                },
            ],
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-effect-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = create_pipeline(device, format, &layout, &shader);
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("argui-effect-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("argui-effect-uniforms"),
            contents: &vec![0; (UNIFORM_STRIDE * UNIFORM_CAPACITY) as usize],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        Self {
            pipeline,
            custom: HashMap::new(),
            layout,
            format,
            sampler,
            uniforms,
            next_slot: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.next_slot = 0;
    }

    pub fn register(
        &mut self,
        device: &wgpu::Device,
        id: ShaderEffectId,
        source: &str,
    ) -> Result<(), RendererError> {
        let source = validated_custom_source(source)?;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-custom-effect"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        self.custom.insert(
            id,
            create_pipeline(device, self.format, &self.layout, &shader),
        );
        Ok(())
    }

    pub fn contains(&self, id: ShaderEffectId) -> bool {
        self.custom.contains_key(&id)
    }

    pub fn draw(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        encoder: &mut wgpu::CommandEncoder,
        draw: EffectDraw<'_>,
    ) {
        let slot = self.next_slot % UNIFORM_CAPACITY;
        self.next_slot += 1;
        let offset = slot * UNIFORM_STRIDE;
        queue.write_buffer(&self.uniforms, offset, bytemuck::bytes_of(&draw.uniform));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-effect-bind-group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(draw.source),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(draw.backdrop),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.uniforms,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<EffectUniform>() as u64),
                    }),
                },
            ],
        });
        let attachment = Some(wgpu::RenderPassColorAttachment {
            view: draw.target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Load,
                store: wgpu::StoreOp::Store,
            },
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("argui-effect-pass"),
            color_attachments: &[attachment],
            ..Default::default()
        });
        pass.set_pipeline(
            draw.shader
                .and_then(|id| self.custom.get(&id))
                .unwrap_or(&self.pipeline),
        );
        pass.set_bind_group(0, &bind_group, &[offset as u32]);
        pass.draw(0..3, 0..1);
    }
}

fn validated_custom_source(source: &str) -> Result<String, RendererError> {
    let source = format!(
        "{}\n{}\n{}",
        include_str!("custom_header.wgsl"),
        source,
        include_str!("custom_footer.wgsl")
    );
    let module = naga::front::wgsl::parse_str(&source)
        .map_err(|error| RendererError::InvalidShader(error.emit_to_string(&source)))?;
    naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::empty(),
    )
    .validate(&module)
    .map_err(|error| RendererError::InvalidShader(error.to_string()))?;
    Ok(source)
}

fn create_pipeline(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    layout: &wgpu::BindGroupLayout,
    shader: &wgpu::ShaderModule,
) -> wgpu::RenderPipeline {
    let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
        label: Some("argui-effect-pipeline-layout"),
        bind_group_layouts: &[Some(layout)],
        immediate_size: 0,
    });
    device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("argui-effect-pipeline"),
        layout: Some(&pipeline_layout),
        vertex: wgpu::VertexState {
            module: shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: wgpu::PrimitiveState::default(),
        depth_stencil: None,
        multisample: wgpu::MultisampleState::default(),
        multiview_mask: None,
        cache: None,
    })
}

const fn texture_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: true },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        },
        count: None,
    }
}

#[cfg(test)]
mod tests {
    use super::validated_custom_source;

    #[test]
    fn custom_effect_abi_accepts_valid_functions_and_rejects_bad_wgsl() {
        let valid = r#"
fn argui_effect(uv: vec2<f32>, source: vec4<f32>, backdrop: vec4<f32>) -> vec4<f32> {
    return mix(source, backdrop, params.data.x * uv.x);
}
"#;
        assert!(validated_custom_source(valid).is_ok());
        assert!(validated_custom_source("not wgsl").is_err());
    }
}
