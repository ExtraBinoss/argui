use argui_core::{Color, Point, Rect, Size, Transform2D};
use argui_inspect::{InspectNodeId, InspectorHandle, StyleProperty, StyleValue};
use argui_layout::{LayoutNode, LayoutOutput};
use argui_paint::{
    Border, ClipBehavior, CustomEffect, Fill, Filter, GradientStop, LayerStyle, LinearGradient,
    RadialGradient, Refraction, ShaderEffectId, Shadow,
};
use argui_ui::{EffectScope, Element, Interaction, Length, UiTree};

use super::{
    CollectState, apply_property_value, apply_tree_overrides, collect_nodes, descendant_count,
    properties, property_value,
};

fn fully_styled() -> Element {
    Element::container([])
        .background(Color::WHITE)
        .border(Border::all(1.0, Color::rgb(0.0, 0.0, 0.0)))
        .paint_opacity(0.5)
        .clip(ClipBehavior::Bounds)
        .transform(Transform2D::IDENTITY.translate(2.0, 3.0))
        .layer(LayerStyle::new(Rect::default()))
        .effect(EffectScope::Content, LayerStyle::new(Rect::default()))
        .width(Length::Px(10.0))
        .height(Length::Px(20.0))
}

#[test]
fn every_typed_override_removes_only_its_resolved_property() {
    let mut root = fully_styled();
    assert!(properties(&root).iter().all(|property| property.authored));
    let tree = UiTree::new(root.clone());
    let node = tree.node_ids()[0];
    let inspector = InspectorHandle::default();
    for property in StyleProperty::ALL {
        assert!(!inspector.toggle(InspectNodeId(node.get()), property));
    }
    apply_tree_overrides(&mut root, tree.node_ids(), &inspector, &mut 0);
    assert!(properties(&root).iter().all(|property| !property.authored));
}

#[test]
fn numeric_overrides_replace_authored_values_without_mutating_the_source() {
    let source = fully_styled();
    let tree = UiTree::new(source.clone());
    let node = tree.node_ids()[0];
    let inspector = InspectorHandle::default();
    let mut transform = properties(&source)
        .into_iter()
        .find(|property| property.property == StyleProperty::Transform)
        .unwrap()
        .value;
    assert!(transform.set_field(0, 18.0));
    inspector.set_property_value(
        InspectNodeId(node.get()),
        StyleProperty::Transform,
        transform,
    );

    let mut overridden = source.clone();
    apply_tree_overrides(&mut overridden, tree.node_ids(), &inspector, &mut 0);
    assert_eq!(source.transform.translation.x, 2.0);
    assert_eq!(overridden.transform.translation.x, 18.0);
    assert!(matches!(
        inspector.property_value(InspectNodeId(node.get()), StyleProperty::Transform),
        Some(StyleValue::Parameters(_))
    ));
}

#[test]
fn snapshots_skip_uninspectable_subtrees_without_losing_siblings() {
    let root = Element::container([
        Element::container([Element::text("hidden")]).inspectable(false),
        Element::image(argui_paint::ImageId(4)).keyed("visible-image"),
        Element::text("visible-text"),
    ]);
    let tree = UiTree::new(root);
    let visible = tree.node_ids()[3];
    let layout = LayoutOutput {
        nodes: vec![LayoutNode {
            index: 3,
            node: visible,
            bounds: Rect::new(Point::new(5.0, 6.0), Size::new(7.0, 8.0)),
            layout_bounds: Rect::default(),
            clip: None,
            text_index: None,
        }],
        ..LayoutOutput::default()
    };
    let mut snapshots = Vec::new();
    collect_nodes(
        tree.root(),
        None,
        0,
        true,
        &mut CollectState {
            ids: tree.node_ids(),
            layout: &layout,
            output: &mut snapshots,
            cursor: 0,
        },
    );
    assert_eq!(descendant_count(tree.root()), 4);
    assert_eq!(snapshots.len(), 3);
    assert_eq!(snapshots[1].key.as_deref(), Some("visible-image"));
    assert_eq!(snapshots[1].bounds, layout.nodes[0].bounds);
    assert_eq!(snapshots[2].kind, "text");
}

