use super::*;
use argui::{
    accessibility::SemanticValue,
    core::{Point, PointerId},
    ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase},
};

/// Delivers one real pan sample to a split separator through the retained tree.
fn pan(app: &Entity<WidgetGallery>, key: &str, phase: GesturePhase, total: Point, delta: Point) {
    let mut tree = UiTree::new(app.render());
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
        delivery: GestureDelivery::FrameCoalesced,
        kind: GestureKind::Pan {
            position: total,
            delta,
            total,
            velocity: Point::default(),
        },
    });
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

/// Reads the controlled numeric size exposed by a split separator.
fn separator_value(root: &Element, key: &str) -> f64 {
    let Some(SemanticValue::Number { value, .. }) =
        keyed(root, key).unwrap().semantics.as_ref().unwrap().value
    else {
        panic!("missing numeric separator value for {key}");
    };
    value
}

#[test]
fn split_pane_examples_resize_across_repeated_controlled_renders() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::split-pane");
    let initial = app.render();
    for title in [
        "Explorer and editor",
        "Preview over console",
        "Trailing inspector",
        "Nested IDE workspace",
    ] {
        assert!(contains_text(&initial, title));
    }
    assert_eq!(separator_value(&initial, "split-navigator"), 220.0);

    pan(
        &app,
        "split-navigator",
        GesturePhase::Started,
        Point::default(),
        Point::default(),
    );
    pan(
        &app,
        "split-navigator",
        GesturePhase::Changed,
        Point::new(40.0, 0.0),
        Point::new(40.0, 0.0),
    );
    pan(
        &app,
        "split-navigator",
        GesturePhase::Changed,
        Point::new(80.0, 0.0),
        Point::new(40.0, 0.0),
    );
    pan(
        &app,
        "split-navigator",
        GesturePhase::Ended,
        Point::new(80.0, 0.0),
        Point::default(),
    );
    assert_eq!(separator_value(&app.render(), "split-navigator"), 300.0);
}

#[test]
fn split_pane_examples_expose_axis_and_trailing_keyboard_controls() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::split-pane");
    keyboard(&app, "split-preview", argui::core::Key::ArrowDown);
    assert_eq!(separator_value(&app.render(), "split-preview"), 128.0);

    keyboard(&app, "split-inspector", argui::core::Key::ArrowLeft);
    assert_eq!(separator_value(&app.render(), "split-inspector"), 255.0);

    keyboard(&app, "split-ide-navigator", argui::core::Key::End);
    keyboard(&app, "split-ide-console", argui::core::Key::Home);
    let root = app.render();
    assert_eq!(separator_value(&root, "split-ide-navigator"), 320.0);
    assert_eq!(separator_value(&root, "split-ide-console"), 64.0);
}
