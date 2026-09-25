use wgpu::util::DeviceExt;

/// Renders an impulse through one blur shader mode and returns RGBA pixels.
///
/// `mode` selects Gaussian horizontal, dual downsample, or dual upsample.
/// Returns an empty vector when no headless GPU adapter is available.
fn render_impulse(mode: u32) -> Vec<u8> {
    const WIDTH: u32 = 64;
    let instance = wgpu::Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        eprintln!("No headless GPU adapter; skipping blur pixel check");
        return Vec::new();
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: Some("blur-regression-shader"),
        source: wgpu::ShaderSource::Wgsl(
            include_str!("../../src/shaders/effects/blur.wgsl").into(),
        ),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: Some("blur-regression-pipeline"),
        layout: None,
        vertex: wgpu::VertexState {
            module: &shader,
            entry_point: Some("vs_main"),
            buffers: &[],
            compilation_options: Default::default(),
        },
        fragment: Some(wgpu::FragmentState {
            module: &shader,
            entry_point: Some("fs_main"),
            targets: &[Some(wgpu::ColorTargetState {
                format: wgpu::TextureFormat::Rgba8Unorm,
                blend: None,
                write_mask: wgpu::ColorWrites::ALL,
            })],
            compilation_options: Default::default(),
        }),
        primitive: Default::default(),
        depth_stencil: None,
        multisample: Default::default(),
        multiview_mask: None,
        cache: None,
    });
    let extent = wgpu::Extent3d {
        width: WIDTH,
        height: 1,
        depth_or_array_layers: 1,
    };
    let texture = |usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some("blur-regression-texture"),
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        })
    };
    let source = texture(wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::TEXTURE_BINDING);
    let output = texture(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let mut input_pixels = [0_u8; (WIDTH * 4) as usize];
    for pixel in input_pixels.as_chunks_mut::<4>().0 {
        pixel[3] = 255;
    }
    input_pixels[32 * 4..32 * 4 + 4].copy_from_slice(&[255; 4]);
    queue.write_texture(
        source.as_image_copy(),
        &input_pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(WIDTH * 4),
            rows_per_image: Some(1),
        },
        extent,
    );
    let mut params = [0_u32; 60];
    params[0] = (WIDTH as f32).to_bits();
    params[1] = 1.0_f32.to_bits();
    params[2] = mode;
    for index in [4, 8, 12, 24] {
        params[index + 2] = (WIDTH as f32).to_bits();
        params[index + 3] = 1.0_f32.to_bits();
    }
    for index in [16, 20] {
        params[index + 2] = 1.0_f32.to_bits();
        params[index + 3] = 1.0_f32.to_bits();
    }
    params[28..32].fill((-1.0_f32).to_bits());
    params[36] = if mode == 1 { 3.5_f32 } else { 1.5_f32 }.to_bits();
    let uniforms = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: Some("blur-regression-uniforms"),
        contents: bytemuck::cast_slice(&params),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let source_view = source.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: Some("blur-regression-bind-group"),
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&source_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: uniforms.as_entire_binding(),
            },
        ],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("blur-regression-readback"),
        size: u64::from(WIDTH * 4),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("blur-regression-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        pass.set_pipeline(&pipeline);
        pass.set_bind_group(0, &bind_group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(WIDTH * 4),
                rows_per_image: Some(1),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (sender, receiver) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            sender.send(result).unwrap();
        });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receiver.recv().unwrap().unwrap();
    let pixels = readback.slice(..).get_mapped_range().unwrap().to_vec();
    if let Ok(directory) = std::env::var("ARGUI_BLUR_CAPTURE_DIR") {
        let directory = std::path::Path::new(&directory);
        std::fs::create_dir_all(directory).unwrap();
        std::fs::write(directory.join(format!("mode-{mode}.rgba")), &pixels).unwrap();
    }
    pixels
}

/// A wide Gaussian blur must fill neighboring pixels smoothly instead of leaving sampled gaps.
#[test]
fn wide_blur_has_no_periodic_holes() {
    let pixels = render_impulse(1);
    if pixels.is_empty() {
        return;
    }
    let red = |offset: usize| pixels[(32 + offset) * 4];
    assert!(red(0) > 20, "blur erased the source pixel");
    for offset in 1..=8 {
        assert!(red(offset) > 0, "gap at offset {offset}");
        assert!(
            red(offset - 1) >= red(offset),
            "blur has a repeated band at offset {offset}"
        );
    }
}

/// Each dual-filter kernel must preserve neighboring pixels and an impulse's center.
#[test]
fn dual_filter_kernels_spread_an_impulse() {
    for mode in [3, 4] {
        let pixels = render_impulse(mode);
        if pixels.is_empty() {
            return;
        }
        let red = |offset: usize| pixels[(32 + offset) * 4];
        assert!(red(0) > 0, "mode {mode} erased the source pixel");
        assert!(red(1) > 0, "mode {mode} left a gap beside the source");
        assert_eq!(pixels[31 * 4], pixels[33 * 4], "mode {mode} is asymmetric");
    }
}
