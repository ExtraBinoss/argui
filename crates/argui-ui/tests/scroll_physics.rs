use argui_core::{Affine2D, Point, Rect, ScrollDelta, Size};
use argui_paint::{ClipChain, ClipRegion};
use argui_ui::{
    Color, Element, QuadStyle, ScrollConfig, ScrollPropagation, ScrollRegion, ScrollbarGeometry,
    ScrollbarPartStyle, ScrollbarRegion, ScrollbarStyle, ScrollbarVisibility, UiTree,
};

fn region(node: argui_ui::NodeId, config: ScrollConfig, max_y: f32) -> ScrollRegion {
    let bounds = Rect::new(Point::default(), Size::new(200.0, 200.0));
    ScrollRegion {
        node,
        bounds,
        clip: bounds,
        transform: Affine2D::IDENTITY,
        clips: ClipChain::from_regions([ClipRegion::new(bounds, Affine2D::IDENTITY)]),
        max_offset: Point::new(0.0, max_y),
        config,
        scrollbar: None,
        interaction_order: 0,
    }
}

fn vertical_scrollbar(track: Rect, thumb: Rect, style: ScrollbarStyle) -> ScrollbarRegion {
    ScrollbarRegion {
        horizontal: None,
        vertical: Some(ScrollbarGeometry { track, thumb }),
        style,
    }
}

#[test]
fn chained_elastic_children_pass_their_excess_to_an_ancestor() {
    let mut tree = UiTree::new(Element::column([
        Element::container([]),
        Element::container([]),
    ]));
    let parent = tree.node_id_at(1).unwrap();
    let child = tree.node_id_at(2).unwrap();
    let regions = [
        region(parent, ScrollConfig::default(), 100.0),
        region(
            child,
            ScrollConfig::default()
                .overscroll(argui_ui::OverscrollBehavior::Elastic(Default::default())),
            0.0,
        ),
    ];
    tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -30.0)),
        &regions,
    );
    assert_eq!(tree.scroll_offset(child), Point::default());
    assert_eq!(tree.scroll_offset(parent), Point::new(0.0, 30.0));
}

#[test]
fn elastic_overscroll_settles_without_changing_the_clamped_offset() {
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let config = ScrollConfig::default()
        .overscroll(argui_ui::OverscrollBehavior::Elastic(Default::default()));
    let regions = [region(node, config, 0.0)];
    let update = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -50.0)),
        &regions,
    );
    let displaced = tree.scroll_offset(node).y;
    assert!(update.scroll_changed);
    assert!(displaced > 0.0);
    assert!(tree.wants_scroll_frame());
    assert!(
        tree.advance_scroll_physics(1.0 / 60.0, &regions)
            .scroll_changed
    );
    assert!(tree.scroll_offset(node).y < displaced);
}

#[test]
fn none_propagation_suppresses_elastic_edges() {
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let config = ScrollConfig::default()
        .propagation(ScrollPropagation::None)
        .overscroll(argui_ui::OverscrollBehavior::Elastic(Default::default()));
    let update = tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -50.0)),
        &[region(node, config, 0.0)],
    );
    assert!(!update.scroll_changed);
    assert_eq!(tree.scroll_offset(node), Point::default());
    assert!(!tree.wants_scroll_frame());
}

#[test]
fn invalid_elastic_values_and_elapsed_time_never_escape_as_nan() {
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let config = ScrollConfig::default().overscroll(argui_ui::OverscrollBehavior::Elastic(
        argui_ui::ElasticScroll {
            resistance: f32::NAN,
            limit: f32::NAN,
            spring: f32::NAN,
            damping: f32::NAN,
        },
    ));
    let regions = [region(node, config, 0.0)];
    tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -30.0)),
        &regions,
    );
    tree.advance_scroll_physics(f32::NAN, &regions);
    assert!(tree.scroll_offset(node).y.is_finite());
}

