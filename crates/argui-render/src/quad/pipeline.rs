use std::{mem::size_of, ops::Range};

use argui_paint::Quad;
use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Clone, Copy, Debug, Pod, Zeroable)]
pub(super) struct QuadInstance {
    rect: [f32; 4],
    background: [f32; 4],
    border_color: [f32; 4],
    radii: [f32; 4],
    border_widths: [f32; 4],
    clip: [f32; 4],
    params: [f32; 4],
}

impl QuadInstance {
    const ATTRIBUTES: [wgpu::VertexAttribute; 7] = wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Float32x4, 3 => Float32x4, 4 => Float32x4, 5 => Float32x4, 6 => Float32x4];
    const LAYOUT: wgpu::VertexBufferLayout<'static> = wgpu::VertexBufferLayout {
        array_stride: size_of::<Self>() as wgpu::BufferAddress,
        step_mode: wgpu::VertexStepMode::Instance,
        attributes: &Self::ATTRIBUTES,
    };

    pub fn new(quad: Quad, scale: f32) -> Self {
        let bounds = quad.bounds;
        let clip = quad.clip;
        Self {
            rect: [
                bounds.origin.x * scale,
                bounds.origin.y * scale,
                bounds.size.width * scale,
                bounds.size.height * scale,
            ],
            background: quad.background.as_array(),
            border_color: quad.border.color.as_array(),
            radii: quad.radii.as_array().map(|value| value * scale),
            border_widths: quad.border.widths.as_array().map(|value| value * scale),
            clip: [
                clip.origin.x * scale,
                clip.origin.y * scale,
                (clip.origin.x + clip.size.width) * scale,
                (clip.origin.y + clip.size.height) * scale,
            ],
            params: [quad.opacity, 0.0, 0.0, 0.0],
        }
    }
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct ViewportUniform {
    size: [f32; 2],
    padding: [f32; 2],
}

pub(super) struct QuadPipeline {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    viewport_buffer: wgpu::Buffer,
    instance_buffer: wgpu::Buffer,
    instance_capacity: usize,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl QuadPipeline {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-quad-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });
        let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-quad-pipeline"),
            layout: None,
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vertex"),
                compilation_options: Default::default(),
                buffers: &[Some(QuadInstance::LAYOUT)],
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
            size: size_of::<ViewportUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let instance_capacity = 1;
        let instance_buffer = create_instance_buffer(device, instance_capacity);
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-quad-bind-group"),
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: viewport_buffer.as_entire_binding(),
            }],
        });
        Self {
            pipeline,
            bind_group,
            viewport_buffer,
            instance_buffer,
            instance_capacity,
        }
    }

    pub fn write(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[QuadInstance],
        viewport: [f32; 2],
    ) {
        if instances.len() > self.instance_capacity {
            self.instance_capacity = instances.len().next_power_of_two();
            self.instance_buffer = create_instance_buffer(device, self.instance_capacity);
        }
        queue.write_buffer(
            &self.viewport_buffer,
            0,
            bytemuck::bytes_of(&ViewportUniform {
                size: viewport,
                padding: [0.0; 2],
            }),
        );
        if !instances.is_empty() {
            queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(instances));
        }
    }

    pub fn draw<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>, instances: Range<u32>) {
        if instances.start >= instances.end {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instance_buffer.slice(..));
        pass.draw(0..6, instances);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
fn create_instance_buffer(device: &wgpu::Device, capacity: usize) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("argui-quad-instances"),
        size: (capacity * size_of::<QuadInstance>()) as u64,
        usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
        mapped_at_creation: false,
    })
}

#[cfg(test)]
mod tests {
    use argui_core::{Color, Point, Rect, Size};
    use argui_paint::{Border, BorderWidths, CornerRadii, Quad};

    use super::QuadInstance;

    #[test]
    fn quad_instances_scale_logical_geometry_once() {
        let quad = Quad {
            bounds: Rect::new(Point::new(2.0, 3.0), Size::new(40.0, 20.0)),
            background: Color::rgb(0.1, 0.2, 0.3),
            border: Border {
                widths: BorderWidths {
                    left: 1.0,
                    right: 2.0,
                    top: 3.0,
                    bottom: 4.0,
                },
                color: Color::WHITE,
            },
            radii: CornerRadii::all(5.0),
            opacity: 0.75,
            clip: Rect::new(Point::new(1.0, 2.0), Size::new(50.0, 30.0)),
        };

        let instance = QuadInstance::new(quad, 2.0);

        assert_eq!(instance.rect, [4.0, 6.0, 80.0, 40.0]);
        assert_eq!(instance.radii, [10.0; 4]);
        assert_eq!(instance.border_widths, [2.0, 4.0, 6.0, 8.0]);
        assert_eq!(instance.clip, [2.0, 4.0, 102.0, 64.0]);
        assert_eq!(instance.params[0], 0.75);
    }
}
