use argui_core::{Affine2D, Point, Rect, Size};
use argui_layout::{LayoutNode, LayoutOutput, PortalLayout};
use argui_paint::ClipChain;
use argui_ui::{Element, RetainedIdentity, ScrollConfig, ScrollRegion, UiTree, WindowLayer};

#[path = "../../../src/app/scroll/request.rs"]
mod implementation;

use implementation::scroll_tracks;

fn region(node: argui_ui::NodeId, y: f32) -> ScrollRegion {
    ScrollRegion {
        node,
        bounds: Rect::new(Point::new(0.0, y), Size::new(100.0, 100.0)),
        clip: Rect::new(Point::new(0.0, y), Size::new(100.0, 100.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        max_offset: Point::new(0.0, 300.0),
        config: ScrollConfig::default(),
        scrollbar: None,
        interaction_order: 0,
    }
}

/// Synthetic layout fixtures isolate target resolution from a native renderer.
fn tree_and_layout() -> (UiTree, LayoutOutput) {
    let tree = UiTree::new(Element::container(
        [Element::text("target").keyed("target")],
    ));
    let nodes = tree.node_ids();
    let mut layout = LayoutOutput::default();
    layout.nodes.push(LayoutNode {
        index: 1,
        node: nodes[1],
        bounds: Rect::new(Point::new(0.0, 240.0), Size::new(20.0, 20.0)),
        layout_bounds: Rect::default(),
        clip: None,
        text_index: None,
    });
    layout.scroll_regions = vec![region(nodes[1], 100.0), region(nodes[0], 0.0)];
    (tree, layout)
}

#[test]
fn scroll_targets_ignore_missing_regions_and_unlaid_elements() {
    use argui_ui::ScrollRequest;
    let (tree, mut layout) = tree_and_layout();
    assert!(
        scroll_tracks(
            &tree,
            &layout,
            &ScrollRequest::offset("missing", Point::new(0.0, 30.0))
        )
        .is_empty()
    );
    assert!(
        scroll_tracks(
            &tree,
            &layout,
            &ScrollRequest::rect("missing", Rect::default())
        )
        .is_empty()
    );
    assert!(scroll_tracks(&tree, &layout, &ScrollRequest::reveal("missing")).is_empty());
    layout.nodes.clear();
    assert!(scroll_tracks(&tree, &layout, &ScrollRequest::reveal("target")).is_empty());
}

#[test]
fn element_reveal_visits_scroll_ancestors_but_stops_at_a_portal() {
    use argui_ui::ScrollRequest;
    let (tree, mut layout) = tree_and_layout();
    let tracks = scroll_tracks(&tree, &layout, &ScrollRequest::reveal("target"));
    assert_eq!(tracks.len(), 2);
    assert_eq!(tracks[0].node, tree.node_ids()[1]);
    assert_eq!(tracks[1].node, tree.node_ids()[0]);

    layout.portals.push(PortalLayout {
        node: tree.node_ids()[1],
        layer: WindowLayer::Popover,
        anchor: None,
        requested: None,
        resolved: None,
        bounds: Rect::default(),
        desired_size: Size::default(),
        available_size: Size::default(),
        constrained_width: false,
        constrained_height: false,
    });
    let tracks = scroll_tracks(&tree, &layout, &ScrollRequest::reveal("target"));
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].node, tree.node_ids()[1]);
}

#[test]
fn explicit_offset_and_rectangle_requests_keep_the_selected_region() {
    use argui_ui::{ScrollAlignment, ScrollRequest};
    let (tree, layout) = tree_and_layout();
    let offset = scroll_tracks(
        &tree,
        &layout,
        &ScrollRequest::offset("target", Point::new(-20.0, 1_000.0)),
    );
    assert_eq!(offset.len(), 1);
    assert_eq!(offset[0].node, tree.node_ids()[1]);
    assert_eq!(offset[0].from, Point::default());
    assert_eq!(offset[0].to, Point::new(0.0, 300.0));

    let rect = scroll_tracks(
        &tree,
        &layout,
        &ScrollRequest::rect(
            "target",
            Rect::new(Point::new(0.0, 240.0), Size::new(20.0, 20.0)),
        )
        .align(ScrollAlignment::Start, ScrollAlignment::Start),
    );
    assert_eq!(rect.len(), 1);
    assert_eq!(rect[0].node, tree.node_ids()[1]);
    assert_eq!(rect[0].to.y, 140.0);
}

#[test]
fn retained_identity_selects_one_of_two_same_key_scroll_views() {
    use argui_ui::ScrollRequest;

    let first = RetainedIdentity::new(100, 9);
    let second = RetainedIdentity::new(101, 9);
    let tree = UiTree::new(Element::container([
        Element::container([])
            .keyed("viewport")
            .retained_identity(first.clone()),
        Element::container([])
            .keyed("viewport")
            .retained_identity(second.clone()),
    ]));
    let mut layout = LayoutOutput::default();
    layout.scroll_regions = vec![
        region(tree.node_ids()[1], 0.0),
        region(tree.node_ids()[2], 0.0),
    ];

    let tracks = scroll_tracks(
        &tree,
        &layout,
        &ScrollRequest::offset(second, Point::new(0.0, 70.0)),
    );
    assert_eq!(tracks.len(), 1);
    assert_eq!(tracks[0].node, tree.node_ids()[2]);
    assert_eq!(tracks[0].to, Point::new(0.0, 70.0));
    assert!(
        scroll_tracks(
            &tree,
            &layout,
            &ScrollRequest::offset(RetainedIdentity::new(999, 9), Point::new(0.0, 70.0)),
        )
        .is_empty()
    );
}

/// Checks every axis alignment and both out-of-viewport `Nearest` cases.
#[test]
fn axis_alignment_covers_explicit_modes_and_nearest_edges() {
    use argui_ui::ScrollAlignment;

    let aligned = |start, end, alignment| {
        implementation::align_axis(10.0, 100.0, 200.0, start, end, alignment)
    };
    assert_eq!(aligned(130.0, 150.0, ScrollAlignment::Start), 40.0);
    assert_eq!(aligned(130.0, 150.0, ScrollAlignment::Center), 0.0);
    assert_eq!(aligned(130.0, 150.0, ScrollAlignment::End), -40.0);
    assert_eq!(aligned(130.0, 150.0, ScrollAlignment::Nearest), 10.0);
    assert_eq!(aligned(80.0, 90.0, ScrollAlignment::Nearest), -10.0);
    assert_eq!(aligned(220.0, 230.0, ScrollAlignment::Nearest), 40.0);
}
