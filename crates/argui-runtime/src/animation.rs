use crate::{AnyEntity, ViewUpdate, app::Application};
use argui_animation::{AnimationId, Clock, Frame, Scheduler, Time};
use argui_ui::{InteractionUpdate, TreeUpdate};
use web_time::Instant;
use winit::{event_loop::ActiveEventLoop, window::Window};

pub(super) struct RuntimeAnimations {
    clock: MonotonicClock,
    scheduler: Scheduler,
    model_animation: Option<AnimationId>,
}

impl RuntimeAnimations {
    pub(super) fn new(model: Option<&AnyEntity>) -> Self {
        let mut animations = Self {
            clock: MonotonicClock::new(),
            scheduler: Scheduler::default(),
            model_animation: None,
        };
        animations.sync(model.is_some_and(AnyEntity::wants_frame));
        animations
    }

    pub(super) fn sync(&mut self, active: bool) -> bool {
        match (active, self.model_animation) {
            (true, None) => {
                self.model_animation = Some(self.scheduler.start());
                true
            }
            (false, Some(id)) => {
                self.scheduler.stop(id);
                self.model_animation = None;
                true
            }
            _ => false,
        }
    }

    /// Samples the platform clock only when active work exists.
    pub(super) fn frame(&mut self) -> Option<Frame> {
        if !self.scheduler.needs_frame() {
            return None;
        }
        self.scheduler.frame(self.clock.now())
    }
}

impl Application {
    pub(super) fn sync_animations(&mut self) -> bool {
        let model_active = self.model.as_ref().is_some_and(AnyEntity::wants_frame);
        let tree_active = self
            .ui_tree
            .as_ref()
            .is_some_and(argui_ui::UiTree::wants_animation_frame);
        self.animations.sync(model_active || tree_active)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn animate(&mut self, window: &Window, _event_loop: &ActiveEventLoop) {
        let Some(frame) = self.animations.frame() else {
            return;
        };
        let model_started = Instant::now();
        let model_update = self.model.as_ref().map_or(ViewUpdate::None, |model| {
            model.animation_frame(frame);
            model.take_effects().update
        });
        let rebuild = model_update == ViewUpdate::Rebuild;
        let model_time = model_started.elapsed();
        let paint_started = Instant::now();
        let tree_animation = self
            .ui_tree
            .as_mut()
            .map_or(TreeUpdate::None, |tree| tree.advance_animations(frame.now));
        let paint_time = paint_started.elapsed();
        self.frame_record.model += model_time;
        self.frame_record.paint += paint_time;
        self.pending_ui_frame.merge(
            &InteractionUpdate {
                paint_changed: tree_animation == TreeUpdate::Paint
                    || model_update == ViewUpdate::Paint,
                layout_changed: tree_animation == TreeUpdate::Layout,
                scroll_changed: tree_animation == TreeUpdate::Scroll,
                ..InteractionUpdate::default()
            },
            rebuild,
        );
        self.sync_animations();
        if self.animations.scheduler.needs_frame() {
            window.request_redraw();
        }
    }
}

struct MonotonicClock {
    origin: Instant,
}

impl MonotonicClock {
    fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }
}

impl Clock for MonotonicClock {
    fn now(&self) -> Time {
        Time::from_nanos(u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX))
    }
}
