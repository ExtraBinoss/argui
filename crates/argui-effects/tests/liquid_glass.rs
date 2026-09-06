#![cfg(feature = "liquid-glass")]
use argui_effects::{LIQUID_GLASS_ID, LiquidGlass};
use argui_paint::{EffectValue, Filter};
use wgpu::util::DeviceExt;

fn effect(glass: LiquidGlass) -> argui_paint::EffectInstance {
    let Filter::Effect(effect) = glass.filter() else {
        panic!("custom effect expected")
    };
    effect
}
#[test]
fn parameters_are_bounded_finite_and_scale_in_logical_pixels() {
    let glass = LiquidGlass::default()
        .refraction(f32::NAN)
        .blur(f32::INFINITY)
        .frequency(0.0)
        .octaves(100)
        .seed(123)
        .turbulence(3.0)
        .tint([f32::NAN, -1.0, 2.0, 0.5])
        .saturation(10.0)
        .highlight(-1.0)
        .edge_width(20.0)
        .chromatic_aberration(2.0);
    let effect = effect(glass);
    assert_eq!(effect.id, LIQUID_GLASS_ID);
    let words = effect.packed_words();
    assert_eq!(f32::from_bits(words[0]), 0.0);
    assert_eq!(f32::from_bits(words[2]), 0.0);
    assert_eq!(words[7], 6);
    assert_eq!(words[8], 123);
    assert_eq!(f32::from_bits(words[9]), 1.0);
    assert_eq!(f32::from_bits(words[12]), 1.0);
    assert_eq!(f32::from_bits(words[10]), 0.0);
    let Filter::Effect(scaled) = Filter::Effect(effect.clone()).scaled(2.0) else {
        panic!()
    };
    let scaled = scaled.packed_words();
    assert_eq!(f32::from_bits(scaled[6]), f32::from_bits(words[6]) * 2.0);
    assert_eq!(scaled[8], words[8]);
    assert!(effect.parameters.iter().all(|p| match p.value {
        EffectValue::F32(v) | EffectValue::LogicalPixels(v) => v.is_finite(),
        _ => true,
    }));
    assert!(
        argui_effects::registry()
            .unwrap()
            .definitions()
            .iter()
            .any(|d| d.id == LIQUID_GLASS_ID)
    );
}

