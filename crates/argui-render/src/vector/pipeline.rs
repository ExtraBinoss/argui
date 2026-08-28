use std::{mem::size_of, ops::Range};

use argui_paint::VectorPrimitive;
use bytemuck::{Pod, Zeroable};

use super::{GpuVertex, Mesh};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct VectorClip {
    inverse_a: [f32; 4],
    inverse_b: [f32; 4],
    bounds: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct VectorInstance {
    rect: [f32; 4],
    source_size: [f32; 2],
    params: [f32; 2],
    transform_a: [f32; 4],
    transform_b: [f32; 4],
    clip_meta: [u32; 4],
}

impl VectorInstance {
    pub fn new(
        vector: &VectorPrimitive,
        source_size: [f32; 2],
        scale: f32,
        clips: &mut Vec<VectorClip>,
    ) -> Self {
        let start = clips.len() as u32;
        clips.extend(vector.clips.regions().iter().filter_map(|clip| {
            let inverse = clip.transform.scaled(scale).inverse()?;
            Some(VectorClip {
                inverse_a: inverse.matrix,
                inverse_b: [inverse.translation.x, inverse.translation.y, 0.0, 0.0],
                bounds: [
                    clip.bounds.origin.x * scale,
                    clip.bounds.origin.y * scale,
                    clip.bounds.size.width * scale,
                    clip.bounds.size.height * scale,
                ],
            })
        }));
        let transform = vector.transform.scaled(scale);
        Self {
            rect: [
                vector.bounds.origin.x * scale,
                vector.bounds.origin.y * scale,
                vector.bounds.size.width * scale,
                vector.bounds.size.height * scale,
            ],
            source_size,
            params: [vector.progress.clamp(0.0, 1.0), vector.opacity],
            transform_a: transform.matrix,
            transform_b: [transform.translation.x, transform.translation.y, 0.0, 0.0],
            clip_meta: [start, clips.len() as u32 - start, 0, 0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Viewport {
    size: [f32; 2],
    origin: [f32; 2],
}

const VIEWPORT_STRIDE: u64 = 256;
const VIEWPORT_CAPACITY: u64 = 1024;

pub(super) struct VectorPipeline {
    pipeline: wgpu::RenderPipeline,
    scene_layout: wgpu::BindGroupLayout,
    scene_group: wgpu::BindGroup,
    viewport: wgpu::Buffer,
    instances: wgpu::Buffer,
    clips: wgpu::Buffer,
    instance_capacity: usize,
    clip_capacity: usize,
    previous_instances: Vec<VectorInstance>,
    previous_clips: Vec<VectorClip>,
    next_viewport: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl VectorPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-vector-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/primitives/vector.wgsl").into(),
            ),
        });
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-vector-scene-layout"),
            entries: &[
                buffer_layout(0, wgpu::BufferBindingType::Uniform, true),
                buffer_layout(
                    1,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    false,
                ),
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("argui-vector-pipeline-layout"),
            bind_group_layouts: &[Some(&scene_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-vector-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[Some(vertex_layout()), Some(instance_layout())],
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
        let viewport = buffer(
            device,
            "argui-vector-viewport",
            VIEWPORT_STRIDE * VIEWPORT_CAPACITY,
            wgpu::BufferUsages::UNIFORM,
        );
        let instances = buffer(
            device,
            "argui-vector-instances",
            size_of::<VectorInstance>() as u64,
            wgpu::BufferUsages::VERTEX,
        );
        let clips = buffer(
            device,
            "argui-vector-clips",
            size_of::<VectorClip>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        let scene_group = scene_group(device, &scene_layout, &viewport, &clips);
        Self {
            pipeline,
            scene_layout,
            scene_group,
            viewport,
            instances,
            clips,
            instance_capacity: 1,
            clip_capacity: 1,
            previous_instances: Vec::new(),
            previous_clips: Vec::new(),
            next_viewport: 0,
        }
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[VectorInstance],
        clips: &[VectorClip],
    ) {
        let instances_reallocated = instances.len() > self.instance_capacity;
        if instances_reallocated {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instances = buffer(
                device,
                "argui-vector-instances",
                (self.instance_capacity * size_of::<VectorInstance>()) as u64,
                wgpu::BufferUsages::VERTEX,
            );
        }
        let clips_reallocated = clips.len() > self.clip_capacity;
        if clips_reallocated {
            self.clip_capacity = clips.len().next_power_of_two();
            self.clips = buffer(
                device,
                "argui-vector-clips",
                (self.clip_capacity * size_of::<VectorClip>()) as u64,
                wgpu::BufferUsages::STORAGE,
            );
            self.scene_group = scene_group(device, &self.scene_layout, &self.viewport, &self.clips);
        }
        crate::upload::write_changed(
            queue,
            &self.instances,
            instances,
            &mut self.previous_instances,
            instances_reallocated,
        );
        crate::upload::write_changed(
            queue,
            &self.clips,
            clips,
            &mut self.previous_clips,
            clips_reallocated,
        );
    }

    pub fn begin_frame(&mut self) {
        self.next_viewport = 0;
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        let offset = self.next_viewport % VIEWPORT_CAPACITY * VIEWPORT_STRIDE;
        self.next_viewport += 1;
        queue.write_buffer(
            &self.viewport,
            offset,
            bytemuck::bytes_of(&Viewport {
                size: [region[2], region[3]],
                origin: [region[0], region[1]],
            }),
        );
        offset as u32
    }

    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        mesh: &'a Mesh,
        range: Range<u32>,
        viewport: u32,
    ) {
        if range.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.scene_group, &[viewport]);
        pass.set_vertex_buffer(0, mesh.vertices.slice(..));
        pass.set_vertex_buffer(1, self.instances.slice(..));
        pass.draw(0..mesh.vertex_count, range);
    }
}

fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![
        0=>Float32x2, 1=>Float32x2, 2=>Float32x4, 3=>Float32x4,
        4=>Float32x3, 5=>Float32x3
    ];
    wgpu::VertexBufferLayout {
        array_stride: size_of::<GpuVertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &ATTRS,
    }
}

fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRS: [wgpu::VertexAttribute; 6] = wgpu::vertex_attr_array![6=>Float32x4,7=>Float32x2,8=>Float32x2,9=>Float32x4,10=>Float32x4,11=>Uint32x4];
    wgpu::VertexBufferLayout {
        array_stride: size_of::<VectorInstance>() as u64,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &ATTRS,
    }
}

fn buffer_layout(
    binding: u32,
    ty: wgpu::BufferBindingType,
    dynamic: bool,
) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer {
            ty,
            has_dynamic_offset: dynamic,
            min_binding_size: None,
        },
        count: None,
    }
}

fn buffer(
    device: &wgpu::Device,
    label: &str,
    size: u64,
    usage: wgpu::BufferUsages,
) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size,
        usage: usage | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

fn scene_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    viewport: &wgpu::Buffer,
    clips: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("argui-vector-scene-group"),
        layout,
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: viewport,
                    offset: 0,
                    size: wgpu::BufferSize::new(size_of::<Viewport>() as u64),
                }),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: clips.as_entire_binding(),
            },
        ],
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn vector_shader_is_valid_wgsl() {
        naga::front::wgsl::parse_str(include_str!("../shaders/primitives/vector.wgsl")).unwrap();
    }
}
