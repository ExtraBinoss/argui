use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::UiTree;
use argui_widgets::{Kbd, shadcn};

#[test]
fn chords_have_one_spoken_name_and_separate_compact_noninteractive_keycaps() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Light);
    for (kbd, label) in [
        (Kbd::new("keys", ["Ctrl", "Shift", "K"]), "Ctrl + Shift + K"),
        (
            Kbd::new("keys", ["⌘", "K"]).label("Command plus K"),
            "Command plus K",
        ),
    ] {
        let mut tree = UiTree::new(kbd.build(theme));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(320.0, 100.0))
            .unwrap();
        assert_eq!(output.nodes[0].bounds.size.height, 24.0);
        let semantic = tree.semantic_tree(&output.semantic_bounds, 1.0);
        assert_eq!(semantic.nodes.len(), 1);
        assert_eq!(semantic.nodes[0].semantics.label.as_deref(), Some(label));
        assert!(output.hit_regions.is_empty());
    }
}

#[test]
fn keycap_size_keeps_accessibility_and_bounds_invalid_values() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(ColorScheme::Dark);
    for (requested, height) in [
        (16.0, 16.0),
        (0.0, 12.0),
        (100.0, 48.0),
        (f32::NAN, 24.0),
        (f32::INFINITY, 24.0),
    ] {
        let mut tree = UiTree::new(Kbd::new("small", ["W"]).size(requested).build(theme));
        let output = LayoutEngine::new()
            .compute(&mut tree, &mut TextEngine::new(), Size::new(320.0, 100.0))
            .unwrap();
        assert_eq!(output.nodes[0].bounds.size.height, height);
        assert!(output.hit_regions.is_empty());
        assert_eq!(
            tree.semantic_tree(&output.semantic_bounds, 1.0).nodes[0]
                .semantics
                .label
                .as_deref(),
            Some("W")
        );
    }
}
