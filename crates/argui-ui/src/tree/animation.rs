use crate::{BindingImpact, Element, PropertyBinding, TreeUpdate, traversal::flattened};

use super::UiTree;

#[derive(Clone, Debug)]
pub(super) struct AnimationRegistry {
    entries: Vec<AnimationEntry>,
    layout_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
struct AnimationEntry {
    binding: PropertyBinding,
    impact: BindingImpact,
}

impl AnimationRegistry {
    pub(super) fn new(root: &Element) -> Self {
        let mut entries = Vec::<AnimationEntry>::new();
        let mut layout_indices = Vec::new();
        for (index, element) in flattened(root).into_iter().enumerate() {
            for binding in &element.bindings {
                let identity = binding.track().identity();
                if let Some(existing) = entries
                    .iter_mut()
                    .find(|entry| entry.binding.track().identity() == identity)
                {
                    existing.impact = strongest_impact(existing.impact, binding.impact());
                } else {
                    entries.push(AnimationEntry {
                        binding: binding.clone(),
                        impact: binding.impact(),
                    });
                }
                if binding.impact() == BindingImpact::Layout && !layout_indices.contains(&index) {
                    layout_indices.push(index);
                }
            }
        }
        Self {
            entries,
            layout_indices,
        }
    }

    fn advance(&self, now: argui_animation::Time) -> TreeUpdate {
        self.entries.iter().fold(TreeUpdate::None, |update, entry| {
            if entry.binding.track().advance(now) {
                strongest_tree_update(update, entry.impact)
            } else {
                update
            }
        })
    }

    fn finish_active(&self) -> TreeUpdate {
        let mut update = TreeUpdate::None;
        for entry in &self.entries {
            let track = entry.binding.track();
            if track.is_active() {
                track.finish();
                update = strongest_tree_update(update, entry.impact);
            }
        }
        update
    }
}

impl UiTree {
    /// Advances active bindings, transitions, and the animated caret.
    ///
    /// * `now` — current animation time.
    ///
    /// Returns the strongest required tree update.
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
        strongest_updates(strongest_updates(bindings, transitions), caret)
    }

    /// Enables or disables reduced-motion behavior for this tree.
    ///
    /// * `reduced` — whether active animations should finish immediately.
    ///
    /// Returns the strongest update caused by changing the setting.
    pub fn set_reduced_motion(&mut self, reduced: bool) -> TreeUpdate {
        let caret_changed = self.focused_animated_caret().is_some();
        self.reduced_motion = reduced;
        self.caret.reset();
        let caret = if caret_changed || self.focused_animated_caret().is_some() {
            TreeUpdate::Paint
        } else {
            TreeUpdate::None
        };
        if reduced {
            strongest_updates(
                strongest_updates(self.animations.finish_active(), self.transitions.finish()),
                caret,
            )
        } else {
            caret
        }
    }

    /// Returns whether an animation needs display-linked frames immediately.
    ///
    /// Held caret keyframes return `false` here and expose their one-shot wake
    /// time through [`UiTree::next_animation_frame_at`].
    #[must_use]
    pub fn wants_animation_frame(&self) -> bool {
        let bindings = self
            .animations
            .entries
            .iter()
            .any(|entry| entry.binding.track().is_active());
        let transitions = self.transitions.wants_frame();
        let caret = self
            .focused_animated_caret()
            .is_some_and(|(node, animation)| self.caret.wants_frame(node, animation));
        bindings || transitions || caret
    }

    /// Returns the next one-shot animation deadline for held visual state.
    ///
    /// Continuously interpolated bindings and transitions are represented by
    /// [`UiTree::wants_animation_frame`] instead.
    #[must_use]
    pub fn next_animation_frame_at(&self) -> Option<argui_animation::Time> {
        self.focused_animated_caret()
            .and_then(|(node, animation)| self.caret.next_frame_at(node, animation))
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
        indices.extend(self.transitions.layout_indices(&self.node_ids));
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
