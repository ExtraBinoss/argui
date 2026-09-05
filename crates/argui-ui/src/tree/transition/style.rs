use argui_core::{Color, Point, Transform2D};
use argui_paint::{Border, Fill, GradientStop, GradientStops, LayerMask, LayerStyle, QuadStyle};

use crate::binding::GradientPointTarget;
use crate::binding::layout::layout_value;
use crate::state::StateValue;
use crate::{Element, NodeId, PropertyKey, StyleCondition, StylePropertyValue, VisualStates};

use super::{NodeSpec, ResolvedProperty, TransitionRegistry, TransitionTarget};

mod effect;
use effect::{apply_effect, effect_values};
mod layout;
pub(in crate::tree) use layout::apply as apply_layout;
mod scrollbar;
mod states;
use states::{ScopeStack, StateContext};
mod values;
use values::{radii, widths_from_array};

pub(super) fn collect_specs<'a>(
    element: &'a Element,
    ids: &[NodeId],
    states_for: &dyn Fn(NodeId) -> VisualStates,
    scrollbar_states_for: &dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    scroll_for: &dyn Fn(NodeId) -> Point,
    container_size: &dyn Fn(NodeId) -> Option<argui_core::Size>,
    output: &mut Vec<NodeSpec<'a>>,
) {
    let states = StateContext::collect(element, ids, states_for);
    let context = SpecContext {
        ids,
        states: &states,
        scrollbar_states_for,
        scroll_for,
        container_size,
    };
    let mut index = 0;
    let mut scope_stack = ScopeStack::default();
    collect_element(element, &mut index, &mut scope_stack, &context, output);
}

struct SpecContext<'a> {
    ids: &'a [NodeId],
    states: &'a StateContext,
    scrollbar_states_for: &'a dyn Fn(NodeId, crate::scroll::ScrollbarPart, bool) -> VisualStates,
    scroll_for: &'a dyn Fn(NodeId) -> Point,
    container_size: &'a dyn Fn(NodeId) -> Option<argui_core::Size>,
}

fn collect_element<'a>(
    element: &'a Element,
    index: &mut usize,
    scope_stack: &mut ScopeStack,
    context: &SpecContext<'_>,
    output: &mut Vec<NodeSpec<'a>>,
) {
    let node_index = *index;
    let node = context.ids[node_index];
    *index += 1;
    context.states.enter(node_index, scope_stack);
    let matched = context
        .states
        .matched(element, node_index, scope_stack, &context.container_size);
    if element.style_transition.is_some() || !element.conditional_styles.is_empty() {
        output.push(NodeSpec {
            target: TransitionTarget::Element(node),
            matched: matched.clone(),
            values: target_values(element, &matched, (context.scroll_for)(node)),
            transition: element.style_transition.as_ref(),
        });
    }
    if let Some(config) = &element.scroll
        && let Some(scrollbar) = &config.scrollbar
    {
        scrollbar::collect(
            scrollbar::ScrollbarContext {
                node,
                node_index,
                enabled: config.enabled,
                states: context.states,
                scope_stack,
                states_for: context.scrollbar_states_for,
                container_size: context.container_size,
            },
            scrollbar,
            output,
        );
    }
    for child in &element.children {
        collect_element(child, index, scope_stack, context, output);
    }
    context.states.exit(*index, scope_stack);
}

fn target_values(
    element: &Element,
    matched: &[StyleCondition],
    scroll: Point,
) -> Vec<ResolvedProperty> {
    let mut values = resolved(base_values(element, scroll));
    for rule in element.conditional_styles.rules() {
        if matched.contains(&rule.condition) {
            for property in rule.style.values() {
                apply_target(&mut values, property, Some(rule.condition.clone()));
            }
        }
    }
    values
}

fn resolved(values: Vec<StylePropertyValue>) -> Vec<ResolvedProperty> {
    values
        .into_iter()
        .map(|property| ResolvedProperty {
            property,
            source: None,
        })
        .collect()
}

fn apply_target(
    values: &mut Vec<ResolvedProperty>,
    property: &StylePropertyValue,
    source: Option<StyleCondition>,
) {
    let mut property = property.clone();
    if property.key == PropertyKey::LayoutStyle {
        values.retain(|value| !matches!(value.property.key, PropertyKey::Layout(_)));
    }
    match (&property.key, &property.value) {
        (PropertyKey::BackgroundColor, StateValue::BackgroundColor(color))
            if values
                .iter()
                .any(|value| value.property.key == PropertyKey::Background) =>
        {
            remove_gradient_values(values);
            property.key = PropertyKey::Background;
            property.value = StateValue::Background(Some(Fill::Solid(*color)));
        }
        (PropertyKey::Background, _) => {
            values.retain(|value| {
                value.property.key != PropertyKey::BackgroundColor
                    && !is_gradient_property(value.property.key)
            });
        }
        (PropertyKey::BorderColor | PropertyKey::BorderWidths, _)
            if values
                .iter()
                .any(|value| value.property.key == PropertyKey::Border) =>
        {
            let current = values
                .iter()
                .find(|value| value.property.key == PropertyKey::Border)
                .and_then(|value| match value.property.value {
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
                    value.property.key,
                    PropertyKey::BorderColor | PropertyKey::BorderWidths
                )
            });
        }
        _ => {}
    }
    if let Some(existing) = values
        .iter_mut()
        .find(|value| value.property.key == property.key)
    {
        existing.property = property;
        existing.source = source;
    } else {
        values.push(ResolvedProperty { property, source });
    }
}

