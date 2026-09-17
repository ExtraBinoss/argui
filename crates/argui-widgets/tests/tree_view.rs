mod tree_view {
    use std::collections::BTreeSet;

    use argui_core::{Key, KeyInput, KeyState, Modifiers};
    use argui_ui::{ActivationSource, ClickEvent, Element, UiEvent, UiEventKind, UiTree};
    use argui_widgets::{TreeAction, TreeNode, TreeView, VList};

    mod cache;

    fn view<'a>(nodes: &'a [TreeNode], collapsed: &'a BTreeSet<String>) -> TreeView<'a> {
        TreeView::new(nodes, None, collapsed, VList::new("tree", 24.0, 120.0, 0.0))
    }

    fn click(key: &str, count: u8) -> UiEvent {
        let tree = UiTree::new(Element::container([]));
        UiEvent::new(
            tree.node_ids()[0],
            Some(key.into()),
            UiEventKind::Click(ClickEvent {
                source: ActivationSource::Accessibility,
                count,
            }),
        )
    }

    fn key(key: &str, input: Key, state: KeyState) -> UiEvent {
        let tree = UiTree::new(Element::container([]));
        UiEvent::new(
            tree.node_ids()[0],
            Some(key.into()),
            UiEventKind::KeyInput(KeyInput {
                key: input,
                state,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        )
    }

    #[test]
    fn tree_actions_distinguish_leaf_double_clicks_and_collapsed_parents() {
        let nodes = [
            TreeNode {
                key: "parent".into(),
                label: "Parent".into(),
                depth: 0,
                icon: None,
            },
            TreeNode {
                key: "child".into(),
                label: "Child".into(),
                depth: 1,
                icon: None,
            },
            TreeNode {
                key: "leaf".into(),
                label: "Leaf".into(),
                depth: 0,
                icon: None,
            },
        ];
        let mut collapsed = BTreeSet::from(["parent".into()]);
        let tree = view(&nodes, &collapsed);

        assert_eq!(
            tree.action(&click("parent", 2)),
            Some(TreeAction::Expand("parent".into()))
        );
        assert_eq!(
            tree.action(&click("leaf", 2)),
            Some(TreeAction::Select("leaf".into()))
        );

        collapsed.clear();
        let tree = view(&nodes, &collapsed);
        assert_eq!(
            tree.action(&click("parent", 2)),
            Some(TreeAction::Collapse("parent".into()))
        );
    }

    #[test]
    fn tree_keyboard_actions_ignore_releases_and_leaf_directional_keys() {
        let nodes = [
            TreeNode {
                key: "root".into(),
                label: "Root".into(),
                depth: 0,
                icon: None,
            },
            TreeNode {
                key: "leaf".into(),
                label: "Leaf".into(),
                depth: 0,
                icon: None,
            },
        ];
        let collapsed = BTreeSet::new();
        let tree = view(&nodes, &collapsed);

        assert_eq!(
            tree.action(&key("leaf", Key::ArrowRight, KeyState::Released)),
            None
        );
        assert_eq!(
            tree.action(&key("leaf", Key::ArrowRight, KeyState::Pressed)),
            None
        );
        assert_eq!(
            tree.action(&key("leaf", Key::ArrowLeft, KeyState::Pressed)),
            None
        );
    }
}
