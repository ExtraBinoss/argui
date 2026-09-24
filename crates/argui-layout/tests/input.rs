use argui_core::{
    CaretAffinity, Color, Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase,
    Rect, ScrollDelta, Size, TextPosition,
};
use argui_layout::{LayoutEngine, TextInputRegion};
use argui_paint::QuadStyle;
use argui_text::{CaretStop, TextEngine, TextOverflow, TextStyle, TextWrap};
use argui_ui::{
    Axes, CaretStyle, CursorIcon, Element, EventHandlerId, EventListener, EventOwnerId, EventType,
    FocusPolicy, GestureCapture, GestureKind, GestureSet, Interaction, Overflow, PanGesture,
    Position, ScrollConfig, ScrollPropagation, ScrollbarGutter, ScrollbarPartStyle, ScrollbarStyle,
    Sides, TextEditorSpec, TextInputFilter, TextPrivacy, UiEventKind, UiTree, auto, length,
    percent, scrollbar_at,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_editor(
    key: &str,
    value: impl AsRef<str>,
    placeholder: &str,
    multiline: bool,
    filter: TextInputFilter,
    text: TextStyle,
    placeholder_text: TextStyle,
) -> Element {
    Element::text_editor(TextEditorSpec {
        value: value.as_ref().to_owned(),
        placeholder: placeholder.to_owned(),
        multiline,
        read_only: false,
        filter,
        text,
        placeholder_text,
        selection: Color::srgba(0.20, 0.68, 0.94, 0.38),
        caret: CaretStyle::default(),
    })
    .keyed(key)
    .width(percent(1.0))
    .interaction(
        Interaction::default()
            .focus_policy(FocusPolicy::TabStop)
            .cursor(CursorIcon::Text),
    )
}

fn single_line_editor(
    key: &str,
    value: impl AsRef<str>,
    placeholder: &str,
    mut text: TextStyle,
) -> Element {
    text.wrap = TextWrap::None;
    let mut placeholder_text = text.clone();
    placeholder_text.color = Color::srgba(0.55, 0.60, 0.68, 1.0);
    placeholder_text.overflow = TextOverflow::Ellipsis(argui_text::EllipsisPosition::End);
    text_editor(
        key,
        value,
        placeholder,
        false,
        TextInputFilter::Any,
        text,
        placeholder_text,
    )
    .padding(argui_ui::sides(13.0, 10.0))
    .align_items(argui_ui::AlignItems::CENTER)
    .shrink(0.0)
}

fn multiline_editor(
    key: &str,
    value: impl AsRef<str>,
    placeholder: &str,
    text: TextStyle,
) -> Element {
    let mut placeholder_text = text.clone();
    placeholder_text.color = Color::srgba(0.55, 0.60, 0.68, 1.0);
    placeholder_text.overflow = TextOverflow::Clip;
    let horizontal_overflow = if text.wrap == TextWrap::None {
        Overflow::Auto
    } else {
        Overflow::Hidden
    };
    text_editor(
        key,
        value,
        placeholder,
        true,
        TextInputFilter::Any,
        text,
        placeholder_text,
    )
    .padding(argui_ui::sides(13.0, 10.0))
    .align_items(argui_ui::AlignItems::START)
    .shrink(0.0)
    .overflow(Axes {
        x: horizontal_overflow,
        y: Overflow::Auto,
    })
    .scrollbar_gutter(ScrollbarGutter::Stable)
    .scroll_config(ScrollConfig::default().propagation(ScrollPropagation::Contain))
}

#[test]
fn password_layout_caret_stops_map_back_to_graphemes_without_shaping_the_secret() {
    let mut ui = UiTree::new(
        single_line_editor("password", "A👩‍🚀e\u{301}", "Password", TextStyle::default())
            .text_privacy(TextPrivacy::Password),
    );
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    let output = engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
    let node = ui.node_ids()[0];
    assert_eq!(ui.text_input_display(node).as_deref(), Some("•••"));
    let region = &output.text_inputs[0];
    assert!(
        region
            .stops
            .iter()
            .all(|stop| stop.position.index <= 9 && stop.position.index % 3 == 0)
    );
    let after_first = region
        .stops
        .iter()
        .find(|stop| stop.position.index == 3)
        .unwrap()
        .position;
    ui.move_text_position(node, after_first, false);
    ui.paste_text(Some(node), "x");
    assert_eq!(ui.text_input_value(node), Some("Ax👩‍🚀e\u{301}"));
    engine
        .compute(&mut ui, &mut text, Size::new(400.0, 100.0))
        .unwrap();
}

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn placeholder_keeps_its_authored_color_on_the_first_frame() {
    let text = TextStyle {
        color: Color::BLACK,
        wrap: TextWrap::None,
        ..TextStyle::default()
    };
    let placeholder = TextStyle {
        color: Color::WHITE,
        overflow: TextOverflow::Ellipsis(argui_text::EllipsisPosition::End),
        ..text.clone()
    };
    let input = text_editor(
        "search",
        "",
        "Search components",
        false,
        TextInputFilter::Any,
        text,
        placeholder,
    );
    let mut ui = UiTree::new(input);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(320.0, 80.0))
        .unwrap();

    assert_eq!(output.text.blocks()[0].style.color, Color::WHITE);
}

