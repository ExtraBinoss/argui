use argui_core::{Affine2D, Transform2D};
use argui_inspect::{
    InspectNodeId, NodeSnapshot, PropertySnapshot, StyleField, StyleLength, StyleProperty,
    StyleUnit, StyleValue, TreeSnapshot,
};
use argui_paint::{
    Border, ClipBehavior, ClipChain, ClipRegion, Color, CornerRadii, Fill, Filter, LayerStyle, Quad,
};
use argui_ui::{Element, ElementKind, Length, NodeId};

use crate::AnyEntity;

use super::Application;

impl Application {
    pub(crate) fn inspected_view(&self) -> Option<Element> {
        let mut root = self.model.as_ref().map(AnyEntity::render)?;
        let Some(inspector) = &self.inspector else {
            return Some(root);
        };
        let ids = self
            .ui_tree
            .as_ref()
            .map(|tree| tree.node_ids())
            .unwrap_or(&[]);
        apply_tree_overrides(&mut root, ids, inspector, &mut 0);
        Some(root)
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn publish_inspection(&self) {
        let (Some(inspector), Some(tree), Some(layout)) =
            (&self.inspector, &self.ui_tree, &self.ui_layout)
        else {
            return;
        };
        let mut nodes = Vec::new();
        collect_nodes(
            tree.root(),
            None,
            0,
            true,
            &mut CollectState {
                ids: tree.node_ids(),
                layout,
                output: &mut nodes,
                cursor: 0,
            },
        );
        inspector.publish_tree(TreeSnapshot {
            revision: tree.revision(),
            nodes,
        });
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn paint_inspection_highlight(&mut self) {
        let node = self
            .inspector
            .as_ref()
            .and_then(|inspector| inspector.highlighted())
            .and_then(|id| self.inspector.as_ref()?.node(id));
        let (Some(node), Some(layout)) = (node, &mut self.ui_layout) else {
            return;
        };
        let mut regions = vec![ClipRegion::new(layout.viewport, Affine2D::IDENTITY)];
        if let Some(clip) = node.clip {
            regions.push(ClipRegion::new(clip, Affine2D::IDENTITY));
        }
        layout.display_list.push_quad(Quad {
            bounds: node.bounds,
            background: Some(Fill::Solid(Color::rgba(0.20, 0.72, 1.0, 0.10))),
            border: Border::all(2.0, Color::rgb(0.25, 0.72, 0.96)),
            radii: CornerRadii::all(0.0),
            opacity: 1.0,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions(regions),
        });
    }
}

struct CollectState<'a> {
    ids: &'a [NodeId],
    layout: &'a argui_layout::LayoutOutput,
    output: &'a mut Vec<NodeSnapshot>,
    cursor: usize,
}

fn collect_nodes(
    element: &Element,
    parent: Option<InspectNodeId>,
    depth: usize,
    parent_visible: bool,
    state: &mut CollectState<'_>,
) {
    let index = state.cursor;
    state.cursor += 1;
    let Some(node) = state.ids.get(index).copied() else {
        return;
    };
    if !element.inspectable {
        state.cursor += descendant_count(element);
        return;
    }
    let id = InspectNodeId(node.get());
    let visible = parent_visible
        && element
            .layer
            .as_ref()
            .is_none_or(|layer| layer.opacity > 0.0);
    let layout_node = state
        .layout
        .nodes
        .iter()
        .find(|candidate| candidate.node == node)
        .copied();
    let bounds = layout_node
        .map(|candidate| candidate.bounds)
        .unwrap_or_default();
    state.output.push(NodeSnapshot {
        id,
        parent,
        depth,
        key: element.key.clone(),
        kind: kind_name(&element.kind).to_owned(),
        summary: summary(&element.kind, element.children.len()),
        bounds,
        clip: layout_node.and_then(|candidate| candidate.clip),
        z_index: element.z_index,
        visible,
        painted: paints_content(element),
        interactive: element
            .interaction
            .as_ref()
            .is_some_and(|interaction| interaction.enabled),
        child_count: element.children.len(),
        properties: properties(element),
    });
    for child in &element.children {
        collect_nodes(child, Some(id), depth + 1, visible, state);
    }
}

fn paints_content(element: &Element) -> bool {
    element.paint.quad.is_visible()
        || !element.effects.is_empty()
        || element.layer.is_some()
        || !matches!(element.kind, ElementKind::Container)
}

fn descendant_count(element: &Element) -> usize {
    element
        .children
        .iter()
        .map(|child| 1 + descendant_count(child))
        .sum()
}

fn kind_name(kind: &ElementKind) -> &'static str {
    match kind {
        ElementKind::Container => "container",
        ElementKind::Text { .. } => "text",
        ElementKind::TextInput { .. } => "text-input",
        ElementKind::Image { .. } => "image",
        ElementKind::Vector { .. } => "vector",
    }
}

