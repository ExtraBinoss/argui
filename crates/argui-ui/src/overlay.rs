use argui_core::{Point, Rect, Size};

use crate::{LengthPercentageAuto, Sides, WritingDirection};

mod portal;
pub use portal::{DismissPolicy, OverlaySurface, Portal, WindowLayer};

#[derive(Clone, Debug, Default, PartialEq)]
pub enum PortalTarget {
    #[default]
    Layout,
    Anchor(AnchorPortal),
    Rect {
        bounds: Rect,
        placement: FloatingPlacement,
    },
    Viewport(ViewportPlacement),
}

#[derive(Clone, Debug, PartialEq)]
pub struct AnchorPortal {
    pub key: String,
    pub placement: FloatingPlacement,
}

impl AnchorPortal {
    #[must_use]
    pub fn new(key: impl Into<String>, placement: FloatingPlacement) -> Self {
        Self {
            key: key.into(),
            placement,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Placement {
    TopStart,
    Top,
    TopEnd,
    #[default]
    BottomStart,
    Bottom,
    BottomEnd,
    LeftStart,
    Left,
    LeftEnd,
    RightStart,
    Right,
    RightEnd,
}

impl Placement {
    const fn side(self) -> Side {
        match self {
            Self::TopStart | Self::Top | Self::TopEnd => Side::Top,
            Self::BottomStart | Self::Bottom | Self::BottomEnd => Side::Bottom,
            Self::LeftStart | Self::Left | Self::LeftEnd => Side::Left,
            Self::RightStart | Self::Right | Self::RightEnd => Side::Right,
        }
    }

    const fn alignment(self) -> Alignment {
        match self {
            Self::TopStart | Self::BottomStart | Self::LeftStart | Self::RightStart => {
                Alignment::Start
            }
            Self::Top | Self::Bottom | Self::Left | Self::Right => Alignment::Center,
            Self::TopEnd | Self::BottomEnd | Self::LeftEnd | Self::RightEnd => Alignment::End,
        }
    }

    const fn with(self, side: Side) -> Self {
        match (side, self.alignment()) {
            (Side::Top, Alignment::Start) => Self::TopStart,
            (Side::Top, Alignment::Center) => Self::Top,
            (Side::Top, Alignment::End) => Self::TopEnd,
            (Side::Bottom, Alignment::Start) => Self::BottomStart,
            (Side::Bottom, Alignment::Center) => Self::Bottom,
            (Side::Bottom, Alignment::End) => Self::BottomEnd,
            (Side::Left, Alignment::Start) => Self::LeftStart,
            (Side::Left, Alignment::Center) => Self::Left,
            (Side::Left, Alignment::End) => Self::LeftEnd,
            (Side::Right, Alignment::Start) => Self::RightStart,
            (Side::Right, Alignment::Center) => Self::Right,
            (Side::Right, Alignment::End) => Self::RightEnd,
        }
    }

    const fn opposite(self) -> Self {
        self.with(self.side().opposite())
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Side {
    Top,
    Bottom,
    Left,
    Right,
}

impl Side {
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Alignment {
    Start,
    Center,
    End,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum AnchorWidth {
    #[default]
    Content,
    AtLeastAnchor,
    MatchAnchor,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CollisionPolicy {
    pub flip: bool,
    pub shift: bool,
    pub constrain: bool,
}

impl CollisionPolicy {
    pub const NONE: Self = Self {
        flip: false,
        shift: false,
        constrain: false,
    };
    pub const FIT_VIEWPORT: Self = Self {
        flip: true,
        shift: true,
        constrain: true,
    };
}

impl Default for CollisionPolicy {
    fn default() -> Self {
        Self::FIT_VIEWPORT
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FloatingPlacement {
    pub preferred: Placement,
    pub offset: f32,
    pub cross_offset: f32,
    pub viewport_padding: f32,
    pub anchor_width: AnchorWidth,
    pub collision: CollisionPolicy,
}

impl FloatingPlacement {
    #[must_use]
    pub const fn new(preferred: Placement) -> Self {
        Self {
            preferred,
            offset: 6.0,
            cross_offset: 0.0,
            viewport_padding: 8.0,
            anchor_width: AnchorWidth::Content,
            collision: CollisionPolicy::FIT_VIEWPORT,
        }
    }

    #[must_use]
    pub const fn offset(mut self, offset: f32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub const fn cross_offset(mut self, offset: f32) -> Self {
        self.cross_offset = offset;
        self
    }

    #[must_use]
    pub const fn viewport_padding(mut self, padding: f32) -> Self {
        self.viewport_padding = padding;
        self
    }

    #[must_use]
    pub const fn anchor_width(mut self, width: AnchorWidth) -> Self {
        self.anchor_width = width;
        self
    }

    #[must_use]
    pub const fn collision(mut self, collision: CollisionPolicy) -> Self {
        self.collision = collision;
        self
    }

    #[must_use]
    pub fn place(
        self,
        viewport: Rect,
        anchor: Rect,
        desired: Size,
        direction: WritingDirection,
    ) -> PlacedOverlay {
        let viewport = finite_rect(viewport);
        let anchor = finite_rect(anchor);
        let inner = inset_rect(viewport, finite_non_negative(self.viewport_padding));
        let offset = finite(self.offset);
        let desired = self.apply_anchor_width(finite_size(desired), anchor.size);
        let placement = self.choose_placement(inner, anchor, desired, offset);
        let available = available_size(placement.side(), inner, anchor, offset);
        let size = if self.collision.constrain {
            Size::new(
                desired.width.min(available.width),
                desired.height.min(available.height),
            )
        } else {
            desired
        };
        let mut point = origin(
            placement,
            anchor,
            size,
            offset,
            finite(self.cross_offset),
            direction,
        );
        if self.collision.shift {
            point.x = clamp_axis(point.x, inner.origin.x, right(inner) - size.width);
            point.y = clamp_axis(point.y, inner.origin.y, bottom(inner) - size.height);
        }
        PlacedOverlay {
            placement,
            bounds: Rect::new(point, size),
            available_size: available,
            constrained_width: size.width < desired.width,
            constrained_height: size.height < desired.height,
        }
    }

    fn apply_anchor_width(self, desired: Size, anchor: Size) -> Size {
        match self.anchor_width {
            AnchorWidth::Content => desired,
            AnchorWidth::AtLeastAnchor => {
                Size::new(desired.width.max(anchor.width), desired.height)
            }
            AnchorWidth::MatchAnchor => Size::new(anchor.width, desired.height),
        }
    }

    fn choose_placement(
        self,
        viewport: Rect,
        anchor: Rect,
        desired: Size,
        offset: f32,
    ) -> Placement {
        if !self.collision.flip || fits(self.preferred, viewport, anchor, desired, offset) {
            return self.preferred;
        }
        let perpendicular = self.preferred.side().perpendicular();
        let candidates = [
            self.preferred.opposite(),
            self.preferred.with(perpendicular[0]),
            self.preferred.with(perpendicular[1]),
        ];
        candidates
            .into_iter()
            .find(|candidate| fits(*candidate, viewport, anchor, desired, offset))
            .unwrap_or_else(|| {
                candidates
                    .into_iter()
                    .chain([self.preferred])
                    .max_by(|left, right| {
                        available_main(*left, viewport, anchor, offset)
                            .total_cmp(&available_main(*right, viewport, anchor, offset))
                    })
                    .unwrap_or(self.preferred)
            })
    }
}

impl Default for FloatingPlacement {
    fn default() -> Self {
        Self::new(Placement::BottomStart)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ViewportAlign {
    Start,
    #[default]
    Center,
    End,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ViewportPlacement {
    Fill {
        margin: f32,
    },
    Positioned {
        horizontal: ViewportAlign,
        vertical: ViewportAlign,
        margin: f32,
    },
}

impl ViewportPlacement {
    #[must_use]
    pub const fn fill() -> Self {
        Self::Fill { margin: 0.0 }
    }

    #[must_use]
    pub const fn centered() -> Self {
        Self::Positioned {
            horizontal: ViewportAlign::Center,
            vertical: ViewportAlign::Center,
            margin: 16.0,
        }
    }

    #[must_use]
    pub const fn align(mut self, horizontal: ViewportAlign, vertical: ViewportAlign) -> Self {
        self = Self::Positioned {
            horizontal,
            vertical,
            margin: self.margin_value(),
        };
        self
    }

    #[must_use]
    pub const fn margin(mut self, margin: f32) -> Self {
        match &mut self {
            Self::Fill { margin: current }
            | Self::Positioned {
                margin: current, ..
            } => {
                *current = margin;
            }
        }
        self
    }

    #[must_use]
    pub fn place(self, viewport: Rect, desired: Size) -> Rect {
        let inner = inset_rect(
            finite_rect(viewport),
            finite_non_negative(self.margin_value()),
        );
        match self {
            Self::Fill { .. } => inner,
            Self::Positioned {
                horizontal,
                vertical,
                ..
            } => {
                let desired = finite_size(desired);
                let size = Size::new(
                    desired.width.min(inner.size.width),
                    desired.height.min(inner.size.height),
                );
                Rect::new(
                    Point::new(
                        align_axis(inner.origin.x, inner.size.width, size.width, horizontal),
                        align_axis(inner.origin.y, inner.size.height, size.height, vertical),
                    ),
                    size,
                )
            }
        }
    }

    const fn margin_value(self) -> f32 {
        match self {
            Self::Fill { margin } | Self::Positioned { margin, .. } => margin,
        }
    }
}

impl Default for ViewportPlacement {
    fn default() -> Self {
        Self::centered()
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlacedOverlay {
    pub placement: Placement,
    pub bounds: Rect,
    pub available_size: Size,
    pub constrained_width: bool,
    pub constrained_height: bool,
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

fn fits(placement: Placement, viewport: Rect, anchor: Rect, desired: Size, offset: f32) -> bool {
    let space = available_size(placement.side(), viewport, anchor, offset);
    desired.width <= space.width && desired.height <= space.height
}

fn available_main(placement: Placement, viewport: Rect, anchor: Rect, offset: f32) -> f32 {
    let size = available_size(placement.side(), viewport, anchor, offset);
    match placement.side() {
        Side::Top | Side::Bottom => size.height,
        Side::Left | Side::Right => size.width,
    }
}

fn available_size(side: Side, viewport: Rect, anchor: Rect, offset: f32) -> Size {
    let width = viewport.size.width;
    let height = viewport.size.height;
    match side {
        Side::Top => Size::new(
            width,
            clamp_zero(anchor.origin.y - viewport.origin.y - offset).min(height),
        ),
        Side::Bottom => Size::new(
            width,
            clamp_zero(bottom(viewport) - bottom(anchor) - offset).min(height),
        ),
        Side::Left => Size::new(
            clamp_zero(anchor.origin.x - viewport.origin.x - offset).min(width),
            height,
        ),
        Side::Right => Size::new(
            clamp_zero(right(viewport) - right(anchor) - offset).min(width),
            height,
        ),
    }
}

fn origin(
    placement: Placement,
    anchor: Rect,
    size: Size,
    offset: f32,
    cross_offset: f32,
    direction: WritingDirection,
) -> Point {
    let align = placement.alignment();
    let aligned_x = logical_align_axis(
        anchor.origin.x,
        anchor.size.width,
        size.width,
        align,
        direction,
    ) + cross_offset;
    let aligned_y =
        physical_align_axis(anchor.origin.y, anchor.size.height, size.height, align) + cross_offset;
    match placement.side() {
        Side::Top => Point::new(aligned_x, anchor.origin.y - offset - size.height),
        Side::Bottom => Point::new(aligned_x, bottom(anchor) + offset),
        Side::Left => Point::new(anchor.origin.x - offset - size.width, aligned_y),
        Side::Right => Point::new(right(anchor) + offset, aligned_y),
    }
}

fn logical_align_axis(
    origin: f32,
    anchor: f32,
    overlay: f32,
    align: Alignment,
    direction: WritingDirection,
) -> f32 {
    let align = if direction == WritingDirection::Rtl {
        match align {
            Alignment::Start => Alignment::End,
            Alignment::End => Alignment::Start,
            Alignment::Center => Alignment::Center,
        }
    } else {
        align
    };
    physical_align_axis(origin, anchor, overlay, align)
}

fn physical_align_axis(origin: f32, anchor: f32, overlay: f32, align: Alignment) -> f32 {
    match align {
        Alignment::Start => origin,
        Alignment::Center => origin + (anchor - overlay) * 0.5,
        Alignment::End => origin + anchor - overlay,
    }
}

fn align_axis(origin: f32, available: f32, size: f32, align: ViewportAlign) -> f32 {
    match align {
        ViewportAlign::Start => origin,
        ViewportAlign::Center => origin + (available - size) * 0.5,
        ViewportAlign::End => origin + available - size,
    }
}

fn clamp_axis(value: f32, minimum: f32, maximum: f32) -> f32 {
    value.max(minimum).min(maximum.max(minimum))
}

fn clamp_zero(value: f32) -> f32 {
    finite(value).max(0.0)
}

fn finite(value: f32) -> f32 {
    if value.is_finite() { value } else { 0.0 }
}

fn finite_non_negative(value: f32) -> f32 {
    finite(value).max(0.0)
}

fn finite_size(size: Size) -> Size {
    Size::new(
        finite_non_negative(size.width),
        finite_non_negative(size.height),
    )
}

fn finite_rect(rect: Rect) -> Rect {
    Rect::new(
        Point::new(finite(rect.origin.x), finite(rect.origin.y)),
        finite_size(rect.size),
    )
}

fn inset_rect(rect: Rect, margin: f32) -> Rect {
    let horizontal = margin.min(rect.size.width * 0.5);
    let vertical = margin.min(rect.size.height * 0.5);
    Rect::new(
        Point::new(rect.origin.x + horizontal, rect.origin.y + vertical),
        Size::new(
            rect.size.width - horizontal * 2.0,
            rect.size.height - vertical * 2.0,
        ),
    )
}

fn right(rect: Rect) -> f32 {
    rect.origin.x + rect.size.width
}

fn bottom(rect: Rect) -> f32 {
    rect.origin.y + rect.size.height
}
