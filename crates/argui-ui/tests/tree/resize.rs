use argui_core::{
    Affine2D, Point, PointerButton, PointerEvent, PointerId, PointerPhase, Rect, Size,
};
use argui_paint::ClipChain;
use argui_ui::{
    Element, EventHandlerId, EventListener, EventOwnerId, EventType, HitRegion, HitShape,
    Interaction, NodeId, ResizeAxis, ResizeHandle, Sides, UiEventKind, UiTree, length,
};

fn root(axis: ResizeAxis, trailing: bool) -> Element {
    let mut interaction = Interaction::default().capture_on_press(true);
    interaction.resize = Some(ResizeHandle {
        target: "pane".into(),
        axis,
        trailing,
        minimum: 100.,
        maximum: 400.,
    });
    Element::row([
        Element::container([])
            .keyed("pane")
            .width(length(200.))
            .height(length(200.)),
        Element::container([])
            .keyed("handle")
            .width(length(8.))
            .height(length(200.))
            .interaction(interaction)
            .on(EventListener::new(
                EventType::ResizeCommit,
                EventHandlerId::new(EventOwnerId(1), 1),
            )),
    ])
}

fn prepare(axis: ResizeAxis, trailing: bool) -> (UiTree, NodeId, HitRegion) {
    let mut tree = UiTree::new(root(axis, trailing));
    let pane = tree.node_ids()[1];
    let handle = tree.node_ids()[2];
    tree.observe_resize_bounds([(pane, Size::new(200., 200.))]);
    let region = HitRegion {
        node: handle,
        bounds: Rect::new(Point::new(200., 0.), Size::new(8., 200.)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focus_policy: argui_ui::FocusPolicy::None,
        cursor: argui_ui::CursorIcon::EwResize,
        gestures: argui_ui::GestureSet::EMPTY,
        window_drag: None,
    };
    (tree, pane, region)
}

fn event(phase: PointerPhase, x: f32, y: f32) -> PointerEvent {
    PointerEvent {
        button: matches!(phase, PointerPhase::Pressed | PointerPhase::Released)
            .then_some(PointerButton::Primary),
        buttons: u16::from(matches!(phase, PointerPhase::Pressed | PointerPhase::Moved)),
        ..PointerEvent::mouse(phase, Point::new(x, y))
    }
}

fn value(tree: &UiTree, pane: NodeId, axis: ResizeAxis) -> argui_ui::Dimension {
    let style = tree.resolved_layout_style(pane, tree.element_for(pane).unwrap());
    match axis {
        ResizeAxis::Horizontal => style.size.width,
        ResizeAxis::Vertical => style.size.height,
    }
}

#[test]
fn native_resize_has_no_move_callbacks_and_one_final_clamped_commit() {
    for axis in [ResizeAxis::Horizontal, ResizeAxis::Vertical] {
        for trailing in [false, true] {
            let (mut tree, pane, region) = prepare(axis, trailing);
            tree.pointer_event(
                event(PointerPhase::Pressed, 204., 40.),
                std::slice::from_ref(&region),
            );
            let origin = if axis == ResizeAxis::Horizontal {
                204.
            } else {
                40.
            };
            for sample in 1..=1000 {
                let delta = sample as f32 * if trailing { -1. } else { 1. };
                let (x, y) = if axis == ResizeAxis::Horizontal {
                    (origin + delta, 40.)
                } else {
                    (204., origin + delta)
                };
                let update = tree.pointer_event(
                    event(PointerPhase::Moved, x, y),
                    std::slice::from_ref(&region),
                );
                assert!(
                    update.events.is_empty(),
                    "resize moves must never reach JavaScript"
                );
            }
            assert_eq!(value(&tree, pane, axis), length(400.));
            let final_at = origin + if trailing { -75. } else { 75. };
            let (x, y) = if axis == ResizeAxis::Horizontal {
                (final_at, 40.)
            } else {
                (204., final_at)
            };
            let update = tree.pointer_event(
                event(PointerPhase::Released, x, y),
                std::slice::from_ref(&region),
            );
            assert_eq!(update.events.len(), 1);
            assert_eq!(
                update.events[0].kind,
                UiEventKind::ResizeCommitted { value: 275. }
            );
            assert_eq!(value(&tree, pane, axis), length(275.));
            assert!(!tree.pointer_captured(PointerId::MOUSE));
        }
    }
}

#[test]
fn native_resize_cancellation_restores_and_never_commits() {
    let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
    tree.pointer_event(
        event(PointerPhase::Pressed, 204., 40.),
        std::slice::from_ref(&region),
    );
    tree.pointer_event(
        event(PointerPhase::Moved, 304., 40.),
        std::slice::from_ref(&region),
    );
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(300.));
    let update = tree.pointer_event(
        event(PointerPhase::Cancelled, 304., 40.),
        std::slice::from_ref(&region),
    );
    assert!(update.layout_changed && update.events.is_empty());
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(200.));
    assert_eq!(tree.layout_animation_indices(), vec![1]);
    tree.mark_layout_clean();
    assert!(tree.layout_animation_indices().is_empty());
    assert!(!tree.pointer_captured(PointerId::MOUSE));
}

