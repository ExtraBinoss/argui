use argui_animation::{Duration, Tween};
use argui_core::{Point, ScrollDelta, Size};
use argui_layout::LayoutEngine;
use argui_paint::{CornerRadii, DisplayCommand, LayerStyle, QuadStyle};
use argui_text::TextEngine;
use argui_ui::{
    Axes, Color, Element, EventHandlerId, EventListener, EventOwnerId, EventType, FlexWrap,
    Overflow, Position, ScrollAnchoring, ScrollAxes, ScrollConfig, ScrollGesture,
    ScrollbarPartStyle, ScrollbarSide, ScrollbarStyle, ScrollbarVisibility, Sides, StylePatch,
    StyleTransition, Transition, UiEventKind, UiTree, ViewportPlacement, VisualState, WindowLayer,
    length, percent, property,
};
use std::time::Duration as StdDuration;

const NOTO_SANS: &[u8] = include_bytes!("../../../assets/fonts/NotoSans-Regular.ttf");

fn text_engine() -> TextEngine {
    TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans")
}

fn content(scrollbar: ScrollbarStyle) -> Element {
    Element::column([
        Element::container([]).height(length(100.0)).shrink(0.0),
        Element::container([]).height(length(200.0)).shrink(0.0),
    ])
    .keyed("scroll")
    .width(length(200.0))
    .height(length(100.0))
    .overflow(Axes {
        x: Overflow::Hidden,
        y: Overflow::Auto,
    })
    .scroll_config(ScrollConfig::default().scrollbar(scrollbar))
    .layer(LayerStyle::new(Default::default()).opacity(0.8))
}

#[test]
/// Wrapped content extends the workspace scroll range and receives chained wheel input.
fn wrapped_children_extend_scroll_content_and_wheel_chains_from_sidebar() {
    let sidebar = Element::column([])
        .width(length(260.0))
        .height(length(628.0))
        .grow(1.0)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default());
    let content = Element::column([])
        .width(percent(0.65))
        .height(length(628.0))
        .grow(3.0);
    let workspace = Element::row([sidebar, content])
        .width(percent(1.0))
        .height(length(628.0))
        .flex_wrap(FlexWrap::Wrap)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default());
    let root = Element::column([
        Element::container([]).height(length(132.0)).shrink(0.0),
        workspace,
    ])
    .width(percent(1.0))
    .height(percent(1.0));
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(420.0, 760.0))
        .unwrap();
    let workspace_node = ui.node_id_at(2).unwrap();
    let workspace_region = output
        .scroll_regions
        .iter()
        .find(|region| region.node == workspace_node)
        .unwrap();
    assert!(workspace_region.max_offset.y >= 628.0);
    assert!(workspace_region.contains(Point::new(100.0, 300.0)));

    let update = ui.scroll(
        Point::new(100.0, 300.0),
        ScrollDelta::Pixels(Point::new(0.0, -700.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    assert!(ui.scroll_offset(workspace_node).y > 0.0);
    layout
        .compute(&mut ui, &mut text, Size::new(420.0, 760.0))
        .unwrap();
    assert!(
        ui.scroll_offset(workspace_node).y > 0.0,
        "a layout pass after wheel input must preserve the user's scroll offset"
    );
}

#[test]
/// The topmost portal viewport receives wheel input before the page behind it.
fn popover_scroll_receives_wheel_before_overlapped_page_scroll() {
    let scroller = |key: &'static str, width: f32, height: f32, content_height: f32| {
        Element::column([Element::container([])
            .height(length(content_height))
            .shrink(0.0)])
        .keyed(key)
        .width(length(width))
        .height(length(height))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default())
    };
    let popup = scroller("settings", 200.0, 120.0, 480.0)
        .viewport_portal(WindowLayer::Popover, ViewportPlacement::centered());
    let page = scroller("page", 400.0, 300.0, 900.0);
    let mut ui = UiTree::new(Element::column([popup, page]));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let output = layout
        .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
        .unwrap();
    let popup_node = ui
        .node_ids()
        .iter()
        .copied()
        .find(|node| ui.key(*node) == Some("settings"))
        .unwrap();
    let page_node = ui
        .node_ids()
        .iter()
        .copied()
        .find(|node| ui.key(*node) == Some("page"))
        .unwrap();
    let point = Point::new(200.0, 150.0);
    assert!(output.scroll_regions.iter().any(|region| {
        region.node == popup_node && region.contains(point) && region.max_offset.y > 0.0
    }));
    assert!(output.scroll_regions.iter().any(|region| {
        region.node == page_node && region.contains(point) && region.max_offset.y > 0.0
    }));

    let target =
        ScrollGesture::default().target(point, StdDuration::ZERO, true, &output.scroll_regions);
    assert_eq!(target, Some(popup_node));
    let update = ui.scroll_from(
        target.unwrap(),
        point,
        ScrollDelta::Pixels(Point::new(0.0, -80.0)),
        &output.scroll_regions,
    );
    assert!(update.scroll_changed);
    assert!(ui.scroll_offset(popup_node).y > 0.0);
    assert_eq!(ui.scroll_offset(page_node).y, 0.0);
}

#[path = "scroll/scrollbar.rs"]
mod scrollbar;

