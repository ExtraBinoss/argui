#[path = "../../src/app/semantic_sync.rs"]
mod implementation;

use argui_accessibility::{Role, SemanticNode, SemanticNodeId, SemanticTree, Semantics};
use argui_core::{Point, Rect, Size};

fn rect(x: f32) -> Rect {
    Rect::new(Point::new(x, 0.0), Size::new(40.0, 20.0))
}

fn snapshot() -> SemanticTree {
    let root = SemanticNodeId::new(1);
    let button = SemanticNodeId::new(2);
    SemanticTree {
        root,
        focus: button,
        nodes: vec![
            SemanticNode {
                id: root,
                bounds: rect(0.0),
                semantics: Semantics::new(Role::Window),
                children: vec![button],
            },
            SemanticNode {
                id: button,
                bounds: rect(20.0),
                semantics: Semantics::new(Role::Button).label("Pause"),
                children: Vec::new(),
            },
        ],
    }
}

#[test]
fn decorative_motion_skips_semantics_but_control_motion_updates_its_bounds() {
    let tree = snapshot();
    let decorative_motion = [(1, rect(0.0)), (2, rect(20.0)), (3, rect(70.0))];
    assert!(!implementation::needs_semantic_sync(
        &tree,
        decorative_motion,
        1.0,
        true,
        true,
    ));
    let button_motion = [(1, rect(0.0)), (2, rect(24.0)), (3, rect(70.0))];
    assert!(implementation::needs_semantic_sync(
        &tree,
        button_motion,
        1.0,
        true,
        true,
    ));
}

#[test]
fn semantic_or_focus_changes_refresh_even_without_geometry_change() {
    let tree = snapshot();
    let same_bounds = [(1, rect(0.0)), (2, rect(20.0))];
    assert!(implementation::needs_semantic_sync(
        &tree,
        same_bounds,
        1.0,
        false,
        true,
    ));
    assert!(implementation::needs_semantic_sync(
        &tree,
        same_bounds,
        1.0,
        true,
        false,
    ));
}

#[test]
fn semantic_bounds_are_compared_in_physical_pixels() {
    let mut tree = snapshot();
    for node in &mut tree.nodes {
        node.bounds = Rect::new(
            Point::new(node.bounds.origin.x * 2.0, node.bounds.origin.y * 2.0),
            Size::new(node.bounds.size.width * 2.0, node.bounds.size.height * 2.0),
        );
    }
    assert!(!implementation::needs_semantic_sync(
        &tree,
        [(1, rect(0.0)), (2, rect(20.0))],
        2.0,
        true,
        true,
    ));
}

/// Missing semantic bounds use an empty rectangle, and empty trees remain unchanged.
#[test]
fn missing_bounds_and_empty_semantic_snapshots_are_handled() {
    let tree = snapshot();
    assert!(implementation::needs_semantic_sync(
        &tree,
        [],
        1.0,
        true,
        true
    ));

    let mut empty = snapshot();
    empty.nodes.clear();
    assert!(!implementation::needs_semantic_sync(
        &empty,
        [],
        1.0,
        true,
        true,
    ));
}
