use argui_core::{Point, ScrollDelta};
use std::time::{Duration, Instant};
use winit::{event::TouchPhase, event_loop::ActiveEventLoop, window::Window};

use crate::{RuntimeError, RuntimeEvent, app::Application};

const INERTIA_DELAY: Duration = Duration::from_millis(18);
const MAX_VELOCITY: f32 = 2_400.0;
const STOP_VELOCITY: f32 = 24.0;

#[derive(Debug, Default)]
pub(super) struct ScrollInertia {
    velocity: Point,
    last_input: Option<Instant>,
    last_frame: Option<Instant>,
    released: bool,
}

impl ScrollInertia {
    fn observe(&mut self, delta: ScrollDelta, phase: TouchPhase, now: Instant) {
        let ScrollDelta::Pixels(delta) = delta else {
            self.cancel();
            return;
        };
        if phase == TouchPhase::Cancelled {
            self.cancel();
            return;
        }
        if phase == TouchPhase::Ended {
            self.released = true;
            self.last_input = Some(now);
            self.last_frame = Some(now);
            return;
        }
        if phase == TouchPhase::Started {
            self.velocity = Point::default();
        }
        let elapsed = self
            .last_input
            .map_or(1.0 / 60.0, |last| now.duration_since(last).as_secs_f32())
            .clamp(1.0 / 240.0, 1.0 / 20.0);
        let sample = Point::new(delta.x / elapsed, delta.y / elapsed);
        self.velocity.x =
            (self.velocity.x * 0.45 + sample.x * 0.55).clamp(-MAX_VELOCITY, MAX_VELOCITY);
        self.velocity.y =
            (self.velocity.y * 0.45 + sample.y * 0.55).clamp(-MAX_VELOCITY, MAX_VELOCITY);
        self.last_input = Some(now);
        self.last_frame = Some(now);
        self.released = false;
    }

    fn advance(&mut self, now: Instant) -> Option<Point> {
        let last_input = self.last_input?;
        if !self.released && now.duration_since(last_input) < INERTIA_DELAY {
            return None;
        }
        let elapsed = now
            .duration_since(self.last_frame.unwrap_or(last_input))
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        self.last_frame = Some(now);
        let delta = Point::new(self.velocity.x * elapsed, self.velocity.y * elapsed);
        let decay = (-8.5 * elapsed).exp();
        self.velocity.x *= decay;
        self.velocity.y *= decay;
        if self.velocity.x.hypot(self.velocity.y) < STOP_VELOCITY {
            self.cancel();
        }
        Some(delta)
    }

    pub(super) fn cancel(&mut self) {
        *self = Self::default();
    }

    fn needs_frame(&self) -> bool {
        self.last_input.is_some()
    }
}

