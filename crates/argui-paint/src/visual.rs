use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use argui_core::{Affine2D, Color, Point, Rect};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradientStop {
    pub offset: f32,
    pub color: Color,
}

impl GradientStop {
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
    pub fn new<const N: usize>(stops: [GradientStop; N]) -> Result<Self, GradientError> {
        Self::from_vec(Vec::from(stops))
    }

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

    #[must_use]
    pub fn as_slice(&self) -> &[GradientStop] {
        &self.0
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.0.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct LinearGradient {
    /// Relative coordinates where `(0, 0)` is the top-left of the primitive.
    pub start: Point,
    pub end: Point,
    pub stops: GradientStops,
}

impl LinearGradient {
    pub fn new<const N: usize>(
        start: Point,
        end: Point,
        stops: [GradientStop; N],
    ) -> Result<Self, GradientError> {
        Ok(Self::with_stops(start, end, GradientStops::new(stops)?))
    }

    pub fn with_stops(start: Point, end: Point, stops: GradientStops) -> Self {
        Self { start, end, stops }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct RadialGradient {
    /// Relative center and radii, allowing circular or elliptical gradients.
    pub center: Point,
    pub radius: Point,
    pub stops: GradientStops,
}

impl RadialGradient {
    pub fn new<const N: usize>(
        center: Point,
        radius: Point,
        stops: [GradientStop; N],
    ) -> Result<Self, GradientError> {
        Ok(Self::with_stops(center, radius, GradientStops::new(stops)?))
    }

    pub fn with_stops(center: Point, radius: Point, stops: GradientStops) -> Self {
        Self {
            center,
            radius,
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
}

impl ClipRegion {
    #[must_use]
    pub const fn new(bounds: Rect, transform: Affine2D) -> Self {
        Self { bounds, transform }
    }

    #[must_use]
    pub fn contains(self, point: Point) -> bool {
        self.transform
            .inverse()
            .is_some_and(|inverse| self.bounds.contains(inverse.transform_point(point)))
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct ClipChain(Arc<[ClipRegion]>);

impl ClipChain {
    #[must_use]
    pub fn from_regions(regions: impl Into<Arc<[ClipRegion]>>) -> Self {
        Self(regions.into())
    }

    #[must_use]
    pub fn appended(&self, region: ClipRegion) -> Self {
        let mut regions = self.0.to_vec();
        regions.push(region);
        Self(regions.into())
    }

    #[must_use]
    pub fn regions(&self) -> &[ClipRegion] {
        &self.0
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.0
            .iter()
            .any(|region| region.bounds.size.width <= 0.0 || region.bounds.size.height <= 0.0)
    }

    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        !self.is_empty() && self.0.iter().all(|clip| clip.contains(point))
    }
}
