use argui_core::{Point, Rect, Size};
use argui_paint::LayerStyle;

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

    /// Returns the overlapping physical region shared with `other`.
    pub fn intersection(self, other: Self) -> Option<Self> {
        let left = self.origin[0].max(other.origin[0]);
        let top = self.origin[1].max(other.origin[1]);
        let right = self.right().min(other.right());
        let bottom = self.bottom().min(other.bottom());
        if right > left && bottom > top {
            Some(Self {
                origin: [left, top],
                size: [right - left, bottom - top],
            })
        } else {
            None
        }
    }

    /// Returns the smallest bounding physical region containing both `self` and `other`.
    pub fn union(self, other: Self) -> Self {
        let left = self.origin[0].min(other.origin[0]);
        let top = self.origin[1].min(other.origin[1]);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self {
            origin: [left, top],
            size: [right - left, bottom - top],
        }
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

/// Returns the backdrop sampling region, including the full composited output.
///
/// `style` determines the filter sampling margin, `viewport` clips the available scene, and
/// `output` includes foreground filters and shadows. The returned region covers `output`, even
/// when its shadow margin exceeds the backdrop filter margin.
pub(crate) fn backdrop_sample_region(
    style: &LayerStyle,
    viewport: PixelRegion,
    output: PixelRegion,
) -> PixelRegion {
    let expansion: f32 = style
        .backdrop_filters
        .iter()
        .map(|filter| filter.expansion())
        .sum();
    if expansion <= 0.0 {
        return output;
    }
    let bounds = style.bounds;
    let expanded = Rect::new(
        Point::new(bounds.origin.x - expansion, bounds.origin.y - expansion),
        Size::new(
            bounds.size.width + expansion * 2.0,
            bounds.size.height + expansion * 2.0,
        ),
    );
    PixelRegion::from_rect(style.transform.transform_rect(expanded), viewport)
        .map_or(output, |sample| sample.union(output))
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
