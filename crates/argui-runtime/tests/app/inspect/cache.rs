use argui_core::{Color, Point, Rect, Size};
use argui_layout::{LayoutNode, LayoutOutput};
use argui_runtime::{Inspection, InspectionCache};
use argui_ui::{Element, UiTree, length};

fn view(label: &str, tools: &str) -> Element {
    Element::column([
        Element::text(label).keyed("app"),
        Element::text(tools).keyed("tools").inspectable(false),
    ])
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
