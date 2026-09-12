use argui_core::{Color, ColorScheme};
use argui_ui::{Element, UiTree};
use argui_widgets::{Button, Input, InputGroup, shadcn};

#[test]
fn addons_do_not_replace_the_editor_or_the_trailing_action() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Dark);
    for invalid in [false, true] {
        let mut group = InputGroup::new(
            "search",
            "Search",
            Input::new("query", "Rust", "", theme.input()).build(),
        );
        group.leading = Some(Element::text("Find"));
        group.trailing = Some(Button::new("go", "Go", theme.button()).build());
        group.invalid = invalid;
        let tree = UiTree::new(group.build(theme));
        assert!(tree.semantic_diagnostics().is_empty());
        let semantics = tree.semantic_tree(&[], 1.0);
        assert_eq!(
            semantics
                .nodes
                .iter()
                .filter(|node| node.semantics.focus_policy.is_tab_stop())
                .count(),
            2
        );
    }
}
