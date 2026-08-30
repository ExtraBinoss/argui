use argui_core::{Color, Point, Transform2D};
use argui_paint::{
    Border, BorderWidths, CornerRadii, Fill, GradientStop, GradientStops, LayerMask, LayerStyle,
    QuadStyle,
};

use crate::binding::layout::{layout_value, set_layout_value};
use crate::binding::{GradientPointTarget, LayoutTarget};
use crate::state::StateValue;
use crate::{Element, NodeId, PropertyKey, StatePropertyValue, VisualState, VisualStates};

use super::{NodeSpec, TransitionRegistry, TransitionTarget};

mod effect;
use effect::{apply_effect, effect_values};
mod scrollbar;

pub(super) fn collect_specs<'a>(
    element: &'a Element,
    ids: &[NodeId],
    states_for: &impl Fn(NodeId) -> VisualStates,
    scrollbar_states_for: &impl Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    scroll_for: &impl Fn(NodeId) -> Point,
    output: &mut Vec<NodeSpec<'a>>,
) {
    let context = SpecContext {
        ids,
        states_for,
        scrollbar_states_for,
        scroll_for,
    };
    let mut index = 0;
    collect_element(element, &mut index, VisualStates::NONE, &context, output);
}

struct SpecContext<'a> {
    ids: &'a [NodeId],
    states_for: &'a dyn Fn(NodeId) -> VisualStates,
    scrollbar_states_for: &'a dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    scroll_for: &'a dyn Fn(NodeId) -> Point,
}

fn collect_element<'a>(
    element: &'a Element,
    index: &mut usize,
    inherited: VisualStates,
    context: &SpecContext<'_>,
    output: &mut Vec<NodeSpec<'a>>,
) {
    let node = context.ids[*index];
    *index += 1;
    let mut own = (context.states_for)(node);
    if element
        .interaction
        .as_ref()
        .is_some_and(|value| !value.enabled)
    {
        own.insert(VisualState::Disabled);
    }
    let states = if element.inherit_interaction_state {
        inherited
    } else {
        own
    };
    if element.style_transition.is_some() || !element.state_styles.is_empty() {
        output.push(NodeSpec {
            target: TransitionTarget::Element(node),
            states,
            values: target_values(element, states, (context.scroll_for)(node)),
            transition: element.style_transition.as_ref(),
        });
    }
    if let Some(config) = &element.scroll
        && let Some(scrollbar) = &config.scrollbar
    {
        scrollbar::collect(
            node,
            config.enabled,
            scrollbar,
            context.scrollbar_states_for,
            output,
        );
    }
    let descendants = if element.interaction.is_some() {
        own
    } else {
        inherited
    };
    for child in &element.children {
        collect_element(child, index, descendants, context, output);
    }
}

fn target_values(
    element: &Element,
    states: VisualStates,
    scroll: Point,
) -> Vec<StatePropertyValue> {
    let mut values = base_values(element, scroll);
    for state in [
        VisualState::Focused,
        VisualState::Hovered,
        VisualState::Pressed,
        VisualState::Disabled,
    ] {
        if states.contains(state)
            && let Some(style) = element.state_styles.get(state)
        {
            for property in style.values() {
                apply_target(&mut values, property);
            }
        }
    }
    values
}

fn apply_target(values: &mut Vec<StatePropertyValue>, property: &StatePropertyValue) {
    let mut property = property.clone();
    match (&property.key, &property.value) {
        (PropertyKey::BackgroundColor, StateValue::BackgroundColor(color))
            if values
                .iter()
                .any(|value| value.key == PropertyKey::Background) =>
        {
            remove_gradient_values(values);
            property.key = PropertyKey::Background;
            property.value = StateValue::Background(Some(Fill::Solid(*color)));
        }
        (PropertyKey::Background, _) => {
            values.retain(|value| {
                value.key != PropertyKey::BackgroundColor && !is_gradient_property(value.key)
            });
        }
        (PropertyKey::BorderColor | PropertyKey::BorderWidths, _)
            if values.iter().any(|value| value.key == PropertyKey::Border) =>
        {
            let current = values
                .iter()
                .find(|value| value.key == PropertyKey::Border)
                .and_then(|value| match value.value {
                    StateValue::Border(border) => border,
                    _ => None,
                })
                .unwrap_or_else(|| Border::all(0.0, Color::TRANSPARENT));
            let border = match property.value {
                StateValue::BorderColor(color) => Border { color, ..current },
                StateValue::BorderWidths(widths) => Border {
                    widths: widths_from_array(widths),
                    ..current
                },
                _ => current,
            };
            property.key = PropertyKey::Border;
            property.value = StateValue::Border(Some(border));
        }
        (PropertyKey::Border, _) => {
            values.retain(|value| {
                !matches!(
                    value.key,
                    PropertyKey::BorderColor | PropertyKey::BorderWidths
                )
            });
        }
        _ => {}
    }
    if let Some(existing) = values.iter_mut().find(|value| value.key == property.key) {
        *existing = property;
    } else {
        values.push(property);
    }
}

