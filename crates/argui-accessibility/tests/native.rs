#![cfg(not(target_arch = "wasm32"))]

use accesskit::{Action, ActionData, ActionRequest, NodeId, TreeId};
use accesskit_consumer::Tree as ConsumerTree;
use argui_accessibility::{
    AccessKitTree, LiveRegion, Orientation, Role, SemanticAction, SemanticNode, SemanticNodeId,
    SemanticRequest, SemanticState, SemanticTree, SemanticValue, Semantics,
};
use argui_core::{Point, Rect, Size};

#[test]
fn complete_snapshots_are_valid_accesskit_trees() {
    let root = SemanticNodeId::new(1);
    let button = SemanticNodeId::new(2);
    let tree = SemanticTree {
        root,
        focus: button,
        nodes: vec![
            SemanticNode {
                id: root,
                bounds: Rect::new(Point::default(), Size::new(800.0, 600.0)),
                semantics: Semantics::new(Role::Window).label("Demo"),
                children: vec![button],
            },
            SemanticNode {
                id: button,
                bounds: Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 40.0)),
                semantics: Semantics::new(Role::Button)
                    .label("Save")
                    .action(SemanticAction::Click),
                children: Vec::new(),
            },
        ],
    };

    let consumer = ConsumerTree::new(AccessKitTree::full(&tree), true);
    assert_eq!(consumer.state().root().label().as_deref(), Some("Demo"));
    assert_eq!(
        consumer.state().focus().and_then(|node| node.label()),
        Some("Save".into())
    );
}

#[test]
fn incremental_updates_only_replace_changed_nodes_and_root_metadata() {
    let root = SemanticNodeId::new(1);
    let child = SemanticNodeId::new(2);
    let original = SemanticTree {
        root,
        focus: root,
        nodes: vec![SemanticNode {
            id: root,
            bounds: Rect::default(),
            semantics: Semantics::new(Role::Window).label("Before"),
            children: Vec::new(),
        }],
    };
    let changed = SemanticTree {
        root,
        focus: root,
        nodes: vec![SemanticNode {
            id: root,
            bounds: Rect::default(),
            semantics: Semantics::new(Role::Window).label("After"),
            children: Vec::new(),
        }],
    };

    let update = AccessKitTree::patch(&original.diff(&changed), &changed);
    assert!(update.tree.is_none());
    assert_eq!(update.nodes.len(), 1);

    let replaced = SemanticTree {
        root: child,
        focus: child,
        nodes: vec![SemanticNode {
            id: child,
            bounds: Rect::default(),
            semantics: Semantics::new(Role::Window).label("Replacement"),
            children: Vec::new(),
        }],
    };
    let update = AccessKitTree::patch(&changed.diff(&replaced), &replaced);
    assert_eq!(update.tree.unwrap().root, NodeId(child.get()));
    assert_eq!(update.focus, NodeId(child.get()));
}