fn summary(kind: &ElementKind, children: usize) -> Option<String> {
    match kind {
        ElementKind::Container => Some(format!("{children} children")),
        ElementKind::Text { content, .. } => Some(short(content)),
        ElementKind::TextInput {
            initial_value,
            placeholder,
            ..
        } => Some(short(if initial_value.is_empty() {
            placeholder
        } else {
            initial_value
        })),
        ElementKind::Image { image, fit, .. } => Some(format!("id={} · {fit:?}", image.0)),
        ElementKind::Vector { vector, progress } => {
            Some(format!("id={} · morph {progress:.2}", vector.0))
        }
    }
}

fn short(value: &str) -> String {
    const LIMIT: usize = 52;
    let mut output = value.chars().take(LIMIT).collect::<String>();
    if value.chars().count() > LIMIT {
        output.push('…');
    }
    output
}

fn properties(element: &Element) -> Vec<PropertySnapshot> {
    StyleProperty::ALL
        .into_iter()
        .map(|property| {
            let authored = match property {
                StyleProperty::Background => element.paint.quad.background.is_some(),
                StyleProperty::Border => element.paint.quad.border.is_some(),
                StyleProperty::Opacity => element.paint.quad.opacity != 1.0,
                StyleProperty::Clip => element.paint.clip != ClipBehavior::None,
                StyleProperty::Transform => element.transform != Transform2D::IDENTITY,
                StyleProperty::Layer => element.layer.is_some(),
                StyleProperty::Effects => !element.effects.is_empty(),
                StyleProperty::Width => element.style.width != Length::Auto,
                StyleProperty::Height => element.style.height != Length::Auto,
            };
            PropertySnapshot {
                property,
                authored,
                value: property_value(element, property),
            }
        })
        .collect()
}