pub(super) fn merge_delta(pending: &mut ScrollDelta, next: ScrollDelta) -> bool {
    let (pending, next) = match (pending, next) {
        (ScrollDelta::Lines(pending), ScrollDelta::Lines(next))
        | (ScrollDelta::Pixels(pending), ScrollDelta::Pixels(next)) => (pending, next),
        _ => return false,
    };
    pending.x += next.x;
    pending.y += next.y;
    true
}

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn queue_pointer_scroll(
        &mut self,
        delta: ScrollDelta,
        phase: TouchPhase,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        self.scroll_inertia.observe(delta, phase, Instant::now());
        if let Some(pending) = &mut self.pending_pointer_scroll {
            if !merge_delta(pending, delta) {
                self.flush_pointer_scroll(window, event_loop);
                self.pending_pointer_scroll = Some(delta);
            }
        } else {
            self.pending_pointer_scroll = Some(delta);
        }
        window.request_redraw();
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_pointer_inertia(&mut self, window: &Window) {
        if let Some(delta) = self.scroll_inertia.advance(Instant::now()) {
            let delta = ScrollDelta::Pixels(delta);
            if let Some(pending) = &mut self.pending_pointer_scroll {
                let _ = merge_delta(pending, delta);
            } else {
                self.pending_pointer_scroll = Some(delta);
            }
        }
        if self.scroll_inertia.needs_frame() {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn flush_pointer_scroll(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let Some(delta) = self.pending_pointer_scroll.take() else {
            return;
        };
        let (Some(point), Some(layout), Some(ui)) =
            (self.pointer, &self.ui_layout, &mut self.ui_tree)
        else {
            return;
        };
        let update = ui.scroll(point, delta, &layout.scroll_regions);
        self.apply_ui_update(update, window, event_loop);
        let hover = match (&self.ui_layout, &mut self.ui_tree) {
            (Some(layout), Some(ui)) => ui.pointer_moved(point, &layout.hit_regions),
            _ => return,
        };
        self.apply_ui_update(hover, window, event_loop);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn scroll_or_exit(&mut self, event_loop: &ActiveEventLoop) -> bool {
        let result = match (&self.ui_tree, &mut self.ui_layout) {
            (Some(ui), Some(layout)) => self.layout_engine.apply_scroll(ui, layout),
            _ => return false,
        };
        if let Err(error) = result {
            (self.on_event)(RuntimeEvent::LayoutFailed(error.to_string()));
            self.fatal_error = Some(RuntimeError::from(error));
            event_loop.exit();
            return false;
        }
        if let (Some(prepared), Some(layout)) = (&mut self.prepared_text, &self.ui_layout) {
            for (index, block) in layout.text.blocks().iter().enumerate() {
                prepared.reposition_block(index, block.bounds.origin, block.clip);
            }
        }
        self.publish_inspection();
        self.paint_inspection_highlight();
        true
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn apply_scroll_request(
        &mut self,
        key: &str,
        offset: Point,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout) else {
            return false;
        };
        let Some(region) = layout
            .scroll_regions
            .iter()
            .find(|region| ui.key(region.node) == Some(key))
        else {
            return false;
        };
        let offset = Point::new(
            offset.x.clamp(0.0, region.max_offset.x),
            offset.y.clamp(0.0, region.max_offset.y),
        );
        if !ui.set_scroll_offset(region.node, offset) {
            return false;
        }
        self.scroll_or_exit(event_loop)
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use argui_core::{Point, ScrollDelta};
    use winit::event::TouchPhase;

    use super::{ScrollInertia, merge_delta};

    #[test]
    fn pointer_scroll_bursts_coalesce_without_losing_distance_or_units() {
        let mut pixels = ScrollDelta::Pixels(Point::new(2.0, 4.0));
        assert!(merge_delta(
            &mut pixels,
            ScrollDelta::Pixels(Point::new(-1.0, 8.0))
        ));
        assert_eq!(pixels, ScrollDelta::Pixels(Point::new(1.0, 12.0)));
        assert!(!merge_delta(
            &mut pixels,
            ScrollDelta::Lines(Point::new(0.0, 1.0))
        ));
        let mut lines = ScrollDelta::Lines(Point::new(1.0, 2.0));
        assert!(merge_delta(
            &mut lines,
            ScrollDelta::Lines(Point::new(3.0, 4.0))
        ));
        assert_eq!(lines, ScrollDelta::Lines(Point::new(4.0, 6.0)));
    }

    #[test]
    fn pixel_scrolls_gain_a_short_tail_while_wheel_lines_stay_direct() {
        let start = Instant::now();
        let mut inertia = ScrollInertia::default();
        inertia.observe(
            ScrollDelta::Pixels(Point::new(0.0, -8.0)),
            TouchPhase::Moved,
            start,
        );
        let tail = inertia
            .advance(start + Duration::from_millis(24))
            .expect("pixel input starts inertia after the quiet period");
        assert!(tail.y < 0.0);
        assert!(inertia.needs_frame());

        inertia.observe(
            ScrollDelta::Lines(Point::new(0.0, -1.0)),
            TouchPhase::Moved,
            start,
        );
        assert!(!inertia.needs_frame());
    }

    #[test]
    fn inertia_handles_gesture_phases_quiet_period_and_settling() {
        let start = Instant::now();
        let mut inertia = ScrollInertia::default();
        assert_eq!(inertia.advance(start), None);

        inertia.observe(
            ScrollDelta::Pixels(Point::new(0.0, -12.0)),
            TouchPhase::Started,
            start,
        );
        assert_eq!(
            inertia.advance(start + Duration::from_millis(8)),
            None,
            "an active gesture has no synthetic motion before the quiet period"
        );
        inertia.observe(
            ScrollDelta::Pixels(Point::new(0.0, -4.0)),
            TouchPhase::Moved,
            start + Duration::from_millis(10),
        );
        inertia.observe(
            ScrollDelta::Pixels(Point::default()),
            TouchPhase::Ended,
            start + Duration::from_millis(12),
        );
        assert!(inertia.advance(start + Duration::from_millis(16)).is_some());

        let mut stopped = ScrollInertia::default();
        stopped.observe(
            ScrollDelta::Pixels(Point::default()),
            TouchPhase::Ended,
            start,
        );
        assert_eq!(
            stopped.advance(start + Duration::from_millis(20)),
            Some(Point::default())
        );
        assert!(!stopped.needs_frame());

        inertia.observe(
            ScrollDelta::Pixels(Point::new(0.0, 3.0)),
            TouchPhase::Cancelled,
            start,
        );
        assert!(!inertia.needs_frame());
    }
}
