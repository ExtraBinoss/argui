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
        let caret = if self.caret.advance(self.focused_animated_caret(), now) {
            TreeUpdate::Paint
        } else {
            TreeUpdate::None
        };
        strongest_updates(strongest_updates(bindings, transitions), caret)
    }

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

    #[must_use]
    pub fn wants_animation_frame(&self) -> bool {
        self.animations
            .entries
            .iter()
            .any(|entry| entry.binding.track().is_active())
            || self.transitions.wants_frame()
            || self.focused_animated_caret().is_some()
    }

    #[must_use]
    pub fn animation_count(&self) -> usize {
        self.animations.entries.len()
    }

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
        _ => TreeUpdate::None,
    }
}

fn strongest_tree_update(current: TreeUpdate, impact: BindingImpact) -> TreeUpdate {
    let next = match impact {
        BindingImpact::Paint => TreeUpdate::Paint,
        BindingImpact::Scroll => TreeUpdate::Scroll,
        BindingImpact::Layout => TreeUpdate::Layout,
    };
    match (current, next) {
        (TreeUpdate::Layout, _) | (_, TreeUpdate::Layout) => TreeUpdate::Layout,
        (TreeUpdate::Scroll, _) | (_, TreeUpdate::Scroll) => TreeUpdate::Scroll,
        (TreeUpdate::Paint, _) | (_, TreeUpdate::Paint) => TreeUpdate::Paint,
        _ => TreeUpdate::None,
    }
}

const fn strongest_impact(left: BindingImpact, right: BindingImpact) -> BindingImpact {
    match (left, right) {
        (BindingImpact::Layout, _) | (_, BindingImpact::Layout) => BindingImpact::Layout,
        (BindingImpact::Scroll, _) | (_, BindingImpact::Scroll) => BindingImpact::Scroll,
        _ => BindingImpact::Paint,
    }
}
