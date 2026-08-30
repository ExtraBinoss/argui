use argui_core::Transform2D;
use argui_inspect::{
    PropertySnapshot, StyleField, StyleLength, StyleProperty, StyleUnit, StyleValue,
};
use argui_paint::{Fill, Filter, LayerStyle};
use argui_ui::{Dimension, Element, ExpandedDimension, Overflow};

pub(super) fn properties(element: &Element) -> Vec<PropertySnapshot> {
    StyleProperty::ALL
        .into_iter()
        .map(|property| PropertySnapshot {
            property,
            authored: authored(element, property),
            value: property_value(element, property),
        })
        .collect()
}

fn authored(element: &Element, property: StyleProperty) -> bool {
    match property {
        StyleProperty::Background => element.paint.quad.background.is_some(),
        StyleProperty::Border => element.paint.quad.border.is_some(),
        StyleProperty::Opacity => element.paint.quad.opacity != 1.0,
        StyleProperty::Overflow => {
            element.style.overflow.x != Overflow::Visible
                || element.style.overflow.y != Overflow::Visible
        }
        StyleProperty::Transform => element.transform != Transform2D::IDENTITY,
        StyleProperty::Layer => element.layer.is_some(),
        StyleProperty::Effects => !element.effects.is_empty(),
        StyleProperty::Width => element.style.size.width != Dimension::auto(),
        StyleProperty::Height => element.style.size.height != Dimension::auto(),
    }
}

pub(super) fn property_value(element: &Element, property: StyleProperty) -> StyleValue {
    match property {
        StyleProperty::Background => background(element),
        StyleProperty::Border => border(element),
        StyleProperty::Opacity => StyleValue::Number(element.paint.quad.opacity),
        StyleProperty::Overflow => StyleValue::Choice(format!(
            "x: {:?}, y: {:?}",
            element.style.overflow.x, element.style.overflow.y
        )),
        StyleProperty::Transform => transform(element),
        StyleProperty::Layer => element.layer.as_ref().map_or_else(
            || StyleValue::Summary("none".into()),
            |layer| StyleValue::Parameters(layer_fields("", layer)),
        ),
        StyleProperty::Effects => effects(element),
        StyleProperty::Width => StyleValue::Length(length_value(element.style.size.width)),
        StyleProperty::Height => StyleValue::Length(length_value(element.style.size.height)),
    }
}

fn background(element: &Element) -> StyleValue {
    match &element.paint.quad.background {
        Some(Fill::Solid(color)) => StyleValue::Color(color.as_array()),
        Some(Fill::Linear(gradient)) => {
            StyleValue::Summary(format!("linear gradient · {} stops", gradient.stops.len()))
        }
        Some(Fill::Radial(gradient)) => {
            StyleValue::Summary(format!("radial gradient · {} stops", gradient.stops.len()))
        }
        None => StyleValue::Summary("none".into()),
    }
}

fn border(element: &Element) -> StyleValue {
    element.paint.quad.border.map_or_else(
        || StyleValue::Summary("none".into()),
        |border| {
            let [left, right, top, bottom] = border.widths.as_array();
            let [red, green, blue, alpha] = border.color.as_array();
            StyleValue::Parameters(fields([
                ("left", left),
                ("right", right),
                ("top", top),
                ("bottom", bottom),
                ("red", red),
                ("green", green),
                ("blue", blue),
                ("alpha", alpha),
            ]))
        },
    )
}

fn transform(element: &Element) -> StyleValue {
    let transform = element.transform;
    StyleValue::Parameters(fields([
        ("translate x", transform.translation.x),
        ("translate y", transform.translation.y),
        ("scale x", transform.scale.x),
        ("scale y", transform.scale.y),
        ("rotation", transform.rotation),
        ("skew x", transform.skew.x),
        ("skew y", transform.skew.y),
    ]))
}

fn effects(element: &Element) -> StyleValue {
    let output = element
        .effects
        .iter()
        .enumerate()
        .flat_map(|(index, effect)| layer_fields(&format!("effect {index} · "), &effect.layer))
        .collect::<Vec<_>>();
    if output.is_empty() {
        StyleValue::Summary("none".into())
    } else {
        StyleValue::Parameters(output)
    }
}

