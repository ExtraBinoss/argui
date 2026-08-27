use argui_core::{Color, Point, Rect, Size, Transform2D};
use argui_inspect::{InspectNodeId, InspectorHandle, StyleProperty, StyleValue};
use argui_layout::{LayoutNode, LayoutOutput};
use argui_paint::{Border, ClipBehavior, LayerStyle};
use argui_ui::{EffectScope, Element, Length, UiTree};

use super::{CollectState, apply_tree_overrides, collect_nodes, descendant_count, properties};

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
