use wgpu::util::DeviceExt;

use crate::DamageRegion;

struct RetainedTarget {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    extent: [u32; 2],
    bytes: u64,
}

impl Drop for RetainedTarget {
    fn drop(&mut self) {
        self._texture.destroy();
    }
}

/// GPU resources used to preserve and selectively refresh direct UI pixels.
pub(crate) struct DamageGpu {
    clear_pipeline: wgpu::RenderPipeline,
    clear_buffer: wgpu::Buffer,
    clear_group: wgpu::BindGroup,
    target: Option<RetainedTarget>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl DamageGpu {
    /// Creates the regional clear pipeline for `format`.
    pub(crate) fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("argui-damage-clear-layout"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: false,
                    min_binding_size: wgpu::BufferSize::new(16),
                },
                count: None,
            }],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("argui-damage-clear-pipeline-layout"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("argui-damage-clear-shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("../shaders/damage/clear.wgsl").into()),
        });
        let clear_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("argui-damage-clear-pipeline"),
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
                    blend: None,
                    write_mask: wgpu::ColorWrites::ALL,
                })],
            }),
            multiview_mask: None,
            cache: None,
        });
        let clear_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("argui-damage-clear-color"),
            contents: &[0; 16],
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let clear_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("argui-damage-clear-group"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: clear_buffer.as_entire_binding(),
            }],
        });
        Self {
            clear_pipeline,
            clear_buffer,
            clear_group,
            target: None,
        }
    }

    /// Ensures a retained target matching `extent` and reports whether it was allocated.
    pub(crate) fn ensure_target(
        &mut self,
        device: &wgpu::Device,
        format: wgpu::TextureFormat,
        extent: [u32; 2],
    ) -> bool {
        if self
            .target
            .as_ref()
            .is_some_and(|target| target.extent == extent)
        {
            return false;
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("argui-retained-surface"),
            size: wgpu::Extent3d {
                width: extent[0].max(1),
                height: extent[1].max(1),
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let bytes = u64::from(extent[0])
            * u64::from(extent[1])
            * u64::from(format.block_copy_size(None).unwrap_or(4));
        self.target = Some(RetainedTarget {
            _texture: texture,
            view,
            extent,
            bytes,
        });
        true
    }

    /// Drops retained pixels, for example after resize or an incompatible frame.
    pub(crate) fn invalidate(&mut self) {
        self.target = None;
    }

    /// Returns whether a retained root texture is currently available.
    pub(crate) fn valid(&self) -> bool {
        self.target.is_some()
    }

    /// Returns a clone of the retained root view, when allocated.
    pub(crate) fn view(&self) -> Option<wgpu::TextureView> {
        self.target.as_ref().map(|target| target.view.clone())
    }

    /// Returns bytes allocated by the retained root texture.
    pub(crate) fn bytes(&self) -> u64 {
        self.target.as_ref().map_or(0, |target| target.bytes)
    }

    /// Uploads the solid clear color used before regional repainting.
    pub(crate) fn prepare_clear(&self, queue: &wgpu::Queue, color: wgpu::Color) {
        let color = [
            color.r as f32,
            color.g as f32,
            color.b as f32,
            color.a as f32,
        ];
        queue.write_buffer(&self.clear_buffer, 0, bytemuck::cast_slice(&color));
    }

    /// Clears the current scissor rectangle without touching retained pixels outside it.
    pub(crate) fn clear<'pass>(&'pass self, pass: &mut wgpu::RenderPass<'pass>) {
        pass.set_pipeline(&self.clear_pipeline);
        pass.set_bind_group(0, &self.clear_group, &[]);
        pass.draw(0..3, 0..1);
    }

    /// Applies `region` as the active render-pass scissor.
    pub(crate) fn scissor(pass: &mut wgpu::RenderPass<'_>, region: DamageRegion) {
        pass.set_scissor_rect(
            region.x,
            region.y,
            region.width.max(1),
            region.height.max(1),
        );
    }
}
