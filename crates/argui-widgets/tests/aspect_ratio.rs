use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree, length};
use argui_widgets::AspectRatio;

#[test]
fn resize_preserves_ratios_and_positions_following_content_after_the_frame() {
    for ratio in [1.0, 4.0 / 3.0, 16.0 / 9.0] {
        let mut tree = UiTree::new(Element::column([
            AspectRatio::new("frame", ratio, Element::text("Content").keyed("inside")).build(),
            Element::text("After").keyed("after").height(length(24.0)),
        ]));
        let mut layout = LayoutEngine::new();
        for width in [180.0, 520.0, 320.0] {
            let output = layout
                .compute(&mut tree, &mut TextEngine::new(), Size::new(width, 900.0))
                .unwrap();
            let bounds = |key| {
                output
                    .nodes
                    .iter()
                    .find(|node| tree.key(node.node) == Some(key))
                    .unwrap()
                    .bounds
            };
            assert!((bounds("frame").size.height - width / ratio).abs() <= 1.0);
            assert_eq!(bounds("inside"), bounds("frame"));
            assert!(bounds("after").origin.y >= bounds("frame").size.height);
        }
    }
}

#[test]
fn invalid_ratios_are_rejected_before_layout() {
    for ratio in [0.0, -1.0, f32::NAN, f32::INFINITY] {
        assert!(
            std::panic::catch_unwind(|| AspectRatio::new("frame", ratio, Element::text("Content")))
                .is_err()
        );
    }
}
