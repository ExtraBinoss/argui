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
        ContainerQuery::max_width(scope, 200.0),
        StylePatch::new().layout(compact),
    );
    let root = Element::container([content])
        .container_scope(scope)
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
        .container_scope(scope)
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
