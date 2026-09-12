use argui_core::{Color, ColorScheme};
use argui_ui::{Element, UiTree};
use argui_widgets::{Bubble, BubbleVariant, Button, shadcn};

#[test]
fn bubble_variants_preserve_content_and_reaction_actions() {
    let palette = shadcn(Color::BLACK);
    let theme = palette.resolve(ColorScheme::Dark);
    for variant in [
        BubbleVariant::Primary,
        BubbleVariant::Secondary,
        BubbleVariant::Muted,
        BubbleVariant::Tinted,
        BubbleVariant::Outline,
        BubbleVariant::Ghost,
        BubbleVariant::Destructive,
    ] {
        for end in [false, true] {
            let mut bubble = Bubble::new("message", Element::text("Hello"));
            bubble.variant = variant;
            bubble.end = end;
            bubble.reactions = Some(Button::new("like", "Like", theme.ghost_button()).build());
            let tree = UiTree::new(bubble.build(theme));
            assert!(tree.semantic_diagnostics().is_empty());
            assert!(
                tree.node_ids()
                    .iter()
                    .any(|node| tree.key(*node) == Some("like"))
            );
        }
    }
}
