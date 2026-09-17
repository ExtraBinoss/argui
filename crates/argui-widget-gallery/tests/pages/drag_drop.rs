use super::*;
use argui::{
    accessibility::SemanticAction,
    animation::{Duration, Frame, Time},
    core::{Point, PointerId},
    runtime::Mount,
    ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase},
};

fn pan(gallery: &Mount<WidgetGallery>, phase: GesturePhase, total: Point, velocity: Point) {
    pan_with_reduced_motion(gallery, phase, total, velocity, false);
}

fn pan_with_reduced_motion(
    gallery: &Mount<WidgetGallery>,
    phase: GesturePhase,
    total: Point,
    velocity: Point,
    reduced_motion: bool,
) {
    pan_target(
        gallery,
        "drag-item-0",
        phase,
        total,
        velocity,
        reduced_motion,
    );
}

fn pan_target(
    gallery: &Mount<WidgetGallery>,
    key: &str,
    phase: GesturePhase,
    total: Point,
    velocity: Point,
    reduced_motion: bool,
) {
    let mut tree = UiTree::new(
        gallery
            .render(WindowEnvironment {
                reduced_motion,
                ..WindowEnvironment::default()
            })
            .unwrap(),
    );
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    let kind = UiEventKind::Gesture(GestureEvent {
        target,
        pointer: PointerId::MOUSE,
        phase,
        delivery: GestureDelivery::Immediate,
        kind: GestureKind::Pan {
            position: Point::default(),
            delta: Point::default(),
            total,
            velocity,
        },
    });
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event).unwrap();
        }
    }
}

fn semantic(gallery: &Mount<WidgetGallery>, key: &str, action: SemanticAction) {
    let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    let kind = UiEventKind::SemanticAction {
        action,
        value: None,
    };
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event).unwrap();
        }
    }
}

fn click_mount(gallery: &Mount<WidgetGallery>, key: &str) {
    let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(target, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event).unwrap();
        }
    }
}

#[test]
fn drag_drop_reorders_live_and_settles_velocity_deformation() {
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    click_mount(&gallery, "nav::drag-drop");
    let initial = gallery.render(Default::default()).unwrap();
    assert!(contains_text(&initial, "Momentum reorder board"));
    assert_eq!(
        keyed(&initial, "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 1")
    );

    pan(
        &gallery,
        GesturePhase::Started,
        Point::default(),
        Point::default(),
    );
    pan(
        &gallery,
        GesturePhase::Changed,
        Point::new(36.0, 190.0),
        Point::new(900.0, 1_100.0),
    );
    for (index, elapsed) in [Duration::from_millis(16), Duration::from_millis(80)]
        .into_iter()
        .enumerate()
    {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos((index as u64 + 1) * 80_000_000),
                elapsed,
            })
            .unwrap();
    }
    let moving = gallery.render(Default::default()).unwrap();
    let held = keyed(&moving, "drag-item-0").unwrap();
    assert_eq!(
        held.semantics.as_ref().unwrap().label.as_deref(),
        Some("Opening titles at position 3")
    );
    assert_ne!(held.transform, argui::core::Transform2D::IDENTITY);
    assert!(held.layer.is_none());
    assert_eq!(held.children.len(), 3);
    assert_eq!(
        held.interaction
            .as_ref()
            .unwrap()
            .gestures
            .pan
            .unwrap()
            .delivery,
        GestureDelivery::FrameCoalesced
    );
    assert!(contains_text(&moving, "HOLDING"));

    pan(
        &gallery,
        GesturePhase::Ended,
        Point::new(36.0, 190.0),
        Point::new(900.0, 1_100.0),
    );
    let dropped = gallery.render(Default::default()).unwrap();
    assert_ne!(
        keyed(&dropped, "drag-item-0").unwrap().transform,
        argui::core::Transform2D::IDENTITY
    );
    for index in 1..=90 {
        gallery
            .animation_frame(Frame {
                now: Time::from_nanos(index * 16_000_000),
                elapsed: Duration::from_millis(16),
            })
            .unwrap();
    }
    assert_eq!(
        keyed(&gallery.render(Default::default()).unwrap(), "drag-item-0")
            .unwrap()
            .transform,
        argui::core::Transform2D::IDENTITY
    );
}

#[test]
fn drag_drop_keyboard_reordering_keeps_stable_item_identity() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::drag-drop");
    keyboard(&gallery, "drag-item-0", argui::core::Key::ArrowDown);
    assert_eq!(
        keyed(&gallery.render(), "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 2")
    );
    keyboard(&gallery, "drag-item-0", argui::core::Key::ArrowUp);
    assert_eq!(
        keyed(&gallery.render(), "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 1")
    );
}

#[test]
fn cancelled_drag_restores_the_original_order() {
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    click_mount(&gallery, "nav::drag-drop");
    pan(
        &gallery,
        GesturePhase::Started,
        Point::default(),
        Point::default(),
    );
    pan(
        &gallery,
        GesturePhase::Changed,
        Point::new(24.0, 190.0),
        Point::new(600.0, 900.0),
    );
    pan(
        &gallery,
        GesturePhase::Cancelled,
        Point::new(24.0, 190.0),
        Point::new(600.0, 900.0),
    );
    let restored = gallery.render(Default::default()).unwrap();
    assert_eq!(
        keyed(&restored, "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 1")
    );
    assert_eq!(
        keyed(&restored, "drag-item-0").unwrap().transform,
        argui::core::Transform2D::IDENTITY
    );
}

#[test]
fn gestures_on_the_list_background_do_not_reorder_items() {
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    click_mount(&gallery, "nav::drag-drop");
    pan_target(
        &gallery,
        "drag-drop-list",
        GesturePhase::Started,
        Point::new(0.0, 190.0),
        Point::new(0.0, 900.0),
        false,
    );
    assert_eq!(
        keyed(&gallery.render(Default::default()).unwrap(), "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 1")
    );
}

#[test]
fn reduced_motion_drops_immediately_and_semantic_actions_reorder() {
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    click_mount(&gallery, "nav::drag-drop");
    pan_with_reduced_motion(
        &gallery,
        GesturePhase::Started,
        Point::default(),
        Point::default(),
        true,
    );
    pan_with_reduced_motion(
        &gallery,
        GesturePhase::Ended,
        Point::new(18.0, 95.0),
        Point::new(700.0, 1_000.0),
        true,
    );
    let dropped = gallery.render(Default::default()).unwrap();
    assert_eq!(
        keyed(&dropped, "drag-item-0").unwrap().transform,
        argui::core::Transform2D::IDENTITY
    );
    semantic(&gallery, "drag-item-0", SemanticAction::Increment);
    assert_eq!(
        keyed(&gallery.render(Default::default()).unwrap(), "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 3")
    );
    semantic(&gallery, "drag-item-0", SemanticAction::Decrement);
    assert_eq!(
        keyed(&gallery.render(Default::default()).unwrap(), "drag-item-0")
            .unwrap()
            .semantics
            .as_ref()
            .unwrap()
            .label
            .as_deref(),
        Some("Opening titles at position 2")
    );
}