fn property_value(element: &Element, property: StyleProperty) -> StyleValue {
    match property {
        StyleProperty::Background => match &element.paint.quad.background {
            Some(Fill::Solid(color)) => StyleValue::Color(color.as_array()),
            Some(Fill::Linear(gradient)) => {
                StyleValue::Summary(format!("linear gradient · {} stops", gradient.stops.len()))
            }
            Some(Fill::Radial(gradient)) => {
                StyleValue::Summary(format!("radial gradient · {} stops", gradient.stops.len()))
            }
            None => StyleValue::Summary("none".into()),
        },
        StyleProperty::Border => element.paint.quad.border.map_or_else(
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
        ),
        StyleProperty::Opacity => StyleValue::Number(element.paint.quad.opacity),
        StyleProperty::Clip => StyleValue::Choice(format!("{:?}", element.paint.clip)),
        StyleProperty::Transform => {
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
        StyleProperty::Layer => element.layer.as_ref().map_or_else(
            || StyleValue::Summary("none".into()),
            |layer| StyleValue::Parameters(layer_fields("", layer)),
        ),
        StyleProperty::Effects => {
            let mut output = Vec::new();
            for (index, effect) in element.effects.iter().enumerate() {
                output.extend(layer_fields(&format!("effect {index} · "), &effect.layer));
            }
            if output.is_empty() {
                StyleValue::Summary("none".into())
            } else {
                StyleValue::Parameters(output)
            }
        }
        StyleProperty::Width => StyleValue::Length(length_value(element.style.width)),
        StyleProperty::Height => StyleValue::Length(length_value(element.style.height)),
    }
}

fn length_value(length: Length) -> StyleLength {
    match length {
        Length::Auto => StyleLength::default(),
        Length::Px(value) => StyleLength {
            value,
            unit: StyleUnit::Px,
        },
        Length::Percent(value) => StyleLength {
            value,
            unit: StyleUnit::Percent,
        },
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
        let shadow_prefix = format!("{prefix}shadow {index} · ");
        let [red, green, blue, alpha] = shadow.color.as_array();
        output.extend(fields([
            (&format!("{shadow_prefix}offset x"), shadow.offset[0]),
            (&format!("{shadow_prefix}offset y"), shadow.offset[1]),
            (&format!("{shadow_prefix}blur"), shadow.blur),
            (&format!("{shadow_prefix}spread"), shadow.spread),
            (&format!("{shadow_prefix}red"), red),
            (&format!("{shadow_prefix}green"), green),
            (&format!("{shadow_prefix}blue"), blue),
            (&format!("{shadow_prefix}alpha"), alpha),
        ]));
    }
    output
}

fn filter_fields(prefix: &str, filter: &Filter, output: &mut Vec<StyleField>) {
    let mut push = |label: &str, value| {
        output.push(StyleField {
            label: format!("{prefix}{label}"),
            value,
        });
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
        Filter::Custom(effect) => {
            for (index, value) in effect.parameters.iter().enumerate() {
                push(&format!("custom {index}"), *value);
            }
        }
    }
}

fn apply_overrides(
    element: &mut Element,
    node: NodeId,
    inspector: &argui_inspect::InspectorHandle,
) {
    let id = InspectNodeId(node.get());
    for property in StyleProperty::ALL {
        match inspector.property_enabled(id, property) {
            Some(false) => reset_property(element, property),
            Some(true) => {
                if let Some(value) = inspector.property_value(id, property) {
                    apply_property_value(element, property, &value);
                }
            }
            None => {}
        }
    }
}

fn reset_property(element: &mut Element, property: StyleProperty) {
    match property {
        StyleProperty::Background => element.paint.quad.background = None,
        StyleProperty::Border => element.paint.quad.border = None,
        StyleProperty::Opacity => element.paint.quad.opacity = 1.0,
        StyleProperty::Clip => element.paint.clip = ClipBehavior::None,
        StyleProperty::Transform => element.transform = Transform2D::IDENTITY,
        StyleProperty::Layer => element.layer = None,
        StyleProperty::Effects => element.effects.clear(),
        StyleProperty::Width => element.style.width = Length::Auto,
        StyleProperty::Height => element.style.height = Length::Auto,
    }
}

fn apply_property_value(element: &mut Element, property: StyleProperty, value: &StyleValue) {
    match (property, value) {
        (StyleProperty::Background, StyleValue::Color([red, green, blue, alpha])) => {
            element.paint.quad.background =
                Some(Fill::Solid(Color::rgba(*red, *green, *blue, *alpha)));
        }
        (StyleProperty::Border, StyleValue::Parameters(fields)) => {
            if let Some(border) = &mut element.paint.quad.border {
                let values = field_values(fields);
                set_if_present(&mut border.widths.left, &values, "left");
                set_if_present(&mut border.widths.right, &values, "right");
                set_if_present(&mut border.widths.top, &values, "top");
                set_if_present(&mut border.widths.bottom, &values, "bottom");
                let [mut red, mut green, mut blue, mut alpha] = border.color.as_array();
                set_if_present(&mut red, &values, "red");
                set_if_present(&mut green, &values, "green");
                set_if_present(&mut blue, &values, "blue");
                set_if_present(&mut alpha, &values, "alpha");
                border.color = Color::rgba(red, green, blue, alpha);
            }
        }
        (StyleProperty::Opacity, StyleValue::Number(value)) => {
            element.paint.quad.opacity = value.clamp(0.0, 1.0);
        }
        (StyleProperty::Transform, StyleValue::Parameters(fields)) => {
            let values = field_values(fields);
            set_if_present(&mut element.transform.translation.x, &values, "translate x");
            set_if_present(&mut element.transform.translation.y, &values, "translate y");
            set_if_present(&mut element.transform.scale.x, &values, "scale x");
            set_if_present(&mut element.transform.scale.y, &values, "scale y");
            set_if_present(&mut element.transform.rotation, &values, "rotation");
            set_if_present(&mut element.transform.skew.x, &values, "skew x");
            set_if_present(&mut element.transform.skew.y, &values, "skew y");
        }
        (StyleProperty::Layer, StyleValue::Parameters(fields)) => {
            if let Some(layer) = &mut element.layer {
                apply_layer_fields("", layer, fields);
            }
        }
        (StyleProperty::Effects, StyleValue::Parameters(fields)) => {
            for (index, effect) in element.effects.iter_mut().enumerate() {
                apply_layer_fields(&format!("effect {index} · "), &mut effect.layer, fields);
            }
        }
        (StyleProperty::Width, StyleValue::Length(value)) => {
            element.style.width = inspect_length(*value);
        }
        (StyleProperty::Height, StyleValue::Length(value)) => {
            element.style.height = inspect_length(*value);
        }
        _ => {}
    }
}

fn inspect_length(value: StyleLength) -> Length {
    match value.unit {
        StyleUnit::Auto => Length::Auto,
        StyleUnit::Px => Length::Px(value.value.max(0.0)),
        StyleUnit::Percent => Length::Percent(value.value.max(0.0)),
    }
}

fn field_values(fields: &[StyleField]) -> std::collections::HashMap<&str, f32> {
    fields
        .iter()
        .map(|field| (field.label.as_str(), field.value))
        .collect()
}

fn set_if_present(target: &mut f32, values: &std::collections::HashMap<&str, f32>, label: &str) {
    if let Some(value) = values.get(label) {
        *target = *value;
    }
}

fn apply_layer_fields(prefix: &str, layer: &mut LayerStyle, fields: &[StyleField]) {
    let values = field_values(fields);
    set_if_present(&mut layer.opacity, &values, &format!("{prefix}opacity"));
    for (index, filter) in layer.filters.iter_mut().enumerate() {
        apply_filter_fields(&format!("{prefix}filter {index} · "), filter, &values);
    }
    for (index, filter) in layer.backdrop_filters.iter_mut().enumerate() {
        apply_filter_fields(&format!("{prefix}backdrop {index} · "), filter, &values);
    }
    for (index, shadow) in layer.shadows.iter_mut().enumerate() {
        let shadow_prefix = format!("{prefix}shadow {index} · ");
        set_if_present(
            &mut shadow.offset[0],
            &values,
            &format!("{shadow_prefix}offset x"),
        );
        set_if_present(
            &mut shadow.offset[1],
            &values,
            &format!("{shadow_prefix}offset y"),
        );
        set_if_present(&mut shadow.blur, &values, &format!("{shadow_prefix}blur"));
        set_if_present(
            &mut shadow.spread,
            &values,
            &format!("{shadow_prefix}spread"),
        );
        let [mut red, mut green, mut blue, mut alpha] = shadow.color.as_array();
        set_if_present(&mut red, &values, &format!("{shadow_prefix}red"));
        set_if_present(&mut green, &values, &format!("{shadow_prefix}green"));
        set_if_present(&mut blue, &values, &format!("{shadow_prefix}blue"));
        set_if_present(&mut alpha, &values, &format!("{shadow_prefix}alpha"));
        shadow.color = Color::rgba(red, green, blue, alpha);
    }
}

fn apply_filter_fields(
    prefix: &str,
    filter: &mut Filter,
    values: &std::collections::HashMap<&str, f32>,
) {
    let value = |label: &str| values.get(format!("{prefix}{label}").as_str()).copied();
    match filter {
        Filter::Blur(current) => set_filter_value(current, value("blur")),
        Filter::Brightness(current) => set_filter_value(current, value("brightness")),
        Filter::Contrast(current) => set_filter_value(current, value("contrast")),
        Filter::Saturation(current) => set_filter_value(current, value("saturation")),
        Filter::HueRotate(current) => set_filter_value(current, value("hue")),
        Filter::Opacity(current) => set_filter_value(current, value("opacity")),
        Filter::ColorMatrix(matrix) => {
            for (index, current) in matrix.iter_mut().enumerate() {
                if let Some(value) = value(&format!("matrix {index}")) {
                    *current = value;
                }
            }
        }
        Filter::Refraction(current) => {
            if let Some(value) = value("strength") {
                current.strength = value;
            }
            if let Some(value) = value("chromatic aberration") {
                current.chromatic_aberration = value;
            }
            if let Some(value) = value("edge") {
                current.edge = value;
            }
        }
        Filter::Custom(current) => {
            for (index, parameter) in current.parameters.iter_mut().enumerate() {
                if let Some(value) = value(&format!("custom {index}")) {
                    *parameter = value;
                }
            }
        }
    }
}

fn set_filter_value(target: &mut f32, value: Option<f32>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn apply_tree_overrides(
    element: &mut Element,
    ids: &[NodeId],
    inspector: &argui_inspect::InspectorHandle,
    cursor: &mut usize,
) {
    if let Some(node) = ids.get(*cursor).copied() {
        apply_overrides(element, node, inspector);
    }
    *cursor += 1;
    for child in &mut element.children {
        apply_tree_overrides(child, ids, inspector, cursor);
    }
}

#[cfg(test)]
mod tests;
