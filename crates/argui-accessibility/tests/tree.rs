use argui_accessibility::{
    Role, SemanticAction, SemanticNode, SemanticNodeId, SemanticTree, Semantics,
};
use argui_core::{Point, Rect, Size};

fn node(id: u64, label: &str) -> SemanticNode {
    SemanticNode {
        id: SemanticNodeId::new(id),
        bounds: Rect::new(Point::new(0.0, 0.0), Size::new(40.0, 20.0)),
        semantics: Semantics::new(Role::Button)
            .label(label)
            .action(SemanticAction::Click),
        children: Vec::new(),
    }
}

#[test]
fn diff_only_contains_changed_semantics() {
    let old = SemanticTree {
        root: SemanticNodeId::new(1),
        focus: SemanticNodeId::new(2),
        nodes: vec![node(1, "root"), node(2, "Save")],
    };
    let next = SemanticTree {
        root: old.root,
        focus: old.focus,
        nodes: vec![node(1, "root"), node(2, "Saved")],
    };
    let patch = old.diff(&next);
    assert_eq!(patch.upserts, vec![node(2, "Saved")]);
    assert!(patch.removed.is_empty());
    assert_eq!(patch.focus, None);
}

#[test]
fn diff_reports_removed_nodes_and_focus() {
    let old = SemanticTree {
        root: SemanticNodeId::new(1),
        focus: SemanticNodeId::new(2),
        nodes: vec![node(1, "root"), node(2, "Save")],
    };
    let next = SemanticTree {
        root: old.root,
        focus: old.root,
        nodes: vec![node(1, "root")],
    };
    let patch = old.diff(&next);
    assert_eq!(patch.removed, vec![SemanticNodeId::new(2)]);
    assert_eq!(patch.focus, Some(SemanticNodeId::new(1)));
}

#[test]
fn node_lookup_root_changes_and_empty_patches_are_explicit() {
    let old = SemanticTree {
        root: SemanticNodeId::new(1),
        focus: SemanticNodeId::new(1),
        nodes: vec![node(1, "old")],
    };
    let unchanged = old.clone();
    let replacement = SemanticTree {
        root: SemanticNodeId::new(2),
        focus: SemanticNodeId::new(2),
        nodes: vec![node(2, "new")],
    };

    assert_eq!(old.node(old.root), Some(&old.nodes[0]));
    assert_eq!(old.node(SemanticNodeId::new(99)), None);
    assert!(old.diff(&unchanged).is_empty());
    let patch = old.diff(&replacement);
    assert!(!patch.is_empty());
    assert_eq!(patch.root, Some(replacement.root));
    assert_eq!(patch.removed, vec![old.root]);
}

#[test]
fn nested_host_coordinates_follow_parent_movement_and_reparenting() {
    use argui_core::{Point, Size};
    let mut tree = SemanticTree {
        root: SemanticNodeId::new(1),
        focus: SemanticNodeId::new(3),
        nodes: vec![node(1, "root"), node(2, "group"), node(3, "handle")],
    };
    tree.nodes[0].bounds.origin = Point::new(10.0, 15.0);
    tree.nodes[0].children = vec![SemanticNodeId::new(2)];
    tree.nodes[1].bounds.origin = Point::new(100.0, 200.0);
    tree.nodes[1].children = vec![SemanticNodeId::new(3)];
    tree.nodes[2].bounds.origin = Point::new(150.0, 230.0);
    tree.nodes[2].bounds.size = Size::new(12.0, 40.0);
    let local = tree.parent_relative_bounds();
    assert_eq!(local[&tree.nodes[0].id].origin, Point::new(10.0, 15.0));
    assert_eq!(local[&tree.nodes[1].id].origin, Point::new(90.0, 185.0));
    assert_eq!(local[&tree.nodes[2].id].origin, Point::new(50.0, 30.0));
    assert_eq!(local[&tree.nodes[2].id].size, Size::new(12.0, 40.0));
    tree.nodes[1].bounds.origin.x = 120.0;
    assert_eq!(
        tree.parent_relative_bounds()[&tree.nodes[2].id].origin.x,
        30.0
    );
    tree.nodes[1].children.clear();
    tree.nodes[0].children.push(SemanticNodeId::new(3));
    assert_eq!(
        tree.parent_relative_bounds()[&tree.nodes[2].id].origin,
        Point::new(140.0, 215.0)
    );
    tree.nodes[0].children.push(SemanticNodeId::new(99));
    assert_eq!(tree.parent_relative_bounds().len(), 3);
}
