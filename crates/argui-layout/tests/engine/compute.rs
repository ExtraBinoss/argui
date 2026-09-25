use super::top_right;
use argui_core::{Point, Size};
use argui_layout::{LayoutEngine, LayoutError};
use argui_text::TextEngine;
use argui_ui::{
    ContainerQuery, ContainerScopeId, Element, FlexDirection, StylePatch, UiTree, length, percent,
};

const NOTO_SANS: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

#[test]
fn safe_area_builder_keeps_layout_content_inside_system_insets() {
    let mut ui = UiTree::new(
        Element::container([Element::container([])
            .keyed("content")
            .width(length(60.0))
            .height(length(40.0))])
        .safe_area(argui_core::Insets::new(21.0, 23.0, 25.0, 17.0))
        .width(percent(1.0))
        .height(percent(1.0)),
    );
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(300.0, 200.0))
        .unwrap();

    assert_eq!(output.nodes[0].bounds.size, Size::new(300.0, 200.0));
    assert_eq!(
        output.nodes[1].bounds.origin,
        argui_core::Point::new(17.0, 21.0)
    );
    assert_eq!(output.nodes[1].bounds.size, Size::new(260.0, 40.0));
}

#[test]
fn container_query_reflows_before_paint_and_reacts_to_resize() {
    let scope = ContainerScopeId::new("panel");
    let base = Element::row([
        Element::container([])
            .width(length(30.0))
            .height(length(20.0)),
        Element::container([])
            .width(length(30.0))
            .height(length(20.0)),
    ])
    .keyed("content")
    .gap(8.0);
    let mut compact = base.style.clone();
    compact.flex_direction = FlexDirection::Column;
    let content = base.when(
        ContainerQuery::max_width(scope.clone(), 200.0),
        StylePatch::new().layout(compact),
    );
    let root = Element::container([content])
        .container_scope(scope.clone())
        .width(percent(1.0))
        .height(percent(1.0));
    let mut ui = UiTree::new(root);
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();

    let compact_output = engine
        .compute(&mut ui, &mut text, Size::new(180.0, 100.0))
        .unwrap();
    assert!(compact_output.nodes[3].bounds.origin.y > compact_output.nodes[2].bounds.origin.y);
    assert_eq!(compact_output.paint_stats.visited_subtrees, 4);

    let wide_output = engine
        .compute(&mut ui, &mut text, Size::new(320.0, 100.0))
        .unwrap();
    assert!(wide_output.nodes[3].bounds.origin.x > wide_output.nodes[2].bounds.origin.x);
}

#[test]
fn oscillating_container_query_returns_a_deterministic_error() {
    let scope = ContainerScopeId::new("cycle");
    let base = Element::container([]).width(length(100.0));
    let mut expanded = base.style.clone();
    expanded.size.width = length(200.0);
    let root = base
        .container_scope(scope.clone())
        .when(
            ContainerQuery::max_width(scope, 150.0),
            StylePatch::new().layout(expanded),
        )
        .height(length(40.0));
    let mut ui = UiTree::new(root);

    let error = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(300.0, 100.0))
        .unwrap_err();
    assert!(matches!(error, LayoutError::NonConvergentContainerQueries));
}

#[test]
fn layout_recovers_after_nonconvergent_queries_are_removed() {
    let scope = ContainerScopeId::new("unstable");
    let base = Element::container([]).width(length(100.0));
    let mut expanded = base.style.clone();
    expanded.size.width = length(200.0);
    let mut ui = UiTree::new(base.container_scope(scope.clone()).when(
        ContainerQuery::max_width(scope, 150.0),
        StylePatch::new().layout(expanded),
    ));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    assert!(matches!(
        engine.compute(&mut ui, &mut text, Size::new(300.0, 100.0)),
        Err(LayoutError::NonConvergentContainerQueries)
    ));
    ui.update(
        Element::container([])
            .keyed("recovered")
            .width(length(40.0))
            .height(length(20.0)),
    );
    let output = engine
        .compute(&mut ui, &mut text, Size::new(300.0, 100.0))
        .unwrap();
    assert_eq!(output.nodes.len(), 1);
    assert_eq!(output.nodes[0].bounds.size, Size::new(40.0, 20.0));
    assert_eq!(ui.key(output.nodes[0].node), Some("recovered"));
}

