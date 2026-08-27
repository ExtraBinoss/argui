use std::{mem::size_of, ops::Range};

use argui_paint::{ImageFit, ImagePrimitive, ImageSampling};
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ImageClip {
    inverse_a: [f32; 4],
    inverse_b: [f32; 4],
    bounds: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub(super) struct ImageInstance {
    rect: [f32; 4],
    uv: [f32; 4],
    radii: [f32; 4],
    transform_a: [f32; 4],
    transform_b: [f32; 4],
    params: [f32; 4],
    clip_meta: [u32; 4],
}

impl ImageInstance {
    pub fn new(
        image: &ImagePrimitive,
        image_size: [u32; 2],
        scale: f32,
        clips: &mut Vec<ImageClip>,
    ) -> Self {
        let start = clips.len() as u32;
        clips.extend(image.clips.regions().iter().filter_map(|clip| {
            let inverse = clip.transform.scaled(scale).inverse()?;
            Some(ImageClip {
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
        let mut rect = [
            image.bounds.origin.x * scale,
            image.bounds.origin.y * scale,
            image.bounds.size.width * scale,
            image.bounds.size.height * scale,
        ];
        let mut uv = [0.0, 0.0, 1.0, 1.0];
        fit(image.fit, image_size, &mut rect, &mut uv);
        let transform = image.transform.scaled(scale);
        Self {
            rect,
            uv,
            radii: image.radii.as_array().map(|value| value * scale),
            transform_a: transform.matrix,
            transform_b: [transform.translation.x, transform.translation.y, 0.0, 0.0],
            params: [image.opacity, 0.0, 0.0, 0.0],
            clip_meta: [start, clips.len() as u32 - start, 0, 0],
        }
    }
}

fn fit(fit: ImageFit, image: [u32; 2], rect: &mut [f32; 4], uv: &mut [f32; 4]) {
    let source = image[0] as f32 / image[1].max(1) as f32;
    let target = rect[2] / rect[3].max(0.000_01);
    match fit {
        ImageFit::Fill => {}
        ImageFit::Contain => {
            if source > target {
                let height = rect[2] / source;
                rect[1] += (rect[3] - height) * 0.5;
                rect[3] = height;
            } else {
                let width = rect[3] * source;
                rect[0] += (rect[2] - width) * 0.5;
                rect[2] = width;
            }
        }
        ImageFit::Cover => {
            if source > target {
                let width = target / source;
                uv[0] = (1.0 - width) * 0.5;
                uv[2] = 1.0 - uv[0];
            } else {
                let height = source / target;
                uv[1] = (1.0 - height) * 0.5;
                uv[3] = 1.0 - uv[1];
            }
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

pub(super) struct ImagePipeline {
    pipeline: wgpu::RenderPipeline,
    scene_layout: wgpu::BindGroupLayout,
    texture_layout: wgpu::BindGroupLayout,
    scene_group: wgpu::BindGroup,
    viewport: wgpu::Buffer,
    instances: wgpu::Buffer,
    clips: wgpu::Buffer,
    instance_capacity: usize,
    clip_capacity: usize,
    next_viewport: u64,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ImagePipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-image-shader"),
            source: wgpu::ShaderSource::Wgsl(
                include_str!("../shaders/primitives/image.wgsl").into(),
            ),
        });
        let scene_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-image-scene-layout"),
            entries: &[
                buffer_layout(0, wgpu::BufferBindingType::Uniform, true),
                buffer_layout(
                    1,
                    wgpu::BufferBindingType::Storage { read_only: true },
                    false,
                ),
            ],
        });
        let texture_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-image-texture-layout"),
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
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("argui-image-pipeline-layout"),
            bind_group_layouts: &[Some(&scene_layout), Some(&texture_layout)],
            immediate_size: 0,
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-image-pipeline"),
            layout: Some(&layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[Some(instance_layout())],
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
            "argui-image-viewport",
            VIEWPORT_STRIDE * VIEWPORT_CAPACITY,
            wgpu::BufferUsages::UNIFORM,
        );
        let instances = buffer(
            device,
            "argui-image-instances",
            size_of::<ImageInstance>() as u64,
            wgpu::BufferUsages::VERTEX,
        );
        let clips = buffer(
            device,
            "argui-image-clips",
            size_of::<ImageClip>() as u64,
            wgpu::BufferUsages::STORAGE,
        );
        let scene_group = scene_group(device, &scene_layout, &viewport, &clips);
        Self {
            pipeline,
            scene_layout,
            texture_layout,
            scene_group,
            viewport,
            instances,
            clips,
            instance_capacity: 1,
            clip_capacity: 1,
            next_viewport: 0,
        }
    }

    pub fn texture_group(
        &self,
        device: &wgpu::Device,
        view: &wgpu::TextureView,
        sampling: ImageSampling,
    ) -> wgpu::BindGroup {
        let filter = match sampling {
            ImageSampling::Nearest => wgpu::FilterMode::Nearest,
            ImageSampling::Linear => wgpu::FilterMode::Linear,
        };
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        });
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-image-texture-group"),
            layout: &self.texture_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
        })
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[ImageInstance],
        clips: &[ImageClip],
    ) {
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instances = buffer(
                device,
                "argui-image-instances",
                (self.instance_capacity * size_of::<ImageInstance>()) as u64,
                wgpu::BufferUsages::VERTEX,
            );
        }
        if clips.len() > self.clip_capacity {
            self.clip_capacity = clips.len().next_power_of_two();
            self.clips = buffer(
                device,
                "argui-image-clips",
                (self.clip_capacity * size_of::<ImageClip>()) as u64,
                wgpu::BufferUsages::STORAGE,
            );
            self.scene_group = scene_group(device, &self.scene_layout, &self.viewport, &self.clips);
        }
        if !instances.is_empty() {
            queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(instances));
        }
        if !clips.is_empty() {
            queue.write_buffer(&self.clips, 0, bytemuck::cast_slice(clips));
        }
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
        texture: &'a wgpu::BindGroup,
        range: Range<u32>,
        viewport: u32,
    ) {
        if range.is_empty() {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.scene_group, &[viewport]);
        pass.set_bind_group(1, texture, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..6, range);
    }
}

