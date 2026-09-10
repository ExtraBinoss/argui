use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, Role, UiTree};
use argui_widgets::{Button, Empty, shadcn};

#[test]
fn empty_state_relates_its_message_and_preserves_action_focus() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for width in [240.0, 580.0] {
        let root = Empty::new("empty", "No projects yet")
            .description("Start your first project by exploring the component library.")
            .media(Element::text("icon"))
            .content(Button::new("create", "Create project", theme.button()).build())
            .build(theme);
        let mut tree = UiTree::new(root);
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 500.0))
            .unwrap();
        assert!(tree.semantic_diagnostics().is_empty());
        let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
        let group = semantic
            .nodes
            .iter()
            .find(|node| node.semantics.role == Role::Group)
            .unwrap();
        assert_eq!(group.semantics.relations.labelled_by.len(), 1);
        assert_eq!(group.semantics.relations.described_by.len(), 1);
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.focus_policy.is_tab_stop())
                .count(),
            1
        );
        assert!(
            !semantic
                .nodes
                .iter()
                .any(|node| node.semantics.label.as_deref() == Some("icon"))
        );
        let button = output
            .nodes
            .iter()
            .find(|node| tree.key(node.node) == Some("create"))
            .unwrap()
            .bounds;
        assert!((button.origin.x + button.size.width / 2.0 - width / 2.0).abs() <= 1.0);
    }
    let bare = Empty::new("bare", "Nothing here")
        .bordered(false)
        .build(theme);
    assert_eq!(bare.children.len(), 1);
    assert!(UiTree::new(bare).semantic_diagnostics().is_empty());
}