#[test]
fn native_resize_preserves_an_unacknowledged_commit_on_next_cancellation() {
    let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
    tree.pointer_event(
        event(PointerPhase::Pressed, 204., 40.),
        std::slice::from_ref(&region),
    );
    tree.pointer_event(
        event(PointerPhase::Released, 264., 40.),
        std::slice::from_ref(&region),
    );
    tree.observe_resize_bounds([(pane, Size::new(260., 200.))]);
    tree.pointer_event(
        event(PointerPhase::Pressed, 204., 40.),
        std::slice::from_ref(&region),
    );
    tree.pointer_event(
        event(PointerPhase::Moved, 304., 40.),
        std::slice::from_ref(&region),
    );
    tree.pointer_event(
        event(PointerPhase::Cancelled, 304., 40.),
        std::slice::from_ref(&region),
    );
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(260.));
}

#[test]
fn native_resize_ignores_other_contacts_secondary_releases_and_nonfinite_positions() {
    let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
    tree.pointer_event(
        event(PointerPhase::Pressed, 204., 40.),
        std::slice::from_ref(&region),
    );
    for mut sample in [
        event(PointerPhase::Moved, 304., 40.),
        event(PointerPhase::Released, 304., 40.),
    ] {
        sample.id = PointerId::new(42);
        assert!(
            tree.pointer_event(sample, std::slice::from_ref(&region))
                .events
                .is_empty()
        );
    }
    let mut secondary = event(PointerPhase::Released, 304., 40.);
    secondary.button = Some(PointerButton::Secondary);
    tree.pointer_event(secondary, std::slice::from_ref(&region));
    tree.pointer_event(
        event(PointerPhase::Moved, f32::NAN, 40.),
        std::slice::from_ref(&region),
    );
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(200.));
    assert!(
        tree.pointer_event(
            event(PointerPhase::Released, 254., 40.),
            std::slice::from_ref(&region)
        )
        .events
        .iter()
        .any(|event| event.kind == UiEventKind::ResizeCommitted { value: 250. })
    );
}

#[test]
fn native_resize_survives_unrelated_tree_updates_and_authored_size_acknowledges_it() {
    let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
    tree.pointer_event(
        event(PointerPhase::Pressed, 204., 40.),
        std::slice::from_ref(&region),
    );
    tree.pointer_event(
        event(PointerPhase::Moved, 254., 40.),
        std::slice::from_ref(&region),
    );
    let mut updated = root(ResizeAxis::Horizontal, false);
    updated.children[0].tooltip = Some("updated media".into());
    tree.update(updated.clone());
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(250.));
    updated.children[0].style.size.width = length(250.);
    tree.update(updated);
    assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(250.));
    tree.mark_layout_clean();
    assert!(tree.layout_animation_indices().is_empty());
}

#[test]
fn native_resize_disabling_or_removing_a_handle_restores_authored_geometry() {
    for remove in [false, true] {
        let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
        tree.pointer_event(
            event(PointerPhase::Pressed, 204., 40.),
            std::slice::from_ref(&region),
        );
        tree.pointer_event(
            event(PointerPhase::Moved, 254., 40.),
            std::slice::from_ref(&region),
        );
        let mut updated = root(ResizeAxis::Horizontal, false);
        if remove {
            updated.children.pop();
        } else {
            updated.children[1].interaction.as_mut().unwrap().enabled = false;
        }
        assert_eq!(tree.update(updated), argui_ui::TreeUpdate::Layout);
        assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(200.));
        assert!(
            tree.pointer_event(event(PointerPhase::Released, 304., 40.), &[])
                .events
                .is_empty()
        );
    }
}

