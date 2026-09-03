use argui_core::{Affine2D, Color, Point, Rect, Size};
use argui_paint::{Border, ClipChain, DisplayList, Fill, Quad};

use super::QuadGpu;

const WIDTH: u32 = 4;
const HEIGHT: u32 = 4;
const ROW_BYTES: u32 = 256;

#[test]
fn srgb_targets_encode_authored_colors_and_blend_in_linear_light() {
    pollster::block_on(async {
        let instance = wgpu::Instance::default();
        let Ok(adapter) = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
        else {
            eprintln!("skipping GPU color-space test: no headless adapter");
            return;
        };
        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let gray = render(&device, &queue, [solid(Color::from_srgb8(128, 128, 128))]).await;
        for channel in &gray[..3] {
            assert!(channel.abs_diff(128) <= 1, "gray pixel={gray:?}");
        }
        assert_eq!(gray[3], 255);

        let blended = render(
            &device,
            &queue,
            [solid(Color::WHITE), solid(Color::srgba(0.0, 0.0, 0.0, 0.5))],
        )
        .await;
        for channel in &blended[..3] {
            assert!(channel.abs_diff(188) <= 1, "blended pixel={blended:?}");
        }
        assert_eq!(blended[3], 255);
    });
}

fn solid(color: Color) -> Quad {
    Quad {
        bounds: Rect::new(Point::default(), Size::new(WIDTH as f32, HEIGHT as f32)),
        background: Some(Fill::Solid(color)),
        border: Border::all(0.0, Color::TRANSPARENT),
        radii: Default::default(),
        opacity: 1.0,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
async fn render<const N: usize>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    quads: [Quad; N],
) -> [u8; 4] {
    let descriptor = wgpu::TextureDescriptor {
        label: Some("argui-srgb-pixel-test"),
        size: wgpu::Extent3d {
            width: WIDTH,
            height: HEIGHT,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format: wgpu::TextureFormat::Rgba8UnormSrgb,
        usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
        view_formats: &[],
    };
    let texture = device.create_texture(&descriptor);
    let view = texture.create_view(&Default::default());
    let mut list = DisplayList::new();
    for quad in quads {
        list.push_quad(quad);
    }
    let mut gpu = QuadGpu::new(device, descriptor.format, 64);
    gpu.prepare(device, queue, &list, 1.0).unwrap();
    gpu.begin_frame();
    let viewport = gpu.target_offset(queue, [0.0, 0.0, WIDTH as f32, HEIGHT as f32]);
    let mut encoder = device.create_command_encoder(&Default::default());
    {
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("argui-srgb-test-pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &view,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        gpu.draw(&mut pass, 0..N as u32, viewport);
    }
    let readback = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("argui-srgb-pixel-readback"),
        size: u64::from(ROW_BYTES * HEIGHT),
        usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
        mapped_at_creation: false,
    });
    encoder.copy_texture_to_buffer(
        texture.as_image_copy(),
        wgpu::TexelCopyBufferInfo {
            buffer: &readback,
            layout: wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(ROW_BYTES),
                rows_per_image: Some(HEIGHT),
            },
        },
        descriptor.size,
    );
    queue.submit([encoder.finish()]);
    let (send, receive) = std::sync::mpsc::channel();
    readback
        .slice(..)
        .map_async(wgpu::MapMode::Read, move |result| {
            send.send(result).unwrap();
        });
    device.poll(wgpu::PollType::wait_indefinitely()).unwrap();
    receive.recv().unwrap().unwrap();
    let bytes = readback.slice(..).get_mapped_range().unwrap();
    let offset = ROW_BYTES as usize * 2 + 2 * 4;
    bytes[offset..offset + 4].try_into().unwrap()
}