#[test]
fn retained_text_edits_refresh_painted_content_without_a_tree_rebuild() {
    let area = multiline_editor("code", "before", "Code", TextStyle::default());
    let mut ui = UiTree::new(area);
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(320.0, 100.0))
        .unwrap();

    ui.replace_text_input(node, "after");
    layout.update_text_inputs(&mut ui, &mut text, &mut output);

    assert_eq!(output.text.blocks()[0].content.as_str(), "after");
}

#[test]
fn non_wrapping_text_area_scrolls_both_axes_without_leaking_past_its_viewport() {
    let long_line =
        "let extremely_long_identifier = build_a_value_that_exceeds_the_editor_width();";
    let value = std::iter::repeat_n(long_line, 24)
        .collect::<Vec<_>>()
        .join("\n");
    let area = multiline_editor(
        "code",
        value,
        "",
        TextStyle {
            wrap: TextWrap::None,
            ..TextStyle::default()
        },
    )
    .width(length(240.0))
    .height(length(96.0));
    let mut ui = UiTree::new(area);
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(240.0, 96.0))
        .unwrap();

    assert!(output.scroll_regions[0].max_offset.x > 0.0);
    assert!(output.scroll_regions[0].max_offset.y > 0.0);
    assert_eq!(output.text.blocks()[0].clip, output.text_inputs[0].clip);
}

#[test]
fn large_non_wrapping_text_areas_shape_only_the_scrolled_viewport() {
    let value = (0..1_000)
        .map(|line| format!("line {line:04}: let value = compute();"))
        .collect::<Vec<_>>()
        .join("\n");
    let area = multiline_editor(
        "code",
        &value,
        "",
        TextStyle {
            line_height: 20.0,
            wrap: TextWrap::None,
            ..TextStyle::default()
        },
    )
    .width(length(320.0))
    .height(length(100.0));
    let mut ui = UiTree::new(area);
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(320.0, 100.0))
        .unwrap();
    assert!(output.text_inputs[0].content_size.height >= 20_000.0);
    assert!(output.text.blocks()[0].content.as_str().len() < value.len() / 10);

    ui.set_scroll_offset(node, Point::new(0.0, 10_000.0));
    layout.update_text_inputs(&mut ui, &mut text, &mut output);

    let visible = output.text.blocks()[0].content.as_str();
    assert!(visible.contains("line 0498") || visible.contains("line 0499"));
    assert!(!visible.contains("line 0000"));
    assert!(output.text_inputs[0].content_size.height >= 20_000.0);
}

#[test]
fn repeated_virtual_editor_scrolls_keep_the_viewport_and_reach_the_last_line() {
    let value = (0..1_000)
        .map(|line| format!("line {line:04}: let value = compute();"))
        .collect::<Vec<_>>()
        .join("\n");
    let area = multiline_editor(
        "code",
        &value,
        "",
        TextStyle {
            line_height: 20.0,
            wrap: TextWrap::None,
            ..TextStyle::default()
        },
    )
    .padding(argui_ui::sides(20.0, 16.0))
    .width(length(640.0))
    .height(length(420.0));
    let mut ui = UiTree::new(area);
    let node = ui.node_id_at(0).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(640.0, 420.0))
        .unwrap();
    let viewport = output.text_inputs[0].viewport;
    let maximum = output.scroll_regions[0].max_offset.y;
    assert!(maximum > 19_000.0);

    ui.set_scroll_offset(node, Point::new(0.0, maximum - 20.0));
    assert!(
        !layout
            .apply_scroll_with_text(&mut ui, &mut text, &mut output)
            .unwrap()
    );

    for step in 1..=20 {
        let offset = maximum * step as f32 / 20.0;
        ui.set_scroll_offset(node, Point::new(0.0, offset));
        assert!(
            layout
                .apply_scroll_with_text(&mut ui, &mut text, &mut output)
                .unwrap()
        );
        assert_eq!(output.text_inputs[0].viewport, viewport);
        assert_eq!(output.scroll_regions[0].max_offset.y, maximum);
    }

    let visible = output.text.blocks()[0].content.as_str();
    assert!(visible.contains("line 0999"), "{visible}");
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
        selection_highlight: argui_ui::TextSelectionHighlight::solid(Color::TRANSPARENT),
        caret_style: argui_ui::CaretStyle::default(),
        content_size: Size::new(100.0, 60.0),
        scroll_x: 0.0,
        scroll_y: 0.0,
    }
}