fn base_values(element: &Element, scroll: Point) -> Vec<StylePropertyValue> {
    let quad = &element.paint.quad;
    let mut values = quad_values(quad);
    values.push(value(
        PropertyKey::LayoutStyle,
        StateValue::LayoutStyle(Box::new(element.style.clone())),
    ));
    values.push(value(
        PropertyKey::Transform,
        StateValue::Transform(element.transform),
    ));
    match &element.kind {
        crate::ElementKind::Text { style, .. } => {
            values.push(value(
                PropertyKey::TextColor,
                StateValue::Color(style.color),
            ));
        }
        crate::ElementKind::TextEditor { text, .. } => {
            values.push(value(PropertyKey::TextColor, StateValue::Color(text.color)));
        }
        crate::ElementKind::Vector { color, .. } => {
            values.push(value(PropertyKey::VectorColor, StateValue::Color(*color)));
        }
        crate::ElementKind::Container | crate::ElementKind::Image { .. } => {}
    }
    if element.conditional_styles.contains(PropertyKey::Scroll) {
        values.push(value(PropertyKey::Scroll, StateValue::Point(scroll)));
    }
    for target in layout::targets(&element.style) {
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

fn quad_values(quad: &QuadStyle) -> Vec<StylePropertyValue> {
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

fn gradient_values(background: &Fill, values: &mut Vec<StylePropertyValue>) {
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

fn layer_values(layer: &LayerStyle, values: &mut Vec<StylePropertyValue>) {
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

fn remove_gradient_values(values: &mut Vec<ResolvedProperty>) {
    values.retain(|value| !is_gradient_property(value.property.key));
}

fn value(key: PropertyKey, value: StateValue) -> StylePropertyValue {
    StylePropertyValue { key, value }
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
    registry.visit(target, PropertyKey::is_quad, |key, value| {
        match (key, value) {
            (PropertyKey::Background, StateValue::Background(value)) => quad.background = value,
            (
                PropertyKey::BackgroundColor,
                StateValue::Color(value) | StateValue::BackgroundColor(value),
            ) => quad.background = Some(Fill::Solid(value)),
            (PropertyKey::Border, StateValue::Border(value)) => quad.border = value,
            (
                PropertyKey::BorderColor,
                StateValue::Color(value) | StateValue::BorderColor(value),
            ) => {
                quad.border.get_or_insert(Border::all(0.0, value)).color = value;
            }
            (
                PropertyKey::BorderWidths,
                StateValue::Vec4(value) | StateValue::BorderWidths(value),
            ) => {
                quad.border
                    .get_or_insert(Border::all(0.0, Color::TRANSPARENT))
                    .widths = widths_from_array(value);
            }
            (
                PropertyKey::CornerRadii,
                StateValue::Vec4(value) | StateValue::CornerRadii(value),
            ) => {
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
        }
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
    registry.visit(
        TransitionTarget::Element(node),
        |key| key == PropertyKey::Transform,
        |key, value| {
            if key == PropertyKey::Transform
                && let StateValue::Transform(value) = value
            {
                *transform = value;
            }
        },
    );
}

pub(in crate::tree) fn apply_scroll(
    registry: &TransitionRegistry,
    node: NodeId,
    offset: &mut Point,
) {
    registry.visit(
        TransitionTarget::Element(node),
        |key| key == PropertyKey::Scroll,
        |key, value| {
            if key == PropertyKey::Scroll
                && let StateValue::Point(value) = value
            {
                *offset = value;
            }
        },
    );
}

pub(in crate::tree) fn apply_text_color(
    registry: &TransitionRegistry,
    node: NodeId,
    color: &mut Color,
) {
    registry.visit(
        TransitionTarget::Element(node),
        |key| key == PropertyKey::TextColor,
        |key, value| {
            if key == PropertyKey::TextColor
                && let StateValue::Color(value) = value
            {
                *color = value;
            }
        },
    );
}

pub(in crate::tree) fn apply_vector_color(
    registry: &TransitionRegistry,
    node: NodeId,
    color: &mut Color,
) {
    registry.visit(
        TransitionTarget::Element(node),
        |key| key == PropertyKey::VectorColor,
        |key, value| {
            if key == PropertyKey::VectorColor
                && let StateValue::Color(value) = value
            {
                *color = value;
            }
        },
    );
}

pub(in crate::tree) fn apply_layer(
    registry: &TransitionRegistry,
    node: NodeId,
    layer: &mut LayerStyle,
) {
    registry.visit(
        TransitionTarget::Element(node),
        |key| {
            key.impact() == crate::BindingImpact::Paint
                && !key.is_quad()
                && !matches!(
                    key,
                    PropertyKey::TextColor | PropertyKey::VectorColor | PropertyKey::Transform
                )
        },
        |key, value| match (key, value) {
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
        },
    );
}
