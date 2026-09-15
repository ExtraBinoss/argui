use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use argui_core::{Affine2D, Color, ColorInterpolation, Point, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

impl GradientStop {
    /// Creates a gradient stop at normalized `offset` with `color`.
    #[must_use]
    pub const fn new(offset: f32, color: Color) -> Self {
        Self { offset, color }
    }
}

impl Default for GradientStop {
    fn default() -> Self {
        Self::new(0.0, Color::TRANSPARENT)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum GradientError {
    TooFewStops,
    InvalidOffset,
    UnsortedStops,
}

impl core::fmt::Display for GradientError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "invalid gradient: {self:?}")
    }
}

impl std::error::Error for GradientError {}

#[derive(Clone, Debug, PartialEq)]
pub struct GradientStops(Arc<[GradientStop]>);

impl GradientStops {
    /// Creates validated stops from a fixed-size array.
    ///
    /// # Errors
    /// Returns a gradient error if there are fewer than two stops, offsets are
    /// outside zero through one, or the stops are not sorted.
    pub fn new<const N: usize>(stops: [GradientStop; N]) -> Result<Self, GradientError> {
        Self::from_vec(Vec::from(stops))
    }

    /// Creates validated stops from a vector.
    ///
    /// # Errors
    /// Returns a gradient error if there are fewer than two stops, offsets are
    /// outside zero through one, or the stops are not sorted.
    pub fn from_vec(stops: Vec<GradientStop>) -> Result<Self, GradientError> {
        if stops.len() < 2 {
            return Err(GradientError::TooFewStops);
        }
        let mut previous = 0.0;
        for (index, stop) in stops.iter().enumerate() {
            if !(0.0..=1.0).contains(&stop.offset) {
                return Err(GradientError::InvalidOffset);
            }
            if index != 0 && stop.offset < previous {
                return Err(GradientError::UnsortedStops);
            }
            previous = stop.offset;
        }
        Ok(Self(stops.into()))
    }

    /// Returns the stops in their validated order.
    #[must_use]
    pub fn as_slice(&self) -> &[GradientStop] {
        &self.0
    }

    /// Returns the number of stops.
    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns whether the collection contains no stops.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BilinearGradient {
    /// Interpolation space shared by both axes, with premultiplied alpha.
    pub interpolation: ColorInterpolation,
    corners: Arc<[Color; 4]>,
}

impl BilinearGradient {
    /// Corners in top-left, top-right, bottom-left, bottom-right order.
    /// Creates a bilinear gradient from top-left, top-right, bottom-left, bottom-right colors.
    /// * `corners` — colors in top-left, top-right, bottom-left, bottom-right order; `interpolation` — color space.
    #[must_use]
    pub fn new(corners: [Color; 4], interpolation: ColorInterpolation) -> Self {
        Self {
            interpolation,
            corners: Arc::new(corners),
        }
    }

    /// Returns the four corner colors in construction order.
    #[must_use]
    pub fn corners(&self) -> &[Color; 4] {
        &self.corners
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Relative coordinates where `(0, 0)` is the top-left of the primitive.
    pub start: Point,
    pub end: Point,
    pub interpolation: ColorInterpolation,
    pub stops: GradientStops,
}

impl LinearGradient {
    /// Creates a linear gradient from endpoints, color space, and validated stops.
    /// * `start`, `end` — gradient endpoints; `interpolation` — color space; `stops` — stop array.
    ///
    /// # Errors
    /// Returns a gradient error if the stop array is invalid.
    pub fn new<const N: usize>(
        start: Point,
        end: Point,
        interpolation: ColorInterpolation,
        stops: [GradientStop; N],
    ) -> Result<Self, GradientError> {
        Ok(Self::with_stops(
            start,
            end,
            interpolation,
            GradientStops::new(stops)?,
        ))
    }