fn instance_layout() -> wgpu::VertexBufferLayout<'static> {
    const ATTRS: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![0=>Float32x4,1=>Float32x4,2=>Float32x4,3=>Float32x4,4=>Float32x4,5=>Float32x4,6=>Uint32x4];
    wgpu::VertexBufferLayout {
        array_stride: size_of::<ImageInstance>() as u64,
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
#[cfg_attr(coverage_nightly, coverage(off))]
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

#[cfg_attr(coverage_nightly, coverage(off))]
fn scene_group(
    device: &wgpu::Device,
    layout: &wgpu::BindGroupLayout,
    viewport: &wgpu::Buffer,
    clips: &wgpu::Buffer,
) -> wgpu::BindGroup {
    device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("argui-image-scene-group"),
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
    use argui_paint::ImageFit;

    use super::fit;

    fn fitted(fit_mode: ImageFit, image: [u32; 2]) -> ([f32; 4], [f32; 4]) {
        let mut rect = [0.0, 0.0, 100.0, 100.0];
        let mut uv = [0.0, 0.0, 1.0, 1.0];
        fit(fit_mode, image, &mut rect, &mut uv);
        (rect, uv)
    }

    #[test]
    fn image_fit_handles_both_aspect_ratio_directions() {
        assert_eq!(
            fitted(ImageFit::Fill, [200, 100]),
            ([0.0, 0.0, 100.0, 100.0], [0.0, 0.0, 1.0, 1.0])
        );
        assert_eq!(
            fitted(ImageFit::Contain, [200, 100]).0,
            [0.0, 25.0, 100.0, 50.0]
        );
        assert_eq!(
            fitted(ImageFit::Contain, [100, 200]).0,
            [25.0, 0.0, 50.0, 100.0]
        );
        assert_eq!(
            fitted(ImageFit::Cover, [200, 100]).1,
            [0.25, 0.0, 0.75, 1.0]
        );
        assert_eq!(
            fitted(ImageFit::Cover, [100, 200]).1,
            [0.0, 0.25, 1.0, 0.75]
        );
    }
}