#[test]
fn inspector_serializes_and_applies_every_filter_parameter() {
    let filters = vec![
        Filter::Blur(1.0),
        Filter::Brightness(1.0),
        Filter::Contrast(1.0),
        Filter::Saturation(1.0),
        Filter::HueRotate(1.0),
        Filter::Opacity(1.0),
        Filter::ColorMatrix([1.0; 20]),
        Filter::Refraction(Refraction {
            strength: 1.0,
            chromatic_aberration: 1.0,
            edge: 1.0,
        }),
        Filter::Custom(CustomEffect::new(ShaderEffectId(7), [1.0, 1.0])),
    ];
    let mut element = Element::container([]).layer(
        filters
            .iter()
            .cloned()
            .fold(LayerStyle::new(Rect::default()), LayerStyle::filter)
            .backdrop(Filter::Blur(1.0))
            .shadow(Shadow::drop([1.0, 1.0], 1.0, Color::rgb(0.1, 0.2, 0.3)).spread(1.0)),
    );
    let StyleValue::Parameters(mut fields) = property_value(&element, StyleProperty::Layer) else {
        panic!("layer properties stay numeric and editable");
    };
    assert!(fields.len() > 40);
    for field in &mut fields {
        field.value = 2.0;
    }
    apply_property_value(
        &mut element,
        StyleProperty::Layer,
        &StyleValue::Parameters(fields),
    );
    let layer = element.layer.as_ref().unwrap();
    assert_eq!(layer.opacity, 2.0);
    assert!(matches!(layer.filters[0], Filter::Blur(2.0)));
    assert_eq!(layer.filters[6], Filter::ColorMatrix([2.0; 20]));
    assert!(matches!(
        layer.filters[7],
        Filter::Refraction(value)
            if value.strength == 2.0
                && value.chromatic_aberration == 2.0
                && value.edge == 2.0
    ));
    assert!(matches!(
        &layer.filters[8],
        Filter::Custom(value) if value.parameters == [2.0, 2.0]
    ));
    assert_eq!(layer.shadows[0].offset, [2.0, 2.0]);
}

#[test]
fn inspector_describes_gradient_and_absent_paints_and_clamps_lengths() {
    let stops = [
        GradientStop::new(0.0, Color::rgb(0.0, 0.0, 0.0)),
        GradientStop::new(1.0, Color::WHITE),
    ];
    let linear = LinearGradient::new(Point::default(), Point::new(1.0, 0.0), stops).unwrap();
    let radial = RadialGradient::new(Point::default(), Point::new(1.0, 1.0), stops).unwrap();
    let mut element = Element::container([]);
    assert_eq!(
        property_value(&element, StyleProperty::Background),
        StyleValue::Summary("none".into())
    );
    element.paint.quad.background = Some(Fill::Linear(linear));
    assert!(matches!(
        property_value(&element, StyleProperty::Background),
        StyleValue::Summary(value) if value.contains("linear")
    ));
    element.paint.quad.background = Some(Fill::Radial(radial));
    assert!(matches!(
        property_value(&element, StyleProperty::Background),
        StyleValue::Summary(value) if value.contains("radial")
    ));

    apply_property_value(
        &mut element,
        StyleProperty::Width,
        &StyleValue::Length(argui_inspect::StyleLength {
            value: -4.0,
            unit: argui_inspect::StyleUnit::Px,
        }),
    );
    apply_property_value(
        &mut element,
        StyleProperty::Height,
        &StyleValue::Length(argui_inspect::StyleLength {
            value: 0.5,
            unit: argui_inspect::StyleUnit::Percent,
        }),
    );
    assert_eq!(element.style.width, Length::Px(0.0));
    assert_eq!(element.style.height, Length::Percent(0.5));
}

#[test]
fn snapshots_distinguish_visual_interactive_and_hidden_structure() {
    let root = Element::container([
        Element::container([]).keyed("empty"),
        Element::container([])
            .keyed("painted")
            .background(Color::WHITE),
        Element::container([])
            .keyed("effect")
            .effect(EffectScope::Content, LayerStyle::new(Rect::default())),
        Element::container([])
            .keyed("layer")
            .layer(LayerStyle::new(Rect::default())),
        Element::container([])
            .keyed("interactive")
            .interaction(Interaction::default()),
        Element::container([Element::text("This text is deliberately longer than fifty-two characters so its summary is shortened")])
            .keyed("hidden-parent")
            .layer(LayerStyle::new(Rect::default()).opacity(0.0)),
    ]);
    let tree = UiTree::new(root);
    let mut snapshots = Vec::new();
    collect_nodes(
        tree.root(),
        None,
        0,
        true,
        &mut CollectState {
            ids: tree.node_ids(),
            layout: &LayoutOutput::default(),
            output: &mut snapshots,
            cursor: 0,
        },
    );
    let find = |key: &str| {
        snapshots
            .iter()
            .find(|node| node.key.as_deref() == Some(key))
            .unwrap()
    };
    assert!(!find("empty").painted);
    assert!(find("painted").painted);
    assert!(find("effect").painted);
    assert!(find("layer").painted);
    assert!(find("interactive").interactive);
    assert!(!find("hidden-parent").visible);
    let text = snapshots.last().unwrap();
    assert!(!text.visible);
    assert!(text.summary.as_ref().unwrap().ends_with('…'));
}
