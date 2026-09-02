use crate::state::StateValue;
use crate::{
    BindingImpact, Element, NodeId, PropertyKey, StyleCondition, StylePropertyValue,
    TransitionDirection, TreeUpdate, VisualStates,
};
use argui_animation::{Motion, MotionTrack, Time, Transition};
use argui_core::{Color, Point, Transform2D};

mod style;
use style::collect_specs;
pub(super) use style::{
    apply_layer, apply_layout, apply_quad, apply_scroll, apply_scrollbar_part, apply_text_color,
    apply_transform, apply_vector_color,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum TransitionTarget {
    Element(NodeId),
    ScrollbarTrack(NodeId),
    ScrollbarThumb(NodeId),
}

#[derive(Clone, Debug, Default)]
pub(super) struct TransitionRegistry {
    entries: Vec<NodeTransition>,
}

#[derive(Clone, Debug)]
struct NodeTransition {
    target: TransitionTarget,
    matched: Vec<StyleCondition>,
    values: Vec<AnimatedProperty>,
    revision: u64,
}

#[derive(Clone, Debug)]
struct AnimatedProperty {
    key: PropertyKey,
    target: StateValue,
    source: Option<StyleCondition>,
    value: AnimatedValue,
}

#[derive(Clone, Debug)]
enum AnimatedValue {
    Transform(Motion<Transform2D>),
    F32(Motion<f32>),
    Color(Motion<Color>),
    Point(Motion<Point>),
    Vec2(Motion<[f32; 2]>),
    Vec3(Motion<[f32; 3]>),
    Vec4(Motion<[f32; 4]>),
    Mat3(Motion<[f32; 9]>),
    Mat4(Motion<[f32; 16]>),
    Discrete {
        from: StateValue,
        target: StateValue,
        progress: Motion<f32>,
    },
}

struct TransitionSync<'a> {
    root: &'a Element,
    node_ids: &'a [NodeId],
    states_for: &'a dyn Fn(NodeId) -> VisualStates,
    scrollbar_states_for: &'a dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    scroll_for: &'a dyn Fn(NodeId) -> Point,
    container_size: &'a dyn Fn(NodeId) -> Option<argui_core::Size>,
    reduced_motion: bool,
}

impl TransitionRegistry {
    fn sync(&mut self, input: TransitionSync<'_>) -> TreeUpdate {
        let mut specs = Vec::new();
        collect_specs(
            input.root,
            input.node_ids,
            input.states_for,
            input.scrollbar_states_for,
            input.scroll_for,
            input.container_size,
            &mut specs,
        );
        let mut update = TreeUpdate::None;
        self.entries
            .retain(|entry| specs.iter().any(|spec| spec.target == entry.target));
        for spec in specs {
            let Some(entry) = self
                .entries
                .iter_mut()
                .find(|entry| entry.target == spec.target)
            else {
                self.entries.push(NodeTransition::new(spec));
                continue;
            };
            update = strongest(update, entry.sync(spec, input.reduced_motion));
        }
        update
    }

    pub(super) fn advance(&mut self, now: Time) -> TreeUpdate {
        self.entries
            .iter_mut()
            .fold(TreeUpdate::None, |update, entry| {
                strongest(update, entry.advance(now))
            })
    }

    pub(super) fn finish(&mut self) -> TreeUpdate {
        self.entries
            .iter_mut()
            .fold(TreeUpdate::None, |update, entry| {
                strongest(update, entry.finish())
            })
    }

    pub(super) fn wants_frame(&self) -> bool {
        self.entries.iter().any(NodeTransition::is_active)
    }

    pub(super) fn set_scroll(&mut self, node: NodeId, offset: Point) {
        let Some(property) = self
            .entries
            .iter_mut()
            .find(|entry| entry.target == TransitionTarget::Element(node))
            .and_then(|entry| {
                entry
                    .values
                    .iter_mut()
                    .find(|value| value.key == PropertyKey::Scroll)
            })
        else {
            return;
        };
        property.target = StateValue::Point(offset);
        property.value.set(StateValue::Point(offset));
    }

    fn visit(&self, target: TransitionTarget, mut visit: impl FnMut(PropertyKey, StateValue)) {
        if let Some(entry) = self.entries.iter().find(|entry| entry.target == target) {
            for property in &entry.values {
                visit(property.key, property.value.value());
            }
        }
    }

    pub(super) fn layout_indices(&self, ids: &[NodeId]) -> Vec<usize> {
        self.entries
            .iter()
            .filter(|entry| {
                entry
                    .values
                    .iter()
                    .any(|value| value.key.impact() == BindingImpact::Layout)
            })
            .filter_map(|entry| match entry.target {
                TransitionTarget::Element(node) => {
                    ids.iter().position(|candidate| *candidate == node)
                }
                TransitionTarget::ScrollbarTrack(_) | TransitionTarget::ScrollbarThumb(_) => None,
            })
            .collect()
    }

    pub(super) fn revision(&self, node: NodeId) -> u64 {
        self.entries
            .iter()
            .find(|entry| entry.target == TransitionTarget::Element(node))
            .map_or(0, |entry| entry.revision)
    }
}

