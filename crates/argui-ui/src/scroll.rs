use argui_core::{Affine2D, Point, Rect, ScrollDelta};
use argui_paint::ClipChain;

use crate::{
    HitRegion, NodeId, OverscrollBehavior, ScrollConfig, ScrollPropagation, ScrollbarStyle,
    VisualState, VisualStates, scroll_physics::ScrollPhysicsState,
};

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollbarRegion {
    pub horizontal: Option<ScrollbarGeometry>,
    pub vertical: Option<ScrollbarGeometry>,
    pub style: ScrollbarStyle,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScrollbarAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ScrollbarGeometry {
    pub track: Rect,
    pub thumb: Rect,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScrollRegion {
    pub node: NodeId,
    pub bounds: Rect,
    pub clip: Rect,
    pub transform: Affine2D,
    pub clips: ClipChain,
    pub max_offset: Point,
    pub config: ScrollConfig,
    pub scrollbar: Option<ScrollbarRegion>,
    pub interaction_order: usize,
}

#[must_use]
pub fn scrollbar_at<'a>(
    point: Point,
    scroll_regions: &'a [ScrollRegion],
    hit_regions: &[HitRegion],
) -> Option<&'a ScrollRegion> {
    let scrollbar = scroll_regions
        .iter()
        .filter(|region| region.config.enabled && region.scrollbar_contains(point))
        .max_by_key(|region| region.interaction_order)?;
    let top_hit = hit_regions
        .iter()
        .enumerate()
        .rev()
        .find(|(_, region)| region.contains(point))
        .map(|(index, _)| index);
    top_hit
        .is_none_or(|index| index < scrollbar.interaction_order)
        .then_some(scrollbar)
}

impl ScrollRegion {
    #[must_use]
    pub fn contains(&self, point: Point) -> bool {
        self.local_point(point)
            .is_some_and(|local| self.bounds.contains(local) && self.clip.contains(local))
            && self.clips.contains(point)
    }

    #[must_use]
    pub fn scrollbar_contains(&self, point: Point) -> bool {
        self.clips.contains(point)
            && self.scrollbar.as_ref().is_some_and(|scrollbar| {
                self.local_point(point).is_some_and(|local| {
                    self.clip.contains(local) && scrollbar.geometry_at(local).is_some()
                })
            })
    }

