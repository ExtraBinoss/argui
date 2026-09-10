use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, Orientation, Role, UiTree, length};
use argui_widgets::{Separator, shadcn};

#[test]
fn dividers_keep_one_pixel_thickness_and_expose_only_semantic_separators() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for orientation in [Orientation::Horizontal, Orientation::Vertical] {
        for decorative in [true, false] {
            let separator = Separator::new("divider")
                .orientation(orientation)
                .decorative(decorative)
                .build(theme);
            let mut tree = UiTree::new(
                Element::row([separator])
                    .width(length(200.0))
                    .height(length(40.0)),
            );
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(200.0, 40.0))
                .unwrap();
            let bounds = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("divider"))
                .unwrap()
                .bounds;
            assert_eq!(
                bounds.size,
                match orientation {
                    Orientation::Horizontal => Size::new(200.0, 1.0),
                    Orientation::Vertical => Size::new(1.0, 40.0),
                }
            );
            let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
            let divider = semantic
                .nodes
                .iter()
                .find(|node| node.semantics.role == Role::Separator);
            assert_eq!(divider.is_none(), decorative);
            if let Some(divider) = divider {
                assert_eq!(divider.semantics.orientation, Some(orientation));
                assert!(!divider.semantics.focus_policy.is_focusable());
            }
        }
    }
    assert!(Separator::new("default").build(theme).semantic_hidden);
}

#[test]
fn labels_stay_centered_between_equal_rules_and_are_announced_once() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for width in [160.0, 480.0] {
        for decorative in [true, false] {
            let separator = Separator::new("divider")
                .label("Or continue with")
                .decorative(decorative)
                .build(theme);
            let mut tree = UiTree::new(separator);
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 100.0))
                .unwrap();
            let label = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("divider::label"))
                .unwrap()
                .bounds;
            assert!((label.origin.x + label.size.width / 2.0 - width / 2.0).abs() <= 1.0);
            let rules: Vec<_> = output
                .nodes
                .iter()
                .filter(|node| node.bounds.size.height == 1.0)
                .collect();
            assert_eq!(rules.len(), 2);
            assert!((rules[0].bounds.size.width - rules[1].bounds.size.width).abs() <= 1.0);
            assert!(rules[0].bounds.origin.x + rules[0].bounds.size.width <= label.origin.x);
            assert!(rules[1].bounds.origin.x >= label.origin.x + label.size.width);
            let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
            assert_eq!(
                semantic
                    .nodes
                    .iter()
                    .filter(|node| node.semantics.label.as_deref() == Some("Or continue with"))
                    .count(),
                1
            );
        }
    }
    let vertical = Separator::new("vertical")
        .orientation(Orientation::Vertical)
        .label("Section")
        .build(theme);
    let mut tree = UiTree::new(Element::row([vertical]).height(length(200.0)));
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut TextEngine::new(), Size::new(100.0, 200.0))
        .unwrap();
    let label = output
        .nodes
        .iter()
        .find(|node| tree.key(node.node) == Some("vertical::label"))
        .unwrap()
        .bounds;
    assert!((label.origin.y + label.size.height / 2.0 - 100.0).abs() <= 1.0);
    assert!(
        Separator::new("empty")
            .label("  ")
            .build(theme)
            .children
            .is_empty()
    );
}
