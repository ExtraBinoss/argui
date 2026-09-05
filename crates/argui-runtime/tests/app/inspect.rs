use argui_core::{Color, Point, Rect, Size, Transform2D};
use argui_inspect::{InspectNodeId, InspectorHandle, StyleField, StyleProperty, StyleValue};
use argui_layout::{LayoutNode, LayoutOutput};
use argui_paint::{
    Border, EffectId, EffectInstance, EffectValue, Fill, Filter, GradientStop, LayerStyle,
    LinearGradient, RadialGradient, Refraction, Shadow,
};
use argui_runtime::Inspection;
use argui_ui::{Axes, Dimension, EffectScope, Element, ElementKind, Interaction, Overflow, UiTree};

mod cache;

fn fully_styled() -> Element {
    Element::container([])
        .background(Color::WHITE)
        .border(Border::all(1.0, Color::srgb(0.0, 0.0, 0.0)))
        .paint_opacity(0.5)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .transform(Transform2D::IDENTITY.translate(2.0, 3.0))
        .layer(LayerStyle::new(Rect::default()))
        .effect(EffectScope::Content, LayerStyle::new(Rect::default()))
        .width(Dimension::length(10.0))
        .height(Dimension::length(20.0))
}

fn property(snapshot: &argui_inspect::NodeSnapshot, property: StyleProperty) -> StyleValue {
    snapshot
        .properties
        .iter()
        .find(|candidate| candidate.property == property)
        .unwrap()
        .value
        .clone()
}

#[test]
fn every_typed_override_removes_only_its_resolved_property() {
    let mut root = fully_styled();
    let tree = UiTree::new(root.clone());
    let node = InspectNodeId(tree.node_ids()[0].get());
    let inspector = InspectorHandle::default();
    for property in StyleProperty::ALL {
        assert!(!inspector.toggle(node, property));
    }

    Inspection::apply_overrides(&mut root, &tree, &inspector);
    let snapshot = Inspection::snapshot(&UiTree::new(root), &LayoutOutput::default());
    assert!(
        snapshot.nodes[0]
            .properties
            .iter()
            .all(|property| !property.authored)
    );
}

