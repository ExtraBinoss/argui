use argui_core::{Color, ColorScheme, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree};
use argui_widgets::{Button, Item, shadcn};

#[test]
fn item_wraps_text_keeps_actions_and_resolves_its_title() {
    let palette = shadcn(Color::BLACK);
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let theme = palette.resolve(scheme);
        for rich in [false, true] {
            let mut item = Item::new("file", "A long file name that should wrap within its row");
            item.bordered = rich;
            if rich {
                item.description = Some("Saved yesterday".into());
                item.media = Some(Element::text("icon"));
                item.actions = Some(Button::new("open", "Open", theme.button()).build());
            }
            let mut tree = UiTree::new(item.build(theme));
            let output = LayoutEngine::new()
                .compute(&mut tree, &mut TextEngine::new(), Size::new(300.0, 400.0))
                .unwrap();
            assert!(tree.semantic_diagnostics().is_empty());
            let title = output
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some("file::title"))
                .unwrap();
            assert!(title.bounds.origin.x + title.bounds.size.width <= 300.0);
            assert_eq!(
                tree.node_ids()
                    .iter()
                    .any(|node| tree.key(*node) == Some("open")),
                rich
            );
        }
    }
}
