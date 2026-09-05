use argui_core::{
    CaretAffinity, Color, Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase,
    Rect, ScrollDelta, Size, TextPosition,
};
use argui_layout::{LayoutEngine, TextInputRegion};
use argui_paint::{PaintStyle, QuadStyle};
use argui_text::{CaretStop, TextEngine, TextOverflow, TextStyle};
use argui_ui::{
    CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType, GestureCapture,
    GestureKind, GestureSet, Interaction, PanGesture, Position, ScrollbarPartStyle, ScrollbarStyle,
    Sides, UiEventKind, UiTree, auto, length, percent, scrollbar_at,
};
use argui_widgets::{Input, InputStyle, TextArea};

const NOTO_SANS: &[u8] = include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn placeholder_keeps_its_authored_color_on_the_first_frame() {
    let text = TextStyle {
        color: Color::BLACK,
        ..TextStyle::default()
    };
    let mut style = InputStyle::new(PaintStyle::default(), text);
    style.placeholder.color = Color::WHITE;
    let mut ui = UiTree::new(Input::new("search", "", "Search components", style).build());
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(320.0, 80.0))
        .unwrap();

    assert_eq!(output.text.blocks()[0].style.color, Color::WHITE);
}

fn stop(index: usize, x: f32, y: f32, word_boundary: bool) -> CaretStop {
    CaretStop {
        position: TextPosition::new(index, CaretAffinity::Before),
        point: Point::new(x, y),
        word_boundary,
    }
}

fn region(stops: Vec<CaretStop>) -> TextInputRegion {
    let tree = UiTree::new(Element::container([]));
    TextInputRegion {
        node: tree.node_ids()[0],
        bounds: Rect::new(Point::default(), Size::new(100.0, 60.0)),
        viewport: Rect::new(Point::default(), Size::new(100.0, 60.0)),
        clip: Rect::new(Point::default(), Size::new(80.0, 60.0)),
        stops,
        selection: Vec::new(),
        caret: Some(Rect::new(Point::new(10.0, 0.0), Size::new(1.0, 16.0))),
        selection_color: Color::TRANSPARENT,
        caret_style: argui_ui::CaretStyle::default(),
        content_size: Size::new(100.0, 60.0),
        scroll_x: 0.0,
        scroll_y: 0.0,
    }
}

#[test]
fn hit_testing_and_empty_regions_have_exact_boundaries() {
    let populated = region(vec![stop(0, 0.0, 0.0, true), stop(1, 20.0, 0.0, true)]);
    assert_eq!(
        populated.hit_position(Point::new(18.0, 2.0)),
        Some(TextPosition::new(1, CaretAffinity::Before))
    );
    assert_eq!(populated.hit_index(Point::new(18.0, 2.0)), Some(1));
    assert_eq!(populated.hit_position(Point::new(90.0, 2.0)), None);
    assert_eq!(populated.hit_position(Point::new(110.0, 2.0)), None);

    let empty = region(Vec::new());
    assert_eq!(
        empty.closest_position(Point::new(4.0, 4.0)),
        TextPosition::default()
    );
    assert_eq!(empty.closest_index(Point::new(4.0, 4.0)), 0);
}

