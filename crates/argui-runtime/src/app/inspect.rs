use argui_core::{Affine2D, Transform2D};
use argui_inspect::{
    InspectNodeId, NodeSnapshot, PortalSnapshot, StyleField, StyleLength, StyleProperty, StyleUnit,
    StyleValue, TreeSnapshot,
};
use argui_paint::{
    Border, ClipChain, ClipRegion, Color, CornerRadii, Fill, Filter, LayerStyle, Quad,
};
use argui_ui::{Axes, Dimension, Element, ElementKind, NodeId, Overflow, UiTree};

use super::Application;

mod cache;
mod memory;
mod values;
pub use cache::InspectionCache;

use values::properties;

impl Application {
    #[cfg_attr(coverage_nightly, coverage(off))]
    pub(super) fn publish_inspection(&mut self) {
        let (Some(inspector), Some(tree), Some(layout)) =
            (&self.inspector, &self.ui_tree, &self.ui_layout)
        else {
            return;
        };
        if inspector.take_memory_request() {
            inspector.publish_memory(Inspection::memory(tree, &self.layout_engine, layout));
        }
        if inspector.enabled()
            && let Some(mut snapshot) = self.inspection_cache.snapshot(tree, layout)
        {
            Inspection::custom_stats(&mut snapshot, &self.layout_engine.custom_stats());
            inspector.publish_tree(snapshot);
        }
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
            background: Some(Fill::Solid(Color::srgba(0.20, 0.72, 1.0, 0.10))),
            border: Border::all(2.0, Color::srgb(0.25, 0.72, 0.96)),
            radii: CornerRadii::all(0.0),
            opacity: 1.0,
            transform: Affine2D::IDENTITY,
            clips: ClipChain::from_regions(regions),
        });
    }
}

/// Public inspection operations for retained UI trees.
pub struct Inspection;

impl Inspection {
    /// Adds extension diagnostics to matching nodes in an inspection snapshot.
    ///
    /// `snapshot` is the snapshot to update; `stats` contains custom layout-element
    /// measurements to attach to nodes with matching identifiers.
    pub fn custom_stats(snapshot: &mut TreeSnapshot, stats: &[argui_layout::CustomElementStats]) {
        for stats in stats {
            if let Some(node) = snapshot
                .nodes
                .iter_mut()
                .find(|node| node.id.0 == stats.node.get())
            {
                node.summary = Some(format!(
                    "{} · layout {} · prepare {} · paint {} · available {:?} · resolved {:?}",
                    stats.type_name,
                    stats.phases.layouts,
                    stats.phases.preparations,
                    stats.phases.paints,
                    stats.phases.available,
                    stats.phases.known_size,
                ));
            }
        }
    }

    /// Builds an inspectable snapshot from a retained tree and its layout.
    ///
    /// `tree` supplies the retained elements and node identifiers; `layout` supplies
    /// their current geometry.
    #[must_use]
    pub fn snapshot(tree: &UiTree, layout: &argui_layout::LayoutOutput) -> TreeSnapshot {
        let mut nodes = Vec::new();
        collect_nodes(
            tree.root(),
            None,
            0,
            true,
            &mut CollectState {
                ids: tree.node_ids(),
                layout,
                layout_nodes: layout.nodes.iter().map(|node| (node.node, *node)).collect(),
                output: &mut nodes,
                cursor: 0,
            },
        );
        TreeSnapshot {
            revision: tree.revision(),
            nodes,
        }
    }

    /// Applies enabled inspector overrides to matching nodes in `root`.
    ///
    /// `tree` provides node identifiers used to match overrides, and `inspector`
    /// supplies the active override values.
    pub fn apply_overrides(
        root: &mut Element,
        tree: &UiTree,
        inspector: &argui_inspect::InspectorHandle,
    ) {
        let edited = inspector.overridden_nodes();
        if !edited.is_empty()
            && let Some(changed) =
                apply_tree_overrides(root, tree.node_ids(), inspector, &edited, &mut 0)
        {
            *root = changed;
        }
    }
}

