use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, LiveRegion, Role, UiTree};
use argui_widgets::{Alert, AlertVariant, shadcn};

#[test]
fn alert_relates_its_message_and_excludes_decorative_icons() {
    let palette = shadcn(Color::WHITE);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for variant in [AlertVariant::Default, AlertVariant::Destructive] {
            let alert = Alert::new("notice", "Unable to sync")
                .description(
                    "Your changes are saved on this device. Check your connection and try again.",
                )
                .icon(Element::text("warning").keyed("icon"))
                .variant(variant)
                .build(theme);
            let mut tree = UiTree::new(alert);
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(280.0, 400.0))
                .unwrap();
            assert!(tree.semantic_diagnostics().is_empty());
            for node in &output.nodes {
                if tree.key(node.node) == Some("notice::title") {
                    let element = tree.element_at(node.index).unwrap();
                    let argui_ui::ElementKind::Text { style, .. } = &element.kind else {
                        panic!("alert title")
                    };
                    assert!(style.color.contrast_ratio(theme.card) >= 4.5);
                }
            }
            let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
            let alert = semantic
                .nodes
                .iter()
                .find(|node| node.semantics.role == Role::Alert)
                .unwrap();
            assert_eq!(alert.semantics.live, LiveRegion::Assertive);
            assert_eq!(alert.semantics.relations.labelled_by.len(), 1);
            assert_eq!(alert.semantics.relations.described_by.len(), 1);
            assert!(
                semantic
                    .nodes
                    .iter()
                    .all(|node| !node.semantics.focus_policy.is_focusable())
            );
            assert!(
                !semantic
                    .nodes
                    .iter()
                    .any(|node| node.semantics.label.as_deref() == Some("warning"))
            );
            let description = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("notice::description"))
                .unwrap()
                .bounds;
            assert!(
                description.size.height > 21.0,
                "descriptions wrap instead of clipping"
            );
            assert!(description.origin.x + description.size.width <= 280.0);
        }
    }
}

#[test]
fn persistent_and_polite_alerts_allow_explicit_announcement_policy() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    for live in [LiveRegion::Off, LiveRegion::Polite] {
        let alert = Alert::new("notice", "Saved").live(live).build(theme);
        assert_eq!(alert.semantics.as_ref().unwrap().live, live);
        assert!(alert.semantic_bindings.described_by.is_empty());
        assert!(UiTree::new(alert).semantic_diagnostics().is_empty());
    }
}
