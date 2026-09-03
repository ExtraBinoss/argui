use argui_core::{Affine2D, CaretAffinity, Point, Rect, Size, TextPosition};
use argui_layout::LayoutEngine;
use argui_paint::{ClipChain, ClipRegion, DisplayCommand, Fill};
use argui_text::TextEngine;
use argui_ui::{
    DocumentTextPoint, Element, SelectionGranularity, TextSelectionStyle, UiTree, UserSelect,
    percent,
};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn document_selection_uses_glyph_regions_and_paints_without_relayout() {
    let color = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let mut ui = UiTree::new(
        Element::column([
            Element::text("first line"),
            Element::text("not selectable").user_select(UserSelect::None),
            Element::text("last line"),
        ])
        .width(percent(1.0))
        .selection_style(TextSelectionStyle {
            background: color,
            handle: color,
        }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 200.0))
        .unwrap();
    assert_eq!(output.text_regions.len(), 2);

    let first = output.text_regions[0].node;
    let last = output.text_regions[1].node;
    ui.begin_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(2, CaretAffinity::After)),
        false,
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        last,
        TextPosition::new(4, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);

    assert_eq!(
        ui.selected_document_text().as_deref(),
        Some("rst line\nlast")
    );
    assert!(output.document_selection_bounds(&ui).is_some());
    assert!(output.display_list.commands().iter().any(|command| {
        matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(color)))
    }));
}

#[test]
fn text_cursor_only_appears_over_shaped_line_geometry() {
    let mut ui = UiTree::new(Element::text("short").width(percent(1.0)));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
    let region = &output.text_regions[0];
    assert!(region.hit_position(Point::new(5.0, 5.0)).is_some());
    assert!(region.hit_position(Point::new(350.0, 5.0)).is_none());

    let center = region.layout.lines[0].bounds.origin;
    assert_eq!(region.distance_squared(center), 0.0);
    assert!(region.distance_squared(Point::new(-20.0, center.y)) > 0.0);
    assert!(region.distance_squared(Point::new(500.0, center.y)) > 0.0);
    assert!(region.distance_squared(Point::new(center.x, 80.0)) > 0.0);

    let mut transformed = region.clone();
    transformed.transform = Affine2D::translation(40.0, 20.0);
    let local_hit = region.hit_position(Point::new(5.0, 5.0)).unwrap();
    assert_eq!(
        transformed.hit_position(Point::new(45.0, 25.0)),
        Some(local_hit)
    );
    transformed.clips = ClipChain::from_regions([ClipRegion::new(
        Rect::new(Point::new(100.0, 100.0), Size::new(20.0, 20.0)),
        Affine2D::IDENTITY,
    )]);
    assert!(transformed.hit_position(Point::new(45.0, 25.0)).is_none());

    transformed.transform = Affine2D {
        matrix: [0.0; 4],
        translation: Point::default(),
    };
    let closest = transformed.closest_position(Point::new(5.0, 5.0));
    assert!(closest.position.index <= 5);
}

#[test]
fn touch_selection_paints_both_handles_in_forward_and_reverse_order() {
    let background = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let handle = argui_core::Color::srgb(0.9, 0.2, 0.4);
    let mut ui = UiTree::new(
        Element::text("select these words")
            .width(percent(1.0))
            .selection_style(TextSelectionStyle { background, handle }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 80.0))
        .unwrap();
    assert!(output.document_selection_bounds(&ui).is_none());
    let node = output.text_regions[0].node;

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(1, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        node,
        TextPosition::new(12, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(node, TextPosition::new(15, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        node,
        TextPosition::new(3, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);
    assert!(output.document_selection_bounds(&ui).is_some());
}

#[test]
fn touch_handles_attach_to_opposite_ends_across_text_nodes() {
    let background = argui_core::Color::srgba(0.2, 0.5, 0.9, 0.4);
    let handle = argui_core::Color::srgb(0.9, 0.2, 0.4);
    let mut ui = UiTree::new(
        Element::column([Element::text("first"), Element::text("second")])
            .width(percent(1.0))
            .selection_style(TextSelectionStyle { background, handle }),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    let first = output.text_regions[0].node;
    let second = output.text_regions[1].node;

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(first, TextPosition::new(1, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        second,
        TextPosition::new(4, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);

    ui.begin_touch_document_selection(
        DocumentTextPoint::new(second, TextPosition::new(5, CaretAffinity::After)),
        SelectionGranularity::Character,
    );
    ui.drag_document_selection(DocumentTextPoint::new(
        first,
        TextPosition::new(2, CaretAffinity::After),
    ));
    engine.repaint(&ui, &mut output);
    assert_eq!(solid_quad_count(&output, handle), 2);
}

fn solid_quad_count(output: &argui_layout::LayoutOutput, color: argui_core::Color) -> usize {
    output
        .display_list
        .commands()
        .iter()
        .filter(|command| {
            matches!(command, DisplayCommand::Quad(quad) if quad.background == Some(Fill::Solid(color)))
        })
        .count()
}
