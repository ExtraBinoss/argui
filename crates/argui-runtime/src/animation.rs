use crate::{AnyEntity, ViewUpdate, app::Application};
use argui_animation::{AnimationId, Clock, Frame, Scheduler, Time};
use argui_ui::{InteractionUpdate, TreeUpdate};
use web_time::Instant;

pub(super) struct RuntimeAnimations {
    clock: MonotonicClock,
    scheduler: Scheduler,
    model_animation: Option<AnimationId>,
    model_active: bool,
    wake_at: Option<Time>,
}

impl RuntimeAnimations {
    pub(super) fn new(model: Option<&AnyEntity>) -> Self {
        let mut animations = Self {
            clock: MonotonicClock::new(),
            scheduler: Scheduler::default(),
            model_animation: None,
            model_active: false,
            wake_at: None,
        };
        animations.model_active = model.is_some_and(AnyEntity::wants_frame);
        animations.sync(animations.model_active);
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
        if !self.presentation_visible {
            self.animations.wake_at = None;
            self.animations.model_active = false;
            return self.animations.sync(false);
        }
        let model_active = self.model.as_ref().is_some_and(AnyEntity::wants_frame);
        self.animations.model_active = model_active;
        let tree_active = self
            .ui_tree
            .as_ref()
            .is_some_and(argui_ui::UiTree::wants_animation_frame);
        let active = model_active || tree_active;
        self.animations.wake_at = (!active)
            .then(|| {
                self.ui_tree
                    .as_ref()
                    .and_then(argui_ui::UiTree::next_animation_frame_at)
            })
            .flatten();
        self.animations.sync(active)
    }

    /// Returns the next event-loop wake deadline for a one-shot animation frame.
    pub(crate) fn next_animation_deadline(&self) -> Option<Instant> {
        self.animations
            .wake_at
            .map(|time| self.animations.clock.instant(time))
    }

    /// Activates a single presentation frame when its scheduled deadline is due.
    pub(crate) fn wake_due_animation(&mut self) {
        if !self.presentation_visible || !self.animations.take_due_wake() {
            return;
        }
        self.animations.sync(true);
        if let Some(window) = self.window() {
            window.request_redraw();
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn animate(
        &mut self,
        window: &dyn crate::host::WindowHost,
        event_loop: &dyn crate::host::LoopControl,
    ) {
        let Some(frame) = self.animations.frame() else {
            return;
        };
        let model_started = self.profile_clock();
        let effects = if self.animations.model_active {
            self.model.as_ref().map(|model| {
                model.animation_frame(frame);
                model.take_effects()
            })
        } else {
            None
        };
        let model_update = effects
            .as_ref()
            .map_or(ViewUpdate::None, |effects| effects.update);
        if let Some(effects) = effects {
            self.apply_model_effects(effects, event_loop);
        }
        let model_time = model_started.map_or(std::time::Duration::ZERO, |start| start.elapsed());
        let paint_started = self.profile_clock();
        let tree_animation = self
            .ui_tree
            .as_mut()
            .map_or(TreeUpdate::None, |tree| tree.advance_animations(frame.now));
        let paint_time = paint_started.map_or(std::time::Duration::ZERO, |start| start.elapsed());
        self.frame_record.model += model_time;
        self.frame_record.paint += paint_time;
        self.pending_ui_frame.merge(
            &InteractionUpdate {
                composite_changed: tree_animation == TreeUpdate::Composite,
                paint_changed: tree_animation == TreeUpdate::Paint
                    || model_update == ViewUpdate::Paint,
                layout_changed: tree_animation == TreeUpdate::Layout,
                scroll_changed: tree_animation == TreeUpdate::Scroll,
                ..InteractionUpdate::default()
            },
            false,
        );
        self.sync_animations();
        if self.animations.scheduler.needs_frame() {
            #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
            if let Some(popup) = self.popups.entries.last() {
                popup.native.window().request_redraw();
            } else {
                window.request_redraw();
            }
            #[cfg(not(all(feature = "native-popups", not(target_arch = "wasm32"))))]
            window.request_redraw();
        }
    }
}

impl RuntimeAnimations {
    /// Clears and reports a one-shot wake whose monotonic deadline has elapsed.
    fn take_due_wake(&mut self) -> bool {
        let Some(deadline) = self.wake_at else {
            return false;
        };
        if self.clock.now() < deadline {
            return false;
        }
        self.wake_at = None;
        true
    }
}

struct MonotonicClock {
    origin: Instant,
}

impl MonotonicClock {
    /// Creates a clock rooted at the current platform instant.
    fn new() -> Self {
        Self {
            origin: Instant::now(),
        }
    }

    /// Converts a clock-relative animation timestamp to a platform instant.
    fn instant(&self, time: Time) -> Instant {
        self.origin + std::time::Duration::from_nanos(time.as_nanos())
    }
}

impl Clock for MonotonicClock {
    fn now(&self) -> Time {
        Time::from_nanos(u64::try_from(self.origin.elapsed().as_nanos()).unwrap_or(u64::MAX))
    }
}