#[test]
fn retained_layout_drops_removed_nodes_across_empty_and_reordered_updates() {
    let build = |keys: &[usize]| {
        Element::column(keys.iter().map(|key| {
            Element::container([])
                .keyed(format!("row-{key}"))
                .width(length(30.0))
                .height(length(12.0))
        }))
        .width(percent(1.0))
    };
    let mut ui = UiTree::new(build(&[0, 1, 2, 3]));
    let mut engine = LayoutEngine::new();
    let mut text = text_engine();
    for keys in [&[0, 1, 2, 3][..], &[3, 1], &[], &[7], &[7, 3, 9]] {
        ui.update(build(keys));
        for width in [300.0, 0.0, 75.0] {
            let output = engine
                .compute(&mut ui, &mut text, Size::new(width, 100.0))
                .unwrap();
            assert_eq!(output.nodes.len(), keys.len() + 1);
            for (index, key) in keys.iter().enumerate() {
                let node = &output.nodes[index + 1];
                assert_eq!(ui.key(node.node), Some(format!("row-{key}").as_str()));
                assert_eq!(node.bounds.origin.y, index as f32 * 12.0);
                assert_eq!(node.bounds.size.height, 12.0);
            }
        }
    }
}

#[test]
fn sticky_edges_constrain_both_axes_and_allow_auto_edges() {
    use argui_ui::{Axes, Overflow, Position, Sides, auto};
    for inset in [
        Sides {
            left: auto(),
            top: auto(),
            right: auto(),
            bottom: auto(),
        },
        Sides::length(0.0),
    ] {
        let child = Element::container([])
            .keyed("sticky")
            .position(Position::Sticky)
            .inset(inset)
            .margin(Sides {
                left: length(90.0),
                top: length(90.0),
                right: length(0.0),
                bottom: length(0.0),
            })
            .width(length(20.0))
            .height(length(20.0))
            .shrink(0.0);
        let mut ui = UiTree::new(
            Element::container([child])
                .width(length(100.0))
                .height(length(100.0))
                .overflow(Axes {
                    x: Overflow::Auto,
                    y: Overflow::Auto,
                }),
        );
        let output = LayoutEngine::new()
            .compute(&mut ui, &mut text_engine(), Size::new(200.0, 200.0))
            .unwrap();
        let expected = if inset.top == auto() { 90.0 } else { 80.0 };
        assert_eq!(
            output.nodes[1].bounds.origin,
            argui_core::Point::new(expected, expected)
        );
    }
    let mut ui = UiTree::new(
        Element::container([])
            .position(Position::Sticky)
            .inset(Sides::length(30.0))
            .width(length(20.0))
            .height(length(20.0)),
    );
    let output = LayoutEngine::new()
        .compute(&mut ui, &mut text_engine(), Size::new(200.0, 200.0))
        .unwrap();
    assert_eq!(output.nodes[0].bounds.size, Size::new(20.0, 20.0));
}

#[test]
fn absolute_overlays_do_not_participate_in_flex_flow() {
    let mut ui = UiTree::new(
        Element::column([
            Element::container([])
                .width(length(80.0))
                .height(length(30.0)),
            Element::container([])
                .width(length(50.0))
                .height(length(20.0))
                .absolute(top_right(5.0, 7.0))
                .z_index(10),
        ])
        .width(length(200.0))
        .height(length(100.0)),
    );
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();

    assert_eq!(output.nodes[1].bounds.origin, Point::default());
    assert_eq!(output.nodes[2].bounds.origin, Point::new(143.0, 5.0));
}