#[test]
fn visual_neighbors_follow_lines_directions_and_word_boundaries() {
    let region = region(vec![
        stop(0, 0.0, 0.0, true),
        stop(1, 10.0, 0.0, false),
        stop(2, 20.0, 0.0, true),
        stop(3, 0.0, 20.0, true),
        stop(4, 11.0, 20.0, false),
        stop(5, 22.0, 20.0, true),
    ]);
    let middle = TextPosition::new(1, CaretAffinity::Before);
    assert_eq!(
        region.visual_neighbor(middle, true, false),
        TextPosition::new(0, CaretAffinity::Before)
    );
    assert_eq!(
        region.visual_neighbor(middle, false, true),
        TextPosition::new(2, CaretAffinity::Before)
    );
    assert_eq!(
        region.vertical_neighbor(middle, false),
        TextPosition::new(4, CaretAffinity::Before)
    );
    assert_eq!(
        region.vertical_neighbor(TextPosition::new(4, CaretAffinity::Before), true),
        middle
    );
    assert_eq!(region.vertical_neighbor(middle, true), middle);
    assert_eq!(
        region.visual_neighbor(TextPosition::new(2, CaretAffinity::Before), false, false),
        TextPosition::new(2, CaretAffinity::Before)
    );

    let affinity_fallback = TextPosition::new(1, CaretAffinity::After);
    assert_eq!(
        region.visual_neighbor(affinity_fallback, false, false),
        TextPosition::new(2, CaretAffinity::Before)
    );
    let missing = TextPosition::new(99, CaretAffinity::Before);
    assert_eq!(region.visual_neighbor(missing, false, false), missing);
    assert_eq!(region.vertical_neighbor(missing, false), missing);

    assert_eq!(
        region.closest_position(Point::new(19.0, 19.0)),
        TextPosition::new(2, CaretAffinity::Before)
    );
    assert_eq!(
        region.closest_position(Point::new(9.0, 20.0)),
        TextPosition::new(4, CaretAffinity::Before)
    );
}

#[test]
fn text_area_shapes_and_clips_scrollable_content_with_a_live_scrollbar() {
    let value = (0..30)
        .map(|line| format!("line {line:02} keeps enough text to exercise wrapping"))
        .collect::<Vec<_>>()
        .join("\n");
    let area = TextArea::new(
        "notes",
        &value,
        "notes",
        InputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .scrollbar(
        ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::default()),
            ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
        )
        .insets(Sides {
            left: 3.0,
            right: 5.0,
            top: 7.0,
            bottom: 21.0,
        }),
    )
    .build()
    .height(length(96.0));
    let mut ui = UiTree::new(area);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
        .unwrap();
    let region = &output.text_inputs[0];
    let scroll = &output.scroll_regions[0];

    assert!(region.content_size.height > region.viewport.size.height);
    assert!(output.text.blocks()[0].bounds.size.height >= region.content_size.height);
    assert_eq!(output.text.blocks()[0].clip, region.clip);
    assert!(scroll.max_offset.y > 0.0);
    let scrollbar = scroll.scrollbar.as_ref().unwrap();
    let scrollbar = scrollbar.vertical.as_ref().unwrap();
    assert_eq!(scrollbar.track.origin.y, scroll.bounds.origin.y + 7.0);
    assert_eq!(
        scrollbar.track.origin.x + scrollbar.track.size.width,
        scroll.bounds.origin.x + scroll.bounds.size.width - 5.0
    );
    assert_eq!(
        scrollbar.track.origin.y + scrollbar.track.size.height,
        scroll.bounds.origin.y + scroll.bounds.size.height - 21.0
    );
    let viewport_size = region.viewport.size;
    ui.pointer_moved(Point::new(20.0, 20.0), &output.hit_regions);
    ui.primary_pressed(&output.hit_regions);
    layout.update_text_inputs(&mut ui, &mut text, &mut output);
    assert_eq!(output.text_inputs[0].viewport.size, viewport_size);
    assert!(output.scroll_regions[0].max_offset.y > 0.0);
    assert!(output.scroll_regions[0].scrollbar.is_some());
    let region = &output.text_inputs[0];
    let prepared = text.prepare(&output.text, 1.0);
    let first_y = prepared.glyphs.iter().map(|glyph| glyph.y).min().unwrap();
    let last_y = prepared.glyphs.iter().map(|glyph| glyph.y).max().unwrap();
    assert!(last_y - first_y > region.viewport.size.height as i32);
    assert!(prepared.glyphs.iter().all(|glyph| {
        glyph.clip
            == [
                region.clip.origin.x,
                region.clip.origin.y,
                region.clip.origin.x + region.clip.size.width,
                region.clip.origin.y + region.clip.size.height,
            ]
    }));

    let before = output.text.blocks()[0].bounds.origin.y;
    let update = ui.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Pixels(Point::new(0.0, 40.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert!(output.text.blocks()[0].bounds.origin.y > before);
    assert!(output.scroll_regions[0].scrollbar.is_some());
}

#[test]
fn a_single_line_input_clips_its_text_after_a_retained_resize() {
    let input = Input::new(
        "search",
        "",
        "Search components with a deliberately long placeholder",
        InputStyle::new(PaintStyle::default(), TextStyle::default()),
    )
    .build()
    .height(length(40.0));
    let mut ui = UiTree::new(Element::column([input]).width(percent(0.5)));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();

    let wide = layout
        .compute(&mut ui, &mut text, Size::new(420.0, 80.0))
        .unwrap();
    let narrow = layout
        .compute(&mut ui, &mut text, Size::new(220.0, 80.0))
        .unwrap();
    let wide_region = &wide.text_inputs[0];
    let narrow_region = &narrow.text_inputs[0];
    let block = &narrow.text.blocks()[0];

    assert!(narrow_region.bounds.size.width < wide_region.bounds.size.width);
    assert_eq!(narrow_region.clip, narrow_region.bounds);
    assert_eq!(block.clip, narrow_region.clip);
    assert_eq!(block.bounds, narrow_region.viewport);
    assert_eq!(
        block.style.overflow,
        TextOverflow::Ellipsis(argui_text::EllipsisPosition::End)
    );
    assert!(
        narrow_region.clip.origin.x + narrow_region.clip.size.width
            < narrow.viewport.origin.x + narrow.viewport.size.width
    );
}

#[test]
fn resizing_a_text_area_preserves_its_scroll_position() {
    let value = (0..30)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let editor = |height| {
        TextArea::new(
            "notes",
            &value,
            "notes",
            InputStyle::new(PaintStyle::default(), TextStyle::default()),
        )
        .scrollbar(ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::default()),
            ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
        ))
        .build()
        .height(length(height))
    };
    let mut ui = UiTree::new(editor(96.0));
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    layout
        .compute(&mut ui, &mut text, Size::new(260.0, 160.0))
        .unwrap();
    ui.set_scroll_offset(node, Point::new(0.0, 40.0));

    ui.update(editor(120.0));
    let output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 160.0))
        .unwrap();

    assert!(output.scroll_regions[0].max_offset.y > 40.0);
    assert_eq!(ui.scroll_offset(node), Point::new(0.0, 40.0));
    assert_eq!(output.text_inputs[0].scroll_y, 40.0);
}

