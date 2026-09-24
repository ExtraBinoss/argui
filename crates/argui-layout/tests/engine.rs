use argui_core::{Point, Rect, ScrollDelta, Size, TextPosition};
use argui_layout::LayoutEngine;
use argui_paint::{DisplayCommand, QuadStyle};
use argui_text::{TextEngine, TextOverflow, TextStyle, TextWrap};
use argui_ui::{
    AlignItems, Axes, CaretStyle, Color, CursorIcon, Element, FlexWrap, FloatingPlacement,
    FocusPolicy, Interaction, LengthPercentageAuto, Overflow, Placement, ScrollConfig, Sides,
    StylePatch, TextEditorSpec, TextInputFilter, UiTree, VisualState, WindowDragBehavior,
    WindowLayer, auto, length, percent, sides,
};

#[path = "engine/compute.rs"]
mod compute_tests;

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

fn top_right(top: f32, right: f32) -> Sides<LengthPercentageAuto> {
    Sides {
        left: auto(),
        right: length(right),
        top: length(top),
        bottom: auto(),
    }
}

fn single_line_editor(key: &str, value: &str, placeholder: &str, mut text: TextStyle) -> Element {
    text.wrap = TextWrap::None;
    let mut placeholder_text = text.clone();
    placeholder_text.overflow = TextOverflow::Ellipsis(argui_text::EllipsisPosition::End);
    Element::text_editor(TextEditorSpec {
        value: value.to_owned(),
        placeholder: placeholder.to_owned(),
        multiline: false,
        read_only: false,
        filter: TextInputFilter::Any,
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

#[test]
fn interaction_repaint_reuses_layout_and_shaped_text() {
    let root = Element::text("Fast repaint")
        .keyed("button")
        .padding(Sides::length(12.0))
        .background(Color::srgb(0.0, 0.0, 0.0))
        .interaction(Interaction::default())
        .when(
            VisualState::Hovered,
            StylePatch::from_quad(QuadStyle::solid(Color::WHITE)),
        );
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    let text_before = output.text.clone();
    let base = output.display_list.commands()[0].clone();

    let update = ui.pointer_moved(Point::new(10.0, 10.0), &output.hit_regions);
    assert!(update.paint_changed);
    layout.repaint(&ui, &mut output);

    assert_eq!(output.text, text_before);
    assert!(output.text.blocks()[0].clip.size.width > output.text.blocks()[0].bounds.size.width);
    assert_ne!(output.display_list.commands()[0], base);
    assert!(matches!(
        &output.display_list.commands()[0],
        DisplayCommand::Quad(quad) if quad.background == Some(argui_paint::Fill::Solid(Color::WHITE))
    ));
}

#[test]
fn unchanged_static_subtree_reuses_its_retained_paint_fragment() {
    let mut ui = UiTree::new(Element::column((0..5_000).map(|index| {
        Element::text(format!("row {index}")).background(Color::srgb(0.1, 0.2, 0.3))
    })));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(800.0, 50_000.0))
        .unwrap();
    let commands = output.display_list.clone();

    layout.repaint(&ui, &mut output);

    assert_eq!(output.display_list, commands);
    assert_eq!(output.paint_stats.visited_subtrees, 1);
    assert_eq!(output.paint_stats.reused_subtrees, 1);
    assert_eq!(output.paint_stats.reused_commands, commands.len());
}

#[test]
fn wrapped_rows_move_whole_items_instead_of_clipping_them() {
    let item = || {
        Element::container([])
            .width(length(80.0))
            .height(length(30.0))
    };
    let mut ui = UiTree::new(
        Element::row([item(), item(), item()])
            .width(percent(1.0))
            .flex_wrap(FlexWrap::Wrap)
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
fn wrapped_flex_labels_keep_every_glyph() {
    let item = |key, label| {
        Element::row([Element::text(label).text_style(TextStyle {
            font_size: 17.0,
            line_height: 22.0,
            wrap: TextWrap::None,
            ..TextStyle::default()
        })])
        .keyed(key)
        .padding(sides(18.0, 11.0))
        .background(Color::srgb(0.1, 0.2, 0.3))
    };
    let mut ui = UiTree::new(
        Element::row([
            item("primary", "Primary action"),
            item("confirm", "Confirm"),
            item("delete", "Delete"),
        ])
        .width(percent(1.0))
        .flex_wrap(FlexWrap::Wrap)
        .gap(12.0),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(260.0, 180.0))
        .unwrap();

    for block in output.text.blocks() {
        let intrinsic = text.measure(block.content.as_str(), &block.style, None);
        assert!(
            block.bounds.size.width + 1.0 >= intrinsic.width,
            "{}: layout {} < intrinsic {}",
            block.content.as_str(),
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
    .width(percent(1.0))
    .height(percent(1.0))
    .padding(Sides::length(20.0))
    .background(Color::srgb(0.1, 0.2, 0.3));
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

#[test]
fn percent_child_stays_inside_padded_parent() {
    let button = |label| {
        Element::text(label)
            .text_style(TextStyle {
                font_size: 17.0,
                line_height: 21.25,
                ..TextStyle::default()
            })
            .padding(sides(20.0, 10.0))
            .shrink(0.0)
    };
    let panel = Element::column([
        Element::text("State updates: 10").text_style(TextStyle {
            font_size: 42.0,
            line_height: 52.5,
            ..TextStyle::default()
        }),
        Element::text(
            "Clicks mutate plain Rust state. The rebuilt tree decides whether it needs no work, a repaint, or a new layout.",
        )
        .text_style(TextStyle {
            font_size: 19.0,
            line_height: 23.75,
            ..TextStyle::default()
        }),
        Element::row([
            button("Increment"),
            button("Toggle paint"),
            button("Reorder keys"),
        ])
        .flex_wrap(FlexWrap::Wrap)
        .gap(12.0),
        Element::row([
            Element::text("Stable alpha")
                .padding(sides(12.0, 7.0)),
            Element::text("Stable beta")
                .padding(sides(12.0, 7.0)),
        ])
        .flex_wrap(FlexWrap::Wrap)
        .gap(10.0),
    ])
        .width(percent(1.0))
        .padding(Sides::length(30.0))
        .gap(22.0);
    let mut ui = UiTree::new(
        Element::column([panel])
            .width(percent(1.0))
            .height(percent(1.0))
            .align_items(AlignItems::CENTER)
            .padding(sides(42.0, 36.0)),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(485.6, 464.0))
        .unwrap();
    let root = output.nodes[0].bounds;
    let panel = output.nodes[1].bounds;
    assert!(panel.origin.x >= root.origin.x + 42.0, "{root:?} {panel:?}");
    assert!(
        panel.origin.x + panel.size.width <= root.origin.x + root.size.width - 42.0,
        "{root:?} {panel:?}"
    );
}

#[test]
fn scroll_translates_geometry_without_rebuilding_taffy() {
    let root = Element::column([
        Element::text("First row").height(length(100.0)).shrink(0.0),
        Element::text("Overscan row")
            .height(length(200.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            })
            .shrink(0.0),
    ])
    .keyed("scroll")
    .width(length(200.0))
    .height(length(100.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default());
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let revision = ui.revision();
    let before = output.nodes[1].bounds;
    assert_eq!(output.text.blocks().len(), 2);
    assert_eq!(output.text.blocks()[1].clip, Rect::default());

    let update = ui.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Lines(Point::new(0.0, -1.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    layout.apply_scroll(&ui, &mut output).unwrap();

    assert_eq!(ui.revision(), revision);
    assert_eq!(output.nodes[1].bounds.origin.y, before.origin.y - 40.0);
    assert_eq!(output.text.blocks()[0].bounds.origin.y, -40.0);
    assert_eq!(output.text.blocks()[1].bounds.origin.y, 60.0);
    assert_eq!(output.text.blocks()[1].clip.size.height, 40.0);
    assert_eq!(output.text.blocks()[0].clip.size.height, 100.0);
}

#[test]
fn anchored_overlay_follows_a_scrolling_anchor_without_relayout() {
    let anchor = Element::container([])
        .keyed("anchor")
        .width(length(80.0))
        .height(length(30.0))
        .shrink(0.0);
    let overlay = Element::container([])
        .keyed("overlay")
        .width(length(120.0))
        .height(length(60.0))
        .background(Color::WHITE)
        .anchored_portal(
            WindowLayer::Popover,
            "anchor",
            FloatingPlacement::new(Placement::BottomStart)
                .offset(6.0)
                .viewport_padding(0.0),
        );
    let mut ui = UiTree::new(
        Element::column([
            Element::container([]).height(length(100.0)).shrink(0.0),
            anchor,
            overlay,
            Element::container([]).height(length(200.0)).shrink(0.0),
        ])
        .keyed("scroll")
        .width(length(240.0))
        .height(length(240.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default()),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 240.0))
        .unwrap();
    let revision = ui.revision();
    let anchor_node = ui.node_id_at(2).unwrap();
    let overlay_node = ui.node_id_at(3).unwrap();
    let before_anchor = output.nodes[2].bounds;
    let before_overlay = output.nodes[3].bounds;
    assert_eq!(before_overlay.origin.x, before_anchor.origin.x);
    assert_eq!(before_overlay.origin.y, before_anchor.origin.y + 36.0);
    assert_eq!(output.nodes[3].clip, Some(output.viewport));

    let update = ui.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Pixels(Point::new(0.0, -40.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    layout.apply_scroll(&ui, &mut output).unwrap();

    let anchor_after = output
        .nodes
        .iter()
        .find(|node| node.node == anchor_node)
        .unwrap()
        .bounds;
    let overlay_after = output
        .nodes
        .iter()
        .find(|node| node.node == overlay_node)
        .unwrap()
        .bounds;
    assert_eq!(ui.revision(), revision);
    assert_eq!(anchor_after.origin.y, before_anchor.origin.y - 40.0);
    assert_eq!(overlay_after.origin.y, anchor_after.origin.y + 36.0);
    assert_eq!(overlay_after.origin.y, before_overlay.origin.y - 40.0);
    output.display_list.validate().unwrap();
}

#[test]
fn focused_text_inputs_emit_cosmic_caret_selection_and_hit_geometry() {
    let input = single_line_editor(
        "field",
        "Hello مرحباً 👋🏽",
        "hint",
        TextStyle {
            font_size: 18.0,
            line_height: 24.0,
            ..TextStyle::default()
        },
    )
    .background(Color::srgb(0.1, 0.1, 0.1));
    let mut ui = UiTree::new(input);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let initial = layout
        .compute(&mut ui, &mut text, Size::new(280.0, 60.0))
        .unwrap();
    ui.pointer_moved(Point::new(20.0, 20.0), &initial.hit_regions);
    ui.primary_pressed(&initial.hit_regions);
    let node = ui.focused_node().unwrap();
    ui.move_text_cursor(node, 0, false);
    ui.move_text_cursor(node, 5, true);
    let output = layout
        .compute(&mut ui, &mut text, Size::new(280.0, 60.0))
        .unwrap();
    let region = &output.text_inputs[0];

    assert!(region.caret.is_some());
    assert!(!region.selection.is_empty());
    assert!(region.stops.len() > 5);
    assert!(region.hit_index(Point::new(20.0, 20.0)).is_some());
    assert!(region.hit_index(Point::new(-20.0, -20.0)).is_none());
    let start = region
        .stops
        .iter()
        .find(|stop| stop.position.index == 0)
        .unwrap()
        .position;
    let right = region.visual_neighbor(start, false, false);
    assert_ne!(right, start);
    assert_eq!(region.visual_neighbor(right, true, false), start);
    assert_ne!(region.visual_neighbor(start, false, true), start);
    let missing = TextPosition::new(usize::MAX, Default::default());
    assert_eq!(region.visual_neighbor(missing, true, false), missing);
    let mut unfocused = region.clone();
    unfocused.caret = None;
    assert_eq!(unfocused.visual_neighbor(missing, true, false), missing);
    let mut clipped = region.clone();
    clipped.clip = argui_core::Rect::new(Point::new(200.0, 0.0), Size::new(80.0, 60.0));
    assert!(clipped.hit_index(Point::new(20.0, 20.0)).is_none());
    assert!(output.display_list.quad_count() >= 3);
}

#[test]
fn scrolling_repositions_text_input_hit_geometry_without_reshaping() {
    let input = single_line_editor("field", "Editable", "hint", TextStyle::default())
        .height(length(40.0))
        .shrink(0.0);
    let root = Element::column([
        Element::container([]).height(length(40.0)).shrink(0.0),
        input,
    ])
    .height(length(60.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default());
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 60.0))
        .unwrap();
    let before = output.text_inputs[0].bounds.origin;

    ui.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -20.0)),
        &output.scroll_regions,
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    assert_eq!(output.text_inputs[0].bounds.origin.y, before.y - 20.0);
}

#[test]
fn caret_and_selection_updates_skip_taffy_and_prepared_text_rebuilds() {
    let input = single_line_editor("field", "Latin العربية Latin", "hint", TextStyle::default());
    let mut ui = UiTree::new(Element::column([Element::text("Label"), input]));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(300.0, 50.0))
        .unwrap();
    let node = ui.node_id_at(2).unwrap();
    let content = output.text.clone();

    let update = ui.move_text_cursor(node, 5, true);
    assert!(update.text_input_changed);
    assert!(!update.layout_changed);
    layout.update_text_inputs(&mut ui, &mut text, &mut output);

    assert_eq!(output.text.blocks()[1].content, content.blocks()[1].content);
    assert!(!output.text_inputs[0].selection.is_empty());

    let mut missing_region = output.clone();
    missing_region.text_inputs.clear();
    layout.update_text_inputs(&mut ui, &mut text, &mut missing_region);
    assert!(missing_region.text_inputs.is_empty());
}

#[test]
fn sibling_z_index_controls_paint_and_hit_test_order() {
    let child = |color, z| {
        Element::container([])
            .background(color)
            .width(length(80.0))
            .height(length(20.0))
            .shrink(0.0)
            .interaction(Interaction::default())
            .z_index(z)
    };
    let low = Color::srgb(0.1, 0.2, 0.3);
    let high = Color::srgb(0.8, 0.7, 0.6);
    let mut ui = UiTree::new(
        Element::column([child(high, 4), child(low, -2)])
            .width(length(100.0))
            .height(length(100.0)),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();

    assert!(matches!(
        &output.display_list.commands()[0],
        DisplayCommand::Quad(quad) if quad.background == Some(argui_paint::Fill::Solid(low))
    ));
    assert_eq!(
        output.hit_regions.last().unwrap().node,
        ui.node_id_at(1).unwrap()
    );
}

#[test]
fn window_drag_regions_preserve_interactive_child_priority() {
    let root = Element::row([Element::container([])
        .width(length(80.0))
        .height(length(40.0))
        .interaction(Interaction::default())])
    .width(length(240.0))
    .height(length(40.0))
    .interaction(Interaction::default().window_drag(WindowDragBehavior::MoveAndToggleMaximize));
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(240.0, 40.0))
        .unwrap();

    assert_eq!(
        output.hit_regions[0].window_drag,
        Some(WindowDragBehavior::MoveAndToggleMaximize)
    );
    assert_eq!(output.hit_regions[1].window_drag, None);
    let top = output
        .hit_regions
        .iter()
        .rev()
        .find(|region| region.contains(Point::new(20.0, 20.0)))
        .unwrap();
    assert_eq!(top.node, ui.node_id_at(1).unwrap());
    assert_eq!(top.window_drag, None);
}