mod input_latency {
    use web_time::Instant;

    use argui_core::{Color, Key, KeyInput, KeyState, Modifiers, Size};
    use argui_layout::LayoutEngine;
    use argui_text::{FontFamily, TextEngine, TextStyle, TextWrap};
    use argui_ui::{
        CaretStyle, Element, FocusPolicy, FocusRequest, Interaction, TextEditorSpec,
        TextInputFilter, UiTree, length,
    };

    const FONT: &[u8] = include_bytes!("../../../../assets/fonts/NotoSans-Regular.ttf");

    fn editor(value: &str, monospace: bool, multiline: bool) -> Element {
        Element::text_editor(TextEditorSpec {
            value: value.to_owned(),
            placeholder: String::new(),
            multiline,
            read_only: false,
            filter: TextInputFilter::Any,
            text: TextStyle {
                family: if monospace {
                    FontFamily::Monospace
                } else {
                    FontFamily::SansSerif
                },
                wrap: TextWrap::None,
                ..TextStyle::default()
            },
            placeholder_text: TextStyle::default(),
            selection: Color::WHITE,
            caret: CaretStyle::default(),
        })
        .width(length(420.0))
        .height(length(100.0))
        .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))
    }

    fn burst(value: &str, count: usize, monospace: bool, multiline: bool) -> (f64, f64) {
        let mut ui = UiTree::new(editor(value, monospace, multiline));
        let node = ui.node_ids()[0];
        let mut layout = LayoutEngine::new();
        let mut text =
            TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
        let size = Size::new(420.0, 100.0);
        let initial = Instant::now();
        let mut output = layout.compute(&mut ui, &mut text, size).unwrap();
        let initial_ms = initial.elapsed().as_secs_f64() * 1_000.0;
        ui.sync_focus(&output.hit_regions, Some(FocusRequest::Focus(node.into())));
        ui.place_text_cursor(node, value.len(), false);
        let mut slowest = 0.0_f64;
        let mut slowest_edit = 0.0_f64;
        let mut slowest_layout = 0.0_f64;
        let mut slowest_prepare = 0.0_f64;
        for _ in 0..count {
            let started = Instant::now();
            let update = ui.edit_text_input(&KeyInput {
                key: Key::Character("x".into()),
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: Some("x".into()),
            });
            let edited = started.elapsed().as_secs_f64() * 1_000.0;
            assert!(update.text_input_changed);
            layout.update_text_inputs(&mut ui, &mut text, &mut output);
            let laid_out = started.elapsed().as_secs_f64() * 1_000.0;
            let prepared = text.prepare(&output.text, 1.0);
            let prepared_at = started.elapsed().as_secs_f64() * 1_000.0;
            assert!(!prepared.glyphs.is_empty());
            slowest_edit = slowest_edit.max(edited);
            slowest_layout = slowest_layout.max(laid_out - edited);
            slowest_prepare = slowest_prepare.max(prepared_at - laid_out);
            slowest = slowest.max(prepared_at);
        }
        assert_eq!(
            ui.text_input_value(node).unwrap().len(),
            value.len() + count
        );
        assert_eq!(
            output.text.blocks()[0].content.as_str().chars().last(),
            Some('x')
        );
        let region = &output.text_inputs[0];
        let caret = region.caret.unwrap();
        assert!(
            caret.origin.x >= region.viewport.origin.x - 1.0
                && caret.origin.x <= region.viewport.origin.x + region.viewport.size.width + 1.0
        );
        assert!(
            caret.origin.y >= region.viewport.origin.y - 1.0
                && caret.origin.y <= region.viewport.origin.y + region.viewport.size.height + 1.0
        );
        assert!(
            region
                .stops
                .iter()
                .any(|stop| stop.position.index == value.len() + count)
        );
        eprintln!(
            "phases (ms): edit {slowest_edit:.2}, layout {slowest_layout:.2}, prepare {slowest_prepare:.2}"
        );
        (initial_ms, slowest)
    }

    #[test]
    fn burst_typing_reaches_visible_text_for_small_and_million_character_editors() {
        let (small_initial, small_slowest) = burst("hello", 32, false, false);
        eprintln!(
            "small input-to-prepared-text initial {small_initial:.2} ms, slowest edit {small_slowest:.2} ms"
        );
        let value = format!("{}\n", "a".repeat(99)).repeat(10_000);
        assert_eq!(value.len(), 1_000_000);
        let (large_initial, large_slowest) = burst(&value, 8, false, true);
        eprintln!(
            "input-to-prepared-text latency (ms): small initial {small_initial:.2}, slowest edit {small_slowest:.2}; million initial {large_initial:.2}, slowest edit {large_slowest:.2}"
        );
    }

    #[test]
    fn burst_typing_reaches_visible_text_in_a_million_character_line() {
        let value = "a".repeat(1_000_000);
        let (initial, slowest) = burst(&value, 8, false, false);
        eprintln!(
            "single-line million field input-to-prepared-text latency (ms): initial {initial:.2}, slowest edit {slowest:.2}"
        );
        let (initial, slowest) = burst(&value, 8, true, false);
        eprintln!(
            "single-line million monospace input-to-prepared-text latency (ms): initial {initial:.2}, slowest edit {slowest:.2}"
        );
    }

    #[test]
    fn burst_typing_reaches_visible_text_in_a_million_byte_unicode_line() {
        let value = "é".repeat(500_000);
        assert_eq!(value.len(), 1_000_000);
        let (initial, slowest) = burst(&value, 4, false, false);
        eprintln!(
            "single-line million-byte Unicode field latency (ms): initial {initial:.2}, slowest edit {slowest:.2}"
        );
    }

    #[test]
    fn middle_of_million_character_editor_updates_its_visible_window() {
        let value = format!("{}\n", "a".repeat(99)).repeat(10_000);
        let mut ui = UiTree::new(editor(&value, false, true));
        let node = ui.node_ids()[0];
        let mut layout = LayoutEngine::new();
        let mut text =
            TextEngine::from_embedded_fonts([FONT], "Noto Sans", "Noto Sans", "Noto Sans");
        let mut output = layout
            .compute(&mut ui, &mut text, Size::new(420.0, 100.0))
            .unwrap();
        ui.sync_focus(&output.hit_regions, Some(FocusRequest::Focus(node.into())));
        ui.place_text_cursor(node, value.len() / 2, false);
        let mut slowest = 0.0_f64;
        for sample in 0..4 {
            let started = Instant::now();
            ui.edit_text_input(&KeyInput {
                key: Key::Character("x".into()),
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: Some("x".into()),
            });
            let edited_ms = started.elapsed().as_secs_f64() * 1_000.0;
            layout.update_text_inputs(&mut ui, &mut text, &mut output);
            let laid_out_ms = started.elapsed().as_secs_f64() * 1_000.0;
            assert!(!text.prepare(&output.text, 1.0).glyphs.is_empty());
            let prepared_ms = started.elapsed().as_secs_f64() * 1_000.0;
            slowest = slowest.max(prepared_ms);
            eprintln!(
                "million-character middle edit sample={sample} edit_ms={edited_ms:.2} layout_ms={:.2} prepare_ms={:.2}",
                laid_out_ms - edited_ms,
                prepared_ms - laid_out_ms,
            );
        }
        let region = &output.text_inputs[0];
        assert!(
            region
                .stops
                .iter()
                .any(|stop| stop.position.index == value.len() / 2 + 4)
        );
        assert!(output.text.blocks()[0].content.as_str().contains("xxxx"));
        eprintln!("million-character middle edit input-to-prepared-text slowest {slowest:.2} ms");
    }
}
