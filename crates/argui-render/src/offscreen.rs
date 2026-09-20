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

impl Drop for Entry {
    fn drop(&mut self) {
        self.texture.destroy();
    }
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

    pub fn begin_frame(&mut self) -> bool {
        self.frame = self.frame.wrapping_add(1);
        self.reused = 0;
        for entry in &mut self.entries {
            entry.used = false;
        }
        let current_frame = self.frame;
        let initial_count = self.entries.len();
        const MAX_IDLE_FRAMES: u64 = 60;
        self.entries
            .retain(|entry| current_frame.saturating_sub(entry.last_frame) <= MAX_IDLE_FRAMES);
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
        self.entries.len() != initial_count
    }

    /// Drops and destroys every allocated texture in the pool.
    pub fn clear(&mut self) {
        self.entries.clear();
        self.reused = 0;
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
        let target_bytes = texture_bytes(self.format, width, height);
        if let Some((index, entry)) = self
            .entries
            .iter_mut()
            .enumerate()
            .filter(|(_, entry)| {
                !entry.used
                    && entry.width >= width
                    && entry.height >= height
                    && entry.bytes <= target_bytes.saturating_mul(2)
            })
            .min_by_key(|(_, entry)| entry.bytes)
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
        let bytes = texture_bytes(self.format, width, height);
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

    pub fn retain(&mut self, index: usize) -> bool {
        let Some(entry) = self.entries.get_mut(index) else {
            return false;
        };
        if entry.used {
            return false;
        }
        entry.used = true;
        entry.last_frame = self.frame;
        true
    }

    pub fn texture(&self, index: usize) -> &Texture {
        &self.entries[index].texture
    }

    pub fn view(&self, index: usize) -> &TextureView {
        &self.entries[index].view
    }

    pub fn extent(&self, index: usize) -> [u32; 2] {
        [self.entries[index].width, self.entries[index].height]
    }

    /// Returns the bytes allocated by the entry at `index`, or zero when it is absent.
    pub fn bytes(&self, index: usize) -> u64 {
        self.entries.get(index).map_or(0, |entry| entry.bytes)
    }

    pub fn stats(&self) -> TexturePoolStats {
        TexturePoolStats {
            textures: self.entries.len(),
            allocated_bytes: self.allocated_bytes(),
            peak_bytes: self.peak,
            reused_this_frame: self.reused,
        }
    }

    /// Returns pool statistics without counting the entry at `index`.
    ///
    /// A missing `index` leaves the aggregate unchanged. Peak and per-frame
    /// reuse counters still describe the complete pool.
    pub fn stats_excluding(&self, index: usize) -> TexturePoolStats {
        let mut stats = self.stats();
        if let Some(entry) = self.entries.get(index) {
            stats.textures = stats.textures.saturating_sub(1);
            stats.allocated_bytes = stats.allocated_bytes.saturating_sub(entry.bytes);
        }
        stats
    }

    fn allocated_bytes(&self) -> u64 {
        self.entries.iter().map(|entry| entry.bytes).sum()
    }
}

fn size_class(size: u32) -> u32 {
    size.max(64).next_power_of_two()
}

fn texture_bytes(format: TextureFormat, width: u32, height: u32) -> u64 {
    u64::from(width) * u64::from(height) * u64::from(format.block_copy_size(None).unwrap_or(4))
}