#[test]
fn rich_nodes_lower_every_value_state_and_relation() {
    let root = SemanticNodeId::new(1);
    let slider = SemanticNodeId::new(2);
    let text = SemanticNodeId::new(3);
    let unbounded = SemanticNodeId::new(4);
    let mut semantics = Semantics::new(Role::Slider)
        .label("Volume")
        .description("Output volume")
        .value(SemanticValue::Number {
            value: 42.0,
            minimum: Some(0.0),
            maximum: Some(100.0),
            step: Some(2.0),
        })
        .state(SemanticState {
            disabled: true,
            selected: true,
            checked: Some(true),
            expanded: Some(false),
            required: true,
            read_only: true,
            invalid: true,
            modal: true,
        })
        .live(LiveRegion::Assertive)
        .orientation(Orientation::Horizontal)
        .level(3)
        .position_in_set(2, 5);
    for action in [
        SemanticAction::Click,
        SemanticAction::Focus,
        SemanticAction::Blur,
        SemanticAction::Increment,
        SemanticAction::Decrement,
        SemanticAction::Expand,
        SemanticAction::Collapse,
        SemanticAction::SetValue,
        SemanticAction::ScrollIntoView,
    ] {
        semantics = semantics.action(action);
    }
    let tree = SemanticTree {
        root,
        focus: slider,
        nodes: vec![
            SemanticNode {
                id: root,
                bounds: Rect::default(),
                semantics: Semantics::new(Role::Window),
                children: vec![slider, text, unbounded],
            },
            SemanticNode {
                id: slider,
                bounds: Rect::default(),
                semantics,
                children: Vec::new(),
            },
            SemanticNode {
                id: text,
                bounds: Rect::default(),
                semantics: Semantics::new(Role::TextInput)
                    .value(SemanticValue::Text("value".into()))
                    .state(SemanticState {
                        checked: Some(false),
                        expanded: Some(true),
                        ..SemanticState::default()
                    })
                    .live(LiveRegion::Polite)
                    .orientation(Orientation::Vertical),
                children: Vec::new(),
            },
            SemanticNode {
                id: unbounded,
                bounds: Rect::default(),
                semantics: Semantics::new(Role::Progress).value(SemanticValue::Number {
                    value: 0.5,
                    minimum: None,
                    maximum: None,
                    step: None,
                }),
                children: Vec::new(),
            },
        ],
    };

    let consumer = ConsumerTree::new(AccessKitTree::full(&tree), true);
    let node = consumer
        .state()
        .node_by_tree_local_id(NodeId(2), TreeId::ROOT)
        .unwrap();
    assert_eq!(node.numeric_value(), Some(42.0));
    assert_eq!(node.min_numeric_value(), Some(0.0));
    assert_eq!(node.max_numeric_value(), Some(100.0));
    assert_eq!(node.numeric_value_step(), Some(2.0));
    assert!(node.is_disabled());
    assert!(node.is_modal());
    assert_eq!(node.is_selected(), Some(true));
    assert_eq!(node.level(), Some(3));
    assert_eq!(node.position_in_set(), Some(2));
    assert_eq!(node.size_of_set(), Some(5));
    assert_eq!(node.orientation(), Some(accesskit::Orientation::Horizontal));
    assert_eq!(node.live(), accesskit::Live::Assertive);
    let unbounded = consumer
        .state()
        .node_by_tree_local_id(NodeId(4), TreeId::ROOT)
        .unwrap();
    assert_eq!(unbounded.numeric_value(), Some(0.5));
    assert_eq!(unbounded.min_numeric_value(), None);
    assert_eq!(unbounded.max_numeric_value(), None);
    assert_eq!(unbounded.numeric_value_step(), None);
}

#[test]
fn accesskit_actions_lower_supported_data_and_reject_unknown_actions() {
    let actions = [
        (Action::Click, SemanticAction::Click),
        (Action::Focus, SemanticAction::Focus),
        (Action::Blur, SemanticAction::Blur),
        (Action::Increment, SemanticAction::Increment),
        (Action::Decrement, SemanticAction::Decrement),
        (Action::Expand, SemanticAction::Expand),
        (Action::Collapse, SemanticAction::Collapse),
        (Action::SetValue, SemanticAction::SetValue),
        (Action::ScrollIntoView, SemanticAction::ScrollIntoView),
    ];
    for (action, expected) in actions {
        let request = SemanticRequest::try_from(ActionRequest {
            action,
            target_tree: TreeId::ROOT,
            target_node: NodeId(7),
            data: None,
        })
        .unwrap();
        assert_eq!(request.target, SemanticNodeId::new(7));
        assert_eq!(request.action, expected);
    }

    let text = SemanticRequest::try_from(ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: NodeId(7),
        data: Some(ActionData::Value("hello".into())),
    })
    .unwrap();
    assert_eq!(text.value, Some(SemanticValue::Text("hello".into())));
    let number = SemanticRequest::try_from(ActionRequest {
        action: Action::SetValue,
        target_tree: TreeId::ROOT,
        target_node: NodeId(7),
        data: Some(ActionData::NumericValue(4.5)),
    })
    .unwrap();
    assert!(matches!(
        number.value,
        Some(SemanticValue::Number { value: 4.5, .. })
    ));
    assert!(
        SemanticRequest::try_from(ActionRequest {
            action: Action::ShowContextMenu,
            target_tree: TreeId::ROOT,
            target_node: NodeId(7),
            data: None,
        })
        .is_err()
    );
}
