use std::collections::HashMap;

use argui_paint::{GpuCanvasId, GpuCanvasPrimitive, ImageSampling, RenderObjectId};

use crate::image::pipeline::ImagePipeline;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct CanvasKey {
    pub canvas: GpuCanvasId,
    pub object: RenderObjectId,
    pub slot: u32,
}

impl From<&GpuCanvasPrimitive> for CanvasKey {
    /// Derives the persistent cache key from a renderer-neutral `primitive`.
    fn from(primitive: &GpuCanvasPrimitive) -> Self {
        Self {
            canvas: primitive.canvas,
            object: primitive.object,
            slot: primitive.slot,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct CanvasExtent {
    pub size: [u32; 2],
    pub bytes: u64,
}

pub(crate) struct CacheEntry {
    _texture: wgpu::Texture,
    view: wgpu::TextureView,
    linear: wgpu::BindGroup,
    nearest: wgpu::BindGroup,
    extent: CanvasExtent,
    last_used: u64,
    attempted_revision: Option<u64>,
    rendered_revision: Option<u64>,
}

impl CacheEntry {
    /// Returns the offscreen view supplied to the application renderer.
    pub fn target(&self) -> &wgpu::TextureView {
        &self.view
    }

    /// Returns the compositor bind group for the requested `sampling` mode.
    pub fn group(&self, sampling: ImageSampling) -> &wgpu::BindGroup {
        match sampling {
            ImageSampling::Nearest => &self.nearest,
            ImageSampling::Linear => &self.linear,
        }
    }

    /// Returns whether `revision` has not yet been attempted for this exact target.
    pub fn needs_render(&self, revision: u64) -> bool {
        self.attempted_revision != Some(revision)
    }

    /// Records `revision` as the valid pixels currently stored in the target.
    pub fn mark_success(&mut self, revision: u64) {
        self.attempted_revision = Some(revision);
        self.rendered_revision = Some(revision);
    }

    /// Records a failed `revision` so an unchanged frame does not retry it.
    pub fn mark_failure(&mut self, revision: u64) {
        self.attempted_revision = Some(revision);
        self.rendered_revision = None;
    }

    /// Returns whether the target contains a successfully rendered revision.
    pub fn valid(&self) -> bool {
        self.rendered_revision.is_some()
    }

    /// Marks the entry as used by `frame` for least-recently-used eviction.
    fn touch(&mut self, frame: u64) {
        self.last_used = frame;
    }
}

pub(crate) struct CanvasCache {
    entries: HashMap<CanvasKey, CacheEntry>,
    budget: u64,
    format: wgpu::TextureFormat,
}

impl CanvasCache {
    /// Creates an empty persistent cache for `format`, bounded to `budget` bytes.
    pub fn new(format: wgpu::TextureFormat, budget: usize) -> Self {
        Self {
            entries: HashMap::new(),
            budget: budget as u64,
            format,
        }
    }

    /// Returns the configured maximum retained texture bytes.
    pub fn budget(&self) -> u64 {
        self.budget
    }

    /// Validates and calculates the exact physical target extent for `primitive`.
    ///
    /// `scale_factor` converts logical bounds to physical pixels and
    /// `max_dimension` enforces the active device limit. A zero-sized viewport
    /// returns `Ok(None)`; invalid or over-budget requests return a message.
    pub fn extent(
        &self,
        primitive: &GpuCanvasPrimitive,
        scale_factor: f32,
        max_dimension: u32,
    ) -> Result<Option<CanvasExtent>, String> {
        if !scale_factor.is_finite() || scale_factor <= 0.0 {
            return Err(format!("invalid window scale factor {scale_factor:?}"));
        }
        if !primitive.resolution_scale.is_finite() || primitive.resolution_scale <= 0.0 {
            return Err(format!(
                "invalid resolution scale {:?}",
                primitive.resolution_scale
            ));
        }
        let logical = primitive.bounds.size;
        if !logical.width.is_finite()
            || !logical.height.is_finite()
            || logical.width < 0.0
            || logical.height < 0.0
        {
            return Err(format!("invalid logical extent {logical:?}"));
        }
        if logical.width == 0.0 || logical.height == 0.0 {
            return Ok(None);
        }
        let scale = f64::from(scale_factor) * f64::from(primitive.resolution_scale);
        let width = f64::from(logical.width).mul_add(scale, 0.0).ceil();
        let height = f64::from(logical.height).mul_add(scale, 0.0).ceil();
        if !width.is_finite()
            || !height.is_finite()
            || width < 1.0
            || height < 1.0
            || width > f64::from(u32::MAX)
            || height > f64::from(u32::MAX)
        {
            return Err(format!(
                "physical extent {width:?} by {height:?} cannot be represented"
            ));
        }
        let size = [width as u32, height as u32];
        if size[0] > max_dimension || size[1] > max_dimension {
            return Err(format!(
                "physical extent {}x{} exceeds max_texture_dimension_2d {max_dimension}",
                size[0], size[1]
            ));
        }
        let bytes_per_block = u64::from(self.format.block_copy_size(None).unwrap_or(4));
        let bytes = u64::from(size[0])
            .checked_mul(u64::from(size[1]))
            .and_then(|pixels| pixels.checked_mul(bytes_per_block))
            .ok_or_else(|| {
                format!(
                    "physical extent {}x{} overflows byte size",
                    size[0], size[1]
                )
            })?;
        if bytes > self.budget {
            return Err(format!(
                "canvas texture needs {bytes} bytes but the cache budget is {} bytes",
                self.budget
            ));
        }
        Ok(Some(CanvasExtent { size, bytes }))
    }

    /// Keeps exact required targets and evicts unused LRU entries until they fit.
    pub fn reserve(&mut self, required: &HashMap<CanvasKey, CanvasExtent>) {
        self.entries.retain(|key, entry| {
            required
                .get(key)
                .is_none_or(|extent| entry.extent == *extent)
        });
        let missing = required
            .iter()
            .filter(|(key, _)| !self.entries.contains_key(key))
            .map(|(_, extent)| extent.bytes)
            .sum::<u64>();
        let mut needed = self.allocated_bytes().saturating_add(missing);
        while needed > self.budget {
            let Some(oldest) = self
                .entries
                .iter()
                .filter(|(key, _)| !required.contains_key(key))
                .min_by_key(|(_, entry)| entry.last_used)
                .map(|(key, _)| *key)
            else {
                break;
            };
            if let Some(entry) = self.entries.remove(&oldest) {
                needed = needed.saturating_sub(entry.extent.bytes);
            }
        }
    }

    /// Returns the exact target for `key`, allocating and touching it when absent.
    pub fn ensure(
        &mut self,
        device: &wgpu::Device,
        pipeline: &ImagePipeline,
        key: CanvasKey,
        extent: CanvasExtent,
        frame: u64,
    ) -> &mut CacheEntry {
        self.entries.entry(key).or_insert_with(|| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some("argui-gpu-canvas-target"),
                size: wgpu::Extent3d {
                    width: extent.size[0],
                    height: extent.size[1],
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: self.format,
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT
                    | wgpu::TextureUsages::TEXTURE_BINDING
                    | wgpu::TextureUsages::COPY_SRC
                    | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
            let linear = pipeline.texture_group(device, &view, ImageSampling::Linear);
            let nearest = pipeline.texture_group(device, &view, ImageSampling::Nearest);
            CacheEntry {
                _texture: texture,
                view,
                linear,
                nearest,
                extent,
                last_used: frame,
                attempted_revision: None,
                rendered_revision: None,
            }
        });
        let entry = self.entries.get_mut(&key).expect("inserted cache entry");
        entry.touch(frame);
        entry
    }

    /// Returns the retained entry for `key`, if one exists.
    pub fn get(&self, key: CanvasKey) -> Option<&CacheEntry> {
        self.entries.get(&key)
    }

    /// Returns the mutable retained entry for `key`, if one exists.
    pub fn get_mut(&mut self, key: CanvasKey) -> Option<&mut CacheEntry> {
        self.entries.get_mut(&key)
    }

    /// Returns the number of retained targets.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns exact texture bytes retained by all entries.
    pub fn allocated_bytes(&self) -> u64 {
        self.entries.values().map(|entry| entry.extent.bytes).sum()
    }
}
