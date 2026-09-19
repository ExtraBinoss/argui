#![cfg(feature = "liquid-glass")]
use argui_effects::{LIQUID_GLASS_ID, LiquidGlass};
use argui_paint::{EffectValue, Filter};
use argui_render::EffectDamage;
use wgpu::util::DeviceExt;

fn effect(glass: LiquidGlass) -> argui_paint::EffectInstance {
    let Filter::Effect(effect) = glass.filter() else {
        panic!("custom effect expected")
    };
    effect
}
#[test]
fn parameters_are_bounded_finite_and_scale_in_logical_pixels() {
    assert_eq!(
        argui_effects::registry()
            .unwrap()
            .get(LIQUID_GLASS_ID)
            .unwrap()
            .damage,
        EffectDamage::Bounded
    );
    let glass = LiquidGlass::default()
        .refraction(f32::NAN)
        .blur(f32::INFINITY)
        .depth_effect(false)
        .brightness(-2.0)
        .contrast(8.0)
        .tint([f32::NAN, -1.0, 2.0, 0.5])
        .saturation(10.0)
        .highlight(-1.0)
        .edge_width(20.0)
        .chromatic_aberration(2.0);
    let effect = effect(glass);
    assert_eq!(effect.id, LIQUID_GLASS_ID);
    let words = effect.packed_words();
    for (index, expected) in [
        (0, 0.0),
        (1, 1.0),
        (2, 0.0),
        (3, 0.0),
        (4, 20.0),
        (5, 4.0),
        (6, -1.0),
        (7, 4.0),
        (9, 0.0),
        (10, 0.0),
        (11, 1.0),
        (12, 0.5),
    ] {
        assert_eq!(f32::from_bits(words[index]), expected);
    }
    assert_eq!(words[8], 0);
    let Filter::Effect(scaled) = Filter::Effect(effect.clone()).scaled(2.0) else {
        panic!()
    };
    assert_eq!(f32::from_bits(scaled.packed_words()[4]), 40.0);
    assert_eq!(
        scaled.packed_words()[1],
        words[1],
        "dispersion is dimensionless"
    );
    assert!(effect.parameters.iter().all(|p| match p.value {
        EffectValue::F32(v) | EffectValue::LogicalPixels(v) => v.is_finite(),
        _ => true,
    }));
    let registry = argui_effects::registry().unwrap();
    let definition = registry
        .definitions()
        .iter()
        .find(|d| d.id == LIQUID_GLASS_ID)
        .unwrap();
    assert_eq!(
        definition.passes.iter().map(|p| p.name).collect::<Vec<_>>(),
        ["blur-x", "blur-y", "glass"]
    );
}

