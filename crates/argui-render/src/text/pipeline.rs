use std::{mem::size_of, ops::Range};

use argui_core::Affine2D;
use argui_paint::ClipRegion;
use argui_text::{PreparedDecoration, PreparedGlyph};
use bytemuck::{Pod, Zeroable};

use super::atlas::AtlasEntry;

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GlyphInstance {
    rect: [f32; 4],
    uv: [f32; 4],
    color: [f32; 4],
    mode: [f32; 4],
    transform_a: [f32; 4],
    transform_b: [f32; 4],
    clip_meta: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct TextClip {
    inverse_a: [f32; 4],
    inverse_b: [f32; 4],
    bounds: [f32; 4],
    radii: [f32; 4],
}

impl TextClip {
    pub fn new(clip: ClipRegion, scale: f32) -> Option<Self> {
        let inverse = clip.transform.scaled(scale).inverse()?;
        Some(Self {
            inverse_a: inverse.matrix,
            inverse_b: [inverse.translation.x, inverse.translation.y, 0.0, 0.0],
            bounds: [
                clip.bounds.origin.x * scale,
                clip.bounds.origin.y * scale,
                clip.bounds.size.width * scale,
                clip.bounds.size.height * scale,
            ],
            radii: clip.radii.as_array().map(|radius| radius * scale),
        })
    }

    pub fn physical(bounds: [f32; 4]) -> Self {
        Self {
            inverse_a: Affine2D::IDENTITY.matrix,
            inverse_b: [0.0; 4],
            bounds: [
                bounds[0],
                bounds[1],
                bounds[2] - bounds[0],
                bounds[3] - bounds[1],
            ],
            radii: [0.0; 4],
        }
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl GlyphInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Uint32x4];
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &Self::ATTRIBUTES,
    };

    pub fn new(
        glyph: PreparedGlyph,
        entry: AtlasEntry,
        atlas_size: u32,
        transform: Affine2D,
        clip_start: u32,
        clip_count: u32,
        backdrop: Option<[f32; 3]>,
    ) -> Self {
        let atlas_size = atlas_size as f32;
        let mode = if entry.color {
            [1.0, 0.0, 0.0, 0.0]
        } else if let Some([red, green, blue]) = backdrop {
            [3.0, red, green, blue]
        } else {
            [0.0; 4]
        };
        Self {
            rect: [
                (glyph.x + entry.left) as f32,
                (glyph.y - entry.top) as f32,
                entry.width as f32,
                entry.height as f32,
            ],
            uv: [
                entry.x as f32 / atlas_size,
                entry.y as f32 / atlas_size,
                (entry.x + entry.width) as f32 / atlas_size,
                (entry.y + entry.height) as f32 / atlas_size,
            ],
            color: glyph.color,
            mode,
            transform_a: transform.matrix,
            transform_b: [transform.translation.x, transform.translation.y, 0.0, 0.0],
            clip_meta: [clip_start, clip_count, 0, 0],
        }
    }

    pub fn solid(
        decoration: PreparedDecoration,
        transform: Affine2D,
        clip_start: u32,
        clip_count: u32,
    ) -> Self {
        Self {
            rect: decoration.rect,
            uv: [0.0; 4],
            color: decoration.color,
            mode: [2.0, 0.0, 0.0, 0.0],
            transform_a: transform.matrix,
            transform_b: [transform.translation.x, transform.translation.y, 0.0, 0.0],
            clip_meta: [clip_start, clip_count, 0, 0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    size: [f32; 2],
    origin: [f32; 2],
}

const VIEWPORT_STRIDE: u64 = 256;
const VIEWPORT_CAPACITY: u64 = 1024;

pub(super) struct TextPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    atlas_view: wgpu::TextureView,
    atlas_sampler: wgpu::Sampler,
    bind_group: wgpu::BindGroup,
    viewport_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    clip_buffer: wgpu::Buffer,
    instance_capacity: usize,
    clip_capacity: usize,
    previous_instances: Vec<GlyphInstance>,
    previous_clips: Vec<TextClip>,
    next_viewport: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl TextPipeline {
    pub fn new(
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        atlas_view: &wgpu::TextureView,
        atlas_sampler: &wgpu::Sampler,
    ) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-text-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/primitives/text.wgsl").into(),
            ),
        });
        let layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("argui-text-layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Storage { read_only: true },
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: true,
                            min_binding_size: wgpu::BufferSize::new(
                                size_of::<ViewportUniform>() as u64
                            ),
                        },
                        count: None,
                    },
                ],
            });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("argui-text-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-text-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[Some(GlyphInstance::LAYOUT)],
            },
            primitive: Default::default(),
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fragment"),
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let viewport_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("argui-text-viewport"),
            size: VIEWPORT_STRIDE * VIEWPORT_CAPACITY,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_capacity = 1;
        let instance_buffer = create_instance_buffer(device, instance_capacity);
        let clip_capacity = 1;
        let clip_buffer = create_clip_buffer(device, clip_capacity);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-text-bind-group"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(atlas_view),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: clip_buffer.as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(atlas_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &viewport_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(size_of::<ViewportUniform>() as u64),
                    }),
                },
            ],
        });
        Self {
            pipeline,
            layout,
            atlas_view: atlas_view.clone(),
            atlas_sampler: atlas_sampler.clone(),
            bind_group,
            viewport_buffer,
            instance_buffer,
            clip_buffer,
            instance_capacity,
            clip_capacity,
            previous_instances: Vec::new(),
            previous_clips: Vec::new(),
            next_viewport: 0,
        }
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[GlyphInstance],
        clips: &[TextClip],
    ) -> bool {
        let mut rebuild = false;
        let instances_reallocated = instances.len() > self.instance_capacity;
        if instances_reallocated {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = create_instance_buffer(device, self.instance_capacity);
            rebuild = true;
        }
        let clips_reallocated = clips.len() > self.clip_capacity;
        if clips_reallocated {
            self.clip_capacity = clips.len().next_power_of_two();
            self.clip_buffer = create_clip_buffer(device, self.clip_capacity);
            rebuild = true;
        }
        if rebuild {
            self.bind_group = create_bind_group(
                device,
                &self.layout,
                &self.atlas_view,
                &self.atlas_sampler,
                &self.viewport_buffer,
                &self.clip_buffer,
            );
        }
        let instances_changed = crate::upload::write_changed(
            queue,
            &self.instance_buffer,
            instances,
            &mut self.previous_instances,
            instances_reallocated,
        );
        let clips_changed = crate::upload::write_changed(
            queue,
            &self.clip_buffer,
            clips,
            &mut self.previous_clips,
            clips_reallocated,
        );
        instances_changed || clips_changed
    }

    pub fn begin_frame(&mut self) {
        self.next_viewport = 0;
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        let slot = self.next_viewport % VIEWPORT_CAPACITY;
        self.next_viewport += 1;
        let offset = slot * VIEWPORT_STRIDE;
        queue.write_buffer(
            &self.viewport_buffer,
            offset,
            bytemuck::bytes_of(&ViewportUniform {
                size: [region[2], region[3]],
                origin: [region[0], region[1]],
            }),
        );
        offset as u32
    }

    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        instances: Range<u32>,
        viewport_offset: u32,
    ) {
        if instances.start >= instances.end {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[viewport_offset]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(0..6, instances);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("argui-text-instances"),
        size: (capacity * size_of::<GlyphInstance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn create_clip_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("argui-text-clips"),
        size: (capacity * size_of::<TextClip>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn create_bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    atlas_view: &wgpu::TextureView,
    atlas_sampler: &wgpu::Sampler,
    viewport: &wgpu::Buffer,
    clips: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("argui-text-bind-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(atlas_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::Sampler(atlas_sampler),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: viewport,
                    offset: 0,
                    size: wgpu::BufferSize::new(size_of::<ViewportUniform>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: clips.as_entire_binding(),
            },
        ],
    })
}