#[test]
fn a_resize_handle_painted_over_a_scrollbar_keeps_pointer_priority() {
    let value = (0..30)
        .map(|line| format!("line {line:02}"))
        .collect::<Vec<_>>()
        .join("\n");
    let mut style = InputStyle::new(PaintStyle::default(), TextStyle::default());
    style.layout.size.height = percent(1.0);
    let area = TextArea::new("notes", value, "notes", style)
        .scrollbar(ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::default()),
            ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
        ))
        .build();
    let handle = Element::container([])
        .keyed("resize")
        .absolute(Sides {
            left: auto(),
            right: length(0.0),
            top: auto(),
            bottom: length(0.0),
        })
        .width(length(18.0))
        .height(length(18.0))
        .on(EventListener::new(
            EventType::Gesture,
            EventHandlerId::new(EventOwnerId(1), 0),
        ))
        .interaction(
            Interaction::default()
                .cursor(CursorIcon::NwseResize)
                .gestures(
                    GestureSet::EMPTY.pan(
                        PanGesture::default()
                            .immediate()
                            .capture(GestureCapture::OnPress),
                    ),
                ),
        );
    let mut ui = UiTree::new(
        Element::container([area, handle])
            .width(length(260.0))
            .height(length(96.0))
            .position(Position::Relative),
    );
    let handle = ui.node_id_at(2).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
        .unwrap();
    let bounds = output
        .nodes
        .iter()
        .find(|node| node.node == handle)
        .unwrap()
        .bounds;
    let point = Point::new(
        bounds.origin.x + bounds.size.width * 0.5,
        bounds.origin.y + bounds.size.height * 0.5,
    );

    assert!(output.scroll_regions[0].scrollbar_contains(point));
    assert!(scrollbar_at(point, &output.scroll_regions, &output.hit_regions).is_none());
    assert_eq!(
        output
            .hit_regions
            .iter()
            .rev()
            .find(|region| region.contains(point))
            .unwrap()
            .node,
        handle
    );

    ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, point),
        &output.hit_regions,
    );
    ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, point),
        &output.hit_regions,
    );
    let dragged = ui.pointer_event(
        PointerEvent::mouse(
            PointerPhase::Moved,
            Point::new(point.x + 36.0, point.y + 24.0),
        ),
        &output.hit_regions,
    );
    assert!(dragged.events.iter().any(|event| {
        event.target_key() == Some("resize")
            && matches!(
                event.kind,
                UiEventKind::Gesture(argui_ui::GestureEvent {
                    kind: GestureKind::Pan { total, .. },
                    ..
                }) if total == Point::new(36.0, 24.0)
            )
    }));
}

