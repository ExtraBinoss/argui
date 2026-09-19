use argui_core::{Point, PointerId};
use argui_ui::{NodeId, ScrollAxes, ScrollPhysics, ScrollRegion};

#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct TouchScrollUpdate {
    pub(super) target: NodeId,
    pub(super) delta: Point,
    pub(super) phase: winit::event::TouchPhase,
    pub(super) natural: bool,
    pub(super) physics: ScrollPhysics,
}

#[derive(Clone, Copy, Debug)]
struct ActiveScroll {
    target: NodeId,
    axes: ScrollAxes,
    natural: bool,
    physics: ScrollPhysics,
}

/// Distinguishes a stationary touch from an intentional, axis-aware scroll drag.
#[derive(Debug, Default)]
pub(super) struct TouchScrollGesture {
    pointer: Option<PointerId>,
    origin: Point,
    last: Point,
    active: Option<ActiveScroll>,
}

impl TouchScrollGesture {
    /// Starts a pending gesture for `pointer` at `point` without moving content.
    pub(super) fn begin(&mut self, pointer: PointerId, point: Point) {
        self.pointer = Some(pointer);
        self.origin = point;
        self.last = point;
        self.active = None;
    }

    /// Advances the pending or active gesture, starting only after `slop` is exceeded.
    pub(super) fn moved(
        &mut self,
        pointer: PointerId,
        point: Point,
        slop: f32,
        regions: &[ScrollRegion],
    ) -> Option<TouchScrollUpdate> {
        if self.pointer != Some(pointer) {
            return None;
        }
        if let Some(active) = self.active {
            let delta = project(difference(point, self.last), active.axes);
            self.last = point;
            return Some(update(active, delta, winit::event::TouchPhase::Moved));
        }
        let total = difference(point, self.origin);
        let axis = dominant_axis(total);
        if axis_value(total, axis).abs() <= slop.max(0.0) {
            return None;
        }
        let region = regions
            .iter()
            .filter(|region| {
                region.config.enabled && region.contains(self.origin) && supports_axis(region, axis)
            })
            .max_by_key(|region| region.interaction_order)?;
        let active = ActiveScroll {
            target: region.node,
            axes: region.config.axes,
            natural: region.config.natural_touch_scroll,
            physics: region.config.physics,
        };
        self.active = Some(active);
        self.last = point;
        Some(update(
            active,
            initial_delta(total, active.axes, slop.max(0.0)),
            winit::event::TouchPhase::Started,
        ))
    }

    /// Ends `pointer` and reports a terminal update only when scrolling had started.
    pub(super) fn end(&mut self, pointer: PointerId, cancelled: bool) -> Option<TouchScrollUpdate> {
        if self.pointer != Some(pointer) {
            return None;
        }
        let active = self.active;
        *self = Self::default();
        active.map(|active| {
            update(
                active,
                Point::default(),
                if cancelled {
                    winit::event::TouchPhase::Cancelled
                } else {
                    winit::event::TouchPhase::Ended
                },
            )
        })
    }

    /// Cancels every pending or active touch-scroll gesture.
    pub(super) fn cancel(&mut self) {
        *self = Self::default();
    }
}

/// Packages one gesture sample with the scroll viewport's latched configuration.
fn update(
    active: ActiveScroll,
    delta: Point,
    phase: winit::event::TouchPhase,
) -> TouchScrollUpdate {
    TouchScrollUpdate {
        target: active.target,
        delta,
        phase,
        natural: active.natural,
        physics: active.physics,
    }
}

/// Returns the dominant movement axis, preferring vertical movement for ties.
fn dominant_axis(point: Point) -> ScrollAxes {
    if point.x.abs() > point.y.abs() {
        ScrollAxes::Horizontal
    } else {
        ScrollAxes::Vertical
    }
}

/// Returns the component of `point` represented by the single-axis intent.
fn axis_value(point: Point, axis: ScrollAxes) -> f32 {
    match axis {
        ScrollAxes::Horizontal => point.x,
        ScrollAxes::Vertical | ScrollAxes::Both => point.y,
    }
}

/// Returns whether `region` can currently scroll along the intended `axis`.
fn supports_axis(region: &ScrollRegion, axis: ScrollAxes) -> bool {
    match (region.config.axes, axis) {
        (ScrollAxes::Both, ScrollAxes::Horizontal) => region.max_offset.x > f32::EPSILON,
        (ScrollAxes::Both, ScrollAxes::Vertical) => region.max_offset.y > f32::EPSILON,
        (ScrollAxes::Horizontal, ScrollAxes::Horizontal) => region.max_offset.x > f32::EPSILON,
        (ScrollAxes::Vertical, ScrollAxes::Vertical) => region.max_offset.y > f32::EPSILON,
        _ => false,
    }
}

/// Projects the first scroll delta and removes the already-consumed touch slop.
fn initial_delta(total: Point, axes: ScrollAxes, slop: f32) -> Point {
    let projected = project(total, axes);
    match axes {
        ScrollAxes::Horizontal => Point::new(remove_slop(projected.x, slop), 0.0),
        ScrollAxes::Vertical => Point::new(0.0, remove_slop(projected.y, slop)),
        ScrollAxes::Both => {
            let magnitude = projected.x.hypot(projected.y);
            if magnitude <= slop || magnitude <= f32::EPSILON {
                Point::default()
            } else {
                let scale = (magnitude - slop) / magnitude;
                Point::new(projected.x * scale, projected.y * scale)
            }
        }
    }
}

/// Removes `slop` from the magnitude of `value` while preserving its direction.
fn remove_slop(value: f32, slop: f32) -> f32 {
    value.signum() * (value.abs() - slop).max(0.0)
}

/// Projects `point` onto the axes owned by the selected scroll viewport.
fn project(point: Point, axes: ScrollAxes) -> Point {
    match axes {
        ScrollAxes::Horizontal => Point::new(point.x, 0.0),
        ScrollAxes::Vertical => Point::new(0.0, point.y),
        ScrollAxes::Both => point,
    }
}

/// Returns the component-wise displacement from `previous` to `point`.
fn difference(point: Point, previous: Point) -> Point {
    Point::new(point.x - previous.x, point.y - previous.y)
}
