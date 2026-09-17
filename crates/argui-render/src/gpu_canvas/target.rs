/// Clears a dirty canvas target to the defined transparent initial value.
pub(super) fn clear_target(encoder: &mut wgpu::CommandEncoder, target: &wgpu::TextureView) {
    drop(encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
        label: Some("argui-gpu-canvas-clear"),
        color_attachments: &[Some(wgpu::RenderPassColorAttachment {
            view: target,
            depth_slice: None,
            resolve_target: None,
            ops: wgpu::Operations {
                load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
                store: wgpu::StoreOp::Store,
            },
        })],
        ..Default::default()
    }));
}

/// Creates the deterministic checkerboard shown for a failed canvas.
pub(super) fn placeholder_texture(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    format: wgpu::TextureFormat,
) -> wgpu::Texture {
    const SIDE: u32 = 8;
    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some("argui-gpu-canvas-placeholder"),
        size: wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
        mip_level_count: 1,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    });
    let mut pixels = Vec::with_capacity((SIDE * SIDE * 4) as usize);
    for y in 0..SIDE {
        for x in 0..SIDE {
            let pixel = if (x / 2 + y / 2) % 2 == 0 {
                [255, 0, 255, 255]
            } else {
                [28, 28, 32, 255]
            };
            pixels.extend_from_slice(&pixel);
        }
    }
    queue.write_texture(
        wgpu::TexelCopyTextureInfo {
            texture: &texture,
            mip_level: 0,
            origin: wgpu::Origin3d::ZERO,
            aspect: wgpu::TextureAspect::All,
        },
        &pixels,
        wgpu::TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(SIDE * 4),
            rows_per_image: Some(SIDE),
        },
        wgpu::Extent3d {
            width: SIDE,
            height: SIDE,
            depth_or_array_layers: 1,
        },
    );
    texture
}