#[test]
fn numeric_overrides_replace_authored_values_without_mutating_the_source() {
    let source = fully_styled();
    let tree = UiTree::new(source.clone());
    let node = InspectNodeId(tree.node_ids()[0].get());
    let inspector = InspectorHandle::default();
    inspector.set_property_value(
        node,
        StyleProperty::Transform,
        StyleValue::Parameters(vec![StyleField {
            label: "translate x".into(),
            value: 18.0,
        }]),
    );

    let mut overridden = source.clone();
    Inspection::apply_overrides(&mut overridden, &tree, &inspector);
    assert_eq!(source.transform.translation.x, 2.0);
    assert_eq!(overridden.transform.translation.x, 18.0);
    assert!(matches!(
        inspector.property_value(node, StyleProperty::Transform),
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
    let snapshot = Inspection::snapshot(&tree, &layout);
    assert_eq!(snapshot.nodes.len(), 3);
    assert_eq!(snapshot.nodes[1].key.as_deref(), Some("visible-image"));
    assert_eq!(snapshot.nodes[1].bounds, layout.nodes[0].bounds);
    assert_eq!(snapshot.nodes[2].kind, "text");
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
        Filter::Effect(EffectInstance::new(
            EffectId::new("test.inspect"),
            [
                ("float", EffectValue::F32(1.0)),
                ("pixels", EffectValue::LogicalPixels(1.0)),
                ("signed", EffectValue::I32(1)),
                ("unsigned", EffectValue::U32(1)),
                ("boolean", EffectValue::Bool(false)),
                ("vec2", EffectValue::Vec2([1.0; 2])),
                ("vec3", EffectValue::Vec3([1.0; 3])),
                ("vec4", EffectValue::Vec4([1.0; 4])),
                ("mat3", EffectValue::Mat3([1.0; 9])),
                ("mat4", EffectValue::Mat4([1.0; 16])),
                ("color", EffectValue::Color(Color::WHITE)),
            ],
        )),
    ];
    let element = Element::container([]).layer(
        filters
            .iter()
            .cloned()
            .fold(LayerStyle::new(Rect::default()), LayerStyle::filter)
            .backdrop(Filter::Blur(1.0))
            .shadow(Shadow::drop([1.0, 1.0], 1.0, Color::srgb(0.1, 0.2, 0.3)).spread(1.0)),
    );
    let tree = UiTree::new(element.clone());
    let node = InspectNodeId(tree.node_ids()[0].get());
    let mut fields = match property(
        &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
        StyleProperty::Layer,
    ) {
        StyleValue::Parameters(fields) => fields,
        _ => panic!("layer properties stay numeric and editable"),
    };
    assert!(fields.len() > 40);
    for field in &mut fields {
        field.value = 2.0;
    }
    let inspector = InspectorHandle::default();
    inspector.set_property_value(node, StyleProperty::Layer, StyleValue::Parameters(fields));
    let mut overridden = element;
    Inspection::apply_overrides(&mut overridden, &tree, &inspector);
    let layer = overridden.layer.as_ref().unwrap();
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
        Filter::Effect(value)
            if value.parameters[0].value == EffectValue::F32(2.0)
                && value.parameters[1].value == EffectValue::LogicalPixels(2.0)
                && value.parameters[2].value == EffectValue::I32(2)
                && value.parameters[3].value == EffectValue::U32(2)
                && value.parameters[4].value == EffectValue::Bool(true)
                && value.parameters[5].value == EffectValue::Vec2([2.0; 2])
                && value.parameters[6].value == EffectValue::Vec3([2.0; 3])
                && value.parameters[7].value == EffectValue::Vec4([2.0; 4])
                && value.parameters[8].value == EffectValue::Mat3([2.0; 9])
                && value.parameters[9].value == EffectValue::Mat4([2.0; 16])
                && value.parameters[10].value == EffectValue::Color(Color::srgba(2.0, 2.0, 2.0, 2.0))
    ));
    assert_eq!(layer.shadows[0].offset, [2.0, 2.0]);

    let mut sparse_fields = match property(
        &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
        StyleProperty::Layer,
    ) {
        StyleValue::Parameters(fields) => fields,
        _ => unreachable!(),
    };
    sparse_fields.retain(|field| !field.label.ends_with("signed"));
    for field in &mut sparse_fields {
        field.value = if field.label.ends_with("boolean") {
            0.0
        } else {
            2.0
        };
    }
    inspector.set_property_value(
        node,
        StyleProperty::Layer,
        StyleValue::Parameters(sparse_fields),
    );
    Inspection::apply_overrides(&mut overridden, &tree, &inspector);
    let layer = overridden.layer.as_ref().unwrap();
    assert!(matches!(
        &layer.filters[8],
        Filter::Effect(value)
            if value.parameters[2].value == EffectValue::I32(2)
                && value.parameters[4].value == EffectValue::Bool(false)
    ));
}

#[test]
fn inspector_describes_gradient_and_absent_paints_and_clamps_lengths() {
    let stops = [
        GradientStop::new(0.0, Color::srgb(0.0, 0.0, 0.0)),
        GradientStop::new(1.0, Color::WHITE),
    ];
    let linear = LinearGradient::new(
        Point::default(),
        Point::new(1.0, 0.0),
        argui_paint::ColorInterpolation::Oklab,
        stops,
    )
    .unwrap();
    let radial = RadialGradient::new(
        Point::default(),
        Point::new(1.0, 1.0),
        argui_paint::ColorInterpolation::Oklab,
        stops,
    )
    .unwrap();
    let mut element = Element::container([]);
    let summary = |element: &Element| {
        let tree = UiTree::new(element.clone());
        property(
            &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
            StyleProperty::Background,
        )
    };
    assert_eq!(summary(&element), StyleValue::Summary("none".into()));
    element.paint.quad.background = Some(Fill::Linear(linear));
    assert!(matches!(summary(&element), StyleValue::Summary(value) if value.contains("linear")));
    element.paint.quad.background = Some(Fill::Radial(radial));
    assert!(matches!(summary(&element), StyleValue::Summary(value) if value.contains("radial")));

    let tree = UiTree::new(element.clone());
    let node = InspectNodeId(tree.node_ids()[0].get());
    let inspector = InspectorHandle::default();
    inspector.set_property_value(
        node,
        StyleProperty::Width,
        StyleValue::Length(argui_inspect::StyleLength {
            value: -4.0,
            unit: argui_inspect::StyleUnit::Px,
        }),
    );
    inspector.set_property_value(
        node,
        StyleProperty::Height,
        StyleValue::Length(argui_inspect::StyleLength {
            value: 0.5,
            unit: argui_inspect::StyleUnit::Percent,
        }),
    );
    Inspection::apply_overrides(&mut element, &tree, &inspector);
    assert_eq!(element.style.size.width, Dimension::length(0.0));
    assert_eq!(element.style.size.height, Dimension::percent(0.5));
}