#[test]
fn text_area_shapes_and_clips_scrollable_content_with_a_live_scrollbar() {
    let value = (0..30)
        .map(|line| format!("line {line:02} keeps enough text to exercise wrapping"))
        .collect::<Vec<_>>()
        .join("\n");
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .insets(Sides {
        left: 3.0,
        right: 5.0,
        top: 7.0,
        bottom: 21.0,
    });
    let area = multiline_editor("notes", &value, "notes", TextStyle::default())
        .scroll_config(
            ScrollConfig::default()
                .propagation(ScrollPropagation::Contain)
                .scrollbar(scrollbar),
        )
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
    let text_right = region.viewport.origin.x + region.viewport.size.width;
    assert!(text_right <= scrollbar.track.origin.x);
    assert!(region.clip.origin.x + region.clip.size.width <= scrollbar.track.origin.x);
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
    let track = output.scroll_regions[0]
        .scrollbar
        .as_ref()
        .unwrap()
        .vertical
        .as_ref()
        .unwrap()
        .track;
    assert!(region.viewport.origin.x + region.viewport.size.width <= track.origin.x);
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
    let clip = output.text_inputs[0].clip;
    assert!(clip.origin.x + clip.size.width <= track.origin.x);
}

#[test]
fn a_single_line_input_clips_its_text_after_a_retained_resize() {
    let input = single_line_editor(
        "search",
        "",
        "Search components with a deliberately long placeholder",
        TextStyle::default(),
    )
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
        let caret = CaretStyle::new(CaretVisual::new(if empty {
            vec![]
        } else {
            vec![CaretPrimitive::new(
                width,
                CaretHeight::Pixels(height),
                QuadStyle::solid(color),
            )]
        }));
        let mut ui = UiTree::new(
            Element::text_editor(TextEditorSpec {
                value: "hello".to_owned(),
                placeholder: String::new(),
                multiline: false,
                read_only: false,
                filter: TextInputFilter::Any,
                text: TextStyle {
                    wrap: TextWrap::None,
                    ..TextStyle::default()
                },
                placeholder_text: TextStyle::default(),
                selection: Color::WHITE,
                caret,
            })
            .keyed("custom-caret")
            .width(percent(1.0))
            .interaction(
                Interaction::default()
                    .focus_policy(FocusPolicy::TabStop)
                    .cursor(CursorIcon::Text),
            ),
        );
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

#[test]
fn narrow_search_row_clips_long_text_and_reveals_its_caret() {
    let value = "searchcomponents".repeat(3);
    let input = single_line_editor("search", &value, "Search components", TextStyle::default())
        .width(percent(1.0))
        .height(length(22.0))
        .padding(argui_ui::sides(0.0, 0.0));
    let input_slot = Element::container([input]).grow(1.0).min_width(length(0.0));
    let icon = Element::container([])
        .width(length(17.0))
        .height(length(17.0))
        .shrink(0.0);
    let row = Element::row([icon, input_slot])
        .width(percent(1.0))
        .min_width(length(0.0))
        .padding(argui_ui::Sides {
            left: length(10.0),
            right: length(10.0),
            top: length(0.0),
            bottom: length(0.0),
        })
        .gap(8.0);
    let frame = Element::container([row])
        .width(length(204.0))
        .height(length(42.0))
        .clip(argui_ui::CornerRadii::all(8.0));
    let mut ui = UiTree::new(frame);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(400.0, 80.0))
        .unwrap();
    let region = &output.text_inputs[0];
    let frame = output.nodes[0].bounds;
    assert!(region.bounds.origin.x >= frame.origin.x);
    assert!(
        region.bounds.origin.x + region.bounds.size.width <= frame.origin.x + frame.size.width,
        "region={region:?}, frame={frame:?}"
    );
    assert!(region.scroll_x > 0.0);
    let end = region
        .stops
        .iter()
        .find(|stop| stop.position.index == value.len())
        .unwrap();
    assert!(end.point.x >= region.viewport.origin.x);
    assert!(end.point.x <= region.viewport.origin.x + region.viewport.size.width);
    assert!(output.text.blocks()[0].clip.size.width <= region.bounds.size.width);
}

#[path = "input/navigation.rs"]
mod navigation;