#[test]
fn gpu_lens_matches_circle_profile_and_composes_blur_color_and_dispersion() {
    let instance = wgpu::Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        eprintln!("No headless GPU adapter; skipping pixel assertions");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let registry = argui_effects::registry().unwrap();
    let definition = registry
        .definitions()
        .iter()
        .find(|d| d.id == LIQUID_GLASS_ID)
        .unwrap();
    let pipelines: Vec<_> = definition
        .passes
        .iter()
        .map(|pass| {
            let source = [
                include_str!("../../argui-render/src/shaders/effects/custom_abi_header.wgsl"),
                pass.wgsl,
                include_str!("../../argui-render/src/shaders/effects/custom_abi_footer.wgsl"),
            ]
            .join("\n");
            let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(pass.name),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
                label: Some(pass.name),
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
            })
        })
        .collect();
    let pixels: Vec<u8> = (0..32 * 32)
        .flat_map(|i| {
            let (x, y) = (i % 32, i / 32);
            [
                if (x / 3 + y / 4) % 2 == 0 { 255 } else { 0 },
                (x * 255 / 32) as u8,
                (y * 255 / 32) as u8,
                255,
            ]
        })
        .collect();
    let base = LiquidGlass::new()
        .refraction(0.0)
        .chromatic_aberration(0.0)
        .blur(0.0)
        .highlight(0.0)
        .tint([0.0; 4])
        .saturation(1.0)
        .brightness(0.0)
        .depth_effect(false)
        .edge_width(8.0);
    let render = |glass| {
        render_pixels(
            &device,
            &queue,
            &pipelines,
            &effect(glass).packed_words(),
            &pixels,
            [0.0; 4],
        )
    };
    let identity = render(base);
    for y in 0..32 {
        assert_eq!(
            &identity[y * 256..y * 256 + 128],
            &pixels[y * 128..(y + 1) * 128]
        );
    }
    let optical = render(base.refraction(12.0));
    assert_ne!(identity, optical);
    // At the left edge the analytic Backdrop gradient points inward. Check the
    // circle-map amount independently against the input's horizontal ramp.
    let x = 1.0_f32 - 0.5 / 8.0;
    let displacement = 12.0 * (1.0 - (1.0 - x * x).sqrt());
    let lo = displacement.floor() as usize;
    let t = displacement.fract();
    let expected = (lo * 255 / 32) as f32 * (1.0 - t) + ((lo + 1) * 255 / 32) as f32 * t;
    assert!((f32::from(optical[15 * 256 + 1]) - expected).abs() <= 1.0);
    for y in 10..22 {
        for x in 10..22 {
            let offset = y * 256 + x * 4;
            assert_eq!(
                &identity[offset..offset + 4],
                &optical[offset..offset + 4],
                "flat center stays sharp"
            );
        }
    }
    assert_eq!(identity, render(base.refraction(12.0).edge_width(0.0)));
    assert_ne!(optical, render(base.refraction(12.0).depth_effect(true)));
    assert_ne!(
        optical,
        render(base.refraction(12.0).chromatic_aberration(1.0))
    );
    let blurred = render(base.blur(3.0));
    assert!(blurred[16 * 256 + 16 * 4] > 30 && blurred[16 * 256 + 16 * 4] < 225);
    assert_ne!(identity, render(base.highlight(1.0)));
    assert_ne!(identity, render(base.saturation(0.0)));
    assert_eq!(
        &render(base.tint([0.0, 0.0, 1.0, 1.0]))[16 * 256 + 64..16 * 256 + 68],
        &[0, 0, 255, 255]
    );
    // Constant premultiplied translucent colors must survive all sampling
    // operations, including rounded corners, without changing alpha or hue.
    let translucent = [64, 32, 16, 128].repeat(32 * 32);
    let output = render_pixels(
        &device,
        &queue,
        &pipelines,
        &effect(
            base.refraction(12.0)
                .blur(4.0)
                .depth_effect(true)
                .chromatic_aberration(1.0),
        )
        .packed_words(),
        &translucent,
        [8.0, 4.0, 12.0, 0.0],
    );
    for y in 0..32 {
        for x in 0..32 {
            let offset = y * 256 + x * 4;
            for (actual, expected) in output[offset..offset + 4].iter().zip([64_u8, 32, 16, 128]) {
                assert!(actual.abs_diff(expected) <= 1, "alpha-safe glass sample");
            }
        }
    }
}
fn render_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipelines: &[wgpu::RenderPipeline],
    words: &[u32],
    pixels: &[u8],
    radii: [f32; 4],
) -> Vec<u8> {
    let size = 32;
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
        pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(size * 4),
            rows_per_image: Some(size),
        },
        extent,
    );
    let outputs: Vec<_> = pipelines
        .iter()
        .map(|_| {
            texture(
                wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::TEXTURE_BINDING,
            )
        })
        .collect();
    let mut views = vec![input.create_view(&Default::default())];
    views.extend(outputs.iter().map(|t| t.create_view(&Default::default())));
    let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Linear,
        min_filter: wgpu::FilterMode::Linear,
        ..Default::default()
    });
    let mut params = [0.0_f32; 60];
    params[..2].copy_from_slice(&[size as f32; 2]);
    for index in [4, 8, 12, 24] {
        params[index..index + 4].copy_from_slice(&[0.0, 0.0, size as f32, size as f32]);
    }
    for index in [16, 20] {
        params[index..index + 4].copy_from_slice(&[0.0, 0.0, 1.0, 1.0]);
    }
    params[28..32].copy_from_slice(&radii);
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
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: None,
        size: u64::from(size) * 256,
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    let mut encoder = device.create_command_encoder(&Default::default());
    for (index, pipeline) in pipelines.iter().enumerate() {
        let group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &pipeline.get_bind_group_layout(0),
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&views[index]),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&views[index]),
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
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &views[index + 1],
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
        outputs.last().unwrap().as_image_copy(),
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