#[test]
fn sticky_children_follow_scroll_without_recomputing_taffy() {
    let sticky = Element::text("Sticky")
        .keyed("sticky")
        .position(Position::Sticky)
        .inset(Sides {
            left: argui_ui::LengthPercentageAuto::auto(),
            right: argui_ui::LengthPercentageAuto::auto(),
            top: argui_ui::LengthPercentageAuto::length(0.0),
            bottom: argui_ui::LengthPercentageAuto::auto(),
        })
        .height(length(24.0));
    let root = Element::column([sticky, Element::container([]).height(length(300.0))])
        .height(length(100.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        });
    let mut ui = UiTree::new(root);
    let sticky_node = ui.node_id_at(1).unwrap();
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let mut output = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    let initial = output
        .nodes
        .iter()
        .find(|node| node.node == sticky_node)
        .unwrap()
        .bounds;
    ui.scroll(
        Point::new(20.0, 20.0),
        ScrollDelta::Pixels(Point::new(0.0, -60.0)),
        &output.scroll_regions,
    );
    layout.apply_scroll(&ui, &mut output).unwrap();
    let moved = output
        .nodes
        .iter()
        .find(|node| node.node == sticky_node)
        .unwrap()
        .bounds;
    assert_eq!(moved.origin.y, initial.origin.y);
}

#[test]
fn automatic_scroll_anchoring_preserves_the_first_visible_keyed_row() {
    let view = |first_height| {
        Element::column([
            Element::text("first")
                .keyed("first")
                .height(length(first_height))
                .shrink(0.0),
            Element::text("anchor")
                .keyed("anchor")
                .height(length(30.0))
                .shrink(0.0),
            Element::container([]).height(length(200.0)).shrink(0.0),
        ])
        .keyed("list")
        .height(length(60.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
    };
    let mut ui = UiTree::new(view(30.0));
    let list = ui.node_id_at(0).unwrap();
    ui.set_scroll_offset(list, Point::new(0.0, 30.0));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let first = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    let anchor = ui
        .resolve_node(&argui_ui::FocusTarget::from("anchor"))
        .unwrap();
    let initial_y = first
        .nodes
        .iter()
        .find(|node| node.node == anchor)
        .unwrap()
        .bounds
        .origin
        .y;

    ui.update(view(50.0));
    let second = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    let anchored_y = second
        .nodes
        .iter()
        .find(|node| node.node == anchor)
        .unwrap()
        .bounds
        .origin
        .y;
    assert_eq!(anchored_y, initial_y);
    assert_eq!(ui.scroll_offset(list), Point::new(0.0, 50.0));
}

#[test]
fn disabled_scroll_anchoring_leaves_the_explicit_offset_unchanged() {
    let view = |first_height| {
        Element::column([
            Element::text("first")
                .keyed("first")
                .height(length(first_height))
                .shrink(0.0),
            Element::text("second")
                .keyed("second")
                .height(length(30.0))
                .shrink(0.0),
            Element::container([]).height(length(200.0)).shrink(0.0),
        ])
        .keyed("list")
        .height(length(60.0))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Auto,
        })
        .scroll_config(ScrollConfig::default().anchoring(ScrollAnchoring::None))
    };
    let mut ui = UiTree::new(view(30.0));
    let list = ui.node_id_at(0).unwrap();
    ui.set_scroll_offset(list, Point::new(0.0, 30.0));
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    ui.update(view(50.0));
    layout
        .compute(&mut ui, &mut text, Size::new(200.0, 60.0))
        .unwrap();
    assert_eq!(ui.scroll_offset(list), Point::new(0.0, 30.0));
}

#[test]
fn variable_virtual_lists_measure_visible_rows_during_layout() {
    let list = argui_ui::VirtualList::variable(6, 20.0, 60.0).overscan(1);
    let view = || {
        list.build("variable-list", 0.0, |index| {
            Element::container([])
                .keyed(format!("variable-{index}"))
                .height(length(12.0 + index as f32 * 4.0))
        })
        .width(length(200.0))
    };
    let mut ui = UiTree::new(view());
    assert!(ui.root().children[0].children[1].virtual_item().is_some());
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();

    let first = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert_eq!(list.item_extent(0), Some(12.0));
    assert_eq!(list.item_extent(3), Some(24.0));
    assert!(first.virtualization_changed);

    ui.update(view());
    let settled = layout
        .compute(&mut ui, &mut text, Size::new(200.0, 100.0))
        .unwrap();
    assert!(!settled.virtualization_changed);
}

#[test]
fn horizontal_virtual_lists_measure_widths_and_request_a_bounded_window() {
    let list = argui_ui::VirtualList::variable(100, 20.0, 90.0)
        .horizontal()
        .overscan(2);
    let root = list
        .build_range("horizontal-list", 0.0, 0..12, |index| {
            Element::container([])
                .keyed(format!("item-{index}"))
                .width(length(12.0 + index as f32 * 3.0))
                .height(length(24.0))
        })
        .width(length(90.0))
        .on(EventListener::new(
            EventType::VirtualMeasure,
            EventHandlerId::new(EventOwnerId(1), 1),
        ))
        .on(EventListener::new(
            EventType::VirtualWindow,
            EventHandlerId::new(EventOwnerId(1), 2),
        ));
    let mut ui = UiTree::new(root);
    let mut layout = LayoutEngine::new();
    let mut text = text_engine();
    let first = layout
        .compute(&mut ui, &mut text, Size::new(90.0, 40.0))
        .unwrap();
    assert!(first.virtualization_changed);
    assert_eq!(list.item_extent(0), Some(12.0));
    assert_eq!(list.item_extent(3), Some(21.0));
    assert!(list.window(0.0).range.len() < 24);
    assert!(first.virtual_events.iter().any(|event| matches!(
        &event.kind,
        UiEventKind::VirtualMeasured { items, viewport_extent: 90.0, .. }
            if items.iter().any(|item| item.index == 3 && item.extent == 21.0)
    )));
    assert!(first.virtual_events.iter().any(|event| matches!(
        event.kind,
        UiEventKind::VirtualWindowChanged { start: 0, end, .. } if end < 24
    )));
}
