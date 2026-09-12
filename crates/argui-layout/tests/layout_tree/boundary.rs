use argui_core::{Color, Point, Rect, Size};
use argui_layout::LayoutEngine;
use argui_text::TextEngine;
use argui_ui::{
    Axes, CustomConstraints, CustomElement, CustomLayoutContext, CustomMeasurement,
    CustomPaintContext, Display, Element, Overflow, ScrollConfig, Sides, UiTree, length, percent,
};

fn content() -> Element {
    Element::column([
        Element::container([])
            .keyed("first")
            .width(length(40.0))
            .height(length(20.0)),
        Element::container([])
            .keyed("last")
            .width(percent(1.0))
            .height(length(1000.0)),
    ])
    .shrink(0.0)
    .background(Color::WHITE)
}

fn panel(content: Element) -> Element {
    Element::layout_boundary(content)
        .width(length(120.0))
        .height(length(80.0))
        .padding(Sides {
            left: percent(0.03),
            right: length(5.0),
            top: length(7.0),
            bottom: length(3.0),
        })
        .border(argui_paint::Border::all(2.0, Color::WHITE))
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Scroll,
        })
        .scroll_config(ScrollConfig::default())
}

#[test]
fn isolated_scroll_content_preserves_geometry_clipping_and_extents_across_resizes() {
    let mut ui = UiTree::new(Element::row([panel(content())]).padding(Sides::length(8.0)));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    for width in [400.0, 301.5, 800.0] {
        let viewport = Size::new(width, 300.0);
        let actual = engine.compute(&mut ui, &mut text, viewport).unwrap();
        let mut reference = ui.clone();
        let mut root = reference.root().clone();
        root.children[0].layout_boundary = false;
        reference.update(root);
        let expected = LayoutEngine::new()
            .compute(&mut reference, &mut text, viewport)
            .unwrap();
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.scroll_regions, expected.scroll_regions);
        assert_eq!(actual.display_list, expected.display_list);
        assert!(actual.scroll_regions[0].max_offset.y > 900.0);
    }
}

#[derive(Debug)]
struct Parent;

#[test]
fn cached_content_rounds_again_when_its_ancestors_move_fractionally() {
    let mut ui = UiTree::new(
        Element::row([
            Element::container([]).width(length(12.3)).shrink(0.0),
            panel(content()).shrink(0.0),
        ])
        .padding(Sides::length(2.4)),
    );
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    for width in [12.3, 12.6, 12.6, 15.2] {
        let mut root = ui.root().clone();
        root.children[0].style.size.width = length(width);
        ui.update(root);
        let actual = engine
            .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        let mut reference = ui.clone();
        let mut root = reference.root().clone();
        root.children[1].layout_boundary = false;
        reference.update(root);
        let expected = LayoutEngine::new()
            .compute(&mut reference, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.scroll_regions, expected.scroll_regions);
        assert_eq!(actual.display_list, expected.display_list);
    }
}

#[test]
fn isolated_portals_restore_styles_after_changing_targets() {
    use argui_ui::{Portal, PortalTarget, ViewportPlacement, WindowLayer};
    let mut ui = UiTree::new(Element::column([
        panel(content()).viewport_portal(WindowLayer::Popover, ViewportPlacement::fill())
    ]));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    for step in 0..6 {
        let mut root = ui.root().clone();
        if step % 3 == 1 {
            root.children[0].portal = Some(Portal::new(WindowLayer::Popover, PortalTarget::Layout));
        } else if step % 3 == 2 {
            root.children[0].portal = None;
        } else {
            root.children[0] = root.children[0]
                .clone()
                .viewport_portal(WindowLayer::Popover, ViewportPlacement::fill());
        }
        root.style.display = if step == 3 {
            Display::None
        } else {
            Display::Flex
        };
        ui.update(root);
        let actual = engine
            .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        let expected = LayoutEngine::new()
            .compute(&mut ui, &mut text, Size::new(400.0, 300.0))
            .unwrap();
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.scroll_regions, expected.scroll_regions);
        assert_eq!(actual.portals, expected.portals);
        if step % 3 != 0 {
            assert_eq!(actual.nodes[1].bounds.size, Size::new(120.0, 80.0));
        }
    }
}

