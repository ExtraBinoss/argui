use argui_core::{Point, ScrollDelta};
use argui_ui::{InertialScroll, NodeId, ScrollBehavior, ScrollPhysics, ScrollRequest};
use web_time::Instant;
use winit::{event::TouchPhase, event_loop::ActiveEventLoop, window::Window};

use crate::{RuntimeError, RuntimeEvent, app::Application};

mod request;
use request::{ScrollTrack, scroll_tracks};

#[derive(Debug)]
pub(super) struct ScrollInertia {
    velocity: Point,
    last_input: Option<Instant>,
    last_frame: Option<Instant>,
    released: bool,
    target: Option<NodeId>,
    config: InertialScroll,
    point: Option<Point>,
    dispatch_wheel: bool,
}

impl Default for ScrollInertia {
    fn default() -> Self {
        Self {
            velocity: Point::default(),
            last_input: None,
            last_frame: None,
            released: false,
            target: None,
            config: InertialScroll::default(),
            point: None,
            dispatch_wheel: true,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub(super) struct PendingScroll {
    delta: ScrollDelta,
    point: Point,
    dispatch_wheel: bool,
    target: Option<NodeId>,
}

#[derive(Clone, Copy, Debug)]
struct ScrollSample {
    delta: ScrollDelta,
    phase: TouchPhase,
    now: Instant,
    target: Option<NodeId>,
    physics: ScrollPhysics,
    point: Point,
    dispatch_wheel: bool,
}

#[derive(Debug)]
pub(super) struct ProgrammaticScroll {
    tracks: Vec<ScrollTrack>,
    started: Instant,
    tween: argui_animation::Tween,
}

impl ScrollInertia {
    fn observe(&mut self, sample: ScrollSample) {
        let ScrollSample {
            delta,
            phase,
            now,
            target,
            physics,
            point,
            dispatch_wheel,
        } = sample;
        let config = match physics {
            ScrollPhysics::Hybrid => InertialScroll::default(),
            ScrollPhysics::Inertial(config) => config,
            ScrollPhysics::Direct | ScrollPhysics::Native => {
                self.cancel();
                return;
            }
        };
        let config = valid_inertia(config);
        let ScrollDelta::Pixels(delta) = delta else {
            self.cancel();
            return;
        };
        if phase == TouchPhase::Cancelled {
            self.cancel();
            return;
        }
        let target_changed = self.target != target;
        self.target = target;
        self.config = config;
        self.point = Some(point);
        self.dispatch_wheel = dispatch_wheel;
        if phase == TouchPhase::Ended {
            self.released = true;
            self.last_input = Some(now);
            self.last_frame = Some(now);
            return;
        }
        if phase == TouchPhase::Started || target_changed {
            self.velocity = Point::default();
        }
        let elapsed = self
            .last_input
            .map_or(1.0 / 60.0, |last| now.duration_since(last).as_secs_f32())
            .clamp(1.0 / 240.0, 1.0 / 20.0);
        let sample = Point::new(delta.x / elapsed, delta.y / elapsed);
        let retained = 1.0 - config.sample_weight.clamp(0.0, 1.0);
        let sample_weight = config.sample_weight.clamp(0.0, 1.0);
        self.velocity.x = (self.velocity.x * retained + sample.x * sample_weight)
            .clamp(-config.velocity_limit, config.velocity_limit);
        self.velocity.y = (self.velocity.y * retained + sample.y * sample_weight)
            .clamp(-config.velocity_limit, config.velocity_limit);
        self.last_input = Some(now);
        self.last_frame = Some(now);
        self.released = false;
    }

    fn advance(&mut self, now: Instant) -> Option<(Point, Point, bool)> {
        let last_input = self.last_input?;
        if !self.released && now.duration_since(last_input) < self.config.continuation_grace {
            return None;
        }
        let elapsed = now
            .duration_since(self.last_frame.unwrap_or(last_input))
            .as_secs_f32()
            .clamp(1.0 / 240.0, 1.0 / 30.0);
        self.last_frame = Some(now);
        let decay = (-self.config.decay * elapsed).exp();
        let distance = if self.config.decay <= f32::EPSILON {
            elapsed
        } else {
            (1.0 - decay) / self.config.decay
        };
        let delta = Point::new(self.velocity.x * distance, self.velocity.y * distance);
        self.velocity.x *= decay;
        self.velocity.y *= decay;
        if self.velocity.x.hypot(self.velocity.y) < self.config.stop_velocity {
            self.cancel();
        }
        self.point.map(|point| (delta, point, self.dispatch_wheel))
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
        self.programmatic_scroll = None;
        let Some(point) = self.pointer else {
            return;
        };
        let Some(layout) = &self.ui_layout else {
            return;
        };
        let target = self.scroll_gesture.target(
            point,
            self.input_epoch.elapsed(),
            phase == TouchPhase::Started,
            &layout.scroll_regions,
        );
        let physics = layout
            .scroll_regions
            .iter()
            .find(|region| Some(region.node) == target)
            .map_or(ScrollPhysics::Direct, |region| region.config.physics);
        self.scroll_inertia.observe(ScrollSample {
            delta,
            phase,
            now: Instant::now(),
            target,
            physics,
            point,
            dispatch_wheel: true,
        });
        if let Some(pending) = &mut self.pending_pointer_scroll {
            if !pending.dispatch_wheel
                || pending.target != target
                || !merge_delta(&mut pending.delta, delta)
            {
                self.flush_pointer_scroll(window, event_loop);
                self.pending_pointer_scroll = Some(PendingScroll {
                    delta,
                    point,
                    dispatch_wheel: true,
                    target,
                });
            } else {
                pending.point = point;
            }
        } else {
            self.pending_pointer_scroll = Some(PendingScroll {
                delta,
                point,
                dispatch_wheel: true,
                target,
            });
        }
        window.request_redraw();
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_pointer_inertia(
        &mut self,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let target = self.scroll_inertia.target;
        if let Some((delta, point, dispatch_wheel)) = self.scroll_inertia.advance(Instant::now()) {
            let delta = ScrollDelta::Pixels(delta);
            if let Some(pending) = &mut self.pending_pointer_scroll {
                if pending.dispatch_wheel != dispatch_wheel
                    || pending.target != target
                    || !merge_delta(&mut pending.delta, delta)
                {
                    self.flush_pointer_scroll(window, event_loop);
                    self.pending_pointer_scroll = Some(PendingScroll {
                        delta,
                        point,
                        dispatch_wheel,
                        target,
                    });
                } else {
                    pending.point = point;
                }
            } else {
                self.pending_pointer_scroll = Some(PendingScroll {
                    delta,
                    point,
                    dispatch_wheel,
                    target,
                });
            }
        }
        if self.scroll_inertia.needs_frame() {
            window.request_redraw();
        }
    }

    pub(super) fn observe_touch_scroll(
        &mut self,
        point: Point,
        delta: Option<Point>,
        phase: argui_core::PointerPhase,
        window: &Window,
    ) {
        let phase = match phase {
            argui_core::PointerPhase::Pressed => TouchPhase::Started,
            argui_core::PointerPhase::Moved => TouchPhase::Moved,
            argui_core::PointerPhase::Released => TouchPhase::Ended,
            argui_core::PointerPhase::Cancelled | argui_core::PointerPhase::Left => {
                TouchPhase::Cancelled
            }
            argui_core::PointerPhase::Entered => return,
        };
        let (target, physics) = self
            .ui_layout
            .as_ref()
            .and_then(|layout| {
                layout
                    .scroll_regions
                    .iter()
                    .rev()
                    .find(|region| region.config.enabled && region.contains(point))
            })
            .map_or((None, ScrollPhysics::Direct), |region| {
                (Some(region.node), region.config.physics)
            });
        self.scroll_inertia.observe(ScrollSample {
            delta: ScrollDelta::Pixels(delta.unwrap_or_default()),
            phase,
            now: Instant::now(),
            target,
            physics,
            point,
            dispatch_wheel: false,
        });
        if self.scroll_inertia.needs_frame() {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_scroll_physics(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let now = Instant::now();
        let elapsed = self
            .last_scroll_physics
            .replace(now)
            .map_or(1.0 / 60.0, |last| now.duration_since(last).as_secs_f32());
        let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout) else {
            return;
        };
        let update = ui.advance_scroll_physics(elapsed, &layout.scroll_regions);
        let active = ui.wants_scroll_frame();
        self.apply_ui_update(update, window, event_loop);
        if active {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn flush_pointer_scroll(&mut self, window: &Window, event_loop: &ActiveEventLoop) {
        let Some(pending) = self.pending_pointer_scroll.take() else {
            return;
        };
        let point = pending.point;
        let delta = pending.delta;
        if pending.dispatch_wheel {
            let wheel_update = match (&self.ui_layout, &mut self.ui_tree) {
                (Some(_), Some(ui)) => pending
                    .target
                    .or_else(|| ui.node_id_at(0))
                    .map_or_else(argui_ui::InteractionUpdate::default, |target| {
                        ui.wheel_event_from(target, point, delta)
                    }),
                _ => return,
            };
            let wheel_event = wheel_update.events.first().cloned();
            self.apply_ui_update(wheel_update, window, event_loop);
            if wheel_event.is_some_and(|event| event.default_prevented()) {
                self.scroll_inertia.cancel();
                return;
            }
        }
        let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) else {
            return;
        };
        let update = pending
            .target
            .map_or_else(argui_ui::InteractionUpdate::default, |target| {
                ui.scroll_from(target, point, delta, &layout.scroll_regions)
            });
        self.apply_ui_update(update, window, event_loop);
        if !pending.dispatch_wheel {
            return;
        }
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
        request: ScrollRequest,
        event_loop: &ActiveEventLoop,
    ) -> bool {
        self.scroll_inertia.cancel();
        let (Some(ui), Some(layout)) = (&self.ui_tree, &self.ui_layout) else {
            return false;
        };
        let tracks = scroll_tracks(ui, layout, &request);
        if tracks.is_empty() {
            return false;
        }
        match request.behavior {
            ScrollBehavior::Instant => {
                let ui = self.ui_tree.as_mut().expect("tree checked above");
                let changed = tracks.iter().fold(false, |changed, track| {
                    let moved = ui.set_scroll_offset(track.node, track.to);
                    if moved {
                        ui.activate_scrollbar(track.node, &layout.scroll_regions);
                    }
                    moved || changed
                });
                changed && self.scroll_or_exit(event_loop)
            }
            ScrollBehavior::Smooth(tween) if !self.environment.reduced_motion => {
                self.programmatic_scroll = Some(ProgrammaticScroll {
                    tracks,
                    started: Instant::now(),
                    tween,
                });
                self.window.as_ref().is_some_and(|window| {
                    window.request_redraw();
                    true
                })
            }
            ScrollBehavior::Smooth(_) => {
                let ui = self.ui_tree.as_mut().expect("tree checked above");
                let changed = tracks.iter().fold(false, |changed, track| {
                    let moved = ui.set_scroll_offset(track.node, track.to);
                    if moved {
                        ui.activate_scrollbar(track.node, &layout.scroll_regions);
                    }
                    moved || changed
                });
                changed && self.scroll_or_exit(event_loop)
            }
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_programmatic_scroll(
        &mut self,
        window: &Window,
        event_loop: &ActiveEventLoop,
    ) {
        let Some(animation) = &self.programmatic_scroll else {
            return;
        };
        let elapsed = Instant::now()
            .duration_since(animation.started)
            .as_secs_f64();
        let delay = animation.tween.delay.as_secs_f64();
        let duration = animation.tween.duration.as_secs_f64();
        let progress = if elapsed <= delay {
            0.0
        } else if duration <= f64::EPSILON {
            1.0
        } else {
            ((elapsed - delay) / duration).clamp(0.0, 1.0) as f32
        };
        let eased = animation.tween.easing.sample(progress);
        let values = animation
            .tracks
            .iter()
            .map(|track| {
                (
                    track.node,
                    Point::new(
                        track.from.x + (track.to.x - track.from.x) * eased,
                        track.from.y + (track.to.y - track.from.y) * eased,
                    ),
                )
            })
            .collect::<Vec<_>>();
        if let (Some(ui), Some(layout)) = (&mut self.ui_tree, &self.ui_layout) {
            for (node, value) in values {
                if ui.set_scroll_offset(node, value) {
                    ui.activate_scrollbar(node, &layout.scroll_regions);
                }
            }
        }
        self.scroll_or_exit(event_loop);
        if progress >= 1.0 {
            self.programmatic_scroll = None;
        } else {
            window.request_redraw();
        }
    }
}

fn valid_inertia(mut config: InertialScroll) -> InertialScroll {
    let defaults = InertialScroll::default();
    config.velocity_limit = non_negative(config.velocity_limit, defaults.velocity_limit);
    config.stop_velocity = non_negative(config.stop_velocity, defaults.stop_velocity);
    config.decay = non_negative(config.decay, defaults.decay);
    config.sample_weight = if config.sample_weight.is_finite() {
        config.sample_weight.clamp(0.0, 1.0)
    } else {
        defaults.sample_weight
    };
    config
}

fn non_negative(value: f32, fallback: f32) -> f32 {
    if value.is_finite() {
        value.max(0.0)
    } else {
        fallback
    }
}
