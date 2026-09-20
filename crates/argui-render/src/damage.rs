use argui_core::Rect;

use crate::DamageTracking;

mod gpu;
mod scene;

pub(crate) use gpu::DamageGpu;
pub use scene::DamageSnapshot;
pub(crate) use scene::scene_damage;

const REGION_ALIGNMENT: u32 = 32;
const COVERAGE_PADDING: f32 = 2.0;

/// Physical-pixel rectangle whose previous surface contents are stale.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DamageRegion {
    /// Horizontal origin in physical pixels.
    pub x: u32,
    /// Vertical origin in physical pixels.
    pub y: u32,
    /// Width in physical pixels.
    pub width: u32,
    /// Height in physical pixels.
    pub height: u32,
}

impl DamageRegion {
    /// Creates one physical-pixel damage rectangle.
    ///
    /// * `x`, `y` — top-left origin; `width`, `height` — damaged extent.
    #[must_use]
    pub const fn new(x: u32, y: u32, width: u32, height: u32) -> Self {
        Self {
            x,
            y,
            width,
            height,
        }
    }

    /// Returns the rectangle's pixel count.
    #[must_use]
    pub const fn pixels(self) -> u64 {
        self.width as u64 * self.height as u64
    }

    /// Returns the exclusive horizontal end.
    #[must_use]
    pub const fn right(self) -> u32 {
        self.x.saturating_add(self.width)
    }

    /// Returns the exclusive vertical end.
    #[must_use]
    pub const fn bottom(self) -> u32 {
        self.y.saturating_add(self.height)
    }

    /// Converts a floating-point rectangle into an aligned, clipped damage region.
    pub(crate) fn from_rect(rect: Rect, viewport: [u32; 2]) -> Option<Self> {
        let left = (rect.origin.x - COVERAGE_PADDING).floor().max(0.0) as u32;
        let top = (rect.origin.y - COVERAGE_PADDING).floor().max(0.0) as u32;
        let right = (rect.origin.x + rect.size.width + COVERAGE_PADDING)
            .ceil()
            .min(viewport[0] as f32) as u32;
        let bottom = (rect.origin.y + rect.size.height + COVERAGE_PADDING)
            .ceil()
            .min(viewport[1] as f32) as u32;
        if right <= left || bottom <= top {
            return None;
        }
        let left = align_down(left);
        let top = align_down(top);
        let right = align_up(right).min(viewport[0]);
        let bottom = align_up(bottom).min(viewport[1]);
        Some(Self::new(left, top, right - left, bottom - top))
    }

    /// Returns whether this rectangle overlaps or touches `other`.
    fn touches(self, other: Self) -> bool {
        self.x <= other.right()
            && other.x <= self.right()
            && self.y <= other.bottom()
            && other.y <= self.bottom()
    }

    /// Returns the smallest rectangle containing this region and `other`.
    pub(crate) fn union(self, other: Self) -> Self {
        let left = self.x.min(other.x);
        let top = self.y.min(other.y);
        let right = self.right().max(other.right());
        let bottom = self.bottom().max(other.bottom());
        Self::new(left, top, right - left, bottom - top)
    }
}

/// Adaptive decision produced from a set of changed physical regions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DamagePlan {
    /// No retained pixels changed.
    Unchanged,
    /// Only the listed regions need repainting.
    Partial(Vec<DamageRegion>),
    /// The complete viewport should be rendered.
    Full,
}

impl DamagePlan {
    /// Resolves raw regions using the supplied viewport and adaptive thresholds.
    ///
    /// Regions are clipped, aligned to 32-pixel tiles and merged. The result is
    /// [`Self::Full`] when tracking is disabled or its count/area thresholds are exceeded.
    ///
    /// * `regions` — physical-pixel rectangles reported by scene comparison.
    /// * `viewport` — physical width and height.
    /// * `tracking` — adaptive thresholds.
    #[must_use]
    pub fn resolve(
        regions: impl IntoIterator<Item = DamageRegion>,
        viewport: [u32; 2],
        tracking: DamageTracking,
    ) -> Self {
        if !tracking.enabled {
            return Self::Full;
        }
        let mut merged = Vec::new();
        for region in regions {
            let Some(region) = aligned(region, viewport) else {
                continue;
            };
            merge_region(&mut merged, region);
        }
        if merged.is_empty() {
            return Self::Unchanged;
        }
        let viewport_pixels = u64::from(viewport[0]) * u64::from(viewport[1]);
        let damaged_pixels = merged
            .iter()
            .copied()
            .map(DamageRegion::pixels)
            .sum::<u64>();
        let maximum_pixels = (viewport_pixels as f64 * f64::from(tracking.max_area_ratio)) as u64;
        if merged.len() > tracking.max_regions || damaged_pixels > maximum_pixels {
            Self::Full
        } else {
            Self::Partial(merged)
        }
    }
}

/// Surface rendering mode selected for the most recent frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum DamageMode {
    /// The complete scene was rendered directly to the surface.
    #[default]
    Full,
    /// The complete scene initialized the retained target before presentation.
    Seed,
    /// Only changed retained-target regions were repainted.
    Partial,
    /// The unchanged retained target was presented without repainting it.
    Reused,
}

/// Retained-surface damage statistics for one rendered frame.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DamageProfile {
    /// Rendering mode selected for this frame.
    pub mode: DamageMode,
    /// Number of repainted regions; full frames report one.
    pub regions: usize,
    /// Number of retained-target pixels repainted.
    pub damaged_pixels: u64,
    /// Bytes allocated by the retained root texture.
    pub retained_bytes: u64,
}

/// Clips `region` to `viewport`, dropping empty results.
fn clipped(region: DamageRegion, viewport: [u32; 2]) -> Option<DamageRegion> {
    let right = region.right().min(viewport[0]);
    let bottom = region.bottom().min(viewport[1]);
    (region.x < right && region.y < bottom)
        .then(|| DamageRegion::new(region.x, region.y, right - region.x, bottom - region.y))
}

/// Aligns `region` to retained-surface tiles and clips it to `viewport`.
fn aligned(region: DamageRegion, viewport: [u32; 2]) -> Option<DamageRegion> {
    let region = clipped(region, viewport)?;
    clipped(
        DamageRegion::new(
            align_down(region.x),
            align_down(region.y),
            align_up(region.right()).saturating_sub(align_down(region.x)),
            align_up(region.bottom()).saturating_sub(align_down(region.y)),
        ),
        viewport,
    )
}

/// Transitively merges `region` into every touching region in `regions`.
fn merge_region(regions: &mut Vec<DamageRegion>, mut region: DamageRegion) {
    let mut index = 0;
    while index < regions.len() {
        if regions[index].touches(region) {
            region = region.union(regions.swap_remove(index));
            index = 0;
        } else {
            index += 1;
        }
    }
    regions.push(region);
}

/// Aligns `value` down to the retained-surface tile size.
const fn align_down(value: u32) -> u32 {
    value / REGION_ALIGNMENT * REGION_ALIGNMENT
}

/// Aligns `value` up to the retained-surface tile size.
const fn align_up(value: u32) -> u32 {
    value
        .div_ceil(REGION_ALIGNMENT)
        .saturating_mul(REGION_ALIGNMENT)
}