impl super::UiTree {
    pub(super) fn sync_transitions(&mut self) -> TreeUpdate {
        self.sync_transitions_with(self.reduced_motion)
    }

    pub(super) fn sync_transitions_with(&mut self, reduced_motion: bool) -> TreeUpdate {
        let interaction = &self.interaction;
        let scroll = &self.scroll;
        let container_sizes = &self.container_sizes;
        let states_for = |node| interaction.visual_states(node);
        let scrollbar_states_for = |node, part, enabled| scroll.visual_states(node, part, enabled);
        let scroll_for = |node| scroll.offset(node);
        let container_size = |node| {
            container_sizes
                .iter()
                .find(|(id, _)| *id == node)
                .map(|(_, size)| *size)
        };
        self.transitions.sync(TransitionSync {
            root: &self.root,
            node_ids: &self.node_ids,
            states_for: &states_for,
            scrollbar_states_for: &scrollbar_states_for,
            scroll_for: &scroll_for,
            container_size: &container_size,
            reduced_motion,
        })
    }

    #[must_use]
    pub fn visual_revision(&self, node: NodeId) -> u64 {
        self.transitions.revision(node)
    }
}

struct NodeSpec<'a> {
    target: TransitionTarget,
    matched: Vec<StyleCondition>,
    values: Vec<ResolvedProperty>,
    transition: Option<&'a crate::StyleTransition>,
}

#[derive(Clone, Debug)]
struct ResolvedProperty {
    property: StylePropertyValue,
    source: Option<StyleCondition>,
}

impl NodeTransition {
    fn new(spec: NodeSpec<'_>) -> Self {
        Self {
            target: spec.target,
            matched: spec.matched,
            revision: 0,
            values: spec
                .values
                .into_iter()
                .map(|resolved| AnimatedProperty {
                    key: resolved.property.key,
                    target: resolved.property.value.clone(),
                    source: resolved.source,
                    value: AnimatedValue::new(resolved.property.value),
                })
                .collect(),
        }
    }