#[test]
fn automatic_scrollbars_show_on_activity_then_fade_to_idle() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    )
    .hide_delay(std::time::Duration::ZERO)
    .fade_duration(std::time::Duration::from_millis(100));
    let config = ScrollConfig::default().scrollbar(style);
    let mut tree = UiTree::new(Element::container([]).scroll_config(config.clone()));
    let node = tree.node_id_at(0).unwrap();
    let regions = [region(node, config.clone(), 100.0)];
    tree.scroll(
        Point::new(10.0, 10.0),
        ScrollDelta::Pixels(Point::new(0.0, -10.0)),
        &regions,
    );
    let opacity = |tree: &UiTree| {
        tree.resolved_scroll_config(node, &config)
            .scrollbar
            .unwrap()
            .thumb
            .base
            .opacity
    };
    assert_eq!(opacity(&tree), 1.0);
    tree.advance_scroll_physics(0.05, &regions);
    assert!(opacity(&tree) > 0.0 && opacity(&tree) < 1.0);
    tree.advance_scroll_physics(0.05, &regions);
    assert!(!tree.wants_scroll_frame());
}

#[test]
fn scrollbar_visibility_is_resolved_before_paint() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    );
    let tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let opacity = |visibility| {
        let config = ScrollConfig::default().scrollbar(style.clone().visibility(visibility));
        tree.resolved_scroll_config(node, &config)
            .scrollbar
            .unwrap()
            .thumb
            .base
            .opacity
    };
    assert_eq!(opacity(ScrollbarVisibility::Always), 1.0);
    assert_eq!(opacity(ScrollbarVisibility::Auto), 0.0);
    assert_eq!(opacity(ScrollbarVisibility::Hidden), 0.0);
}

#[test]
fn hovered_auto_scrollbars_repaint_without_running_idle_frames() {
    let style = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::default()),
        ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
    );
    let config = ScrollConfig::default().scrollbar(style.clone());
    let mut tree = UiTree::new(Element::container([]).scroll_config(config.clone()));
    let node = tree.node_id_at(0).unwrap();
    let mut region = region(node, config, 100.0);
    region.scrollbar = Some(vertical_scrollbar(
        region.bounds,
        Rect::new(Point::default(), Size::new(200.0, 40.0)),
        style,
    ));

    let regions = [region];
    assert!(
        tree.scrollbar_pointer_moved(Some(Point::new(20.0, 20.0)), &regions)
            .paint_changed
    );
    assert!(!tree.wants_scroll_frame());
    tree.advance_scroll_physics(0.1, &regions);
    assert!(tree.scrollbar_pointer_moved(None, &[]).paint_changed);
    assert!(tree.wants_scroll_frame());
}

#[test]
fn scrollbar_activity_reuses_entries_and_discards_stale_configuration() {
    let parts = || {
        ScrollbarStyle::new(
            ScrollbarPartStyle::new(QuadStyle::default()),
            ScrollbarPartStyle::new(QuadStyle::solid(Color::WHITE)),
        )
    };
    let mut tree = UiTree::new(Element::container([]));
    let node = tree.node_id_at(0).unwrap();
    let always = ScrollConfig::default().scrollbar(
        parts()
            .visibility(ScrollbarVisibility::Always)
            .fade_duration(std::time::Duration::ZERO),
    );
    tree.activate_scrollbar(node, &[region(node, always, 100.0)]);
    assert!(!tree.wants_scroll_frame());

    let automatic = ScrollConfig::default().scrollbar(
        parts()
            .hide_delay(std::time::Duration::ZERO)
            .fade_duration(std::time::Duration::ZERO),
    );
    let regions = [region(node, automatic, 100.0)];
    tree.activate_scrollbar(node, &regions);
    tree.activate_scrollbar(node, &regions);
    assert!(tree.wants_scroll_frame());
    assert!(
        !tree
            .advance_scroll_physics(f32::NAN, &regions)
            .paint_changed
    );
    assert!(tree.advance_scroll_physics(0.01, &regions).paint_changed);
    assert!(!tree.wants_scroll_frame());

    tree.activate_scrollbar(node, &regions);
    tree.advance_scroll_physics(0.0, &[region(node, ScrollConfig::default(), 100.0)]);
    assert!(!tree.wants_scroll_frame());
}
