use std::collections::HashMap;

use argui_text::{GlyphContent, GlyphKey, TextEngine};

use crate::RendererError;

#[derive(Clone, Copy, Debug)]
pub(super) struct AtlasEntry {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
    pub left: i32,
    pub top: i32,
    pub color: bool,
}

pub(super) struct GlyphAtlas {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    sampler: wgpu::Sampler,
    entries: HashMap<GlyphKey, Option<AtlasEntry>>,
    size: u32,
    cursor_x: u32,
    cursor_y: u32,
    row_height: u32,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl GlyphAtlas {
    pub fn new(device: &wgpu::Device, size: u32) -> Self {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("argui-glyph-atlas"),
            size: wgpu::Extent3d {
                width: size,
                height: size,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("argui-glyph-sampler"),
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
            cursor_x: 1,
            cursor_y: 1,
            row_height: 0,
        }
    }

    pub const fn size(&self) -> u32 {
        self.size
    }

    pub const fn view(&self) -> &wgpu::TextureView {
        &self.view
    }

    pub const fn sampler(&self) -> &wgpu::Sampler {
        &self.sampler
    }

    pub fn get_or_insert(
        &mut self,
        queue: &wgpu::Queue,
        engine: &mut TextEngine,
        key: GlyphKey,
    ) -> Result<Option<AtlasEntry>, RendererError> {
        if let Some(entry) = self.entries.get(&key) {
            return Ok(*entry);
        }
        let Some(image) = engine.rasterize(key) else {
            self.entries.insert(key, None);
            return Ok(None);
        };
        if image.width == 0 || image.height == 0 {
            self.entries.insert(key, None);
            return Ok(None);
        }
        let (x, y) = self.allocate(image.width, image.height)?;
        let pixels = rgba_pixels(image.content, &image.data);
        queue.write_texture(
            wgpu::TexelCopyTextureInfo {
                texture: &self.texture,
                mip_level: 0,
                origin: wgpu::Origin3d { x, y, z: 0 },
                aspect: wgpu::TextureAspect::All,
            },
            &pixels,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(image.width * 4),
                rows_per_image: Some(image.height),
            },
            wgpu::Extent3d {
                width: image.width,
                height: image.height,
                depth_or_array_layers: 1,
            },
        );
        let entry = AtlasEntry {
            x,
            y,
            width: image.width,
            height: image.height,
            left: image.left,
            top: image.top,
            color: image.content == GlyphContent::Color,
        };
        self.entries.insert(key, Some(entry));
        Ok(Some(entry))
    }

    pub fn reset(&mut self) {
        self.entries.clear();
        self.cursor_x = 1;
        self.cursor_y = 1;
        self.row_height = 0;
    }

    fn allocate(&mut self, width: u32, height: u32) -> Result<(u32, u32), RendererError> {
        if self.cursor_x + width + 1 > self.size {
            self.cursor_x = 1;
            self.cursor_y += self.row_height + 1;
            self.row_height = 0;
        }
        if self.cursor_y + height + 1 > self.size {
            return Err(RendererError::GlyphAtlasFull);
        }
        let position = (self.cursor_x, self.cursor_y);
        self.cursor_x += width + 1;
        self.row_height = self.row_height.max(height);
        Ok(position)
    }
}

fn rgba_pixels(content: GlyphContent, data: &[u8]) -> Vec<u8> {
    match content {
        GlyphContent::Color => data.to_vec(),
        GlyphContent::Mask => data
            .iter()
            .flat_map(|alpha| [255, 255, 255, *alpha])
            .collect(),
        GlyphContent::SubpixelMask => data
            .as_chunks::<3>()
            .0
            .iter()
            .flat_map(|rgb| [255, 255, 255, *rgb.iter().max().unwrap_or(&0)])
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use argui_text::GlyphContent;

    use super::rgba_pixels;

    #[test]
    fn expands_masks_to_rgba() {
        assert_eq!(
            rgba_pixels(GlyphContent::Mask, &[0, 128]),
            [255, 255, 255, 0, 255, 255, 255, 128]
        );
        assert_eq!(
            rgba_pixels(GlyphContent::Color, &[1, 2, 3, 4]),
            [1, 2, 3, 4]
        );
        assert_eq!(
            rgba_pixels(GlyphContent::SubpixelMask, &[1, 9, 3]),
            [255, 255, 255, 9]
        );
    }
}