fn length_value(length: Dimension) -> StyleLength {
    match length.expand() {
        ExpandedDimension::Auto => StyleLength::default(),
        ExpandedDimension::Length(value) => StyleLength {
            value,
            unit: StyleUnit::Px,
        },
        ExpandedDimension::Percent(value) => StyleLength {
            value,
            unit: StyleUnit::Percent,
        },
        _ => StyleLength::default(),
    }
}

fn fields<const N: usize>(values: [(&str, f32); N]) -> Vec<StyleField> {
    values
        .into_iter()
        .map(|(label, value)| StyleField {
            label: label.into(),
            value,
        })
        .collect()
}

fn layer_fields(prefix: &str, layer: &LayerStyle) -> Vec<StyleField> {
    let mut output = vec![StyleField {
        label: format!("{prefix}opacity"),
        value: layer.opacity,
    }];
    for (index, filter) in layer.filters.iter().enumerate() {
        filter_fields(&format!("{prefix}filter {index} · "), filter, &mut output);
    }
    for (index, filter) in layer.backdrop_filters.iter().enumerate() {
        filter_fields(&format!("{prefix}backdrop {index} · "), filter, &mut output);
    }
    for (index, shadow) in layer.shadows.iter().enumerate() {
        let prefix = format!("{prefix}shadow {index} · ");
        let [red, green, blue, alpha] = shadow.color.as_array();
        output.extend(fields([
            (&format!("{prefix}offset x"), shadow.offset[0]),
            (&format!("{prefix}offset y"), shadow.offset[1]),
            (&format!("{prefix}blur"), shadow.blur),
            (&format!("{prefix}spread"), shadow.spread),
            (&format!("{prefix}red"), red),
            (&format!("{prefix}green"), green),
            (&format!("{prefix}blue"), blue),
            (&format!("{prefix}alpha"), alpha),
        ]));
    }
    output
}

fn filter_fields(prefix: &str, filter: &Filter, output: &mut Vec<StyleField>) {
    let mut push = |label: &str, value| {
        output.push(StyleField {
            label: format!("{prefix}{label}"),
            value,
        })
    };
    match filter {
        Filter::Blur(value) => push("blur", *value),
        Filter::Brightness(value) => push("brightness", *value),
        Filter::Contrast(value) => push("contrast", *value),
        Filter::Saturation(value) => push("saturation", *value),
        Filter::HueRotate(value) => push("hue", *value),
        Filter::Opacity(value) => push("opacity", *value),
        Filter::ColorMatrix(values) => {
            for (index, value) in values.iter().enumerate() {
                push(&format!("matrix {index}"), *value);
            }
        }
        Filter::Refraction(value) => {
            push("strength", value.strength);
            push("chromatic aberration", value.chromatic_aberration);
            push("edge", value.edge);
        }
        Filter::Effect(effect) => {
            for argument in &effect.parameters {
                effect_value_fields(
                    &format!("effect {}", argument.name),
                    &argument.value,
                    &mut push,
                );
            }
        }
    }
}

fn effect_value_fields(
    label: &str,
    value: &argui_ui::EffectValue,
    push: &mut impl FnMut(&str, f32),
) {
    use argui_ui::EffectValue;
    match value {
        EffectValue::F32(value) | EffectValue::LogicalPixels(value) => push(label, *value),
        EffectValue::I32(value) => push(label, *value as f32),
        EffectValue::U32(value) => push(label, *value as f32),
        EffectValue::Bool(value) => push(label, f32::from(*value)),
        EffectValue::Vec2(values) => push_components(label, values, push),
        EffectValue::Vec3(values) => push_components(label, values, push),
        EffectValue::Vec4(values) => push_components(label, values, push),
        EffectValue::Mat3(values) => push_components(label, values, push),
        EffectValue::Mat4(values) => push_components(label, values, push),
        EffectValue::Color(color) => push_components(label, &color.as_array(), push),
    }
}

fn push_components(label: &str, values: &[f32], push: &mut impl FnMut(&str, f32)) {
    for (component, value) in values.iter().enumerate() {
        push(&format!("{label} {component}"), *value);
    }
}