#[test]
fn native_resize_interruptions_restore_size_and_allow_a_fresh_gesture() {
    for interruption in 0..4 {
        let (mut tree, pane, region) = prepare(ResizeAxis::Horizontal, false);
        tree.pointer_event(
            event(PointerPhase::Pressed, 204., 40.),
            std::slice::from_ref(&region),
        );
        tree.pointer_event(
            event(PointerPhase::Moved, 304., 40.),
            std::slice::from_ref(&region),
        );
        let mut authored = root(ResizeAxis::Horizontal, false);
        if interruption == 0 {
            let update = tree.pointer_event(
                event(PointerPhase::Released, f32::NAN, 40.),
                std::slice::from_ref(&region),
            );
            assert!(update.layout_changed && update.events.is_empty());
        } else {
            match interruption {
                1 => {
                    authored.children[1]
                        .interaction
                        .as_mut()
                        .unwrap()
                        .resize
                        .as_mut()
                        .unwrap()
                        .axis = ResizeAxis::Vertical
                }
                2 => {
                    authored.children[1]
                        .interaction
                        .as_mut()
                        .unwrap()
                        .resize
                        .as_mut()
                        .unwrap()
                        .maximum = 350.
                }
                _ => authored.children[0].style.size.width = length(240.),
            }
            assert_eq!(tree.update(authored), argui_ui::TreeUpdate::Layout);
            assert!(
                tree.pointer_event(
                    event(PointerPhase::Released, 354., 40.),
                    std::slice::from_ref(&region)
                )
                .events
                .is_empty()
            );
        }
        let width = if interruption == 3 { 240. } else { 200. };
        assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(width));
        assert_eq!(value(&tree, pane, ResizeAxis::Vertical), length(200.));
        tree.observe_resize_bounds([(pane, Size::new(width, 200.))]);
        tree.pointer_event(
            event(PointerPhase::Pressed, 204., 40.),
            std::slice::from_ref(&region),
        );
        let at = if interruption == 1 {
            (204., 90.)
        } else {
            (254., 40.)
        };
        let committed = tree.pointer_event(
            event(PointerPhase::Released, at.0, at.1),
            std::slice::from_ref(&region),
        );
        assert_eq!(committed.events.len(), 1);
        assert_eq!(
            committed.events[0].kind,
            UiEventKind::ResizeCommitted {
                value: if interruption == 1 { 250. } else { width + 50. }
            }
        );
    }
}

#[test]
fn native_resize_rejects_missing_duplicate_unmeasured_and_invalid_targets() {
    for invalid in 0..5 {
        let mut root = root(ResizeAxis::Horizontal, false);
        match invalid {
            0 => root.children[0].key = Some("other".into()),
            1 => {
                let duplicate = root.children[0].clone();
                root.children.push(duplicate);
            }
            2 => {
                root.children[1]
                    .interaction
                    .as_mut()
                    .unwrap()
                    .resize
                    .as_mut()
                    .unwrap()
                    .minimum = f32::NAN
            }
            3 => {
                root.children[1]
                    .interaction
                    .as_mut()
                    .unwrap()
                    .resize
                    .as_mut()
                    .unwrap()
                    .maximum = 1.
            }
            _ => {}
        }
        let (_, _, mut region) = prepare(ResizeAxis::Horizontal, false);
        let mut tree = UiTree::new(root);
        let pane = tree.node_ids()[1];
        region.node = tree.node_ids()[2];
        if invalid != 4 {
            tree.observe_resize_bounds([(pane, Size::new(200., 200.))]);
        }
        tree.pointer_event(
            event(PointerPhase::Pressed, 204., 40.),
            std::slice::from_ref(&region),
        );
        assert!(
            tree.pointer_event(
                event(PointerPhase::Moved, 304., 40.),
                std::slice::from_ref(&region)
            )
            .events
            .is_empty()
        );
        assert_eq!(value(&tree, pane, ResizeAxis::Horizontal), length(200.));
    }
}