fn base_values(element: &Element, scroll: Point) -> Vec<StatePropertyValue> {
    let quad = &element.paint.quad;
    let mut values = quad_values(quad);
    values.push(value(
        PropertyKey::Transform,
        StateValue::Transform(element.transform),
    ));
    if element.state_styles.contains(PropertyKey::Scroll) {
        values.push(value(PropertyKey::Scroll, StateValue::Point(scroll)));
    }
    for target in layout_targets(&element.style) {
        values.push(value(
            PropertyKey::Layout(target),
            StateValue::F32(layout_value(&element.style, target)),
        ));
    }
    if let Some(layer) = &element.layer {
        layer_values(layer, &mut values);
    }
    for effect in &element.effects {
        effect_values(&effect.layer, &mut values);
    }
    values
}

fn quad_values(quad: &QuadStyle) -> Vec<StatePropertyValue> {
    let mut values = vec![
        value(
            PropertyKey::CornerRadii,
            StateValue::CornerRadii(quad.radii.as_array()),
        ),
        value(PropertyKey::Opacity, StateValue::Opacity(quad.opacity)),
    ];
    match &quad.background {
        Some(Fill::Solid(color)) => values.push(value(
            PropertyKey::BackgroundColor,
            StateValue::BackgroundColor(*color),
        )),
        background => {
            values.push(value(
                PropertyKey::Background,
                StateValue::Background(background.clone()),
            ));
            if let Some(background) = background {
                gradient_values(background, &mut values);
            }
        }
    }
    match quad.border {
        Some(border) => {
            values.push(value(
                PropertyKey::BorderColor,
                StateValue::BorderColor(border.color),
            ));
            values.push(value(
                PropertyKey::BorderWidths,
                StateValue::BorderWidths(border.widths.as_array()),
            ));
        }
        None => values.push(value(PropertyKey::Border, StateValue::Border(None))),
    }
    values
}

fn gradient_values(background: &Fill, values: &mut Vec<StatePropertyValue>) {
    let (first, second, stops) = match background {
        Fill::Linear(gradient) => (
            (GradientPointTarget::LinearStart, gradient.start),
            (GradientPointTarget::LinearEnd, gradient.end),
            &gradient.stops,
        ),
        Fill::Radial(gradient) => (
            (GradientPointTarget::RadialCenter, gradient.center),
            (GradientPointTarget::RadialRadius, gradient.radius),
            &gradient.stops,
        ),
        Fill::Solid(_) => return,
    };
    values.push(value(
        PropertyKey::GradientPoint(first.0),
        StateValue::Point(first.1),
    ));
    values.push(value(
        PropertyKey::GradientPoint(second.0),
        StateValue::Point(second.1),
    ));
    for (index, stop) in stops.as_slice().iter().enumerate() {
        values.push(value(
            PropertyKey::GradientStopOffset(index),
            StateValue::F32(stop.offset),
        ));
        values.push(value(
            PropertyKey::GradientStopColor(index),
            StateValue::Color(stop.color),
        ));
    }
}

fn layer_values(layer: &LayerStyle, values: &mut Vec<StatePropertyValue>) {
    values.push(value(
        PropertyKey::LayerOpacity,
        StateValue::F32(layer.opacity),
    ));
    if let LayerMask::Rounded(radii) = layer.mask {
        values.push(value(
            PropertyKey::LayerMaskRadii,
            StateValue::CornerRadii(radii.as_array()),
        ));
    }
    for (index, shadow) in layer.shadows.iter().enumerate() {
        values.push(value(
            PropertyKey::ShadowOffset(index),
            StateValue::Vec2(shadow.offset),
        ));
        values.push(value(
            PropertyKey::ShadowBlur(index),
            StateValue::F32(shadow.blur),
        ));
        values.push(value(
            PropertyKey::ShadowSpread(index),
            StateValue::F32(shadow.spread),
        ));
        values.push(value(
            PropertyKey::ShadowColor(index),
            StateValue::Color(shadow.color),
        ));
    }
    effect_values(layer, values);
}

