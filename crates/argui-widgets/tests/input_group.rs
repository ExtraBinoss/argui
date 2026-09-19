use argui_core::{Color, ColorScheme};
use argui_paint::{BorderWidths, CornerRadii};
use argui_ui::{Element, UiTree, sides};
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

#[test]
fn the_outer_surface_owns_the_border_without_increasing_control_height() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Dark);
    let group = InputGroup::new(
        "message",
        "Message",
        Input::new("body", "", "Send a message", theme.input()).build(),
    )
    .build(theme);

    assert_eq!(group.style.padding, sides(4.0, 0.0));
    assert_eq!(
        group.children[0].paint.quad.border.as_ref().unwrap().widths,
        BorderWidths::all(0.0)
    );
    assert!(group.children[0].paint.quad.background.is_none());
    assert_eq!(group.children[0].paint.quad.radii, CornerRadii::all(0.0));
    assert_eq!(group.children[0].style.flex_shrink, 1.0);
}
