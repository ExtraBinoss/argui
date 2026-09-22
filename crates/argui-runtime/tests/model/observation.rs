use argui_core::{Affine2D, Point, PointerButton, PointerEvent, PointerPhase, Rect, Size};
use argui_paint::ClipChain;
use argui_runtime::{ObservationReader, ObservedInteraction, SourceIdentityIndex};
use argui_ui::{
    CursorIcon, Element, FocusPolicy, GestureSet, HitRegion, HitShape, RetainedIdentity,
    ScrollConfig, ScrollRegion, Sides, UiTree, VisualState,
};
use std::collections::HashSet;

fn region(node: argui_ui::NodeId) -> HitRegion {
    HitRegion {
        node,
        bounds: Rect::new(Point::new(10.0, 20.0), Size::new(100.0, 100.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        },
        enabled: true,
        focus_policy: FocusPolicy::None,
        cursor: CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }
}

#[test]
fn pointer_motion_changes_observed_local_coordinates_without_paint_invalidation() {
    let identity = RetainedIdentity::new(4, 9);
    let mut tree = UiTree::new(Element::container([]).retained_identity(identity));
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, Point::new(15.0, 25.0)),
        &regions,
    );
    let before = ObservedInteraction::from_node(&tree, node, &regions, None);
    assert!(before.states.contains(VisualState::Hovered));
    assert_eq!(before.pointer_position, Some(Point::new(5.0, 5.0)));

    let update = tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, Point::new(30.0, 45.0)),
        &regions,
    );
    let after = ObservedInteraction::from_node(&tree, node, &regions, None);
    assert!(!update.paint_changed);
    assert_eq!(after.pointer_position, Some(Point::new(20.0, 25.0)));
    assert_ne!(before, after);
}

#[test]
fn observation_reader_defaults_to_idle_and_press_origin_survives_release() {
    let identity = RetainedIdentity::new(1, 2);
    assert_eq!(
        ObservationReader::default().get(&identity),
        ObservedInteraction::default()
    );

    let mut tree = UiTree::new(Element::container([]).retained_identity(identity));
    let node = tree.node_ids()[0];
    let regions = [region(node)];
    let press = PointerEvent {
        button: Some(PointerButton::Primary),
        buttons: 1,
        ..PointerEvent::mouse(PointerPhase::Pressed, Point::new(34.0, 54.0))
    };
    tree.pointer_event(press, &regions);
    let active = ObservedInteraction::from_node(&tree, node, &regions, None);
    assert!(active.states.contains(VisualState::Pressed));
    assert_eq!(active.pressed_position, Some(Point::new(24.0, 34.0)));

    tree.pointer_event(
        PointerEvent {
            button: Some(PointerButton::Primary),
            ..PointerEvent::mouse(PointerPhase::Released, Point::new(34.0, 54.0))
        },
        &regions,
    );
    let released = ObservedInteraction::from_node(&tree, node, &regions, None);
    assert!(!released.states.contains(VisualState::Pressed));
    assert_eq!(released.pressed_position, active.pressed_position);
}