struct CollectState<'a> {
    ids: &'a [NodeId],
    layout: &'a argui_layout::LayoutOutput,
    layout_nodes: std::collections::HashMap<NodeId, argui_layout::LayoutNode>,
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
    let layout_node = state.layout_nodes.get(&node).copied();
    let bounds = layout_node
        .map(|candidate| candidate.bounds)
        .unwrap_or_default();
    state.output.push(NodeSnapshot {
        id,
        parent,
        depth,
        key: element.key.clone(),
        kind: kind_name(&element.kind).to_owned(),
        summary: if element.text_privacy.protected() {
            Some("Protected text".into())
        } else {
            summary(&element.kind, element.children.len())
        },
        bounds,
        clip: layout_node.and_then(|candidate| candidate.clip),
        z_index: element.z_index,
        portal: state
            .layout
            .portals
            .iter()
            .find(|portal| portal.node == node)
            .map(|portal| PortalSnapshot {
                layer: format!("{:?}", portal.layer),
                anchor: portal.anchor.clone(),
                requested: portal.requested.map(|value| format!("{value:?}")),
                resolved: portal.resolved.map(|value| format!("{value:?}")),
                available_size: portal.available_size,
                constrained_width: portal.constrained_width,
                constrained_height: portal.constrained_height,
            }),
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
        ElementKind::Custom(_) => "custom",
        ElementKind::GpuCanvas(_) => "gpu-canvas",
        ElementKind::Container => "container",
        ElementKind::Text { .. } => "text",
        ElementKind::TextEditor { multiline, .. } => {
            if *multiline {
                "text-area"
            } else {
                "text-input"
            }
        }
        ElementKind::Image { .. } => "image",
        ElementKind::Vector { .. } => "vector",
    }
}

fn summary(kind: &ElementKind, children: usize) -> Option<String> {
    match kind {
        ElementKind::Custom(custom) => Some(custom.type_name().to_owned()),
        ElementKind::GpuCanvas(canvas) => Some(format!(
            "id={} · revision={}",
            canvas.canvas().get(),
            canvas.revision()
        )),
        ElementKind::Container => Some(format!("{children} children")),
        ElementKind::Text { content, .. } => Some(short(content.as_str())),
        ElementKind::TextEditor {
            value, placeholder, ..
        } => Some(short(if value.is_empty() { placeholder } else { value })),
        ElementKind::Image { image, fit, .. } => Some(format!("id={} · {fit:?}", image.0)),
        ElementKind::Vector { vector, fit, .. } => Some(format!("id={} · {fit:?}", vector.0)),
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
        StyleProperty::Background => element.override_background(None),
        StyleProperty::Border => element.override_border(None),
        StyleProperty::Opacity => element.override_paint_opacity(1.0),
        StyleProperty::Overflow => {
            element.style.overflow = Axes {
                x: Overflow::Visible,
                y: Overflow::Visible,
            };
        }
        StyleProperty::Transform => element.transform = Transform2D::IDENTITY,
        StyleProperty::Layer => element.layer = None,
        StyleProperty::Effects => element.effects.clear(),
        StyleProperty::Width => element.style.size.width = Dimension::auto(),
        StyleProperty::Height => element.style.size.height = Dimension::auto(),
    }
}

fn apply_property_value(element: &mut Element, property: StyleProperty, value: &StyleValue) {
    match (property, value) {
        (StyleProperty::Background, StyleValue::Srgba([red, green, blue, alpha])) => {
            element
                .override_background(Some(Fill::Solid(Color::srgba(*red, *green, *blue, *alpha))));
        }
        (StyleProperty::Border, StyleValue::Parameters(fields)) => {
            if let Some(border) = &mut element.paint.quad.border {
                let values = field_values(fields);
                set_if_present(&mut border.widths.left, &values, "left");
                set_if_present(&mut border.widths.right, &values, "right");
                set_if_present(&mut border.widths.top, &values, "top");
                set_if_present(&mut border.widths.bottom, &values, "bottom");
                let [mut red, mut green, mut blue, mut alpha] = border.color.to_srgba();
                set_if_present(&mut red, &values, "red");
                set_if_present(&mut green, &values, "green");
                set_if_present(&mut blue, &values, "blue");
                set_if_present(&mut alpha, &values, "alpha");
                border.color = Color::srgba(red, green, blue, alpha);
            }
            element.override_border(element.paint.quad.border);
        }
        (StyleProperty::Opacity, StyleValue::Number(value)) => {
            element.override_paint_opacity(*value);
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
            element.style.size.width = inspect_length(*value);
        }
        (StyleProperty::Height, StyleValue::Length(value)) => {
            element.style.size.height = inspect_length(*value);
        }
        (StyleProperty::Overflow, StyleValue::Choice(value)) => {
            let parse = |value: &str| match value.trim() {
                "Visible" => Some(Overflow::Visible),
                "Hidden" => Some(Overflow::Hidden),
                "Auto" => Some(Overflow::Auto),
                "Scroll" => Some(Overflow::Scroll),
                _ => None,
            };
            if let Some((x, y)) = value.split_once('/')
                && let (Some(x), Some(y)) = (parse(x), parse(y))
            {
                element.style.overflow = Axes { x, y };
            }
        }
        _ => {}
    }
}

