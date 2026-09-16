pub(crate) mod pipeline;

use std::collections::HashMap;

use argui_paint::{DisplayCommand, DisplayList, ImageAsset, ImageId, ImageSampling};
use pipeline::{ImageInstance, ImagePipeline};

use crate::RendererError;

struct TextureEntry {
    _texture: wgpu::Texture,
    linear: wgpu::BindGroup,
    nearest: wgpu::BindGroup,
    bytes: usize,
    size: [u32; 2],
}

pub(crate) struct ImageGpu {
    pipeline: ImagePipeline,
    textures: HashMap<ImageId, TextureEntry>,
    capacity: usize,
    used: usize,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ImageGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, capacity: usize) -> Self {
        Self {
            pipeline: ImagePipeline::new(device, format),
            textures: HashMap::new(),
            capacity,
            used: 0,
        }
    }

    pub fn register(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        asset: &ImageAsset,
    ) -> Result<(), RendererError> {
        let previous = self.textures.get(&asset.id).map_or(0, |entry| entry.bytes);
        let requested = self.used - previous + asset.byte_len();
        if requested > self.capacity {
            return Err(RendererError::ImageCacheFull {
                requested,
                capacity: self.capacity,
            });
        }
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("argui-image"),
            size: wgpu::Extent3d {
                width: asset.width,
                height: asset.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &asset.rgba8,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(asset.width * 4),
                rows_per_image: Some(asset.height),
            },
            wgpu::Extent3d {
                width: asset.width,
                height: asset.height,
                depth_or_array_layers: 1,
            },
        );
        let view = texture.create_view(&Default::default());
        let entry = TextureEntry {
            linear: self
                .pipeline
                .texture_group(device, &view, ImageSampling::Linear),
            nearest: self
                .pipeline
                .texture_group(device, &view, ImageSampling::Nearest),
            _texture: texture,
            bytes: asset.byte_len(),
            size: [asset.width, asset.height],
        };
        self.textures.insert(asset.id, entry);
        self.used = requested;
        Ok(())
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale_factor: f32,
    ) -> Result<bool, RendererError> {
        let images = display_list
            .commands()
            .iter()
            .filter_map(|command| match command {
                DisplayCommand::Image(image) => Some(image),
                _ => None,
            });
        let mut instances = Vec::new();
        let mut clips = Vec::new();
        for image in images {
            if !self.textures.contains_key(&image.image) {
                return Err(RendererError::MissingImage(image.image.0));
            }
            let size = self.textures[&image.image].size;
            instances.push(ImageInstance::new(image, size, scale_factor, &mut clips));
        }
        Ok(self.pipeline.write(device, queue, &instances, &clips))
    }

    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    pub fn draw<'pass>(
        &'pass self,
        pass: &mut wgpu::RenderPass<'pass>,
        image: ImageId,
        sampling: ImageSampling,
        instances: std::ops::Range<u32>,
        viewport_offset: u32,
    ) {
        let Some(texture) = self.textures.get(&image) else {
            return;
        };
        let group = match sampling {
            ImageSampling::Linear => &texture.linear,
            ImageSampling::Nearest => &texture.nearest,
        };
        self.pipeline.draw(pass, group, instances, viewport_offset);
    }
}