#[test]
fn primitive_and_optional_property_edits_cover_present_and_absent_targets() {
    let mut element = fully_styled();
    let tree = UiTree::new(element.clone());
    let node = InspectNodeId(tree.node_ids()[0].get());
    let inspector = InspectorHandle::default();
    inspector.set_property_value(
        node,
        StyleProperty::Background,
        StyleValue::Srgba([0.1, 0.2, 0.3, 0.4]),
    );
    Inspection::apply_overrides(&mut element, &tree, &inspector);
    assert_eq!(
        element.paint.quad.background,
        Some(Fill::Solid(Color::srgba(0.1, 0.2, 0.3, 0.4)))
    );

    let border = property(
        &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
        StyleProperty::Border,
    );
    inspector.set_property_value(node, StyleProperty::Border, border.clone());
    Inspection::apply_overrides(&mut element, &tree, &inspector);
    assert!(element.paint.quad.border.is_some());

    let mut without_border = Element::container([]);
    let absent_tree = UiTree::new(without_border.clone());
    let absent_node = InspectNodeId(absent_tree.node_ids()[0].get());
    let absent_inspector = InspectorHandle::default();
    absent_inspector.set_property_value(absent_node, StyleProperty::Border, border);
    Inspection::apply_overrides(&mut without_border, &absent_tree, &absent_inspector);
    assert!(without_border.paint.quad.border.is_none());

    inspector.set_property_value(node, StyleProperty::Opacity, StyleValue::Number(2.0));
    inspector.set_property_value(
        node,
        StyleProperty::Layer,
        property(
            &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
            StyleProperty::Layer,
        ),
    );
    inspector.set_property_value(
        node,
        StyleProperty::Effects,
        property(
            &Inspection::snapshot(&tree, &LayoutOutput::default()).nodes[0],
            StyleProperty::Effects,
        ),
    );
    Inspection::apply_overrides(&mut element, &tree, &inspector);
    assert_eq!(element.paint.quad.opacity, 1.0);
}

#[test]
fn snapshots_distinguish_visual_interactive_and_hidden_structure() {
    let root = Element::container([
        Element::container([]).keyed("empty"),
        Element::container([]).keyed("painted").background(Color::WHITE),
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
    let snapshots = Inspection::snapshot(&tree, &LayoutOutput::default()).nodes;
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

#[test]
fn snapshots_cover_every_media_summary_and_empty_identity_input() {
    let text_input = |initial_value: &str| {
        let mut element = Element::container([]);
        element.kind = ElementKind::TextEditor {
            value: initial_value.into(),
            placeholder: "placeholder".into(),
            multiline: false,
            read_only: false,
            filter: argui_ui::TextInputFilter::Any,
            text: argui_text::TextStyle::default(),
            placeholder_text: argui_text::TextStyle::default(),
            selection: Color::WHITE,
            caret: argui_ui::CaretStyle::default(),
        };
        element
    };
    let root = Element::container([
        text_input(""),
        text_input("value"),
        Element::vector(argui_paint::VectorId(8)),
    ]);
    let tree = UiTree::new(root);
    let snapshots = Inspection::snapshot(&tree, &LayoutOutput::default()).nodes;
    assert_eq!(snapshots[1].kind, "text-input");
    assert_eq!(snapshots[1].summary.as_deref(), Some("placeholder"));
    assert_eq!(snapshots[2].summary.as_deref(), Some("value"));
    assert_eq!(snapshots[3].kind, "vector");
    assert_eq!(snapshots[3].summary.as_deref(), Some("id=8 · Contain"));

    let empty = UiTree::new(Element::container([]));
    assert_eq!(
        Inspection::snapshot(&empty, &LayoutOutput::default())
            .nodes
            .len(),
        1
    );
}
