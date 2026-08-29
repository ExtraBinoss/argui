use argui_animation::{
    Duration, Easing, FillMode, Interpolate, Keyframe, Keyframes, Time, Timeline, Timing,
};
use argui_core::Transform2D;
use argui_paint::{
    Border, BorderWidths, Color, CornerRadii, Fill, GradientStop, GradientStops, LinearGradient,
    QuadStyle, RadialGradient,
};

use crate::{Element, NodeId};

#[derive(Clone, Debug, PartialEq)]
pub struct Transition {
    pub duration: Duration,
    pub delay: Duration,
    pub easing: Easing,
}

impl Transition {
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self {
            duration,
            delay: Duration::ZERO,
            easing: Easing::Linear,
        }
    }

    #[must_use]
    pub const fn delay(mut self, delay: Duration) -> Self {
        self.delay = delay;
        self
    }

    #[must_use]
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PaintTransitions {
    entries: Vec<Entry>,
    reduced_motion: bool,
}

#[derive(Clone, Debug)]
struct Entry {
    node: NodeId,
    displayed: QuadStyle,
    displayed_transform: Transform2D,
    from: QuadStyle,
    from_transform: Transform2D,
    target: QuadStyle,
    target_transform: Transform2D,
    pending: Option<Transition>,
    timeline: Option<Timeline<f32>>,
}

impl PaintTransitions {
    pub(crate) fn set_reduced_motion(&mut self, reduced: bool) -> bool {
        self.reduced_motion = reduced;
        let changed = reduced && !self.entries.is_empty();
        if changed {
            self.entries.clear();
        }
        changed
    }

    pub(crate) fn sync(
        &mut self,
        old_root: &Element,
        old_ids: &[NodeId],
        new_root: &Element,
        new_ids: &[NodeId],
    ) {
        if self.reduced_motion {
            self.entries.clear();
            return;
        }
        let old = flattened(old_root);
        for (new, node) in flattened(new_root).into_iter().zip(new_ids.iter().copied()) {
            let Some(transition) = new.transition.clone() else {
                self.remove(node);
                continue;
            };
            let Some(old_index) = old_ids.iter().position(|candidate| *candidate == node) else {
                continue;
            };
            let old_style = old[old_index].paint.quad.clone();
            if old_style != new.paint.quad || old[old_index].transform != new.transform {
                self.queue(
                    node,
                    old_style,
                    old[old_index].transform,
                    new.paint.quad.clone(),
                    new.transform,
                    transition,
                );
            }
        }
        self.entries.retain(|entry| new_ids.contains(&entry.node));
    }

    fn queue(
        &mut self,
        node: NodeId,
        old: QuadStyle,
        old_transform: Transform2D,
        target: QuadStyle,
        target_transform: Transform2D,
        transition: Transition,
    ) {
        if transition.duration == Duration::ZERO {
            self.remove(node);
            return;
        }
        if let Some(entry) = self.entries.iter_mut().find(|entry| entry.node == node) {
            if entry.target != target || entry.target_transform != target_transform {
                entry.from = entry.displayed.clone();
                entry.from_transform = entry.displayed_transform;
                entry.target = target;
                entry.target_transform = target_transform;
                entry.pending = Some(transition);
                entry.timeline = None;
            }
            return;
        }
        self.entries.push(Entry {
            node,
            displayed: old.clone(),
            displayed_transform: old_transform,
            from: old,
            from_transform: old_transform,
            target,
            target_transform,
            pending: Some(transition),
            timeline: None,
        });
    }

    pub(crate) fn advance(&mut self, now: Time) -> bool {
        let mut changed = false;
        for entry in &mut self.entries {
            if let Some(transition) = entry.pending.take() {
                entry.timeline = make_timeline(&transition, now);
                changed = true;
            }
            let Some(timeline) = &mut entry.timeline else {
                continue;
            };
            if let Some(progress) = timeline.sample(now).value {
                let displayed =
                    interpolate_quad(entry.from.clone(), entry.target.clone(), progress);
                changed |= displayed != entry.displayed;
                entry.displayed = displayed;
                let displayed_transform = entry
                    .from_transform
                    .interpolate(entry.target_transform, progress);
                changed |= displayed_transform != entry.displayed_transform;
                entry.displayed_transform = displayed_transform;
            }
        }
        self.entries.retain(|entry| {
            entry.pending.is_some() || entry.timeline.as_ref().is_some_and(Timeline::needs_frame)
        });
        changed
    }

    pub(crate) fn resolve(&self, node: NodeId, target: QuadStyle) -> QuadStyle {
        self.entries
            .iter()
            .find(|entry| entry.node == node)
            .map_or(target, |entry| entry.displayed.clone())
    }

    pub(crate) fn resolve_transform(&self, node: NodeId, target: Transform2D) -> Transform2D {
        self.entries
            .iter()
            .find(|entry| entry.node == node)
            .map_or(target, |entry| entry.displayed_transform)
    }

    pub(crate) fn needs_frame(&self) -> bool {
        !self.entries.is_empty()
    }