    fn sync(&mut self, spec: NodeSpec<'_>, reduced_motion: bool) -> TreeUpdate {
        self.values.retain(|property| {
            spec.values
                .iter()
                .any(|value| value.property.key == property.key)
        });
        let mut update = TreeUpdate::None;
        for resolved in spec.values {
            let target = resolved.property;
            let Some(property) = self.values.iter_mut().find(|value| value.key == target.key)
            else {
                self.values.push(AnimatedProperty {
                    key: target.key,
                    target: target.value.clone(),
                    source: resolved.source,
                    value: AnimatedValue::new(target.value),
                });
                update = strongest(update, target.key.impact().into());
                continue;
            };
            if property.target == target.value {
                property.source = resolved.source;
                continue;
            }
            let direction = property_direction(
                &self.matched,
                &spec.matched,
                property.source.as_ref(),
                resolved.source.as_ref(),
            );
            property.target = target.value.clone();
            property.source = resolved.source;
            if reduced_motion || spec.transition.is_none() {
                property.value.set(target.value);
            } else if let Some(style_transition) = spec.transition {
                property.value.retarget(
                    target.value,
                    style_transition.resolve(target.key, direction),
                );
            }
            update = strongest(update, target.key.impact().into());
        }
        self.matched = spec.matched;
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    fn advance(&mut self, now: Time) -> TreeUpdate {
        let update = self
            .values
            .iter()
            .fold(TreeUpdate::None, |update, property| {
                if property.value.advance(now) {
                    strongest(update, property.key.impact().into())
                } else {
                    update
                }
            });
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    fn finish(&mut self) -> TreeUpdate {
        let update = self
            .values
            .iter()
            .fold(TreeUpdate::None, |update, property| {
                if property.value.finish() {
                    strongest(update, property.key.impact().into())
                } else {
                    update
                }
            });
        if update != TreeUpdate::None {
            self.revision = self.revision.wrapping_add(1);
        }
        update
    }

    fn is_active(&self) -> bool {
        self.values
            .iter()
            .any(|property| property.value.is_active())
    }
}

impl AnimatedValue {
    fn new(value: StateValue) -> Self {
        match value {
            StateValue::Transform(value) => Self::Transform(Motion::new(value)),
            StateValue::Background(_) | StateValue::Border(_) => Self::Discrete {
                from: value.clone(),
                target: value,
                progress: Motion::new(1.0),
            },
            StateValue::BackgroundColor(value)
            | StateValue::BorderColor(value)
            | StateValue::Color(value) => Self::Color(Motion::new(value)),
            StateValue::BorderWidths(value)
            | StateValue::CornerRadii(value)
            | StateValue::Vec4(value) => Self::Vec4(Motion::new(value)),
            StateValue::Opacity(value) | StateValue::F32(value) => Self::F32(Motion::new(value)),
            StateValue::Point(value) => Self::Point(Motion::new(value)),
            StateValue::Vec2(value) => Self::Vec2(Motion::new(value)),
            StateValue::Vec3(value) => Self::Vec3(Motion::new(value)),
            StateValue::Mat3(value) => Self::Mat3(Motion::new(value)),
            StateValue::Mat4(value) => Self::Mat4(Motion::new(value)),
            StateValue::LayoutStyle(value) => Self::Discrete {
                from: StateValue::LayoutStyle(value.clone()),
                target: StateValue::LayoutStyle(value),
                progress: Motion::new(1.0),
            },
        }
    }

    fn value(&self) -> StateValue {
        match self {
            Self::Transform(value) => StateValue::Transform(value.value()),
            Self::F32(value) => StateValue::F32(value.value()),
            Self::Color(value) => StateValue::Color(value.value()),
            Self::Point(value) => StateValue::Point(value.value()),
            Self::Vec2(value) => StateValue::Vec2(value.value()),
            Self::Vec3(value) => StateValue::Vec3(value.value()),
            Self::Vec4(value) => StateValue::Vec4(value.value()),
            Self::Mat3(value) => StateValue::Mat3(value.value()),
            Self::Mat4(value) => StateValue::Mat4(value.value()),
            Self::Discrete {
                from,
                target,
                progress,
            } => {
                if progress.value() < 0.5 {
                    from.clone()
                } else {
                    target.clone()
                }
            }
        }
    }

    fn set(&mut self, target: StateValue) {
        *self = Self::new(target);
    }

    fn retarget(&mut self, target: StateValue, transition: &Transition) {
        if matches!(target, StateValue::LayoutStyle(_)) {
            self.set(target);
            return;
        }
        match (self, target) {
            (Self::Transform(motion), StateValue::Transform(value)) => {
                transition.retarget(motion, value)
            }
            (Self::F32(motion), StateValue::Opacity(value) | StateValue::F32(value)) => {
                transition.retarget(motion, value)
            }
            (
                Self::Color(motion),
                StateValue::BackgroundColor(value)
                | StateValue::BorderColor(value)
                | StateValue::Color(value),
            ) => transition.retarget(motion, value),
            (Self::Point(motion), StateValue::Point(value)) => transition.retarget(motion, value),
            (Self::Vec2(motion), StateValue::Vec2(value)) => transition.retarget(motion, value),
            (Self::Vec3(motion), StateValue::Vec3(value)) => transition.retarget(motion, value),
            (
                Self::Vec4(motion),
                StateValue::BorderWidths(value)
                | StateValue::CornerRadii(value)
                | StateValue::Vec4(value),
            ) => transition.retarget(motion, value),
            (Self::Mat3(motion), StateValue::Mat3(value)) => transition.retarget(motion, value),
            (Self::Mat4(motion), StateValue::Mat4(value)) => transition.retarget(motion, value),
            (slot, target) => {
                let from = slot.value();
                let progress = Motion::new(0.0);
                transition.retarget(&progress, 1.0);
                *slot = Self::Discrete {
                    from,
                    target,
                    progress,
                };
            }
        }
    }

    fn track(&self) -> &dyn MotionTrack {
        match self {
            Self::Transform(value) => value,
            Self::F32(value) => value,
            Self::Color(value) => value,
            Self::Point(value) => value,
            Self::Vec2(value) => value,
            Self::Vec3(value) => value,
            Self::Vec4(value) => value,
            Self::Mat3(value) => value,
            Self::Mat4(value) => value,
            Self::Discrete { progress, .. } => progress,
        }
    }

    fn advance(&self, now: Time) -> bool {
        self.track().advance(now)
    }

    fn finish(&self) -> bool {
        if self.track().is_active() {
            self.track().finish();
            true
        } else {
            false
        }
    }

    fn is_active(&self) -> bool {
        self.track().is_active()
    }
}

fn property_direction(
    old: &[StyleCondition],
    new: &[StyleCondition],
    old_source: Option<&StyleCondition>,
    new_source: Option<&StyleCondition>,
) -> Option<TransitionDirection> {
    if old_source == new_source {
        return None;
    }
    if let Some(condition) = new_source.filter(|condition| !old.contains(condition)) {
        return Some(TransitionDirection::Enter(condition.clone()));
    }
    old_source
        .filter(|condition| !new.contains(condition))
        .map(|condition| TransitionDirection::Exit(condition.clone()))
}

impl From<BindingImpact> for TreeUpdate {
    fn from(value: BindingImpact) -> Self {
        match value {
            BindingImpact::Paint => Self::Paint,
            BindingImpact::Layout => Self::Layout,
            BindingImpact::Scroll => Self::Scroll,
        }
    }
}

fn strongest(left: TreeUpdate, right: TreeUpdate) -> TreeUpdate {
    match (left, right) {
        (TreeUpdate::Layout, _) | (_, TreeUpdate::Layout) => TreeUpdate::Layout,
        (TreeUpdate::Scroll, _) | (_, TreeUpdate::Scroll) => TreeUpdate::Scroll,
        (TreeUpdate::Paint, _) | (_, TreeUpdate::Paint) => TreeUpdate::Paint,
        (TreeUpdate::Semantics, _) | (_, TreeUpdate::Semantics) => TreeUpdate::Semantics,
        _ => TreeUpdate::None,
    }
}
