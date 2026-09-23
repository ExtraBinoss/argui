use argui_core::{Point, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{Axes, Color, Element, Overflow, UiTree, length};

#[test]
fn scroll_repaint_omits_clipped_offscreen_cards_but_keeps_visible_geometry() {
    let cards = (0..80)
        .map(|index| {
            Element::container([Element::container([])
                .keyed(format!("card-{index}"))
                .width(length(48.0))
                .height(length(48.0))
                .background(Color::WHITE)])
            .width(length(120.0))
            .height(length(100.0))
            .shrink(0.0)
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            })
        })
        .collect::<Vec<_>>();
    let mut ui = UiTree::new(
        Element::column(cards)
            .width(length(120.0))
            .height(length(200.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Auto,
            }),
    );
    let viewport = ui.node_id_at(0).unwrap();
    let mut engine = LayoutEngine::new();
    let mut output = engine
        .compute(&mut ui, &mut TextEngine::new(), Size::new(120.0, 200.0))
        .unwrap();
    let node_count = output.nodes.len();
    assert!(node_count > 100);
    ui.set_scroll_offset(viewport, Point::new(0.0, 3_000.0));
    engine.apply_scroll(&ui, &mut output).unwrap();

    assert!(
        output.paint_stats.visited_subtrees < 20,
        "{:?}",
        output.paint_stats
    );
    assert!(output.display_list.len() < 20);
    assert_eq!(output.semantic_bounds.len(), node_count);
    let visible = ui
        .node_ids()
        .iter()
        .find(|node| ui.key(**node) == Some("card-30"))
        .unwrap();
    let bounds = output
        .semantic_bounds
        .iter()
        .find(|(node, _)| node == visible)
        .unwrap()
        .1;
    assert!(bounds.size.height > 0.0);
    let hidden = ui
        .node_ids()
        .iter()
        .find(|node| ui.key(**node) == Some("card-0"))
        .unwrap();
    let bounds = output
        .semantic_bounds
        .iter()
        .find(|(node, _)| node == hidden)
        .unwrap()
        .1;
    assert_eq!(bounds.size, Size::default());
}
