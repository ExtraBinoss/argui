use argui_core::Size;
use argui_layout::LayoutEngine;
use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_text::{TextEngine, TextStyle};
use argui_ui::{
    Element, Length, OverlayPlacement, PlacementSide, ScrollConfig, ScrollbarStyle, TextInput,
    TextInputStyle, UiTree,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn scrollable_overlay_translates_its_input_region_and_scrollbar() {
    let scrollbar = ScrollbarStyle::new(
        QuadStyle::solid(Color::rgb(0.1, 0.1, 0.1)),
        QuadStyle::solid(Color::WHITE),
    );
    let anchor = Element::container([])
        .keyed("overlay-anchor")
        .width(Length::Px(60.0))
        .height(Length::Px(24.0));
    let overlay = Element::column([
        TextInput::new(
            "overlay-input",
            "value",
            "placeholder",
            TextInputStyle::new(PaintStyle::default(), TextStyle::default()),
        )
        .build()
        .height(Length::Px(28.0))
        .shrink(0.0),
        Element::container([]).height(Length::Px(180.0)).shrink(0.0),
    ])
    .keyed("scrollable-overlay")
    .width(Length::Px(140.0))
    .height(Length::Px(80.0))
    .scrollable(ScrollConfig::default().scrollbar(scrollbar))
    .anchored_to(
        "overlay-anchor",
        OverlayPlacement::new(PlacementSide::Bottom).margin(0.0),
    );
    let missing_anchor = Element::container([])
        .width(Length::Px(20.0))
        .height(Length::Px(20.0))
        .anchored_to("absent-anchor", OverlayPlacement::new(PlacementSide::Right));
    let mut ui = UiTree::new(Element::column([anchor, overlay, missing_anchor]));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(320.0, 240.0))
        .unwrap();

    assert_eq!(output.text_inputs.len(), 1);
    assert_eq!(output.scroll_regions.len(), 1);
    assert!(output.scroll_regions[0].scrollbar.is_some());
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(output.text_inputs.len(), 1);
    assert_eq!(output.scroll_regions.len(), 1);
    assert!(output.scroll_regions[0].scrollbar.is_some());
}