    #[must_use]
    pub fn local_point(&self, point: Point) -> Option<Point> {
        self.transform
            .inverse()
            .map(|inverse| inverse.transform_point(point))
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ScrollChange {
    pub node: NodeId,
    pub delta: Point,
    pub offset: Point,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum ScrollOutcome {
    Changed(Vec<ScrollChange>),
    Consumed,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct ScrollDrag {
    node: NodeId,
    axis: ScrollbarAxis,
    grab: f32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ScrollbarPart {
    Track,
    Thumb,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScrollHover {
    node: NodeId,
    axis: ScrollbarAxis,
    part: ScrollbarPart,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct ScrollState {
    offsets: Vec<(NodeId, Point)>,
    drag: Option<ScrollDrag>,
    hover: Option<ScrollHover>,
    physics: ScrollPhysicsState,
}

impl ScrollState {
    pub fn dragging(&self) -> bool {
        self.drag.is_some()
    }

    pub fn visual_states(&self, node: NodeId, part: ScrollbarPart, enabled: bool) -> VisualStates {
        let mut states = VisualStates::NONE;
        if !enabled {
            states.insert(VisualState::Disabled);
        }
        if self
            .hover
            .is_some_and(|hover| hover.node == node && hover.part == part)
        {
            states.insert(VisualState::Hovered);
        }
        if part == ScrollbarPart::Thumb && self.drag.is_some_and(|drag| drag.node == node) {
            states.insert(VisualState::Pressed);
        }
        states
    }

    pub fn scrollbar_opacity(&self, node: NodeId, visibility: crate::ScrollbarVisibility) -> f32 {
        self.physics.scrollbar_opacity(node, visibility)
    }

    pub fn update_hover(&mut self, point: Option<Point>, regions: &[ScrollRegion]) -> bool {
        let hover = point.and_then(|point| {
            let region = regions
                .iter()
                .filter(|region| region.config.enabled && region.scrollbar_contains(point))
                .max_by_key(|region| region.interaction_order)?;
            let scrollbar = region.scrollbar.as_ref()?;
            let local = region.local_point(point)?;
            let (axis, geometry) = scrollbar.geometry_at(local)?;
            Some(ScrollHover {
                node: region.node,
                axis,
                part: if geometry.thumb.contains(local) {
                    ScrollbarPart::Thumb
                } else {
                    ScrollbarPart::Track
                },
            })
        });
        if self.hover == hover {
            return false;
        }
        self.hover = hover;
        if let Some(hover) = hover {
            let visibility = regions
                .iter()
                .find(|region| region.node == hover.node)
                .and_then(|region| region.config.scrollbar.as_ref())
                .map(|style| style.visibility);
            self.physics.activate_scrollbar(hover.node, visibility);
        }
        true
    }

    pub fn offset(&self, node: NodeId) -> Point {
        self.offsets
            .iter()
            .find_map(|(candidate, offset)| (*candidate == node).then_some(*offset))
            .unwrap_or_default()
    }

    pub fn visual_offset(&self, node: NodeId) -> Point {
        self.physics.visual_offset(node, self.offset(node))
    }

    pub fn set_offset(&mut self, node: NodeId, offset: Point) -> bool {
        self.physics.clear_overscroll(node);
        let offset = Point::new(offset.x.max(0.0), offset.y.max(0.0));
        if let Some((_, stored)) = self
            .offsets
            .iter_mut()
            .find(|(candidate, _)| *candidate == node)
        {
            if *stored == offset {
                return false;
            }
            *stored = offset;
        } else {
            self.offsets.push((node, offset));
        }
        true
    }

    pub fn scroll(
        &mut self,
        point: Point,
        delta: ScrollDelta,
        regions: &[ScrollRegion],
    ) -> Option<ScrollOutcome> {
        let mut remaining = delta;
        let mut changes = Vec::new();
        for (index, region) in regions
            .iter()
            .enumerate()
            .rev()
            .filter(|(_, region)| region.contains(point))
        {
            if !region.config.enabled {
                continue;
            }
            let requested = region.config.logical_delta(remaining);
            let previous = self.offset(region.node);
            let maximum = Point::new(
                non_negative(region.max_offset.x),
                non_negative(region.max_offset.y),
            );
            let offset = Point::new(
                (previous.x + requested.x).clamp(0.0, maximum.x),
                (previous.y + requested.y).clamp(0.0, maximum.y),
            );
            let applied = Point::new(offset.x - previous.x, offset.y - previous.y);
            if applied != Point::default() {
                self.store_offset(region.node, offset);
                self.physics.activate_scrollbar(
                    region.node,
                    region
                        .config
                        .scrollbar
                        .as_ref()
                        .map(|style| style.visibility),
                );
                changes.push(ScrollChange {
                    node: region.node,
                    delta: applied,
                    offset,
                });
            }
            remaining = residual_delta(remaining, requested, applied);
            let has_chain_target = region.config.propagation == ScrollPropagation::Chain
                && regions[..index]
                    .iter()
                    .rev()
                    .any(|candidate| candidate.config.enabled && candidate.contains(point));
            if let OverscrollBehavior::Elastic(config) = region.config.overscroll
                && !has_chain_target
                && region.config.propagation != ScrollPropagation::None
            {
                let excess = Point::new(requested.x - applied.x, requested.y - applied.y);
                if excess != Point::default() {
                    self.physics.activate_scrollbar(
                        region.node,
                        region
                            .config
                            .scrollbar
                            .as_ref()
                            .map(|style| style.visibility),
                    );
                    let visual_delta = self.physics.push_overscroll(region.node, excess, config);
                    if visual_delta != Point::default() {
                        changes.push(ScrollChange {
                            node: region.node,
                            delta: visual_delta,
                            offset,
                        });
                    }
                    remaining = zero_delta(remaining);
                }
            }
            if delta_is_zero(remaining) || region.config.propagation != ScrollPropagation::Chain {
                return Some(if changes.is_empty() {
                    ScrollOutcome::Consumed
                } else {
                    ScrollOutcome::Changed(changes)
                });
            }
        }
        (!changes.is_empty()).then_some(ScrollOutcome::Changed(changes))
    }

    pub fn scrollbar_pressed(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<Option<ScrollChange>> {
        let region = regions
            .iter()
            .filter(|region| region.config.enabled && region.scrollbar_contains(point))
            .max_by_key(|region| region.interaction_order)?;
        let scrollbar = region.scrollbar.as_ref()?;
        let local = region.local_point(point)?;
        let (axis, geometry) = scrollbar.geometry_at(local)?;
        let cursor = axis.value(local);
        let grab = if geometry.thumb.contains(local) {
            cursor - axis.origin(geometry.thumb)
        } else {
            axis.extent(geometry.thumb) * 0.5
        };
        self.drag = Some(ScrollDrag {
            node: region.node,
            axis,
            grab,
        });
        self.physics.activate_scrollbar(
            region.node,
            region
                .config
                .scrollbar
                .as_ref()
                .map(|style| style.visibility),
        );
        Some(self.drag_to(point, regions))
    }

    pub fn scrollbar_dragged(
        &mut self,
        point: Point,
        regions: &[ScrollRegion],
    ) -> Option<ScrollChange> {
        self.drag?;
        self.drag_to(point, regions)
    }

    pub fn scrollbar_released(&mut self) -> bool {
        self.drag.take().is_some()
    }

    fn drag_to(&mut self, point: Point, regions: &[ScrollRegion]) -> Option<ScrollChange> {
        let drag = self.drag?;
        let region = regions.iter().find(|region| region.node == drag.node)?;
        let scrollbar = region.scrollbar.as_ref()?;
        let point = region.local_point(point)?;
        let geometry = scrollbar.geometry(drag.axis)?;
        let travel = drag.axis.extent(geometry.track) - drag.axis.extent(geometry.thumb);
        let maximum = non_negative(drag.axis.point_value(region.max_offset));
        if travel <= 0.0 || maximum <= 0.0 {
            return None;
        }
        let thumb = (drag.axis.value(point) - drag.grab - drag.axis.origin(geometry.track))
            .clamp(0.0, travel);
        let previous = self.offset(region.node);
        let offset = drag.axis.with_value(previous, thumb / travel * maximum);
        let applied = Point::new(offset.x - previous.x, offset.y - previous.y);
        if applied == Point::default() {
            return None;
        }
        self.store_offset(region.node, offset);
        Some(ScrollChange {
            node: region.node,
            delta: applied,
            offset,
        })
    }

    fn store_offset(&mut self, node: NodeId, offset: Point) {
        if let Some((_, stored)) = self
            .offsets
            .iter_mut()
            .find(|(candidate, _)| *candidate == node)
        {
            *stored = offset;
        } else {
            self.offsets.push((node, offset));
        }
    }

    pub fn advance_overscroll(&mut self, elapsed: f32) -> Vec<ScrollChange> {
        self.physics
            .advance_overscroll(elapsed)
            .into_iter()
            .map(|(node, delta)| ScrollChange {
                node,
                delta,
                offset: self.offset(node),
            })
            .collect()
    }

    pub fn advance_scrollbars(&mut self, elapsed: f32, regions: &[ScrollRegion]) -> bool {
        let hover = self.hover;
        let drag = self.drag;
        self.physics.advance_scrollbars(elapsed, regions, |node| {
            hover.is_some_and(|hover| hover.node == node)
                || drag.is_some_and(|drag| drag.node == node)
        })
    }

    pub fn activate_scrollbar(&mut self, node: NodeId, regions: &[ScrollRegion]) {
        let visibility = regions
            .iter()
            .find(|region| region.node == node)
            .and_then(|region| region.config.scrollbar.as_ref())
            .map(|style| style.visibility);
        self.physics.activate_scrollbar(node, visibility);
    }

    pub fn scrollbar_activity_active(&self) -> bool {
        let hover = self.hover;
        let drag = self.drag;
        self.physics.scrollbar_activity_active(|node| {
            hover.is_some_and(|hover| hover.node == node)
                || drag.is_some_and(|drag| drag.node == node)
        })
    }

    pub fn overscroll_active(&self) -> bool {
        self.physics.overscroll_active()
    }

    pub fn retain(&mut self, ids: &[NodeId]) {
        self.offsets.retain(|(node, _)| ids.contains(node));
        self.physics.retain(ids);
        if self.drag.is_some_and(|drag| !ids.contains(&drag.node)) {
            self.drag = None;
        }
        if self.hover.is_some_and(|hover| !ids.contains(&hover.node)) {
            self.hover = None;
        }
    }
}

impl ScrollbarRegion {
    fn geometry_at(&self, point: Point) -> Option<(ScrollbarAxis, &ScrollbarGeometry)> {
        self.vertical
            .as_ref()
            .filter(|geometry| geometry.track.contains(point))
            .map(|geometry| (ScrollbarAxis::Vertical, geometry))
            .or_else(|| {
                self.horizontal
                    .as_ref()
                    .filter(|geometry| geometry.track.contains(point))
                    .map(|geometry| (ScrollbarAxis::Horizontal, geometry))
            })
    }

    fn geometry(&self, axis: ScrollbarAxis) -> Option<&ScrollbarGeometry> {
        match axis {
            ScrollbarAxis::Horizontal => self.horizontal.as_ref(),
            ScrollbarAxis::Vertical => self.vertical.as_ref(),
        }
    }
}

impl ScrollbarAxis {
    const fn value(self, point: Point) -> f32 {
        match self {
            Self::Horizontal => point.x,
            Self::Vertical => point.y,
        }
    }

    const fn origin(self, rect: Rect) -> f32 {
        self.value(rect.origin)
    }

    const fn extent(self, rect: Rect) -> f32 {
        match self {
            Self::Horizontal => rect.size.width,
            Self::Vertical => rect.size.height,
        }
    }

    const fn point_value(self, point: Point) -> f32 {
        self.value(point)
    }

    const fn with_value(self, point: Point, value: f32) -> Point {
        match self {
            Self::Horizontal => Point::new(value, point.y),
            Self::Vertical => Point::new(point.x, value),
        }
    }
}

fn residual_delta(input: ScrollDelta, requested: Point, applied: Point) -> ScrollDelta {
    let residual = |raw: f32, wanted: f32, used: f32| {
        if wanted.abs() <= f32::EPSILON {
            raw
        } else {
            raw * ((wanted - used) / wanted).clamp(0.0, 1.0)
        }
    };
    match input {
        ScrollDelta::Lines(raw) => ScrollDelta::Lines(Point::new(
            residual(raw.x, requested.x, applied.x),
            residual(raw.y, requested.y, applied.y),
        )),
        ScrollDelta::Pixels(raw) => ScrollDelta::Pixels(Point::new(
            residual(raw.x, requested.x, applied.x),
            residual(raw.y, requested.y, applied.y),
        )),
    }
}

fn delta_is_zero(delta: ScrollDelta) -> bool {
    let point = match delta {
        ScrollDelta::Lines(point) | ScrollDelta::Pixels(point) => point,
    };
    point.x.abs() <= f32::EPSILON && point.y.abs() <= f32::EPSILON
}

fn zero_delta(delta: ScrollDelta) -> ScrollDelta {
    match delta {
        ScrollDelta::Lines(_) => ScrollDelta::Lines(Point::default()),
        ScrollDelta::Pixels(_) => ScrollDelta::Pixels(Point::default()),
    }
}

fn non_negative(value: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        0.0
    }
}
