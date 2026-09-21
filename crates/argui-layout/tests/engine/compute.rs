use argui_core::Size;
use argui_layout::{LayoutEngine, LayoutError};
use argui_text::TextEngine;
use argui_ui::{
    ContainerQuery, ContainerScopeId, Element, FlexDirection, StylePatch, UiTree, length, percent,
};

const NOTO_SANS: &[u8] =
    include_bytes!("../../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf");

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