    fn remove(&mut self, node: NodeId) {
        self.entries.retain(|entry| entry.node != node);
    }
}

fn make_timeline(transition: &Transition, now: Time) -> Option<Timeline<f32>> {
    let frames = Keyframes::new(vec![
        Keyframe::new(0.0, 0.0).easing(transition.easing.clone()),
        Keyframe::new(1.0, 1.0),
    ])
    .ok()?;
    let mut timeline = Timeline::new(
        frames,
        Timing::new(transition.duration)
            .delay(transition.delay)
            .fill(FillMode::Both),
    )
    .ok()?;
    timeline.play(now);
    Some(timeline)
}

fn interpolate_quad(from: QuadStyle, target: QuadStyle, progress: f32) -> QuadStyle {
    QuadStyle {
        background: interpolate_fill(from.background, target.background, progress),
        border: interpolate_border(from.border, target.border, progress),
        radii: CornerRadii {
            top_left: from
                .radii
                .top_left
                .interpolate(target.radii.top_left, progress),
            top_right: from
                .radii
                .top_right
                .interpolate(target.radii.top_right, progress),
            bottom_right: from
                .radii
                .bottom_right
                .interpolate(target.radii.bottom_right, progress),
            bottom_left: from
                .radii
                .bottom_left
                .interpolate(target.radii.bottom_left, progress),
        },
        opacity: from.opacity.interpolate(target.opacity, progress),
    }
}

fn interpolate_fill(from: Option<Fill>, target: Option<Fill>, progress: f32) -> Option<Fill> {
    if from.is_none() && target.is_none() {
        return None;
    }
    if from == target {
        return from;
    }
    if let (Some(from), Some(target)) = (&from, &target)
        && let Some(fill) = interpolate_gradient(from, target, progress)
    {
        return Some(fill);
    }
    let from = from.as_ref().map_or(Color::TRANSPARENT, solid_color);
    let target_color = target.as_ref().map_or(Color::TRANSPARENT, solid_color);
    if progress >= 1.0 && target.is_none() {
        None
    } else {
        Some(Fill::Solid(from.interpolate(target_color, progress)))
    }
}

fn interpolate_gradient(from: &Fill, target: &Fill, progress: f32) -> Option<Fill> {
    match (from, target) {
        (Fill::Linear(from), Fill::Linear(target)) => Some(Fill::Linear(LinearGradient {
            start: from.start.interpolate(target.start, progress),
            end: from.end.interpolate(target.end, progress),
            stops: interpolate_stops(&from.stops, &target.stops, progress)?,
        })),
        (Fill::Radial(from), Fill::Radial(target)) => Some(Fill::Radial(RadialGradient {
            center: from.center.interpolate(target.center, progress),
            radius: from.radius.interpolate(target.radius, progress),
            stops: interpolate_stops(&from.stops, &target.stops, progress)?,
        })),
        _ => None,
    }
}

fn interpolate_stops(
    from: &GradientStops,
    target: &GradientStops,
    progress: f32,
) -> Option<GradientStops> {
    if from.len() != target.len() {
        return None;
    }
    let stops = from
        .as_slice()
        .iter()
        .zip(target.as_slice())
        .map(|(from, target)| {
            GradientStop::new(
                from.offset.interpolate(target.offset, progress),
                from.color.interpolate(target.color, progress),
            )
        })
        .collect();
    GradientStops::from_vec(stops).ok()
}

fn interpolate_border(
    from: Option<Border>,
    target: Option<Border>,
    progress: f32,
) -> Option<Border> {
    if from.is_none() && target.is_none() {
        return None;
    }
    let from = from.unwrap_or(Border::all(0.0, Color::TRANSPARENT));
    let target_border = target.unwrap_or(Border::all(0.0, Color::TRANSPARENT));
    if progress >= 1.0 && target.is_none() {
        None
    } else {
        Some(Border {
            widths: BorderWidths {
                left: from
                    .widths
                    .left
                    .interpolate(target_border.widths.left, progress),
                right: from
                    .widths
                    .right
                    .interpolate(target_border.widths.right, progress),
                top: from
                    .widths
                    .top
                    .interpolate(target_border.widths.top, progress),
                bottom: from
                    .widths
                    .bottom
                    .interpolate(target_border.widths.bottom, progress),
            },
            color: from.color.interpolate(target_border.color, progress),
        })
    }
}

fn solid_color(fill: &Fill) -> Color {
    match fill {
        Fill::Solid(color) => *color,
        Fill::Linear(gradient) => gradient.stops.as_slice()[0].color,
        Fill::Radial(gradient) => gradient.stops.as_slice()[0].color,
    }
}

fn flattened(root: &Element) -> Vec<&Element> {
    fn visit<'a>(element: &'a Element, output: &mut Vec<&'a Element>) {
        output.push(element);
        for child in &element.children {
            visit(child, output);
        }
    }
    let mut output = Vec::new();
    visit(root, &mut output);
    output
}
