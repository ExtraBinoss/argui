#![cfg(feature = "scroll")]
use argui_effects::{EDGE_FADE_ID, EDGE_SHADOW_ID, EdgeFade, EdgeShadow, registry};
use argui_paint::{Color, EffectValue, Filter};
use argui_ui::{ScrollMetric, ScrollMetrics};
use wgpu::util::DeviceExt;

#[test]
fn gpu_edge_shader_preserves_center_alpha_and_scales_fade_width() {
    let instance = wgpu::Instance::default();
    let Ok(adapter) = pollster::block_on(instance.request_adapter(&Default::default())) else {
        eprintln!("No headless GPU adapter; skipping pixel assertions");
        return;
    };
    let (device, queue) = pollster::block_on(adapter.request_device(&Default::default())).unwrap();
    let source = [
        include_str!("../../argui-render/src/shaders/effects/custom_abi_header.wgsl"),
        include_str!("../src/shaders/effects/edges.wgsl"),
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
    for scale in [1.0, 2.0] {
        for (filter, shadow) in [
            (
                EdgeFade::new(8.0).strengths([0.0, 1.0, 0.0, 0.0]).filter(),
                false,
            ),
            (
                EdgeShadow::new(8.0, Color::BLACK)
                    .strengths([0.0, 1.0, 0.0, 0.0])
                    .filter(),
                true,
            ),
        ] {
            let size = (32.0 * scale) as u32;
            let Filter::Effect(effect) = filter.scaled(scale) else {
                panic!()
            };
            let pixels = render_pixels(
                &device,
                &queue,
                &pipeline,
                size,
                &effect.packed_words(),
                [255, 0, 0, 255],
            );
            let pixel = |x: u32, y: u32| {
                &pixels[(y * 256 + x * 4) as usize..(y * 256 + x * 4 + 4) as usize]
            };
            let edge = pixel(size / 2, 0);
            let middle = pixel(size / 2, size / 2);
            assert_eq!(middle, &[255, 0, 0, 255]);
            assert!(edge[0] < 10, "edge should fade or darken: {edge:?}");
            assert_eq!(edge[3], if shadow { 255 } else { edge[0] });
            let half = pixel(size / 2, (4.0 * scale) as u32);
            assert!(half[0] > 110 && half[0] < 160, "half width: {half:?}");
        }
    }
    // The shadow must cover empty gaps as well as opaque content on every axis.
    for edge in 0..4 {
        let mut strengths = [0.0; 4];
        strengths[edge] = 1.0;
        let Filter::Effect(effect) =
            EdgeShadow::new(8.0, Color::srgb(0.0, 0.0, 1.0).with_alpha(0.5))
                .strengths(strengths)
                .filter()
        else {
            panic!()
        };
        for input in [[0, 0, 0, 0], [128, 0, 0, 128], [255, 0, 0, 255]] {
            let pixels = render_pixels(
                &device,
                &queue,
                &pipeline,
                32,
                &effect.packed_words(),
                input,
            );
            let (x, y) = [(0, 16), (16, 0), (31, 16), (16, 31)][edge];
            let pixel = &pixels[y * 256 + x * 4..y * 256 + x * 4 + 4];
            assert!(
                (i16::from(pixel[2]) - 128).abs() <= 2,
                "blue overlay: {pixel:?}"
            );
            assert!((i16::from(pixel[0]) - i16::from(input[0]) / 2).abs() <= 2);
            assert!((i16::from(pixel[3]) - (128 + i16::from(input[3]) / 2)).abs() <= 2);
            assert_eq!(&pixels[16 * 256 + 16 * 4..16 * 256 + 16 * 4 + 4], &input);
        }
    }
}

fn render_pixels(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    pipeline: &wgpu::RenderPipeline,
    size: u32,
    words: &[u32],
    input_pixel: [u8; 4],
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
        &input_pixel.repeat((size * size) as usize),
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

fn metrics() -> ScrollMetrics {
    ScrollMetrics {
        viewport: Default::default(),
        transform: Default::default(),
        offset: Default::default(),
        max_offset: Default::default(),
    }
}

#[test]
fn presets_validate_static_parameters_and_registered_wgsl() {
    let registry = registry().unwrap();
    for (filter, id) in [
        (
            EdgeFade::default()
                .intensity(0.5)
                .strengths([0.0, 0.5, 1.0, 0.0])
                .filter(),
            EDGE_FADE_ID,
        ),
        (
            EdgeShadow::new(16.0, Color::BLACK)
                .intensity(0.2)
                .strengths([1.0; 4])
                .filter(),
            EDGE_SHADOW_ID,
        ),
    ] {
        let Filter::Effect(effect) = filter else {
            panic!()
        };
        assert_eq!(effect.id, id);
        let definition = registry
            .definitions()
            .iter()
            .find(|definition| definition.id == id)
            .unwrap();
        definition.validate().unwrap();
        definition.validate_instance(&effect).unwrap();
        assert_eq!(effect.parameters[0].name, "width");
        assert_eq!(effect.expansion, 0.0);
    }
}

#[test]
fn invalid_width_and_intensity_disable_scroll_layers_and_sanitize_static_filters() {
    for width in [0.0, -2.0, f32::NAN, f32::INFINITY] {
        let fade = EdgeFade::new(width)
            .intensity(f32::NAN)
            .strengths([f32::NAN, -1.0, 3.0, 0.5]);
        let Filter::Effect(filter) = fade.filter() else {
            panic!()
        };
        assert_eq!(filter.parameters[0].value, EffectValue::LogicalPixels(0.0));
        assert_eq!(filter.parameters[1].value, EffectValue::F32(0.0));
        assert_eq!(
            filter.parameters[2].value,
            EffectValue::Vec4([0.0, 0.0, 1.0, 0.5])
        );
        assert!(fade.scroll().resolve(metrics()).is_none());
    }
}

#[test]
fn scroll_bindings_preserve_per_edge_strength_and_use_logical_viewport_sizes() {
    let mut m = metrics();
    m.viewport.size.width = 100.0;
    m.viewport.size.height = 80.0;
    m.max_offset.y = 200.0;
    m.offset.y = 10.0;
    for scroll in [
        EdgeFade::new(20.0)
            .strengths([0.0, 0.5, 0.0, 1.0])
            .scroll_with(4.0, 12.0),
        EdgeShadow::new(20.0, Color::BLACK)
            .strengths([0.0, 0.5, 0.0, 1.0])
            .scroll_with(4.0, 12.0),
    ] {
        let layer = scroll.resolve(m).unwrap();
        let Filter::Effect(filter) = &layer.filters[0] else {
            panic!()
        };
        assert_eq!(
            filter.parameters[2].value,
            EffectValue::Vec4([0.0, 0.25, 0.0, 1.0])
        );
        assert_eq!(
            filter.parameters[5].value,
            m.value(ScrollMetric::ViewportWidth)
        );
        assert_eq!(
            filter.parameters[6].value,
            m.value(ScrollMetric::ViewportHeight)
        );
        let scaled = layer.scaled(2.0);
        let Filter::Effect(filter) = &scaled.filters[0] else {
            panic!()
        };
        assert_eq!(filter.parameters[0].value, EffectValue::LogicalPixels(40.0));
        assert_eq!(
            filter.parameters[5].value,
            EffectValue::LogicalPixels(200.0)
        );
    }
    assert!(EdgeFade::new(0.0).scroll().resolve(m).is_none());
    assert!(
        EdgeFade::new(20.0)
            .intensity(0.0)
            .scroll()
            .resolve(m)
            .is_none()
    );
    assert!(
        EdgeShadow::new(20.0, Color::BLACK)
            .scroll()
            .resolve(m)
            .is_some()
    );
    m.offset.y = 0.0;
    assert!(
        EdgeFade::new(20.0)
            .strengths([0.0, 1.0, 0.0, 0.0])
            .scroll()
            .resolve(m)
            .is_none()
    );
}
