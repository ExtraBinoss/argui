use argui_core::{Point, Rect, Size};

const LAYER_TILE: u32 = 32;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct PixelRegion {
    pub origin: [u32; 2],
    pub size: [u32; 2],
}

impl PixelRegion {
    pub fn viewport(width: u32, height: u32) -> Self {
        Self {
            origin: [0, 0],
            size: [width.max(1), height.max(1)],
        }
    }

    pub fn from_rect(rect: Rect, viewport: Self) -> Option<Self> {
        let clipped_left = rect.origin.x.floor().max(viewport.origin[0] as f32) as u32;
        let clipped_top = rect.origin.y.floor().max(viewport.origin[1] as f32) as u32;
        let clipped_right = (rect.origin.x + rect.size.width)
            .ceil()
            .min(viewport.right() as f32) as u32;
        let clipped_bottom = (rect.origin.y + rect.size.height)
            .ceil()
            .min(viewport.bottom() as f32) as u32;
        if clipped_right > clipped_left && clipped_bottom > clipped_top {
            let left = align_down(clipped_left).max(viewport.origin[0]);
            let top = align_down(clipped_top).max(viewport.origin[1]);
            let right = align_up(clipped_right).min(viewport.right());
            let bottom = align_up(clipped_bottom).min(viewport.bottom());
            Some(Self {
                origin: [left, top],
                size: [right - left, bottom - top],
            })
        } else {
            None
        }
    }

    pub const fn right(self) -> u32 {
        self.origin[0] + self.size[0]
    }

    pub const fn bottom(self) -> u32 {
        self.origin[1] + self.size[1]
    }

    pub fn as_rect(self) -> Rect {
        Rect::new(
            Point::new(self.origin[0] as f32, self.origin[1] as f32),
            Size::new(self.size[0] as f32, self.size[1] as f32),
        )
    }

    pub const fn as_f32(self) -> [f32; 4] {
        [
            self.origin[0] as f32,
            self.origin[1] as f32,
            self.size[0] as f32,
            self.size[1] as f32,
        ]
    }
}

const fn align_down(value: u32) -> u32 {
    value / LAYER_TILE * LAYER_TILE
}

const fn align_up(value: u32) -> u32 {
    value.div_ceil(LAYER_TILE).saturating_mul(LAYER_TILE)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct TextureTarget {
    pub texture: usize,
    pub region: PixelRegion,
    pub extent: [u32; 2],
    pub allocation: [u32; 2],
}

impl TextureTarget {
    pub const fn new(texture: usize, region: PixelRegion, extent: [u32; 2]) -> Self {
        Self {
            texture,
            region,
            extent,
            allocation: extent,
        }
    }

    pub const fn with_allocation(
        texture: usize,
        region: PixelRegion,
        extent: [u32; 2],
        allocation: [u32; 2],
    ) -> Self {
        Self {
            texture,
            region,
            extent,
            allocation,
        }
    }

    pub fn uv_rect(self) -> [f32; 4] {
        [
            0.0,
            0.0,
            self.extent[0] as f32 / self.allocation[0].max(1) as f32,
            self.extent[1] as f32 / self.allocation[1].max(1) as f32,
        ]
    }

    pub fn viewport_for(self, output: PixelRegion) -> [f32; 4] {
        let scale_x = self.extent[0] as f32 / self.region.size[0] as f32;
        let scale_y = self.extent[1] as f32 / self.region.size[1] as f32;
        [
            (output.origin[0] - self.region.origin[0]) as f32 * scale_x,
            (output.origin[1] - self.region.origin[1]) as f32 * scale_y,
            output.size[0] as f32 * scale_x,
            output.size[1] as f32 * scale_y,
        ]
    }
}

#[cfg(test)]
mod tests {
    use argui_core::{Point, Rect, Size};

    use super::{PixelRegion, TextureTarget};

    #[test]
    fn layer_regions_align_outward_and_clip_to_the_parent() {
        let parent = PixelRegion {
            origin: [10, 20],
            size: [100, 80],
        };
        let rect = Rect::new(Point::new(9.5, 22.2), Size::new(40.1, 90.0));
        assert_eq!(
            PixelRegion::from_rect(rect, parent),
            Some(PixelRegion {
                origin: [10, 20],
                size: [54, 80],
            })
        );
        assert_eq!(PixelRegion::from_rect(Rect::default(), parent), None);
    }

    #[test]
    fn nearby_animated_bounds_share_one_offscreen_size_class() {
        let viewport = PixelRegion::viewport(1_000, 700);
        let first = PixelRegion::from_rect(
            Rect::new(Point::new(101.0, 99.0), Size::new(398.0, 402.0)),
            viewport,
        )
        .unwrap();
        let second = PixelRegion::from_rect(
            Rect::new(Point::new(104.0, 102.0), Size::new(392.0, 396.0)),
            viewport,
        )
        .unwrap();
        assert_eq!(first.size, second.size);
    }

    #[test]
    fn logical_regions_map_to_downsampled_texture_viewports() {
        let region = PixelRegion {
            origin: [100, 50],
            size: [200, 100],
        };
        let target = TextureTarget::new(0, region, [100, 50]);
        let output = PixelRegion {
            origin: [120, 70],
            size: [40, 20],
        };
        assert_eq!(target.viewport_for(output), [10.0, 10.0, 20.0, 10.0]);
        assert_eq!(region.as_rect().origin, Point::new(100.0, 50.0));
        let allocated = TextureTarget::with_allocation(0, region, [100, 50], [128, 64]);
        assert_eq!(allocated.uv_rect(), [0.0, 0.0, 0.78125, 0.78125]);
    }
}