fn is_gradient_property(key: PropertyKey) -> bool {
    matches!(
        key,
        PropertyKey::GradientPoint(_)
            | PropertyKey::GradientStopOffset(_)
            | PropertyKey::GradientStopColor(_)
    )
}

fn remove_gradient_values(values: &mut Vec<StatePropertyValue>) {
    values.retain(|value| !is_gradient_property(value.key));
}

fn value(key: PropertyKey, value: StateValue) -> StatePropertyValue {
    StatePropertyValue { key, value }
}

fn layout_targets(style: &crate::LayoutStyle) -> Vec<LayoutTarget> {
    use crate::Length::{Auto, Percent, Px};
    let mut values = vec![
        LayoutTarget::PaddingLeft,
        LayoutTarget::PaddingRight,
        LayoutTarget::PaddingTop,
        LayoutTarget::PaddingBottom,
        LayoutTarget::Gap,
        LayoutTarget::Grow,
        LayoutTarget::Shrink,
    ];
    let mut add = |length, px, percent| match length {
        Px(_) => values.push(px),
        Percent(_) => values.push(percent),
        Auto => {}
    };
    add(
        style.width,
        LayoutTarget::WidthPx,
        LayoutTarget::WidthPercent,
    );
    add(
        style.height,
        LayoutTarget::HeightPx,
        LayoutTarget::HeightPercent,
    );
    add(
        style.min_width,
        LayoutTarget::MinWidthPx,
        LayoutTarget::MinWidthPercent,
    );
    add(
        style.min_height,
        LayoutTarget::MinHeightPx,
        LayoutTarget::MinHeightPercent,
    );
    add(
        style.max_width,
        LayoutTarget::MaxWidthPx,
        LayoutTarget::MaxWidthPercent,
    );
    add(
        style.max_height,
        LayoutTarget::MaxHeightPx,
        LayoutTarget::MaxHeightPercent,
    );
    add(
        style.inset.left,
        LayoutTarget::InsetLeftPx,
        LayoutTarget::InsetLeftPx,
    );
    add(
        style.inset.right,
        LayoutTarget::InsetRightPx,
        LayoutTarget::InsetRightPx,
    );
    add(
        style.inset.top,
        LayoutTarget::InsetTopPx,
        LayoutTarget::InsetTopPx,
    );
    add(
        style.inset.bottom,
        LayoutTarget::InsetBottomPx,
        LayoutTarget::InsetBottomPx,
    );
    values
}

pub(in crate::tree) fn apply_quad(
    registry: &TransitionRegistry,
    node: NodeId,
    quad: &mut QuadStyle,
) {
    apply_quad_target(registry, TransitionTarget::Element(node), quad);
}

pub(in crate::tree) fn apply_scrollbar_part(
    registry: &TransitionRegistry,
    node: NodeId,
    part: crate::scroll::ScrollbarPart,
    quad: &mut QuadStyle,
) {
    let target = match part {
        crate::scroll::ScrollbarPart::Track => TransitionTarget::ScrollbarTrack(node),
        crate::scroll::ScrollbarPart::Thumb => TransitionTarget::ScrollbarThumb(node),
    };
    apply_quad_target(registry, target, quad);
}

fn apply_quad_target(
    registry: &TransitionRegistry,
    target: TransitionTarget,
    quad: &mut QuadStyle,
) {
    registry.visit(target, |key, value| match (key, value) {
        (PropertyKey::Background, StateValue::Background(value)) => quad.background = value,
        (
            PropertyKey::BackgroundColor,
            StateValue::Color(value) | StateValue::BackgroundColor(value),
        ) => quad.background = Some(Fill::Solid(value)),
        (PropertyKey::Border, StateValue::Border(value)) => quad.border = value,
        (PropertyKey::BorderColor, StateValue::Color(value) | StateValue::BorderColor(value)) => {
            quad.border.get_or_insert(Border::all(0.0, value)).color = value;
        }
        (PropertyKey::BorderWidths, StateValue::Vec4(value) | StateValue::BorderWidths(value)) => {
            quad.border
                .get_or_insert(Border::all(0.0, Color::TRANSPARENT))
                .widths = widths_from_array(value);
        }
        (PropertyKey::CornerRadii, StateValue::Vec4(value) | StateValue::CornerRadii(value)) => {
            quad.radii = radii(value);
        }
        (PropertyKey::Opacity, StateValue::F32(value) | StateValue::Opacity(value)) => {
            quad.opacity = value;
        }
        (PropertyKey::GradientPoint(target), StateValue::Point(value)) => {
            apply_gradient_point(&mut quad.background, target, value);
        }
        (PropertyKey::GradientStopOffset(index), StateValue::F32(value)) => {
            apply_gradient_stop(&mut quad.background, index, |stop| stop.offset = value);
        }
        (PropertyKey::GradientStopColor(index), StateValue::Color(value)) => {
            apply_gradient_stop(&mut quad.background, index, |stop| stop.color = value);
        }
        _ => {}
    });
}

