use argui_core::{Point, Size};
use argui_layout::LayoutEngine;
use argui_paint::{DisplayCommand, PaintStyle, QuadStyle};
use argui_text::{TextEngine, TextStyle};
use argui_ui::{Button, ButtonStyle, Color, Edges, Element, Interaction, Length, UiTree, Wrap};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn interaction_repaint_reuses_layout_and_shaped_text() {
    let root = Element::text("Fast repaint")
        .keyed("button")
        .padding(Edges::all(12.0))
        .background(Color::rgb(0.0, 0.0, 0.0))
        .interaction(Interaction::default().hovered(QuadStyle::solid(Color::WHITE)));
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    let text_before = output.text.clone();
    let base = output.display_list.commands()[0];

    let update = ui.pointer_moved(Point::new(10.0, 10.0), &output.hit_regions);
    assert!(update.paint_changed);
    layout.repaint(&ui, &mut output);

    assert_eq!(output.text, text_before);
    assert!(output.text.blocks()[0].clip.size.width > output.text.blocks()[0].bounds.size.width);
    assert_ne!(output.display_list.commands()[0], base);
    assert!(matches!(
        output.display_list.commands()[0],
        DisplayCommand::Quad(quad) if quad.background == Color::WHITE
    ));
}

#[test]
fn wrapped_rows_move_whole_items_instead_of_clipping_them() {
    let item = || {
        Element::container([])
            .width(Length::Px(80.0))
            .height(Length::Px(30.0))
    };
    let mut ui = UiTree::new(
        Element::row([item(), item(), item()])
            .width(Length::Percent(1.0))
            .wrap(Wrap::Wrap)
            .gap(8.0),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();

    let output = layout
        .compute(&mut ui, &mut text, Size::new(180.0, 120.0))
        .unwrap();

    assert_eq!(
        output.nodes[1].bounds.origin.y,
        output.nodes[2].bounds.origin.y
    );
    assert!(output.nodes[3].bounds.origin.y > output.nodes[1].bounds.origin.y);
    assert_eq!(output.nodes[3].bounds.size.width, 80.0);
}

#[test]
fn wrapped_button_labels_keep_every_glyph() {
    let button = |key, label| {
        Button::new(
            key,
            label,
            ButtonStyle::new(
                PaintStyle::new(QuadStyle::solid(Color::rgb(0.1, 0.2, 0.3))),
                TextStyle {
                    font_size: 17.0,
                    line_height: 22.0,
                    ..TextStyle::default()
                },
            ),
        )
        .build()
    };
    let mut ui = UiTree::new(
        Element::row([
            button("primary", "Primary action"),
            button("confirm", "Confirm"),
            button("delete", "Delete"),
        ])
        .width(Length::Percent(1.0))
        .wrap(Wrap::Wrap)
        .gap(12.0),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 180.0))
        .unwrap();

    for block in output.text.blocks() {
        let intrinsic = text.measure(&block.text, &block.style, None);
        assert!(
            block.bounds.size.width + 1.0 >= intrinsic.width,
            "{}: layout {} < intrinsic {}",
            block.text,
            block.bounds.size.width,
            intrinsic.width
        );
        assert!(block.clip.size.width > block.bounds.size.width);
        assert_eq!(block.bounds.size.height, block.style.line_height);
    }
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
    .padding(Edges::all(20.0))
    .background(Color::rgb(0.1, 0.2, 0.3));
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
    assert_eq!(wide.display_list.quad_count(), 1);
    assert_eq!(wide.display_list.commands().len(), 2);
}
