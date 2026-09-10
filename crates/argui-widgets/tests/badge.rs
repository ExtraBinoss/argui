use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_paint::Fill;
use argui_text::TextEngine;
use argui_ui::{Element, Role, UiTree};
use argui_widgets::{Badge, BadgeVariant, shadcn};

#[test]
fn variants_keep_compact_labels_readable_and_announce_them_once() {
    let palette = shadcn(Color::srgb(0.2, 0.5, 0.9));
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for variant in [
            BadgeVariant::Primary,
            BadgeVariant::Secondary,
            BadgeVariant::Destructive,
            BadgeVariant::Outline,
            BadgeVariant::Ghost,
        ] {
            let badge = Badge::new("status", "Waiting for review")
                .variant(variant)
                .leading(Element::text("check"))
                .trailing(Element::text("arrow"))
                .build(theme);
            assert!(badge.interaction.is_none());
            let background = badge.paint.quad.background.clone();
            match variant {
                BadgeVariant::Primary => assert_eq!(background, Some(Fill::Solid(theme.primary))),
                BadgeVariant::Secondary => {
                    assert_eq!(background, Some(Fill::Solid(theme.secondary)))
                }
                BadgeVariant::Destructive => {
                    assert_eq!(background, Some(Fill::Solid(theme.destructive)))
                }
                _ => assert_eq!(background, Some(Fill::Solid(Color::TRANSPARENT))),
            }
            let mut tree = UiTree::new(Element::column([badge]));
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(360.0, 200.0))
                .unwrap();
            let bounds = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("status"))
                .unwrap()
                .bounds;
            assert!(bounds.size.height >= 22.0 && bounds.size.height < 40.0);
            assert!(
                bounds.size.width < 360.0,
                "badges must not stretch across the row"
            );
            let semantics = tree.semantic_tree(&output.semantic_bounds, 1.0);
            assert_eq!(
                semantics
                    .nodes
                    .iter()
                    .filter(|node| node.semantics.role == Role::Text)
                    .count(),
                1
            );
            assert!(
                semantics
                    .nodes
                    .iter()
                    .any(|node| node.semantics.label.as_deref() == Some("Waiting for review"))
            );
        }
        let simple = Badge::new("simple", "New").build(theme);
        assert_eq!(simple.children.len(), 1);
    }
}