fn inspect_length(value: StyleLength) -> Dimension {
    match value.unit {
        StyleUnit::Auto => Dimension::auto(),
        StyleUnit::Px => Dimension::length(value.value.max(0.0)),
        StyleUnit::Percent => Dimension::percent(value.value.max(0.0)),
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
        let [mut red, mut green, mut blue, mut alpha] = shadow.color.to_srgba();
        set_if_present(&mut red, &values, &format!("{shadow_prefix}red"));
        set_if_present(&mut green, &values, &format!("{shadow_prefix}green"));
        set_if_present(&mut blue, &values, &format!("{shadow_prefix}blue"));
        set_if_present(&mut alpha, &values, &format!("{shadow_prefix}alpha"));
        shadow.color = Color::srgba(red, green, blue, alpha);
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
        Filter::DropShadow(shadow) => {
            set_filter_value(&mut shadow.offset[0], value("offset x"));
            set_filter_value(&mut shadow.offset[1], value("offset y"));
            set_filter_value(&mut shadow.blur, value("blur"));
            let [mut red, mut green, mut blue, mut alpha] = shadow.color.to_srgba();
            set_filter_value(&mut red, value("red"));
            set_filter_value(&mut green, value("green"));
            set_filter_value(&mut blue, value("blue"));
            set_filter_value(&mut alpha, value("alpha"));
            shadow.color = Color::srgba(red, green, blue, alpha);
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
        Filter::Effect(current) => {
            for argument in &mut current.parameters {
                apply_effect_value(
                    &format!("effect {}", argument.name),
                    &mut argument.value,
                    &value,
                );
            }
        }
    }
}

fn apply_effect_value(
    label: &str,
    target: &mut argui_ui::EffectValue,
    value: &impl Fn(&str) -> Option<f32>,
) {
    use argui_ui::EffectValue;
    match target {
        EffectValue::F32(current) | EffectValue::LogicalPixels(current) => {
            set_filter_value(current, value(label));
        }
        EffectValue::I32(current) => {
            if let Some(next) = value(label) {
                *current = next.round() as i32;
            }
        }
        EffectValue::U32(current) => {
            if let Some(next) = value(label) {
                *current = next.max(0.0).round() as u32;
            }
        }
        EffectValue::Bool(current) => {
            if let Some(next) = value(label) {
                *current = next >= 0.5;
            }
        }
        EffectValue::Vec2(values) => apply_components(label, values, value),
        EffectValue::Vec3(values) => apply_components(label, values, value),
        EffectValue::Vec4(values) => apply_components(label, values, value),
        EffectValue::Mat3(values) => apply_components(label, values, value),
        EffectValue::Mat4(values) => apply_components(label, values, value),
        EffectValue::Color(color) => {
            let mut components = color.to_srgba();
            apply_components(label, &mut components, value);
            *color = Color::srgba(components[0], components[1], components[2], components[3]);
        }
    }
}

fn apply_components(label: &str, values: &mut [f32], value: &impl Fn(&str) -> Option<f32>) {
    for (component, current) in values.iter_mut().enumerate() {
        set_filter_value(current, value(&format!("{label} {component}")));
    }
}

fn set_filter_value(target: &mut f32, value: Option<f32>) {
    if let Some(value) = value {
        *target = value;
    }
}

fn apply_tree_overrides(
    element: &Element,
    ids: &[NodeId],
    inspector: &argui_inspect::InspectorHandle,
    edited: &std::collections::HashSet<InspectNodeId>,
    cursor: &mut usize,
) -> Option<Element> {
    let mut changed = None;
    if let Some(node) = ids.get(*cursor).copied()
        && edited.contains(&InspectNodeId(node.get()))
    {
        let next = changed.insert(element.clone());
        apply_overrides(next, node, inspector);
    }
    *cursor += 1;
    for (index, child) in element.children.iter().enumerate() {
        if let Some(next) = apply_tree_overrides(child, ids, inspector, edited, cursor) {
            changed.get_or_insert_with(|| element.clone()).children[index] = next;
        }
    }
    changed
}