#[test]
fn gpu_glass_displaces_source_without_blur_and_seed_changes_the_image() {
    let instance = wgpu::Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        eprintln!("No headless GPU adapter; skipping pixel assertions");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let source = [
        include_str!("../../argui-render/src/shaders/effects/custom_abi_header.wgsl"),
        include_str!("../src/shaders/effects/liquid_glass.wgsl"),
        include_str!("../../argui-render/src/shaders/effects/custom_abi_footer.wgsl"),
    ]
    .join("\n");
    let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(source.into()),
    });
    let pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
        label: None,
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

    let base = LiquidGlass::new()
        .refraction(0.0)
        .chromatic_aberration(0.0)
        .blur(0.0)
        .highlight(0.0)
        .fresnel(0.0)
        .tint([0.0; 4])
        .turbulence(1.0);
    let render = |glass| {
        render_pixels(
            &device,
            &queue,
            &pipeline,
            32,
            &effect(glass).packed_words(),
        )
    };
    let identity = render(base);
    // Every input pixel survives when all optical modifications are disabled.
    for y in 0..32usize {
        for x in 0..32usize {
            let pixel = &identity[y * 256 + x * 4..y * 256 + x * 4 + 4];
            let expected = [
                if (x / 3 + y / 4).is_multiple_of(2) {
                    255
                } else {
                    0
                },
                (x * 255 / 32) as u8,
                (y * 255 / 32) as u8,
                255,
            ];
            assert_eq!(pixel, expected);
        }
    }
    let warped = render(base.refraction(20.0).frequency(0.1));
    let optical = render(base.refraction(12.0).turbulence(0.0));
    assert_ne!(identity, optical, "curved rim refracts with noise disabled");
    for y in 10..22usize {
        for x in 10..22usize {
            let offset = y * 256 + x * 4;
            assert_eq!(
                &identity[offset..offset + 4],
                &optical[offset..offset + 4],
                "flat center must remain undistorted"
            );
        }
    }
    assert_eq!(
        identity,
        render(base.refraction(12.0).turbulence(0.0).ior(1.0)),
        "no optical refraction at IOR 1"
    );
    let other = render(base.refraction(20.0).frequency(0.1).seed(17));
    assert_ne!(
        identity, warped,
        "refraction must move pixels even with blur disabled"
    );
    assert_ne!(warped, other, "seed must change displacement");
    assert_eq!(
        warped,
        render(base.refraction(20.0).frequency(0.1)),
        "deterministic fractal noise"
    );
    let tint = render(base.tint([0.0, 0.0, 1.0, 1.0]));
    assert_eq!(
        &tint[16 * 256 + 16 * 4..16 * 256 + 16 * 4 + 4],
        &[0, 0, 255, 255]
    );
}
fn render_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    size: u32,
    words: &[u32],
) -> Vec<u8> {
    let extent = wgpu::Extent3d {
        width: size,
        height: size,
        depth_or_array_layers: 1,
    };
    let texture = |usage| {
        device.create_texture(&wgpu::TextureDescriptor {
            label: None,
            size: extent,
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage,
            view_formats: &[],
        })
    };
    let input = texture(wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST);
    queue.write_texture(
        input.as_image_copy(),
        &(0..size * size)
            .flat_map(|i| {
                let x = i % size;
                let y = i / size;
                [
                    if (x / 3 + y / 4).is_multiple_of(2) {
                        255
                    } else {
                        0
                    },
                    (x * 255 / size) as u8,
                    (y * 255 / size) as u8,
                    255,
                ]
            })
            .collect::<Vec<_>>(),
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size * 4),
            rows_per_image: Some(size),
        },
        extent,
    );
    let output = texture(wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC);
    let input_view = input.create_view(&Default::default());
    let output_view = output.create_view(&Default::default());
    let sampler = device.create_sampler(&Default::default());
    let mut params = [0.0_f32; 60];
    params[..2].copy_from_slice(&[size as f32; 2]);
    for index in [4, 8, 12, 24] {
        params[index..index + 4].copy_from_slice(&[0.0, 0.0, size as f32, size as f32]);
    }
    for index in [16, 20] {
        params[index..index + 4].copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    }
    let uniform = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &params
            .iter()
            .flat_map(|value| value.to_le_bytes())
            .collect::<Vec<_>>(),
        usage: wgpu::BufferUsages::UNIFORM,
    });
    let parameters = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
        label: None,
        contents: &words
            .iter()
            .flat_map(|word| word.to_le_bytes())
            .collect::<Vec<_>>(),
        usage: wgpu::BufferUsages::STORAGE,
    });
    let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &pipeline.get_bind_group_layout(0),
        entries: &[
            wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::TextureView(&input_view),
            },
            wgpu::BindGroupEntry {
                binding: 1,
                resource: wgpu::BindingResource::TextureView(&input_view),
            },
            wgpu::BindGroupEntry {
                binding: 2,
                resource: wgpu::BindingResource::Sampler(&sampler),
            },
            wgpu::BindGroupEntry {
                binding: 3,
                resource: uniform.as_entire_binding(),
            },
            wgpu::BindGroupEntry {
                binding: 4,
                resource: parameters.as_entire_binding(),
            },
        ],
    });
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(size) * 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &output_view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        });
        pass.set_pipeline(pipeline);
        pass.set_bind_group(0, &group, &[]);
        pass.draw(0..3, 0..1);
    }
    encoder.copy_texture_to_buffer(
        output.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(256),
                rows_per_image: Some(size),
            },
        },
        extent,
    );
    queue.submit([encoder.finish()]);
    let (tx, rx) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| tx.send(result).unwrap());
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    rx.recv().unwrap().unwrap();
    readback.slice(..).get_mapped_range().unwrap().to_vec()
}
