use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_text::{TextEngine, TextStyle};
use argui_ui::{Edges, Element, Length, UiTree};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn taffy_layout_uses_real_text_measurement_and_viewport_constraints() {
    let root = Element::column([Element::text(
        "A responsive line that wraps when its viewport gets narrow.",
    )
    .text_style(TextStyle {
        font_size: 20.0,
        line_height: 25.0,
        ..TextStyle::default()
    })])
    .width(Length::Percent(1.0))
    .height(Length::Percent(1.0))
    .padding(Edges::all(20.0));
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();

    let wide = layout
        .compute(&mut ui, &mut text, Size::new(600.0, 300.0))
        .unwrap();
    let narrow = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 300.0))
        .unwrap();

    assert_eq!(wide.nodes[0].bounds.size, Size::new(600.0, 300.0));
    assert_eq!(narrow.nodes[0].bounds.size, Size::new(260.0, 300.0));
    assert!(narrow.text.blocks()[0].bounds.size.width < wide.text.blocks()[0].bounds.size.width);
    assert!(narrow.text.blocks()[0].bounds.size.height > wide.text.blocks()[0].bounds.size.height);
    assert!(!ui.layout_dirty());
}