#[test]
fn source_index_samples_only_watched_nodes_and_skips_empty_watches() {
    let children = (0..512)
        .map(|site| Element::container([]).retained_identity(RetainedIdentity::new(3, site)))
        .collect::<Vec<_>>();
    let mut tree = UiTree::new(Element::container(children));
    let watched_identity = RetainedIdentity::new(3, 251);
    let watched_node = tree.node_ids()[252];
    let regions = [region(watched_node)];
    let mut index = SourceIdentityIndex::default();

    let empty = index.observations(Some(&tree), &regions, &[], None, &HashSet::new());
    assert!(empty.is_empty());
    assert_eq!(index.indexed_len(), 0);

    let watched = HashSet::from([watched_identity.clone()]);
    tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, Point::new(15.0, 25.0)),
        &regions,
    );
    let sampled = index.observations(Some(&tree), &regions, &[], None, &watched);
    assert_eq!(sampled.len(), 1);
    assert_eq!(index.indexed_len(), 512);
    assert_eq!(
        sampled[&watched_identity].pointer_position,
        Some(Point::new(5.0, 5.0))
    );

    tree.pointer_event(
        PointerEvent::mouse(PointerPhase::Moved, Point::new(25.0, 35.0)),
        &regions,
    );
    let sampled_again = index.observations(Some(&tree), &regions, &[], None, &watched);
    assert_eq!(sampled_again.len(), 1);
    assert_eq!(
        sampled_again[&watched_identity].pointer_position,
        Some(Point::new(15.0, 15.0))
    );
    assert_eq!(index.indexed_len(), 512);

    tree.update(Element::container([]));
    assert!(
        index
            .observations(Some(&tree), &[], &[], None, &watched)
            .is_empty()
    );
    assert_eq!(index.indexed_len(), 0);
    assert!(
        index
            .observations(None, &[], &[], None, &watched)
            .is_empty()
    );
}

#[test]
fn scroll_observation_is_bounded_and_changes_with_retained_offset() {
    let identity = RetainedIdentity::new(5, 6);
    let mut tree = UiTree::new(Element::container([]).retained_identity(identity.clone()));
    let node = tree.node_ids()[0];
    let scroll = [ScrollRegion {
        node,
        bounds: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0)),
        clip: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 50.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        max_offset: Point::new(200.0, 300.0),
        config: ScrollConfig::default(),
        scrollbar: None,
        interaction_order: 0,
    }];
    let watched = HashSet::from([identity.clone()]);
    let mut index = SourceIdentityIndex::default();
    let initial = index.observations(Some(&tree), &[], &scroll, None, &watched);
    let geometry = initial[&identity].scroll.unwrap();
    assert_eq!(geometry.offset, Point::new(0.0, 0.0));
    assert_eq!(geometry.viewport, Size::new(100.0, 50.0));
    assert_eq!(geometry.content, Size::new(300.0, 350.0));

    tree.set_scroll_offset(node, Point::new(35.0, 120.0));
    let moved = index.observations(Some(&tree), &[], &scroll, None, &watched);
    assert_eq!(
        moved[&identity].scroll.unwrap().offset,
        Point::new(35.0, 120.0)
    );
    assert_ne!(initial, moved);

    tree.set_scroll_offset(node, Point::new(500.0, -20.0));
    let bounded = index.observations(Some(&tree), &[], &scroll, None, &watched);
    assert_eq!(
        bounded[&identity].scroll.unwrap().offset,
        Point::new(200.0, 0.0)
    );
    assert!(
        index.observations(Some(&tree), &[], &[], None, &watched)[&identity]
            .scroll
            .is_none()
    );
}

/// Invalid layout extents cannot leak non-finite dimensions into observations.
#[test]
fn scroll_observation_normalizes_non_finite_and_negative_extents() {
    let identity = RetainedIdentity::new(8, 9);
    let mut tree = UiTree::new(Element::container([]).retained_identity(identity.clone()));
    let node = tree.node_ids()[0];
    tree.set_scroll_offset(node, Point::new(f32::INFINITY, f32::INFINITY));
    let scroll = [ScrollRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(f32::NAN, -20.0)),
        clip: Rect::new(Point::default(), Size::new(10.0, 10.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        max_offset: Point::new(f32::INFINITY, -10.0),
        config: ScrollConfig::default(),
        scrollbar: None,
        interaction_order: 0,
    }];
    let watched = HashSet::from([identity.clone()]);
    let observed =
        SourceIdentityIndex::default().observations(Some(&tree), &[], &scroll, None, &watched);
    let geometry = observed[&identity].scroll.unwrap();
    assert_eq!(geometry.offset, Point::default());
    assert_eq!(geometry.viewport, Size::new(0.0, 0.0));
    assert_eq!(geometry.content, Size::new(0.0, 0.0));
}
