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
