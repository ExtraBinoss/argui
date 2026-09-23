mod atlas;
mod pipeline;

use std::collections::HashMap;

use argui_core::Size;
use argui_paint::{DisplayCommand, DisplayList, ImageFit, VectorAsset, VectorId, VectorPrimitive};
use atlas::{RasterKey, VectorAtlas};
use pipeline::{VectorInstance, VectorPipeline};

use crate::{RendererError, profile::VectorAtlasStats};

const MAX_ATLAS_SIZE: u32 = 2048;

struct RegisteredVector {
    tree: resvg::usvg::Tree,
    size: Size,
    tintable: bool,
}

pub(crate) struct VectorGpu {
    pipeline: VectorPipeline,
    atlas: VectorAtlas,
    texture_group: wgpu::BindGroup,
    assets: HashMap<VectorId, RegisteredVector>,
    variants: HashMap<VectorId, Vec<RasterKey>>,
    hits_this_frame: usize,
    rasterizations_this_frame: usize,
}

impl VectorGpu {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let pipeline = VectorPipeline::new(device, format);
        let atlas = VectorAtlas::new(device, 256);
        let texture_group = pipeline.texture_group(device, &atlas.view, &atlas.sampler);
        Self {
            pipeline,
            atlas,
            texture_group,
            assets: HashMap::new(),
            variants: HashMap::new(),
            hits_this_frame: 0,
            rasterizations_this_frame: 0,
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn register(&mut self, asset: &VectorAsset) -> Result<(), RendererError> {
        let tree = resvg::usvg::Tree::from_data(&asset.svg, &resvg::usvg::Options::default())
            .map_err(|error| RendererError::VectorRasterization(error.to_string()))?;
        self.assets.insert(
            asset.id,
            RegisteredVector {
                tree,
                size: asset.size,
                tintable: asset.tintable,
            },
        );
        self.variants.remove(&asset.id);
        Ok(())
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale: f32,
    ) -> Result<bool, RendererError> {
        self.clear_frame_stats();
        let mut reset = false;
        let mut changed = false;
        loop {
            match self.prepare_once(device, queue, display_list, scale) {
                Err(RendererError::VectorAtlasFull) if !reset => {
                    if self.atlas.size() < MAX_ATLAS_SIZE {
                        self.atlas = VectorAtlas::new(device, self.atlas.size() * 2);
                        self.texture_group = self.pipeline.texture_group(
                            device,
                            &self.atlas.view,
                            &self.atlas.sampler,
                        );
                    } else {
                        self.atlas.clear();
                        reset = true;
                    }
                    self.variants.clear();
                    changed = true;
                }
                result => return result.map(|updated| updated || changed),
            }
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn prepare_once(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale: f32,
    ) -> Result<bool, RendererError> {
        let vectors = display_list.commands().iter().filter_map(|command| {
            if let DisplayCommand::Vector(vector) = command {
                Some(vector)
            } else {
                None
            }
        });
        let mut instances = Vec::new();
        let mut clips = Vec::new();
        for vector in vectors {
            let asset = self
                .assets
                .get(&vector.vector)
                .ok_or(RendererError::MissingVector(vector.vector.0))?;
            let [width, height] = raster_dimensions(vector, asset.size, scale);
            let requested = RasterKey {
                id: vector.vector,
                width,
                height,
            };
            let key = choose_variant(
                requested,
                self.variants
                    .get(&vector.vector)
                    .map(Vec::as_slice)
                    .unwrap_or_default(),
            );
            let entry = if let Some(entry) = self.atlas.entry(key) {
                self.hits_this_frame += 1;
                entry
            } else {
                if !self.atlas.can_insert(requested) {
                    return Err(RendererError::VectorAtlasFull);
                }
                let pixels = rasterize(asset, width, height)?;
                let entry = self.atlas.insert(queue, requested, &pixels)?;
                self.rasterizations_this_frame += 1;
                self.variants
                    .entry(vector.vector)
                    .or_default()
                    .push(requested);
                entry
            };
            instances.push(VectorInstance::new(
                vector,
                entry,
                asset.tintable,
                scale,
                &mut clips,
            ));
        }
        Ok(self.pipeline.write(device, queue, &instances, &clips))
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn clear_frame_stats(&mut self) {
        self.hits_this_frame = 0;
        self.rasterizations_this_frame = 0;
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn stats(&self) -> VectorAtlasStats {
        VectorAtlasStats {
            entries: self.atlas.entry_count(),
            hits_this_frame: self.hits_this_frame,
            rasterizations_this_frame: self.rasterizations_this_frame,
            allocated_bytes: self.atlas.allocated_bytes(),
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub fn draw<'a>(
        &'a self,
        pass: &mut wgpu::RenderPass<'a>,
        range: std::ops::Range<u32>,
        viewport: u32,
    ) {
        self.pipeline
            .draw(pass, &self.texture_group, range, viewport);
    }
}

fn choose_variant(requested: RasterKey, variants: &[RasterKey]) -> RasterKey {
    variants
        .iter()
        .copied()
        .filter(|variant| {
            let width = requested.width as f32 / variant.width as f32;
            let height = requested.height as f32 / variant.height as f32;
            (0.75..=1.125).contains(&width) && (0.75..=1.125).contains(&height)
        })
        .min_by_key(|variant| {
            variant.width.abs_diff(requested.width) + variant.height.abs_diff(requested.height)
        })
        .unwrap_or(requested)
}

fn raster_dimensions(vector: &VectorPrimitive, source: Size, scale: f32) -> [u32; 2] {
    let matrix = vector.transform.matrix;
    let transform_scale_x = matrix[0].hypot(matrix[1]);
    let transform_scale_y = matrix[2].hypot(matrix[3]);
    let target_width = vector.bounds.size.width.abs() * scale * transform_scale_x;
    let target_height = vector.bounds.size.height.abs() * scale * transform_scale_y;
    let source_width = source.width.max(0.000_01);
    let source_height = source.height.max(0.000_01);
    let (width, height) = match vector.fit {
        ImageFit::Fill => (target_width, target_height),
        ImageFit::Contain => {
            let factor = (target_width / source_width).min(target_height / source_height);
            (source_width * factor, source_height * factor)
        }
        ImageFit::Cover => {
            let factor = (target_width / source_width).max(target_height / source_height);
            (source_width * factor, source_height * factor)
        }
    };
    [pixel_extent(width), pixel_extent(height)]
}

fn pixel_extent(value: f32) -> u32 {
    if value.is_finite() {
        value.ceil().clamp(1.0, (MAX_ATLAS_SIZE - 8) as f32) as u32
    } else {
        1
    }
}

fn rasterize(asset: &RegisteredVector, width: u32, height: u32) -> Result<Vec<u8>, RendererError> {
    let mut pixmap = resvg::tiny_skia::Pixmap::new(width, height).ok_or_else(|| {
        RendererError::VectorRasterization(format!("invalid raster size {width}x{height}"))
    })?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        width as f32 / asset.size.width,
        height as f32 / asset.size.height,
    );
    resvg::render(&asset.tree, transform, &mut pixmap.as_mut());
    let mut pixels = pixmap.take();
    if asset.tintable {
        for pixel in pixels.as_chunks_mut::<4>().0 {
            pixel[..3].fill(255);
        }
    } else {
        unpremultiply(&mut pixels);
    }
    Ok(pixels)
}

fn unpremultiply(pixels: &mut [u8]) {
    for pixel in pixels.as_chunks_mut::<4>().0 {
        let alpha = u32::from(pixel[3]);
        if alpha == 0 {
            pixel[..3].fill(0);
        } else if alpha < 255 {
            for channel in &mut pixel[..3] {
                *channel = ((u32::from(*channel) * 255 + alpha / 2) / alpha).min(255) as u8;
            }
        }
    }
}
