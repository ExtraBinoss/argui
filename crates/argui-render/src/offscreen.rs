use wgpu::{
    Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TexturePoolStats {
    pub textures: usize,
    pub allocated_bytes: u64,
    pub peak_bytes: u64,
    pub reused_this_frame: usize,
}

struct Entry {
    texture: Texture,
    view: TextureView,
    width: u32,
    height: u32,
    bytes: u64,
    used: bool,
    last_frame: u64,
}

pub(crate) struct TexturePool {
    entries: Vec<Entry>,
    format: TextureFormat,
    budget: u64,
    frame: u64,
    peak: u64,
    reused: usize,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl TexturePool {
    pub fn new(format: TextureFormat, budget: u64) -> Self {
        Self {
            entries: Vec::new(),
            format,
            budget,
            frame: 0,
            peak: 0,
            reused: 0,
        }
    }

    pub fn begin_frame(&mut self) {
        self.frame = self.frame.wrapping_add(1);
        self.reused = 0;
        for entry in &mut self.entries {
            entry.used = false;
        }
        while self.allocated_bytes() > self.budget && self.entries.len() > 1 {
            let oldest = self
                .entries
                .iter()
                .enumerate()
                .min_by_key(|(_, entry)| entry.last_frame)
                .map(|(index, _)| index)
                .unwrap_or(0);
            self.entries.swap_remove(oldest);
        }
    }

    pub fn acquire(&mut self, device: &wgpu::Device, width: u32, height: u32) -> usize {
        let width = size_class(width);
        let height = size_class(height);
        if let Some((index, entry)) = self
            .entries
            .iter_mut()
            .enumerate()
            .find(|(_, entry)| !entry.used && entry.width == width && entry.height == height)
        {
            entry.used = true;
            entry.last_frame = self.frame;
            self.reused += 1;
            return index;
        }
        let texture = device.create_texture(&TextureDescriptor {
            label: Some("argui-offscreen"),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::RENDER_ATTACHMENT
                | TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_SRC
                | TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let view = texture.create_view(&TextureViewDescriptor::default());
        let bytes = u64::from(width) * u64::from(height) * 4;
        self.entries.push(Entry {
            texture,
            view,
            width,
            height,
            bytes,
            used: true,
            last_frame: self.frame,
        });
        self.peak = self.peak.max(self.allocated_bytes());
        self.entries.len() - 1
    }

    pub fn texture(&self, index: usize) -> &Texture {
        &self.entries[index].texture
    }

    pub fn view(&self, index: usize) -> &TextureView {
        &self.entries[index].view
    }

    pub fn stats(&self) -> TexturePoolStats {
        TexturePoolStats {
            textures: self.entries.len(),
            allocated_bytes: self.allocated_bytes(),
            peak_bytes: self.peak,
            reused_this_frame: self.reused,
        }
    }

    fn allocated_bytes(&self) -> u64 {
        self.entries.iter().map(|entry| entry.bytes).sum()
    }
}

fn size_class(size: u32) -> u32 {
    size.max(1)
}

#[cfg(test)]
mod tests {
    use super::size_class;

    #[test]
    fn offscreen_dimensions_preserve_exact_viewport_pixels() {
        assert_eq!(size_class(0), 1);
        assert_eq!(size_class(1), 1);
        assert_eq!(size_class(801), 801);
    }
}
