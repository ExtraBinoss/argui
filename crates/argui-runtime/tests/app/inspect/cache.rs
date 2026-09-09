use argui_core::{Color, Point, Rect, Size, Transform2D};
use argui_layout::{LayoutNode, LayoutOutput, PortalLayout};
use argui_paint::LayerStyle;
use argui_runtime::{Inspection, InspectionCache};
use argui_ui::{EffectScope, Element, UiTree, WindowLayer, length};

fn view(label: &str, tools: &str) -> Element {
    Element::column([
        Element::text(label).keyed("app"),
        Element::text(tools).keyed("tools").inspectable(false),
    ])
}

#[test]
fn retained_leaf_snapshots_hide_private_text_and_republish_privacy_changes() {
    let leaf = Element::text("private value").text_privacy(argui_ui::TextPrivacy::Password);
    let mut tree = UiTree::new(leaf.clone());
    let mut cache = InspectionCache::default();
    let layout = LayoutOutput::default();
    let snapshot = cache.snapshot(&tree, &layout).unwrap();
    assert_eq!(snapshot.nodes[0].summary.as_deref(), Some("Protected text"));
    assert!(cache.snapshot(&tree, &layout).is_none());
    tree.update(leaf.clone().text_privacy(argui_ui::TextPrivacy::Public));
    let snapshot = cache.snapshot(&tree, &layout).unwrap();
    assert_ne!(snapshot.nodes[0].summary.as_deref(), Some("Protected text"));
    assert_eq!(snapshot, Inspection::snapshot(&tree, &layout));
    tree.update(leaf.text_privacy(argui_ui::TextPrivacy::RevealedPassword));
    let snapshot = cache.snapshot(&tree, &layout).unwrap();
    assert_eq!(snapshot.nodes[0].summary.as_deref(), Some("Protected text"));
    assert!(cache.snapshot(&tree, &layout).is_none());
}

#[test]
fn tools_updates_do_not_republish_the_application() {
    let mut cache = InspectionCache::default();
    let mut tree = UiTree::new(view("application", "row 1"));
    let layout = LayoutOutput::default();
    assert_eq!(
        cache.snapshot(&tree, &layout),
        Some(Inspection::snapshot(&tree, &layout))
    );
    assert!(cache.snapshot(&tree, &layout).is_none());
    tree.update(view("application", "row 400"));
    assert!(cache.snapshot(&tree, &layout).is_none());
    tree.update(view("changed application", "row 400"));
    assert_eq!(
        cache.snapshot(&tree, &layout),
        Some(Inspection::snapshot(&tree, &layout))
    );
}

#[test]
fn geometry_style_and_removal_invalidate_the_cache() {
    let mut cache = InspectionCache::default();
    let mut tree = UiTree::new(view("app", "tools"));
    let mut layout = LayoutOutput::default();
    cache.snapshot(&tree, &layout);
    layout.nodes.push(LayoutNode {
        index: 1,
        node: tree.node_ids()[1],
        bounds: Rect::new(Point::new(1.0, 2.0), Size::new(30.0, 40.0)),
        layout_bounds: Rect::default(),
        clip: None,
        text_index: None,
    });
    assert!(cache.snapshot(&tree, &layout).is_some());
    layout.nodes[0].clip = Some(Rect::new(Point::default(), Size::new(10.0, 10.0)));
    assert!(cache.snapshot(&tree, &layout).is_some());
    tree.update(
        view("app", "tools")
            .width(length(200.0))
            .background(Color::WHITE),
    );
    assert_eq!(
        cache.snapshot(&tree, &layout),
        Some(Inspection::snapshot(&tree, &layout))
    );
    tree.update(Element::column([]));
    assert_eq!(
        cache.snapshot(&tree, &layout),
        Some(Inspection::snapshot(&tree, &layout))
    );
    assert!(cache.snapshot(&tree, &layout).is_none());
}

#[test]
fn hidden_subtrees_preserve_sibling_identity_and_empty_snapshots_settle() {
    let mut cache = InspectionCache::default();
    let mut tree = UiTree::new(Element::column([
        Element::column([Element::text("hidden")]).inspectable(false),
        Element::text("visible").keyed("visible"),
    ]));
    let layout = LayoutOutput::default();
    assert_eq!(
        cache.snapshot(&tree, &layout),
        Some(Inspection::snapshot(&tree, &layout))
    );
    assert!(cache.snapshot(&tree, &layout).is_none());
    tree.update(Element::column([]).inspectable(false));
    assert!(cache.snapshot(&tree, &layout).unwrap().nodes.is_empty());
    assert!(cache.snapshot(&tree, &layout).is_none());
}

fn republishes_for_distinct_content(changed: Element) {
    let mut cache = InspectionCache::default();
    let original = UiTree::new(Element::container([]));
    let layout = LayoutOutput::default();
    assert!(cache.snapshot(&original, &layout).is_some());
    let changed = UiTree::new(changed);
    assert!(cache.snapshot(&changed, &layout).is_some());
}

#[test]
fn content_field_changes_invalidate_the_matching_cached_entry() {
    republishes_for_distinct_content(Element::container([]).keyed("changed"));
    republishes_for_distinct_content(Element::container([]).background(Color::WHITE));
    republishes_for_distinct_content(
        Element::container([]).transform(Transform2D::IDENTITY.translate(1.0, 2.0)),
    );
    republishes_for_distinct_content(
        Element::container([]).layer(LayerStyle::new(Rect::default())),
    );
    republishes_for_distinct_content(
        Element::container([]).effect(EffectScope::Content, LayerStyle::new(Rect::default())),
    );
    republishes_for_distinct_content(Element::container([]).z_index(1));
}

#[test]
fn replacing_a_child_with_an_incompatible_kind_invalidates_its_identity() {
    let mut cache = InspectionCache::default();
    let mut tree = UiTree::new(Element::column([Element::text("old")]));
    let layout = LayoutOutput::default();
    assert!(cache.snapshot(&tree, &layout).is_some());
    let old_child = tree.node_ids()[1];

    tree.update(Element::column([Element::image(argui_paint::ImageId(1))]));
    assert_ne!(tree.node_ids()[1], old_child);
    assert!(cache.snapshot(&tree, &layout).is_some());
}

#[test]
fn portal_metadata_changes_republish_without_content_changes() {
    let mut cache = InspectionCache::default();
    let tree = UiTree::new(Element::container([]));
    let node = tree.node_ids()[0];
    let mut layout = LayoutOutput::default();
    assert!(cache.snapshot(&tree, &layout).is_some());
    layout.portals.push(PortalLayout {
        node,
        layer: WindowLayer::Popover,
        anchor: Some("trigger".into()),
        requested: None,
        resolved: None,
        bounds: Rect::default(),
        available_size: Size::default(),
        constrained_width: false,
        constrained_height: false,
    });
    assert!(cache.snapshot(&tree, &layout).is_some());
    assert!(cache.snapshot(&tree, &layout).is_none());

    layout.portals[0].anchor = Some("other-trigger".into());
    assert!(cache.snapshot(&tree, &layout).is_some());
}
