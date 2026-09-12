use argui_core::{Color, ColorScheme};
use argui_ui::{Orientation, Role, UiTree};
use argui_widgets::{Button, ButtonGroup, shadcn};

#[test]
fn groups_preserve_independent_button_targets_and_names_in_both_axes() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Light);
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        let mut group = ButtonGroup::new(
            "actions",
            "Actions",
            [
                Button::new("save", "Save", theme.button()).build(),
                Button::new("cancel", "Cancel", theme.outline_button()).build(),
            ],
        );
        group.orientation = orientation;
        let tree = UiTree::new(group.build());
        let semantic = tree.semantic_tree(&[], 1.0);
        assert_eq!(
            semantic
                .nodes
                .iter()
                .filter(|node| node.semantics.role == Role::Button)
                .count(),
            2
        );
        assert!(
            semantic
                .nodes
                .iter()
                .any(|node| node.semantics.label.as_deref() == Some("Actions"))
        );
    }
}
