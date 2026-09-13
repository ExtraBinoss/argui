use std::{mem::size_of, ops::Range};

use argui_paint::{ClipRegion, ColorInterpolation, Fill, Quad};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct ClipInstance {
    inverse_a: [f32; 4],
    inverse_b: [f32; 4],
    bounds: [f32; 4],
    radii: [f32; 4],
}

impl ClipInstance {
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
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct QuadInstance {
    rect: [f32; 4],
    background: [f32; 4],
    border_color: [f32; 4],
    radii: [f32; 4],
    border_widths: [f32; 4],
    transform_a: [f32; 4],
    transform_b: [f32; 4],
    fill_geometry: [f32; 4],
    params: [f32; 4],
    clip_meta: [u32; 4],
    gradient_meta: [u32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct GradientStopInstance {
    offset: [f32; 4],
    color: [f32; 4],
}

impl QuadInstance {
    pub fn new(
        quad: &Quad,
        scale: f32,
        clip_start: u32,
        clip_count: u32,
        stops: &mut Vec<GradientStopInstance>,
    ) -> Self {
        let transform = quad.transform.scaled(scale);
        let mut instance = Self {
            rect: [
                quad.bounds.origin.x * scale,
                quad.bounds.origin.y * scale,
                quad.bounds.size.width * scale,
                quad.bounds.size.height * scale,
            ],
            background: [0.0; 4],
            border_color: quad.border.color.to_linear_rgba(),
            radii: quad.radii.as_array().map(|value| value * scale),
            border_widths: quad.border.widths.as_array().map(|value| value * scale),
            transform_a: transform.matrix,
            transform_b: [transform.translation.x, transform.translation.y, 0.0, 0.0],
            fill_geometry: [0.0; 4],
            params: [quad.opacity, 0.0, 0.0, 0.0],
            clip_meta: [clip_start, clip_count, 0, 0],
            gradient_meta: [0; 4],
        };
        instance.set_fill(quad.background.as_ref(), stops);
        instance
    }

    fn set_fill(&mut self, fill: Option<&Fill>, stops: &mut Vec<GradientStopInstance>) {
        match fill {
            Some(Fill::Solid(color)) => self.background = color.to_linear_rgba(),
            Some(Fill::Linear(gradient)) => {
                self.params[1] = 1.0;
                self.fill_geometry = [
                    gradient.start.x,
                    gradient.start.y,
                    gradient.end.x,
                    gradient.end.y,
                ];
                self.set_stops(gradient.stops.as_slice(), gradient.interpolation, stops);
            }
            Some(Fill::Radial(gradient)) => {
                self.params[1] = 2.0;
                self.fill_geometry = [
                    gradient.center.x,
                    gradient.center.y,
                    gradient.radius.x,
                    gradient.radius.y,
                ];
                self.set_stops(gradient.stops.as_slice(), gradient.interpolation, stops);
            }
            Some(Fill::Bilinear(gradient)) => {
                self.params[1] = 3.0;
                let corners = gradient
                    .corners()
                    .map(|color| argui_paint::GradientStop::new(0.0, color));
                self.set_stops(&corners, gradient.interpolation, stops);
            }
            None => {}
        }
    }

    fn set_stops(
        &mut self,
        gradient: &[argui_paint::GradientStop],
        interpolation: ColorInterpolation,
        stops: &mut Vec<GradientStopInstance>,
    ) {
        let interpolation = match interpolation {
            ColorInterpolation::Oklab => 0,
            ColorInterpolation::LinearSrgb => 1,
            ColorInterpolation::Srgb => 2,
        };
        self.gradient_meta = [stops.len() as u32, gradient.len() as u32, interpolation, 0];
        stops.extend(gradient.iter().map(|stop| {
            let [first, second, third, alpha] = stop
                .color
                .to_interpolation_components(interpolation_space(interpolation));
            GradientStopInstance {
                offset: [stop.offset, 0.0, 0.0, 0.0],
                color: [first * alpha, second * alpha, third * alpha, alpha],
            }
        }));
    }
}

const fn interpolation_space(value: u32) -> ColorInterpolation {
    match value {
        0 => ColorInterpolation::Oklab,
        1 => ColorInterpolation::LinearSrgb,
        _ => ColorInterpolation::Srgb,
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

pub(super) struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    layout: wgpu::BindGroupLayout,
    bind_group: wgpu::BindGroup,
    viewport_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    clip_buffer: wgpu::Buffer,
    gradient_buffer: wgpu::Buffer,
    instance_capacity: usize,
    clip_capacity: usize,
    gradient_capacity: usize,
    previous_instances: Vec<QuadInstance>,
    previous_clips: Vec<ClipInstance>,
    previous_gradients: Vec<GradientStopInstance>,
    next_viewport: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl QuadPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-quad-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/primitives/quad.wgsl").into(),
            ),
        });
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-quad-layout"),
            entries: &[
                buffer_layout(0, wgpu::BufferBindingType::Uniform, true),
                buffer_layout(
                    1,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    false,
                ),
                buffer_layout(
                    2,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    false,
                ),
                buffer_layout(
                    3,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    false,
                ),
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("argui-quad-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-quad-pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[],
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
            label: Some("argui-quad-viewport"),
            size: VIEWPORT_STRIDE * VIEWPORT_CAPACITY,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_buffer = storage_buffer::<QuadInstance>(device, "argui-quad-instances", 1);
        let clip_buffer = storage_buffer::<ClipInstance>(device, "argui-quad-clips", 1);
        let gradient_buffer =
            storage_buffer::<GradientStopInstance>(device, "argui-gradient-stops", 1);
        let bind_group = bind_group(
            device,
            &layout,
            &viewport_buffer,
            &instance_buffer,
            &clip_buffer,
            &gradient_buffer,
        );
        Self {
            pipeline,
            layout,
            bind_group,
            viewport_buffer,
            instance_buffer,
            clip_buffer,
            gradient_buffer,
            instance_capacity: 1,
            clip_capacity: 1,
            gradient_capacity: 1,
            previous_instances: Vec::new(),
            previous_clips: Vec::new(),
            previous_gradients: Vec::new(),
            next_viewport: 0,
        }
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[QuadInstance],
        clips: &[ClipInstance],
        gradients: &[GradientStopInstance],
    ) -> bool {
        let mut changed = false;
        let mut instances_reallocated = false;
        let mut clips_reallocated = false;
        let mut gradients_reallocated = false;
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = storage_buffer::<QuadInstance>(
                device,
                "argui-quad-instances",
                self.instance_capacity,
            );
            changed = true;
            instances_reallocated = true;
        }
        if clips.len() > self.clip_capacity {
            self.clip_capacity = clips.len().next_power_of_two();
            self.clip_buffer =
                storage_buffer::<ClipInstance>(device, "argui-quad-clips", self.clip_capacity);
            changed = true;
            clips_reallocated = true;
        }
        if gradients.len() > self.gradient_capacity {
            self.gradient_capacity = gradients.len().next_power_of_two();
            self.gradient_buffer = storage_buffer::<GradientStopInstance>(
                device,
                "argui-gradient-stops",
                self.gradient_capacity,
            );
            changed = true;
            gradients_reallocated = true;
        }
        if changed {
            self.bind_group = bind_group(
                device,
                &self.layout,
                &self.viewport_buffer,
                &self.instance_buffer,
                &self.clip_buffer,
                &self.gradient_buffer,
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
        let gradients_changed = crate::upload::write_changed(
            queue,
            &self.gradient_buffer,
            gradients,
            &mut self.previous_gradients,
            gradients_reallocated,
        );
        instances_changed || clips_changed || gradients_changed
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
        if instances.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[viewport_offset]);
        pass.draw(0..6, instances);
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

#[cfg_attr(coverage_nightly, coverage(off))]
fn storage_buffer<T>(device: &wgpu::Device, label: &str, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: (capacity * size_of::<T>()) as u64,
        usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn bind_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    viewport: &wgpu::Buffer,
    instances: &wgpu::Buffer,
    clips: &wgpu::Buffer,
    gradients: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("argui-quad-bind-group"),
        layout,
        entries: &[
            binding(0, viewport, size_of::<ViewportUniform>() as u64),
            binding(1, instances, instances.size()),
            binding(2, clips, clips.size()),
            binding(3, gradients, gradients.size()),
        ],
    })
}

fn binding(binding: u32, buffer: &wgpu::Buffer, size: u64) -> wgpu::BindGroupEntry<'_> {
    wgpu::BindGroupEntry {
        binding,
        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
            buffer,
            offset: 0,
            size: wgpu::BufferSize::new(size),
        }),
    }
}