    /// Creates a linear gradient using an existing validated stop collection.
    /// * `start`, `end` — gradient endpoints; `interpolation` — color space; `stops` — color stops.
    pub fn with_stops(
        start: Point,
        end: Point,
        interpolation: ColorInterpolation,
        stops: GradientStops,
    ) -> Self {
        Self {
            start,
            end,
            interpolation,
            stops,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    /// Relative center and radii, allowing circular or elliptical gradients.
    pub center: Point,
    pub radius: Point,
    pub interpolation: ColorInterpolation,
    pub stops: GradientStops,
}

impl RadialGradient {
    /// Creates an elliptical radial gradient from center, radii, color space, and stops.
    /// * `center` — gradient center; `radius` — horizontal and vertical radii; `interpolation` — color space.
    ///
    /// # Errors
    /// Returns a gradient error if the stop array is invalid.
    pub fn new<const N: usize>(
        center: Point,
        radius: Point,
        interpolation: ColorInterpolation,
        stops: [GradientStop; N],
    ) -> Result<Self, GradientError> {
        Ok(Self::with_stops(
            center,
            radius,
            interpolation,
            GradientStops::new(stops)?,
        ))
    }

    /// Creates a radial gradient using an existing validated stop collection.
    /// * `center` — gradient center; `radius` — horizontal and vertical radii; `interpolation` — color space; `stops` — color stops.
    pub fn with_stops(
        center: Point,
        radius: Point,
        interpolation: ColorInterpolation,
        stops: GradientStops,
    ) -> Self {
        Self {
            center,
            radius,
            interpolation,
            stops,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct ImageId(pub u64);

static NEXT_IMAGE_ID: AtomicU64 = AtomicU64::new(1);

impl ImageId {
    /// Allocates a process-local opaque handle. Asset libraries normally call
    /// this on behalf of applications.
    ///
    /// # Panics
    /// Panics if the process-local image handle space is exhausted.
    #[must_use]
    pub fn fresh() -> Self {
        let id = NEXT_IMAGE_ID.fetch_add(1, Ordering::Relaxed);
        assert_ne!(id, u64::MAX, "image handle space exhausted");
        Self(id)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ImageAsset {
    pub id: ImageId,
    pub width: u32,
    pub height: u32,
    pub rgba8: Arc<[u8]>,
}

impl ImageAsset {
    /// Creates an RGBA8 image asset after checking its dimensions and byte length.
    ///
    /// # Errors
    /// Returns [`ImageAssetError::InvalidDimensions`] if the byte length cannot be
    /// represented, or [`ImageAssetError::InvalidByteLength`] if the buffer does not
    /// contain exactly four bytes per pixel.
    /// * `id` — image identity; `width`, `height` — pixel dimensions; `rgba8` — row-major RGBA bytes.
    pub fn rgba8(
        id: ImageId,
        width: u32,
        height: u32,
        rgba8: impl Into<Arc<[u8]>>,
    ) -> Result<Self, ImageAssetError> {
        let rgba8 = rgba8.into();
        let expected = usize::try_from(width)
            .ok()
            .and_then(|width| usize::try_from(height).ok().map(|height| width * height))
            .and_then(|pixels| pixels.checked_mul(4))
            .ok_or(ImageAssetError::InvalidDimensions)?;
        if rgba8.len() != expected {
            return Err(ImageAssetError::InvalidByteLength {
                expected,
                actual: rgba8.len(),
            });
        }
        Ok(Self {
            id,
            width,
            height,
            rgba8,
        })
    }

    /// Returns the number of bytes in the pixel buffer.
    #[must_use]
    pub fn byte_len(&self) -> usize {
        self.rgba8.len()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ImageAssetError {
    InvalidDimensions,
    InvalidByteLength { expected: usize, actual: usize },
}

impl core::fmt::Display for ImageAssetError {
    fn fmt(&self, formatter: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(formatter, "invalid RGBA image: {self:?}")
    }
}

impl std::error::Error for ImageAssetError {}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ImageFit {
    Fill,
    Contain,
    #[default]
    Cover,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ImageSampling {
    Nearest,
    #[default]
    Linear,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ClipRegion {
    pub bounds: Rect,
    pub transform: Affine2D,
    pub radii: crate::CornerRadii,
}

impl ClipRegion {
    /// Creates an unrounded clip region in the supplied transform.
    /// * `bounds` — local clipping rectangle; `transform` — transform into surface coordinates.
    #[must_use]
    pub const fn new(bounds: Rect, transform: Affine2D) -> Self {
        Self {
            bounds,
            transform,
            radii: crate::CornerRadii::all(0.0),
        }
    }

    /// Creates a clip region with per-corner radii.
    /// * `bounds` — local clipping rectangle; `transform` — surface transform; `radii` — corner radii.
    #[must_use]
    pub const fn rounded(bounds: Rect, transform: Affine2D, radii: crate::CornerRadii) -> Self {
        Self {
            bounds,
            transform,
            radii,
        }
    }

    /// Returns whether `point` lies inside the transformed rounded region.
    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        self.transform.inverse().is_some_and(|inverse| {
            let point = inverse.transform_point(point);
            self.bounds.contains(point) && rounded_rect_contains(self.bounds, self.radii, point)
        })
    }
}

fn rounded_rect_contains(bounds: Rect, radii: crate::CornerRadii, point: Point) -> bool {
    let local = Point::new(point.x - bounds.origin.x, point.y - bounds.origin.y);
    let width = bounds.size.width.max(0.0);
    let height = bounds.size.height.max(0.0);
    let radius = if local.y < height * 0.5 {
        if local.x < width * 0.5 {
            radii.top_left
        } else {
            radii.top_right
        }
    } else if local.x < width * 0.5 {
        radii.bottom_left
    } else {
        radii.bottom_right
    }
    .clamp(0.0, width.min(height) * 0.5);
    let center = Point::new(
        local.x.clamp(radius, width - radius),
        local.y.clamp(radius, height - radius),
    );
    let delta = Point::new(local.x - center.x, local.y - center.y);
    delta.x * delta.x + delta.y * delta.y <= radius * radius
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipChain(Arc<[ClipRegion]>);

impl ClipChain {
    /// Creates a clip chain from an existing shared region collection.
    /// * `regions` — clip regions to retain in the chain.
    #[must_use]
    pub fn from_regions(regions: impl Into<Arc<[ClipRegion]>>) -> Self {
        Self(regions.into())
    }

    /// Returns a copy of the chain with `region` appended.
    #[must_use]
    pub fn appended(&self, region: ClipRegion) -> Self {
        let mut regions = self.0.to_vec();
        regions.push(region);
        Self(regions.into())
    }

    /// Returns the regions in clipping order.
    #[must_use]
    pub fn regions(&self) -> &[ClipRegion] {
        &self.0
    }

    /// Returns whether any region has an empty or non-positive extent.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0
            .iter()
            .any(|region| region.bounds.size.width <= 0.0 || region.bounds.size.height <= 0.0)
    }

    /// Returns whether `point` lies within every region in the chain.
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        !self.is_empty() && self.0.iter().all(|clip| clip.contains(point))
    }
}
