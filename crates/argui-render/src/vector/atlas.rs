use std::collections::HashMap;

use argui_paint::VectorId;

use crate::RendererError;

const GUTTER: u32 = 2;

#[derive(Clone, Copy, Debug)]
pub(super) struct AtlasEntry {
    pub uv: [f32; 4],
    pub size: [u32; 2],
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(super) struct RasterKey {
    pub id: VectorId,
    pub width: u32,
    pub height: u32,
}

pub(super) struct VectorAtlas {
    texture: wgpu::Texture,
    pub view: wgpu::TextureView,
    pub sampler: wgpu::Sampler,
    entries: HashMap<RasterKey, AtlasEntry>,
    size: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl VectorAtlas {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("argui-vector-atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("argui-vector-atlas-sampler"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });
        Self {
            texture,
            view,
            sampler,
            entries: HashMap::new(),
            size,
            cursor_x: GUTTER,
            cursor_y: GUTTER,
            row_height: 0,
        }
    }

    pub fn entry(&self, key: RasterKey) -> Option<AtlasEntry> {
        self.entries.get(&key).copied()
    }

    /// Reports whether `key` fits at the current shelf cursor without changing atlas state.
    pub fn can_insert(&self, key: RasterKey) -> bool {
        let allocated_width = key.width + GUTTER * 2;
        let allocated_height = key.height + GUTTER * 2;
        if allocated_width + GUTTER > self.size || allocated_height + GUTTER > self.size {
            return false;
        }
        let next_y = if self.cursor_x + allocated_width > self.size {
            self.cursor_y + self.row_height
        } else {
            self.cursor_y
        };
        next_y + allocated_height <= self.size
    }

    /// Uploads `pixels` for `key`, returning its atlas entry or a capacity error.
    pub fn insert(
        &mut self,
        queue: &wgpu::Queue,
        key: RasterKey,
        pixels: &[u8],
    ) -> Result<AtlasEntry, RendererError> {
        if !self.can_insert(key) {
            return Err(RendererError::VectorAtlasFull);
        }
        let allocated_width = key.width + GUTTER * 2;
        let allocated_height = key.height + GUTTER * 2;
        if self.cursor_x + allocated_width > self.size {
            self.cursor_x = GUTTER;
            self.cursor_y += self.row_height;
            self.row_height = 0;
        }
        let padded = pad_pixels(pixels, key.width, key.height);
        let x = self.cursor_x + GUTTER;
        let y = self.cursor_y + GUTTER;
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d {
                    x: self.cursor_x,
                    y: self.cursor_y,
                    z: 0,
                },
                aspect: wgpu::TextureAspect::All,
            },
            &padded,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(allocated_width * 4),
                rows_per_image: Some(allocated_height),
            },
            wgpu::Extent3d {
                width: allocated_width,
                height: allocated_height,
                depth_or_array_layers: 1,
            },
        );
        let divisor = self.size as f32;
        let entry = AtlasEntry {
            uv: [
                x as f32 / divisor,
                y as f32 / divisor,
                (x + key.width) as f32 / divisor,
                (y + key.height) as f32 / divisor,
            ],
            size: [key.width, key.height],
        };
        self.cursor_x += allocated_width;
        self.row_height = self.row_height.max(allocated_height);
        self.entries.insert(key, entry);
        Ok(entry)
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.cursor_x = GUTTER;
        self.cursor_y = GUTTER;
        self.row_height = 0;
    }

    pub fn entry_count(&self) -> usize {
        self.entries.len()
    }

    pub const fn allocated_bytes(&self) -> u64 {
        self.size as u64 * self.size as u64 * 4
    }

    pub const fn size(&self) -> u32 {
        self.size
    }
}

fn pad_pixels(pixels: &[u8], width: u32, height: u32) -> Vec<u8> {
    let padded_width = width + GUTTER * 2;
    let padded_height = height + GUTTER * 2;
    let mut padded = vec![0; (padded_width * padded_height * 4) as usize];
    let source_stride = (width * 4) as usize;
    let destination_stride = (padded_width * 4) as usize;
    for row in 0..height as usize {
        let source = row * source_stride;
        let destination = (row + GUTTER as usize) * destination_stride + (GUTTER * 4) as usize;
        padded[destination..destination + source_stride]
            .copy_from_slice(&pixels[source..source + source_stride]);
    }
    padded
}
