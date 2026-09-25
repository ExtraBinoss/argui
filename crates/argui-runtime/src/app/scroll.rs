use argui_core::{Point, ScrollDelta};
use argui_ui::{NodeId, ScrollBehavior, ScrollPhysics, ScrollRequest};
use web_time::Instant;
use winit::event::TouchPhase;

use crate::{
    RuntimeError, RuntimeEvent,
    app::{Application, inertia::ScrollSample, touch_scroll::TouchScrollUpdate},
};

mod request;
use request::{ScrollTrack, scroll_tracks};

#[derive(Clone, Copy, Debug)]
pub(super) struct PendingScroll {
    delta: ScrollDelta,
    point: Point,
    dispatch_wheel: bool,
    target: Option<NodeId>,
}

#[derive(Debug)]
pub(super) struct ProgrammaticScroll {
    tracks: Vec<ScrollTrack>,
    started: Instant,
    tween: argui_animation::Tween,
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
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
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
        let config = layout
            .scroll_regions
            .iter()
            .find(|region| Some(region.node) == target)
            .map(|region| &region.config);
        let physics = config.map_or(ScrollPhysics::Direct, |config| config.physics);
        let inertia_delta = match delta {
            ScrollDelta::Lines(lines) => {
                let line_size = config.map_or(40.0, |config| config.line_size.max(0.0));
                ScrollDelta::Pixels(Point::new(lines.x * line_size, lines.y * line_size))
            }
            ScrollDelta::Pixels(_) => delta,
        };
        self.scroll_inertia.observe(ScrollSample {
            delta: inertia_delta,
            phase,
            now: self.input_epoch.elapsed(),
            target,
            physics,
            point,
            dispatch_wheel: true,
            reduced_motion: self.environment.reduced_motion,
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
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        if self.environment.reduced_motion {
            self.scroll_inertia.cancel();
        }
        let target = self.scroll_inertia.target();
        if let Some((delta, point, dispatch_wheel)) =
            self.scroll_inertia.advance(self.input_epoch.elapsed())
        {
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

    pub(super) fn apply_touch_scroll(
        &mut self,
        update: TouchScrollUpdate,
        point: Point,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let delta = super::inertia::touch_scroll_delta(update.delta, update.natural);
        if matches!(update.phase, TouchPhase::Started | TouchPhase::Moved)
            && (delta.x.abs() > f32::EPSILON || delta.y.abs() > f32::EPSILON)
        {
            self.programmatic_scroll = None;
            if let (Some(layout), Some(ui)) = (&self.ui_layout, &mut self.ui_tree) {
                let scroll = ui.scroll_from(
                    update.target,
                    point,
                    ScrollDelta::Pixels(delta),
                    &layout.scroll_regions,
                );
                self.apply_ui_update(scroll, window, event_loop);
            }
        }
        self.scroll_inertia.observe(ScrollSample {
            delta: ScrollDelta::Pixels(delta),
            phase: update.phase,
            now: self.input_epoch.elapsed(),
            target: Some(update.target),
            physics: update.physics,
            point,
            dispatch_wheel: false,
            reduced_motion: self.environment.reduced_motion,
        });
        if self.scroll_inertia.needs_frame() {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn advance_scroll_physics(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
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
        if !update.is_empty() {
            self.apply_ui_update(update, window, event_loop);
        }
        if active {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn flush_pointer_scroll(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let Some(pending) = self.pending_pointer_scroll.take() else {
            return;
        };
        let point = pending.point;
        let delta = pending.delta;
        if pending.dispatch_wheel {
            // Deliver the event to the pointer hit target; default scrolling still
            // uses the independently latched scroll-region target below.
            let wheel_update = match (&self.ui_layout, &mut self.ui_tree) {
                (Some(layout), Some(ui)) => layout
                    .hit_regions
                    .iter()
                    .rev()
                    .find(|region| region.enabled && region.contains(point))
                    .map(|region| region.node)
                    .or(pending.target)
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
        let did_scroll = update.scroll_changed;
        self.apply_ui_update(update, window, event_loop);
        if !did_scroll {
            self.scroll_inertia.cancel();
        }
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
    pub(super) fn scroll_or_exit(&mut self, event_loop: &dyn crate::host::LoopControl) -> bool {
        let result = match (&mut self.ui_tree, &mut self.ui_layout) {
            (Some(ui), Some(layout)) => {
                self.layout_engine
                    .apply_scroll_with_text(ui, &mut self.text_engine, layout)
            }
            _ => return false,
        };
        let refreshed_text = match result {
            Ok(refreshed) => refreshed,
            Err(error) => {
                (self.on_event)(RuntimeEvent::LayoutFailed(error.to_string()));
                self.fatal_error = Some(RuntimeError::from(error));
                event_loop.exit();
                return false;
            }
        };
        // A scroll repaints primitive geometry even when a compositor motion
        // was sampled earlier in this redraw cycle.
        self.composite_frame = false;
        if refreshed_text {
            self.prepared_text = self
                .ui_layout
                .as_ref()
                .map(|layout| self.text_engine.prepare(&layout.text, self.scale_factor));
        } else if let (Some(prepared), Some(layout)) = (&mut self.prepared_text, &self.ui_layout) {
            for (index, block) in layout.text.blocks().iter().enumerate() {
                prepared.reposition_block(index, block.bounds.origin, block.clip);
            }
        }
        #[cfg(feature = "inspect")]
        self.publish_inspection();
        #[cfg(feature = "inspect")]
        self.paint_inspection_highlight();
        true
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn apply_scroll_request(
        &mut self,
        request: ScrollRequest,
        event_loop: &dyn crate::host::LoopControl,
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
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
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