#[test]
fn repeated_newlines_keep_the_multiline_caret_visible_and_scroll_monotonic() {
    let editor = |value: &str| {
        TextArea::new(
            "notes",
            value,
            "notes",
            InputStyle::new(PaintStyle::default(), TextStyle::default()),
        )
        .build()
        .height(length(96.0))
    };
    let mut ui = UiTree::new(editor("start"));
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
        .unwrap();
    ui.pointer_moved(Point::new(20.0, 20.0), &output.hit_regions);
    ui.primary_pressed(&output.hit_regions);
    ui.place_text_cursor(node, "start".len(), false);
    let mut previous_scroll = 0.0;

    for repeat in 0..12 {
        ui.edit_text_input(&KeyInput {
            key: Key::Enter,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: repeat > 0,
            text: None,
        });
        let value = ui.text_input_value(node).unwrap().to_owned();
        ui.update(editor(&value));
        output = layout
            .compute(&mut ui, &mut text, Size::new(260.0, 96.0))
            .unwrap();
        let region = &output.text_inputs[0];
        let scroll = ui.scroll_offset(node).y;
        assert!(scroll >= previous_scroll);
        assert!(region.caret.unwrap().origin.y >= region.viewport.origin.y);
        assert!(
            region.caret.unwrap().origin.y + region.caret.unwrap().size.height
                <= region.viewport.origin.y + region.viewport.size.height
        );
        previous_scroll = scroll;
    }
    assert!(previous_scroll > 0.0);
}

#[test]
fn empty_or_degenerate_custom_carets_do_not_emit_invalid_quads() {
    use argui_ui::{CaretHeight, CaretPrimitive, CaretStyle, CaretVisual, FocusRequest};
    let color = Color::srgb(0.3, 0.8, 0.4);
    for (width, height, empty, expected) in [
        (2.0, 10.0, true, 0),
        (0.0, 10.0, false, 0),
        (2.0, 0.0, false, 0),
        (-2.0, 10.0, false, 0),
        (2.0, -10.0, false, 0),
        (2.0, 10.0, false, 1),
    ] {
        let mut style = InputStyle::new(PaintStyle::default(), TextStyle::default());
        style.caret = CaretStyle::new(CaretVisual::new(if empty {
            vec![]
        } else {
            vec![CaretPrimitive::new(
                width,
                CaretHeight::Pixels(height),
                QuadStyle::solid(color),
            )]
        }));
        let mut ui = UiTree::new(Input::new("custom-caret", "hello", "", style).build());
        let mut engine = LayoutEngine::new();
        let mut text = text_engine();
        let mut output = engine
            .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
            .unwrap();
        ui.sync_focus(
            &output.hit_regions,
            Some(FocusRequest::Focus("custom-caret".into())),
        );
        engine.update_text_inputs(&mut ui, &mut text, &mut output);
        assert!(output.text_inputs[0].caret.is_some());
        let painted = output
            .display_list
            .commands()
            .iter()
            .filter(|command| {
                matches!(command, argui_paint::DisplayCommand::Quad(quad)
                if quad.background == Some(argui_paint::Fill::Solid(color)))
            })
            .count();
        assert_eq!(painted, expected, "caret {width} x {height}, empty={empty}");
    }
}