impl CustomElement for Parent {
    type State = ();
    fn create_state(&self) {}
    fn layout_revision(&self) -> u64 {
        0
    }
    fn paint_revision(&self) -> u64 {
        0
    }
    fn prepare(&self, _: &mut (), _: Size) {}
    fn paint(&self, _: &mut (), _: &mut CustomPaintContext<'_>) {}
    fn layout(
        &self,
        _: &mut (),
        cx: &mut dyn CustomLayoutContext,
    ) -> Result<CustomMeasurement, String> {
        let child = cx.measure_child(
            0,
            CustomConstraints {
                width: cx.available().width,
                height: None,
            },
        )?;
        cx.place_child(0, Rect::new(Point::default(), child.size))?;
        Ok(child)
    }
}

#[test]
fn content_changes_do_not_remeasure_ancestors_but_external_size_changes_do() {
    let root = Element::custom_container(Parent, [panel(content())]);
    let mut ui = UiTree::new(root);
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(400.0, 300.0);
    engine.compute(&mut ui, &mut text, viewport).unwrap();
    let before = engine.custom_stats()[0].phases.layouts;
    let mut root = ui.root().clone();
    root.children[0].children[0].children[1].style.size.height = length(1500.0);
    ui.update(root);
    let output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(engine.custom_stats()[0].phases.layouts, before);
    assert_eq!(output.nodes[4].bounds.size.height, 1500.0);
    assert!(output.scroll_regions[0].max_offset.y > 1400.0);
    let mut root = ui.root().clone();
    root.children[0].style.size.width = length(160.0);
    ui.update(root);
    engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert!(engine.custom_stats()[0].phases.layouts > before);
}

#[test]
fn nested_roots_follow_hidden_content_boundary_toggles_and_removal() {
    let mut ui = UiTree::new(panel(Element::column([panel(content())])));
    let mut engine = LayoutEngine::new();
    let mut text = TextEngine::from_embedded_fonts([], "sans", "sans", "mono");
    let viewport = Size::new(400.0, 300.0);
    assert_eq!(ui.layout_root_indices(), [0, 2]);
    for step in 0..8 {
        let mut root = ui.root().clone();
        root.style.display = if step % 2 == 0 {
            Display::None
        } else {
            Display::Flex
        };
        root.layout_boundary = step % 3 != 0;
        root.children[0].children[0].children[0].children[1]
            .style
            .size
            .height = length(1100.0 + step as f32);
        ui.update(root);
        let actual = engine.compute(&mut ui, &mut text, viewport).unwrap();
        let expected = LayoutEngine::new()
            .compute(&mut ui, &mut text, viewport)
            .unwrap();
        assert_eq!(actual.nodes, expected.nodes);
        assert_eq!(actual.scroll_regions, expected.scroll_regions);
        if step % 2 == 0 {
            assert!(
                actual
                    .nodes
                    .iter()
                    .all(|n| n.bounds.size == Size::default())
            );
        }
        assert_eq!(
            engine.retained_node_count(),
            ui.node_ids().len() + ui.layout_root_indices().len()
        );
    }
    ui.update(Element::container([]));
    engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(engine.retained_node_count(), 1);
    assert!(ui.layout_root_indices().is_empty());
}

#[test]
fn invalid_boundaries_report_the_contract_instead_of_using_stale_geometry() {
    let mut too_many = panel(content());
    too_many.children.push(content());
    let mut wrong_kind = panel(content());
    wrong_kind.kind = Element::text("invalid").kind.clone();
    for root in [
        too_many,
        wrong_kind,
        panel(content()).overflow(Axes {
            x: Overflow::Visible,
            y: Overflow::Hidden,
        }),
        panel(content()).overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Visible,
        }),
    ] {
        let error = LayoutEngine::new()
            .compute(
                &mut UiTree::new(root),
                &mut TextEngine::from_embedded_fonts([], "sans", "sans", "mono"),
                Size::new(400.0, 300.0),
            )
            .unwrap_err();
        assert!(error.to_string().contains("layout boundary:"));
        assert!(std::error::Error::source(&error).is_none());
    }
}
