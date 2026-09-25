use std::{collections::HashMap, mem::size_of};

use argui_paint::EffectId;
use bytemuck::{Pod, Zeroable};
use wgpu::util::DeviceExt;

use crate::{
    RendererError,
    gpu_profile::GpuFrameCapture,
    target::{PixelRegion, TextureTarget},
};

const UNIFORM_STRIDE: u64 = 256;
const UNIFORM_CAPACITY: u64 = 1024;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(crate) struct EffectUniform {
    pub viewport: [f32; 2],
    pub mode: u32,
    pub blend: u32,
    pub target: [f32; 4],
    pub source: [f32; 4],
    pub backdrop: [f32; 4],
    pub source_uv: [f32; 4],
    pub backdrop_uv: [f32; 4],
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
            target: [0.0, 0.0, 1.0, 1.0],
            source: [0.0, 0.0, 1.0, 1.0],
            backdrop: [0.0, 0.0, 1.0, 1.0],
            source_uv: [0.0, 0.0, 1.0, 1.0],
            backdrop_uv: [0.0, 0.0, 1.0, 1.0],
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

impl EffectUniform {
    /// Stores an inverse surface transform used by the built-in compositor pass.
    ///
    /// * `transform` — inverse transform mapping output pixels to retained source pixels.
    pub(crate) fn set_inverse_transform(&mut self, transform: argui_core::Affine2D) {
        self.matrix[0] = [
            transform.matrix[0],
            transform.matrix[2],
            transform.translation.x,
            0.0,
        ];
        self.matrix[1] = [
            transform.matrix[1],
            transform.matrix[3],
            transform.translation.y,
            0.0,
        ];
    }
}

pub(crate) struct EffectGpu {
    pipeline: wgpu::RenderPipeline,
    blur_pipeline: wgpu::RenderPipeline,
    custom: HashMap<(EffectId, u64, usize), wgpu::RenderPipeline>,
    layout: wgpu::BindGroupLayout,
    format: wgpu::TextureFormat,
    sampler: wgpu::Sampler,
    uniforms: wgpu::Buffer,
    parameters: wgpu::Buffer,
    parameter_stride: u64,
    parameter_binding_size: u64,
    parameter_word_capacity: usize,
    next_slot: u64,
}

pub(crate) struct EffectDraw<'a> {
    pub target: &'a wgpu::TextureView,
    pub target_region: PixelRegion,
    pub target_extent: [u32; 2],
    pub output_region: PixelRegion,
    pub source: &'a wgpu::TextureView,
    pub backdrop: &'a wgpu::TextureView,
    pub uniform: EffectUniform,
    pub shader: Option<(EffectId, u64, usize)>,
    pub parameters: &'a [u32],
    pub profiler: Option<&'a GpuFrameCapture>,
    pub profile_label: &'a str,
    pub profile_object: Option<argui_paint::RenderObjectId>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl EffectGpu {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        maximum_parameter_words: usize,
    ) -> Self {
        let parameter_binding_size = (maximum_parameter_words.max(1) * size_of::<u32>()) as u64;
        let alignment = u64::from(device.limits().min_storage_buffer_offset_alignment.max(1));
        let parameter_stride = parameter_binding_size.div_ceil(alignment) * alignment;
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
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(parameter_binding_size),
                    },
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
            source: wgpu::ShaderSource::Wgsl(built_in_source().into()),
        });
        let pipeline = create_pipeline(device, format, &layout, &shader);
        let blur_shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-blur-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/effects/blur.wgsl").into()),
        });
        let blur_pipeline = create_pipeline(device, format, &layout, &blur_shader);
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
        let parameters = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("argui-effect-parameters"),
            size: parameter_stride * UNIFORM_CAPACITY,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        Self {
            pipeline,
            blur_pipeline,
            custom: HashMap::new(),
            layout,
            format,
            sampler,
            uniforms,
            parameters,
            parameter_stride,
            parameter_binding_size,
            parameter_word_capacity: maximum_parameter_words.max(1),
            next_slot: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.next_slot = 0;
    }

    pub fn register(
        &mut self,
        device: &wgpu::Device,
        id: EffectId,
        revision: u64,
        pass: usize,
        source: &str,
    ) -> Result<(), RendererError> {
        let source =
            crate::shader::validate_effect_source(format!("effect://{id}/{pass}"), source, &[])
                .map_err(|error| RendererError::InvalidShader(error.to_string()))?
                .source;
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-custom-effect"),
            source: wgpu::ShaderSource::Wgsl(source.into()),
        });
        self.custom.insert(
            (id, revision, pass),
            create_pipeline(device, self.format, &self.layout, &shader),
        );
        Ok(())
    }

    pub fn contains(&self, id: &EffectId, revision: u64) -> bool {
        self.custom
            .keys()
            .any(|(candidate, candidate_revision, _)| {
                candidate == id && *candidate_revision == revision
            })
    }

    pub fn parameter_word_capacity(&self) -> usize {
        self.parameter_word_capacity
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
        let parameter_offset = slot * self.parameter_stride;
        queue.write_buffer(&self.uniforms, offset, bytemuck::bytes_of(&draw.uniform));
        if !draw.parameters.is_empty() {
            queue.write_buffer(
                &self.parameters,
                parameter_offset,
                bytemuck::cast_slice(draw.parameters),
            );
        }
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-effect-bind-group"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(draw.source),
                },
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.parameters,
                        offset: 0,
                        size: wgpu::BufferSize::new(self.parameter_binding_size),
                    }),
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
        let load = if (1..=4).contains(&draw.uniform.mode) {
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT)
        } else {
            wgpu::LoadOp::Load
        };
        let attachment = Some(wgpu::RenderPassColorAttachment {
            view: draw.target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load,
                store: wgpu::StoreOp::Store,
            },
        });
        let timestamp_writes = draw.profiler.and_then(|profiler| {
            profiler.timestamp_writes(
                draw.profile_label,
                draw.profile_object,
                u64::from(draw.output_region.size[0]) * u64::from(draw.output_region.size[1]),
            )
        });
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("argui-effect-pass"),
            color_attachments: &[attachment],
            timestamp_writes,
            ..Default::default()
        });
        let viewport = TextureTarget::new(0, draw.target_region, draw.target_extent)
            .viewport_for(draw.output_region);
        pass.set_viewport(viewport[0], viewport[1], viewport[2], viewport[3], 0.0, 1.0);
        pass.set_scissor_rect(
            viewport[0].round() as u32,
            viewport[1].round() as u32,
            viewport[2].round().max(1.0) as u32,
            viewport[3].round().max(1.0) as u32,
        );
        let pipeline = draw.shader.and_then(|id| self.custom.get(&id)).unwrap_or(
            if (1..=4).contains(&draw.uniform.mode) {
                &self.blur_pipeline
            } else {
                &self.pipeline
            },
        );
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &bind_group, &[offset as u32, parameter_offset as u32]);
        pass.draw(0..3, 0..1);
    }
}

fn built_in_source() -> String {
    format!(
        "{}\n{}",
        include_str!("../shaders/effects/compositor.wgsl"),
        include_str!("../shaders/effects/refraction.wgsl")
    )
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
