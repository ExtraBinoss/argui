use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, Role, UiTree};
use argui_widgets::{Button, Card, shadcn};

#[test]
fn card_keeps_sections_in_flow_and_actions_focusable_at_narrow_widths() {
    let palette = shadcn(Color::WHITE);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for width in [280.0, 520.0] {
            let card = Card::new("project", Element::text("Body content").keyed("body"))
                .title("A shared workspace with a long descriptive title")
                .description("Keep all the components of your project in one place.")
                .heading_level(2)
                .action(Button::new("edit", "Edit", theme.outline_button()).build())
                .footer(Button::new("save", "Save", theme.button()).build())
                .build(theme);
            let mut tree = UiTree::new(card);
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 600.0))
                .unwrap();
            let bounds = |key| {
                output
                    .nodes
                    .iter()
                    .find(|node| tree.key(node.node) == Some(key))
                    .unwrap()
                    .bounds
            };
            let body = bounds("body");
            let description = bounds("project::description");
            let footer = bounds("save");
            assert!(body.origin.y >= description.origin.y + description.size.height);
            assert!(footer.origin.y >= body.origin.y + body.size.height);
            assert!(bounds("edit").origin.x + bounds("edit").size.width <= width);
            assert!(tree.semantic_diagnostics().is_empty());
            let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
            let group = semantic
                .nodes
                .iter()
                .find(|node| node.semantics.role == Role::Group)
                .unwrap();
            assert_eq!(group.semantics.relations.labelled_by.len(), 1);
            assert_eq!(group.semantics.relations.described_by.len(), 1);
            assert!(semantic.nodes.iter().any(
                |node| node.semantics.role == Role::Heading && node.semantics.level == Some(2)
            ));
            assert_eq!(
                semantic
                    .nodes
                    .iter()
                    .filter(|node| node.semantics.focus_policy.is_tab_stop())
                    .count(),
                2
            );
        }
    }
}

#[test]
fn optional_sections_do_not_leave_empty_headers_or_dangling_references() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    let plain = Card::new("plain", Element::text("Body")).build(theme);
    assert_eq!(plain.children.len(), 1);
    for card in [
        plain,
        Card::new("description", Element::text("Body"))
            .description("Description only")
            .build(theme),
        Card::new("action", Element::text("Body"))
            .action(Element::text("Action only"))
            .build(theme),
        Card::new("low", Element::text("Body"))
            .title("Heading")
            .heading_level(0)
            .build(theme),
        Card::new("high", Element::text("Body"))
            .title("Heading")
            .heading_level(20)
            .build(theme),
    ] {
        let tree = UiTree::new(card);
        assert!(tree.semantic_diagnostics().is_empty());
        for node in tree.semantic_tree(&[], 1.0).nodes {
            if let Some(level) = node.semantics.level {
                assert!((1..=6).contains(&level));
            }
        }
    }
}