fn apply_gradient_point(fill: &mut Option<Fill>, target: GradientPointTarget, value: Point) {
    match (fill, target) {
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearStart) => gradient.start = value,
        (Some(Fill::Linear(gradient)), GradientPointTarget::LinearEnd) => gradient.end = value,
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialCenter) => {
            gradient.center = value;
        }
        (Some(Fill::Radial(gradient)), GradientPointTarget::RadialRadius) => {
            gradient.radius = value;
        }
        _ => {}
    }
}

fn apply_gradient_stop(
    fill: &mut Option<Fill>,
    index: usize,
    apply: impl FnOnce(&mut GradientStop),
) {
    let stops = match fill {
        Some(Fill::Linear(gradient)) => &mut gradient.stops,
        Some(Fill::Radial(gradient)) => &mut gradient.stops,
        _ => return,
    };
    let mut resolved = stops.as_slice().to_vec();
    let Some(stop) = resolved.get_mut(index) else {
        return;
    };
    apply(stop);
    if let Ok(resolved) = GradientStops::from_vec(resolved) {
        *stops = resolved;
    }
}

pub(in crate::tree) fn apply_transform(
    registry: &TransitionRegistry,
    node: NodeId,
    transform: &mut Transform2D,
) {
    registry.visit(TransitionTarget::Element(node), |key, value| {
        if key == PropertyKey::Transform
            && let StateValue::Transform(value) = value
        {
            *transform = value;
        }
    });
}

pub(in crate::tree) fn apply_scroll(
    registry: &TransitionRegistry,
    node: NodeId,
    offset: &mut Point,
) {
    registry.visit(TransitionTarget::Element(node), |key, value| {
        if key == PropertyKey::Scroll
            && let StateValue::Point(value) = value
        {
            *offset = value;
        }
    });
}

pub(in crate::tree) fn apply_layout(
    registry: &TransitionRegistry,
    node: NodeId,
    style: &mut crate::LayoutStyle,
) {
    registry.visit(TransitionTarget::Element(node), |key, value| {
        if let (PropertyKey::Layout(target), StateValue::F32(value)) = (key, value) {
            set_layout_value(style, target, value);
        }
    });
}

pub(in crate::tree) fn apply_layer(
    registry: &TransitionRegistry,
    node: NodeId,
    layer: &mut LayerStyle,
) {
    registry.visit(TransitionTarget::Element(node), |key, value| {
        match (key, value) {
            (PropertyKey::LayerOpacity, StateValue::F32(value)) => layer.opacity = value,
            (
                PropertyKey::LayerMaskRadii,
                StateValue::Vec4(value) | StateValue::CornerRadii(value),
            ) => {
                layer.mask = LayerMask::Rounded(radii(value));
            }
            (PropertyKey::ShadowOffset(index), StateValue::Vec2(value)) => {
                if let Some(shadow) = layer.shadows.get_mut(index) {
                    shadow.offset = value;
                }
            }
            (PropertyKey::ShadowBlur(index), StateValue::F32(value)) => {
                if let Some(shadow) = layer.shadows.get_mut(index) {
                    shadow.blur = value;
                }
            }
            (PropertyKey::ShadowSpread(index), StateValue::F32(value)) => {
                if let Some(shadow) = layer.shadows.get_mut(index) {
                    shadow.spread = value;
                }
            }
            (PropertyKey::ShadowColor(index), StateValue::Color(value)) => {
                if let Some(shadow) = layer.shadows.get_mut(index) {
                    shadow.color = value;
                }
            }
            (key, value) => apply_effect(layer, key, &value),
        }
    });
}

const fn widths_from_array(value: [f32; 4]) -> BorderWidths {
    BorderWidths {
        left: value[0],
        right: value[1],
        top: value[2],
        bottom: value[3],
    }
}

const fn radii(value: [f32; 4]) -> CornerRadii {
    CornerRadii {
        top_left: value[0],
        top_right: value[1],
        bottom_right: value[2],
        bottom_left: value[3],
    }
}
