mod atlas;
mod pipeline;

use std::collections::HashMap;

use argui_core::Size;
use argui_paint::{DisplayCommand, DisplayList, ImageFit, VectorAsset, VectorId, VectorPrimitive};
use atlas::{RasterKey, VectorAtlas};
use pipeline::{VectorInstance, VectorPipeline};

use crate::{RendererError, profile::VectorAtlasStats};

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

#[cfg_attr(coverage_nightly, coverage(off))]
impl VectorGpu {
    pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat) -> Self {
        let pipeline = VectorPipeline::new(device, format);
        let atlas = VectorAtlas::new(device);
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
        Ok(())
    }

    pub fn prepare(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale: f32,
    ) -> Result<(), RendererError> {
        self.clear_frame_stats();
        match self.prepare_once(device, queue, display_list, scale) {
            Err(RendererError::VectorAtlasFull) => {
                self.atlas.clear();
                self.variants.clear();
                self.prepare_once(device, queue, display_list, scale)
            }
            result => result,
        }
    }

    fn prepare_once(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        display_list: &DisplayList,
        scale: f32,
    ) -> Result<(), RendererError> {
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
        self.pipeline.write(device, queue, &instances, &clips);
        Ok(())
    }

    pub fn begin_frame(&mut self) {
        self.pipeline.begin_frame();
    }

    pub fn clear_frame_stats(&mut self) {
        self.hits_this_frame = 0;
        self.rasterizations_this_frame = 0;
    }

    pub fn stats(&self) -> VectorAtlasStats {
        VectorAtlasStats {
            entries: self.atlas.entry_count(),
            hits_this_frame: self.hits_this_frame,
            rasterizations_this_frame: self.rasterizations_this_frame,
            allocated_bytes: self.atlas.allocated_bytes(),
        }
    }

    pub fn target_offset(&mut self, queue: &wgpu::Queue, region: [f32; 4]) -> u32 {
        self.pipeline.target_offset(queue, region)
    }

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
        value.ceil().clamp(1.0, 2040.0) as u32
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

#[cfg(test)]
mod tests {
    use argui_core::{Affine2D, Color, Point, Rect, Size};
    use argui_paint::{ClipChain, ImageFit, VectorId, VectorPrimitive};

    use super::{
        RasterKey, RegisteredVector, choose_variant, pixel_extent, raster_dimensions, rasterize,
        unpremultiply,
    };

    fn primitive(fit: ImageFit) -> VectorPrimitive {
        VectorPrimitive {
            vector: VectorId(1),
            bounds: Rect::new(Point::default(), Size::new(32.0, 16.0)),
            fit,
            color: Color::WHITE,
            opacity: 1.0,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::default(),
        }
    }

    fn registered(svg: &[u8], tintable: bool) -> RegisteredVector {
        let tree = resvg::usvg::Tree::from_data(svg, &resvg::usvg::Options::default()).unwrap();
        RegisteredVector {
            size: Size::new(tree.size().width(), tree.size().height()),
            tree,
            tintable,
        }
    }

    #[test]
    fn raster_size_obeys_fit_and_scale() {
        let source = Size::new(24.0, 24.0);
        assert_eq!(
            raster_dimensions(&primitive(ImageFit::Contain), source, 2.0),
            [32, 32]
        );
        assert_eq!(
            raster_dimensions(&primitive(ImageFit::Cover), source, 2.0),
            [64, 64]
        );
        assert_eq!(
            raster_dimensions(&primitive(ImageFit::Fill), source, 2.0),
            [64, 32]
        );
    }

    #[test]
    fn raster_extent_is_finite_and_bounded() {
        assert_eq!(pixel_extent(f32::NAN), 1);
        assert_eq!(pixel_extent(50_000.0), 2040);
    }

    #[test]
    fn straight_alpha_is_recovered() {
        let mut pixels = [64, 32, 16, 128, 9, 8, 7, 0, 3, 4, 5, 255];
        unpremultiply(&mut pixels);
        assert_eq!(pixels, [128, 64, 32, 128, 0, 0, 0, 0, 3, 4, 5, 255]);
    }

    #[test]
    fn raster_variants_use_quality_hysteresis() {
        let cached = RasterKey {
            id: VectorId(1),
            width: 64,
            height: 64,
        };
        let nearby = RasterKey {
            id: VectorId(1),
            width: 70,
            height: 70,
        };
        let larger = RasterKey {
            id: VectorId(1),
            width: 80,
            height: 80,
        };
        assert_eq!(choose_variant(nearby, &[cached]), cached);
        assert_eq!(choose_variant(larger, &[cached]), larger);
    }

    #[test]
    fn curved_icons_keep_antialiased_coverage_for_gpu_tinting() {
        let icon = registered(
            br#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24"><circle cx="12" cy="12" r="8" fill="currentColor"/></svg>"#,
            true,
        );
        let pixels = rasterize(&icon, 16, 16).unwrap();
        let pixels = pixels.as_chunks::<4>().0;
        assert!(pixels.iter().any(|pixel| pixel[3] == 0));
        assert!(pixels.iter().any(|pixel| pixel[3] == 255));
        assert!(pixels.iter().any(|pixel| (1..255).contains(&pixel[3])));
        assert!(
            pixels
                .iter()
                .filter(|pixel| pixel[3] != 0)
                .all(|pixel| pixel[..3] == [255, 255, 255])
        );
    }

    #[test]
    fn gradients_and_clips_survive_static_svg_rasterization() {
        let asset = registered(
            br##"<svg xmlns="http://www.w3.org/2000/svg" width="16" height="8"><defs><linearGradient id="g"><stop stop-color="#ff0000"/><stop offset="1" stop-color="#0000ff"/></linearGradient><clipPath id="c"><path d="M0 0H12V8H0Z"/></clipPath></defs><path clip-path="url(#c)" fill="url(#g)" d="M0 0H16V8H0Z"/></svg>"##,
            false,
        );
        let pixels = rasterize(&asset, 16, 8).unwrap();
        let left = &pixels[(4 * 4)..(4 * 5)];
        let right = &pixels[(4 * 10)..(4 * 11)];
        let clipped = &pixels[(4 * 15)..(4 * 16)];
        assert!(left[0] > left[2]);
        assert!(right[2] > right[0]);
        assert_eq!(clipped[3], 0);
    }
}
