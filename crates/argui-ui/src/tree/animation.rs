use crate::{
    ActivationSource, BindingImpact, Element, NodeId, PropertyBinding, TreeUpdate, UiEventKind,
    traversal::flattened_visible,
};
use argui_animation::{
    CubicBezier, Duration, Easing, FrameSchedule, Keyframe, Keyframes, Motion, Time, Timeline,
    Timing,
};
use argui_core::PointerPhase;
use std::collections::HashMap;

use super::UiTree;

#[derive(Clone, Debug)]
pub(super) struct AnimationRegistry {
    entries: Vec<AnimationEntry>,
    layout_indices: Vec<usize>,
}

/// One native scale pulse for a retained interactive node.
#[derive(Clone, Debug)]
pub(super) struct PressBounce {
    motion: Motion<f32>,
}

impl PressBounce {
    /// Creates an idle pulse at full scale.
    fn new() -> Self {
        Self {
            motion: Motion::new(1.0),
        }
    }

    /// Restarts the full down, overshoot, and return sequence at `minimum` scale.
    fn restart(&self, minimum: f32) {
        let ease = Easing::CubicBezier(
            CubicBezier::new(0.2, 0.8, 0.2, 1.0).expect("constant easing is valid"),
        );
        let frames = Keyframes::new([
            Keyframe::new(0.0, 1.0).easing(ease.clone()),
            Keyframe::new(0.22, minimum).easing(ease.clone()),
            Keyframe::new(0.65, 1.0 + (1.0 - minimum) * 0.2).easing(ease),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("constant press keyframes are valid");
        let timeline = Timeline::new(frames, Timing::new(Duration::from_millis(180)))
            .expect("constant press timing is valid");
        self.motion.set(1.0);
        self.motion.play(timeline);
    }

    /// Advances the pulse and returns its compositor or final paint update.
    fn advance(&self, now: Time) -> TreeUpdate {
        if !self.motion.is_active() {
            return TreeUpdate::None;
        }
        let changed = self
            .motion
            .advance(now)
            .expect("validated press timeline advances");
        if !self.motion.is_active() {
            TreeUpdate::Paint
        } else if changed {
            TreeUpdate::Composite
        } else {
            TreeUpdate::None
        }
    }

    /// Returns the scale presented by the current pulse frame.
    pub(super) fn scale(&self) -> f32 {
        self.motion.value()
    }

    /// Returns whether another display-linked pulse frame is needed.
    fn is_active(&self) -> bool {
        self.motion.is_active()
    }
}

#[derive(Clone, Debug)]
struct AnimationEntry {
    binding: PropertyBinding,
    impact: BindingImpact,
}

impl AnimationEntry {
    /// Returns this binding's invalidation, repainting transforms when `settled`.
    /// A final paint refreshes retained text at its resting scale and pixel phase.
    fn impact(&self, settled: bool) -> BindingImpact {
        if settled && matches!(self.binding, PropertyBinding::Transform(_)) {
            strongest_impact(self.impact, BindingImpact::Paint)
        } else {
            self.impact
        }
    }
}

impl AnimationRegistry {
    /// Collects visible bindings from `root`, deduplicating shared motion tracks.
    ///
    /// Returns a registry with each track's strongest invalidation impact and
    /// the preorder indices of layout-affected nodes.
    pub(super) fn new(root: &Element) -> Self {
        let mut entries = Vec::<AnimationEntry>::new();
        let mut entry_indices = HashMap::<usize, usize>::new();
        let mut layout_indices = Vec::new();
        for (index, element) in flattened_visible(root) {
            for binding in &element.bindings {
                let identity = binding.track().identity();
                if let Some(&entry_index) = entry_indices.get(&identity) {
                    let existing = &mut entries[entry_index];
                    existing.impact = strongest_impact(existing.impact, binding.impact());
                } else {
                    entry_indices.insert(identity, entries.len());
                    entries.push(AnimationEntry {
                        binding: binding.clone(),
                        impact: binding.impact(),
                    });
                }
                if binding.impact() == BindingImpact::Layout
                    && layout_indices.last().copied() != Some(index)
                {
                    layout_indices.push(index);
                }
            }
        }
        Self {
            entries,
            layout_indices,
        }
    }

    /// Samples each track at `now`, returning its strongest update including settled transforms.
    fn advance(&self, now: argui_animation::Time) -> TreeUpdate {
        self.entries.iter().fold(TreeUpdate::None, |update, entry| {
            let track = entry.binding.track();
            let was_active = track.is_active();
            let changed = track.advance(now);
            let settled = was_active && !track.is_active();
            if changed || settled {
                strongest_tree_update(update, entry.impact(settled))
            } else {
                update
            }
        })
    }

    /// Finishes active tracks and returns the invalidation for their final presentation.
    fn finish_active(&self) -> TreeUpdate {
        let mut update = TreeUpdate::None;
        for entry in &self.entries {
            let track = entry.binding.track();
            if track.is_active() {
                track.finish();
                update = strongest_tree_update(update, entry.impact(true));
            }
        }
        update
    }
}

impl UiTree {
    /// Replays the native bounce for pointer presses and non-pointer activations.
    pub(in crate::tree) fn start_press_bounces(
        &mut self,
        events: &[(NodeId, UiEventKind)],
    ) -> TreeUpdate {
        if self.reduced_motion {
            return TreeUpdate::None;
        }
        let mut update = TreeUpdate::None;
        for (node, event) in events {
            let activated = matches!(event, UiEventKind::Pointer(pointer) if pointer.phase == PointerPhase::Pressed)
                || matches!(event, UiEventKind::Click(click) if !matches!(click.source, ActivationSource::Pointer(_)));
            if !activated {
                continue;
            }
            let Some(minimum) = self
                .element_for(*node)
                .and_then(|element| element.interaction.as_ref())
                .filter(|interaction| interaction.enabled)
                .and_then(|interaction| interaction.press_bounce_scale)
                .filter(|scale| scale.is_finite() && *scale > 0.0 && *scale <= 1.0)
            else {
                continue;
            };
            self.press_bounces
                .entry(*node)
                .or_insert_with(PressBounce::new)
                .restart(minimum);
            update = TreeUpdate::Composite;
        }
        update
    }

    /// Advances active bindings, transitions, and the animated caret.
    ///
    /// * `now` — current animation time.
    ///
    /// Returns the strongest required tree update. Transform animations request
    /// a paint when they settle so retained text is rasterized at its final scale.
    pub fn advance_animations(&mut self, now: argui_animation::Time) -> TreeUpdate {
        let bindings = if self.reduced_motion {
            self.animations.finish_active()
        } else {
            self.animations.advance(now)
        };
        let transitions = if self.reduced_motion {
            self.transitions.finish()
        } else {
            self.transitions.advance(now)
        };
        let mut press_bounces = TreeUpdate::None;
        self.press_bounces.retain(|_, bounce| {
            press_bounces = strongest_updates(press_bounces, bounce.advance(now));
            bounce.is_active()
        });
        let caret_animation = self.focused_animated_caret();
        let caret_composited =
            caret_animation.is_some_and(|(_, animation)| animation.supports_composition());
        let caret_node = caret_animation.map(|(node, _)| node);
        let caret = if self.caret.advance(caret_node, now) {
            if caret_composited {
                TreeUpdate::Composite
            } else {
                TreeUpdate::Paint
            }
        } else {
            TreeUpdate::None
        };
        strongest_updates(
            strongest_updates(bindings, transitions),
            strongest_updates(press_bounces, caret),
        )
    }

    /// Enables or disables reduced-motion behavior for this tree.
    ///
    /// * `reduced` — whether active animations should finish immediately.
    ///
    /// Returns the strongest update caused by changing the setting.
    pub fn set_reduced_motion(&mut self, reduced: bool) -> TreeUpdate {
        let caret_changed = self.focused_animated_caret().is_some();
        self.reduced_motion = reduced;
        let scroll_widths = if reduced && self.scroll.finish_hover_widths() {
            TreeUpdate::Scroll
        } else {
            TreeUpdate::None
        };
        let press_bounces = if reduced && !self.press_bounces.is_empty() {
            self.press_bounces.clear();
            TreeUpdate::Paint
        } else {
            TreeUpdate::None
        };
        self.caret.reset();
        let caret = if caret_changed || self.focused_animated_caret().is_some() {
            TreeUpdate::Paint
        } else {
            TreeUpdate::None
        };
        if reduced {
            strongest_updates(
                scroll_widths,
                strongest_updates(
                    press_bounces,
                    strongest_updates(
                        strongest_updates(
                            self.animations.finish_active(),
                            self.transitions.finish(),
                        ),
                        caret,
                    ),
                ),
            )
        } else {
            strongest_updates(scroll_widths, strongest_updates(press_bounces, caret))
        }
    }

    /// Returns whether an animation needs display-linked frames immediately.
    ///
    /// Held keyframes and step easing return `false` between visual changes
    /// and expose one-shot wakes through [`UiTree::next_animation_frame_at`].
    #[must_use]
    pub fn wants_animation_frame(&self) -> bool {
        let bindings = self.animations.entries.iter().any(|entry| {
            let track = entry.binding.track();
            (self.reduced_motion && track.is_active())
                || track.frame_schedule() == FrameSchedule::Continuous
        });
        let transitions = self.transitions.wants_frame();
        let caret = self
            .focused_animated_caret()
            .is_some_and(|(node, animation)| self.caret.wants_frame(node, animation));
        bindings || transitions || caret || !self.press_bounces.is_empty()
    }

    /// Returns the earliest one-shot deadline for held visual state.
    ///
    /// Continuously interpolated bindings and transitions are represented by
    /// [`UiTree::wants_animation_frame`] instead.
    #[must_use]
    pub fn next_animation_frame_at(&self) -> Option<argui_animation::Time> {
        let bindings = (!self.reduced_motion)
            .then(|| {
                self.animations
                    .entries
                    .iter()
                    .filter_map(|entry| match entry.binding.track().frame_schedule() {
                        FrameSchedule::At(time) => Some(time),
                        FrameSchedule::None | FrameSchedule::Continuous => None,
                    })
                    .min()
            })
            .flatten();
        let caret = self
            .focused_animated_caret()
            .and_then(|(node, animation)| self.caret.next_frame_at(node, animation));
        bindings.into_iter().chain(caret).min()
    }

    /// Returns the number of active property animations.
    #[must_use]
    pub fn animation_count(&self) -> usize {
        self.animations.entries.len()
    }

    /// Returns preorder indices whose layout is affected by active animations.
    #[must_use]
    pub fn layout_animation_indices(&self) -> Vec<usize> {
        let mut indices = self.animations.layout_indices.clone();
        indices.extend(self.transitions.layout_indices(&self.index));
        indices.sort_unstable();
        indices.dedup();
        indices
    }
}

fn strongest_updates(left: TreeUpdate, right: TreeUpdate) -> TreeUpdate {
    match (left, right) {
        (TreeUpdate::Layout, _) | (_, TreeUpdate::Layout) => TreeUpdate::Layout,
        (TreeUpdate::Scroll, _) | (_, TreeUpdate::Scroll) => TreeUpdate::Scroll,
        (TreeUpdate::Paint, _) | (_, TreeUpdate::Paint) => TreeUpdate::Paint,
        (TreeUpdate::Composite, _) | (_, TreeUpdate::Composite) => TreeUpdate::Composite,
        _ => TreeUpdate::None,
    }
}

fn strongest_tree_update(current: TreeUpdate, impact: BindingImpact) -> TreeUpdate {
    let next = match impact {
        BindingImpact::Composite => TreeUpdate::Composite,
        BindingImpact::Paint => TreeUpdate::Paint,
        BindingImpact::Scroll => TreeUpdate::Scroll,
        BindingImpact::Layout => TreeUpdate::Layout,
    };
    match (current, next) {
        (TreeUpdate::Layout, _) | (_, TreeUpdate::Layout) => TreeUpdate::Layout,
        (TreeUpdate::Scroll, _) | (_, TreeUpdate::Scroll) => TreeUpdate::Scroll,
        (TreeUpdate::Paint, _) | (_, TreeUpdate::Paint) => TreeUpdate::Paint,
        (TreeUpdate::Composite, _) | (_, TreeUpdate::Composite) => TreeUpdate::Composite,
        _ => TreeUpdate::None,
    }
}

const fn strongest_impact(left: BindingImpact, right: BindingImpact) -> BindingImpact {
    match (left, right) {
        (BindingImpact::Layout, _) | (_, BindingImpact::Layout) => BindingImpact::Layout,
        (BindingImpact::Scroll, _) | (_, BindingImpact::Scroll) => BindingImpact::Scroll,
        (BindingImpact::Paint, _) | (_, BindingImpact::Paint) => BindingImpact::Paint,
        _ => BindingImpact::Composite,
    }
}
