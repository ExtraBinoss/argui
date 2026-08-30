use argui_core::{Point, Rect, Size};

use crate::{LengthPercentageAuto, Sides};

#[derive(Clone, Debug, PartialEq)]
pub struct OverlayAnchor {
    pub key: String,
    pub placement: OverlayPlacement,
}

impl OverlayAnchor {
    #[must_use]
    pub fn new(key: impl Into<String>, placement: OverlayPlacement) -> Self {
        Self {
            key: key.into(),
            placement,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum PlacementSide {
    Top,
    #[default]
    Bottom,
    Left,
    Right,
}

impl PlacementSide {
    const fn opposite(self) -> Self {
        match self {
            Self::Top => Self::Bottom,
            Self::Bottom => Self::Top,
            Self::Left => Self::Right,
            Self::Right => Self::Left,
        }
    }

    const fn perpendicular(self) -> [Self; 2] {
        match self {
            Self::Top | Self::Bottom => [Self::Right, Self::Left],
            Self::Left | Self::Right => [Self::Bottom, Self::Top],
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum OverlayAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct OverlayPlacement {
    pub preferred: PlacementSide,
    pub align: OverlayAlign,
    pub gap: f32,
    pub margin: f32,
}

impl OverlayPlacement {
    #[must_use]
    pub const fn new(preferred: PlacementSide) -> Self {
        Self {
            preferred,
            align: OverlayAlign::Center,
            gap: 8.0,
            margin: 8.0,
        }
    }

    #[must_use]
    pub const fn align(mut self, align: OverlayAlign) -> Self {
        self.align = align;
        self
    }

    #[must_use]
    pub const fn gap(mut self, gap: f32) -> Self {
        self.gap = gap;
        self
    }

    #[must_use]
    pub const fn margin(mut self, margin: f32) -> Self {
        self.margin = margin;
        self
    }

    #[must_use]
    pub fn place(self, viewport: Rect, anchor: Rect, desired: Size) -> PlacedOverlay {
        let inner = inset_rect(viewport, self.margin.max(0.0));
        let perpendicular = self.preferred.perpendicular();
        let candidates = [
            self.preferred,
            self.preferred.opposite(),
            perpendicular[0],
            perpendicular[1],
        ];
        let side = candidates
            .into_iter()
            .find(|side| fits(*side, inner, anchor, desired, self.gap))
            .unwrap_or_else(|| {
                candidates
                    .into_iter()
                    .max_by(|left, right| {
                        available(*left, inner, anchor, self.gap)
                            .total_cmp(&available(*right, inner, anchor, self.gap))
                    })
                    .unwrap_or(self.preferred)
            });
        let max_size = available_size(side, inner, anchor, self.gap);
        let size = Size::new(
            desired.width.min(max_size.width).max(0.0),
            desired.height.min(max_size.height).max(0.0),
        );
        let origin = origin(side, self.align, anchor, size, self.gap);
        let origin = Point::new(
            clamp_axis(origin.x, inner.origin.x, right(inner) - size.width),
            clamp_axis(origin.y, inner.origin.y, bottom(inner) - size.height),
        );
        PlacedOverlay {
            side,
            bounds: Rect::new(origin, size),
            max_size,
        }
    }
}

impl Default for OverlayPlacement {
    fn default() -> Self {
        Self::new(PlacementSide::Bottom)
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedOverlay {
    pub side: PlacementSide,
    pub bounds: Rect,
    pub max_size: Size,
}

impl PlacedOverlay {
    #[must_use]
    pub fn inset_from(self, containing_block: Rect) -> Sides<LengthPercentageAuto> {
        Sides {
            left: LengthPercentageAuto::length(self.bounds.origin.x - containing_block.origin.x),
            right: LengthPercentageAuto::auto(),
            top: LengthPercentageAuto::length(self.bounds.origin.y - containing_block.origin.y),
            bottom: LengthPercentageAuto::auto(),
        }
    }
}

fn fits(side: PlacementSide, viewport: Rect, anchor: Rect, desired: Size, gap: f32) -> bool {
    let space = available_size(side, viewport, anchor, gap);
    desired.width <= space.width && desired.height <= space.height
}

fn available(side: PlacementSide, viewport: Rect, anchor: Rect, gap: f32) -> f32 {
    let size = available_size(side, viewport, anchor, gap);
    match side {
        PlacementSide::Top | PlacementSide::Bottom => size.height,
        PlacementSide::Left | PlacementSide::Right => size.width,
    }
}

fn available_size(side: PlacementSide, viewport: Rect, anchor: Rect, gap: f32) -> Size {
    let width = viewport.size.width.max(0.0);
    let height = viewport.size.height.max(0.0);
    match side {
        PlacementSide::Top => Size::new(
            width,
            (anchor.origin.y - viewport.origin.y - gap).clamp(0.0, height),
        ),
        PlacementSide::Bottom => Size::new(
            width,
            (bottom(viewport) - bottom(anchor) - gap).clamp(0.0, height),
        ),
        PlacementSide::Left => Size::new(
            (anchor.origin.x - viewport.origin.x - gap).clamp(0.0, width),
            height,
        ),
        PlacementSide::Right => Size::new(
            (right(viewport) - right(anchor) - gap).clamp(0.0, width),
            height,
        ),
    }
}

fn origin(side: PlacementSide, align: OverlayAlign, anchor: Rect, size: Size, gap: f32) -> Point {
    let aligned_x = align_axis(anchor.origin.x, anchor.size.width, size.width, align);
    let aligned_y = align_axis(anchor.origin.y, anchor.size.height, size.height, align);
    match side {
        PlacementSide::Top => Point::new(aligned_x, anchor.origin.y - gap - size.height),
        PlacementSide::Bottom => Point::new(aligned_x, bottom(anchor) + gap),
        PlacementSide::Left => Point::new(anchor.origin.x - gap - size.width, aligned_y),
        PlacementSide::Right => Point::new(right(anchor) + gap, aligned_y),
    }
}

fn align_axis(origin: f32, anchor: f32, overlay: f32, align: OverlayAlign) -> f32 {
    match align {
        OverlayAlign::Start => origin,
        OverlayAlign::Center => origin + (anchor - overlay) * 0.5,
        OverlayAlign::End => origin + anchor - overlay,
    }
}

fn clamp_axis(value: f32, minimum: f32, maximum: f32) -> f32 {
    // Reassociating `(origin + size) - size` can land a few ULPs below
    // `origin`. `f32::clamp` treats that harmless rounding error as an invalid
    // interval and panics, so collapse a transiently inverted interval first.
    value.max(minimum).min(maximum.max(minimum))
}

fn inset_rect(rect: Rect, margin: f32) -> Rect {
    let width = rect.size.width.max(0.0);
    let height = rect.size.height.max(0.0);
    let horizontal = margin.min(width * 0.5);
    let vertical = margin.min(height * 0.5);
    Rect::new(
        Point::new(rect.origin.x + horizontal, rect.origin.y + vertical),
        Size::new(
            (width - horizontal * 2.0).max(0.0),
            (height - vertical * 2.0).max(0.0),
        ),
    )
}

fn right(rect: Rect) -> f32 {
    rect.origin.x + rect.size.width
}

fn bottom(rect: Rect) -> f32 {
    rect.origin.y + rect.size.height
}
