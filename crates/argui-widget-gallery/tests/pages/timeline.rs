use argui::{
    runtime::{Entity, Mount},
    ui::{ClickEvent, Element, ElementKind, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn find<'a>(root: &'a Element, key: &str) -> &'a Element {
    fn search<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
        if root.key.as_deref() == Some(key) {
            Some(root)
        } else {
            root.children.iter().find_map(|child| search(child, key))
        }
    }
    search(root, key).expect("gallery element")
}
fn dispatch(app: &Mount<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event).unwrap();
        }
    }
}
fn click(app: &Mount<WidgetGallery>, key: &str) {
    dispatch(app, key, UiEventKind::Click(ClickEvent::accessibility()));
}
fn bounds(root: &Element, key: &str) -> argui::core::Rect {
    let mut tree = UiTree::new(root.clone());
    let layout = argui::layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui::text::TextEngine::new(),
            argui::core::Size::new(1400.0, 2000.0),
        )
        .unwrap();
    layout
        .nodes
        .iter()
        .find(|node| tree.key(node.node) == Some(key))
        .unwrap()
        .bounds
}
#[test]
fn timeline_uses_custom_element_and_standard_keyboard_accessible_controls() {
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::custom-timeline");
    let before = app.render(Default::default()).unwrap();
    assert!(matches!(
        find(&before, "timeline-ruler").kind,
        ElementKind::Custom(_)
    ));
    let clip = find(&before, "timeline-clip-0");
    assert!(
        clip.interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
    let size = clip.style.size;
    let position = bounds(&before, "timeline-clip-0").origin;
    click(&app, "timeline-extend");
    let after = app.render(Default::default()).unwrap();
    assert_ne!(find(&after, "timeline-clip-0").style.size, size);
    dispatch(
        &app,
        "timeline-clip-0",
        UiEventKind::KeyInput(argui::core::KeyInput {
            key: argui::core::Key::ArrowRight,
            state: argui::core::KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    let after = app.render(Default::default()).unwrap();
    assert_ne!(bounds(&after, "timeline-clip-0").origin, position);
    let ruler = find(&after, "timeline-ruler").kind.clone();
    click(&app, "timeline-zoom-in");
    let after = app.render(Default::default()).unwrap();
    assert_ne!(find(&after, "timeline-ruler").kind, ruler);
}

#[test]
fn two_timelines_share_edits_but_not_selection_or_zoom() {
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::custom-timeline");
    let before = app.render(Default::default()).unwrap();
    let first_size = find(&before, "timeline-clip-0").style.size;
    let second_size = find(&before, "timeline-clip-1").style.size;
    click(&app, "timeline-clip-1");
    click(&app, "timeline-extend");
    let after = app.render(Default::default()).unwrap();
    assert!(
        find(&after, "timeline-clip-1")
            .semantics
            .as_ref()
            .unwrap()
            .state
            .selected
    );
    assert!(
        !find(&after, "timeline-secondary-clip-1")
            .semantics
            .as_ref()
            .unwrap()
            .state
            .selected
    );
    assert_ne!(find(&after, "timeline-clip-1").style.size, second_size);
    assert_eq!(
        find(&after, "timeline-clip-1").style.size,
        find(&after, "timeline-secondary-clip-1").style.size
    );
    assert_eq!(
        find(&after, "timeline-detail-1").kind,
        find(&after, "timeline-secondary-detail-1").kind
    );
    click(&app, "timeline-secondary-extend");
    let after = app.render(Default::default()).unwrap();
    assert_ne!(
        find(&after, "timeline-clip-0").style.size,
        first_size,
        "the second view retains its own selected clip"
    );
    let secondary_ruler = find(&after, "timeline-secondary-ruler").kind.clone();
    click(&app, "timeline-zoom-in");
    let after = app.render(Default::default()).unwrap();
    assert_eq!(
        find(&after, "timeline-secondary-ruler").kind,
        secondary_ruler
    );
    assert_ne!(
        find(&after, "timeline-clip-0").style.size,
        find(&after, "timeline-secondary-clip-0").style.size
    );
}

#[test]
fn cancelling_drag_preserves_duration_changed_in_the_other_view() {
    use argui::{
        core::{Point, PointerId},
        ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase},
    };
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::custom-timeline");
    let initial = app.render(Default::default()).unwrap();
    let initial_x = bounds(&initial, "timeline-clip-0").origin.x;
    let pan = |phase, x| {
        let mut tree = UiTree::new(app.render(Default::default()).unwrap());
        let target = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some("timeline-clip-0"))
            .unwrap();
        let kind = UiEventKind::Gesture(GestureEvent {
            target,
            pointer: PointerId::new(1),
            phase,
            delivery: GestureDelivery::Immediate,
            kind: GestureKind::Pan {
                position: Point::default(),
                delta: Point::default(),
                total: Point::new(x, 0.0),
                velocity: Point::default(),
            },
        });
        for event in tree.event_deliveries(target, kind) {
            if event.should_dispatch() {
                app.dispatch_event(&event).unwrap();
            }
        }
    };
    pan(GesturePhase::Started, 0.0);
    pan(GesturePhase::Changed, 80.0);
    let dragged = app.render(Default::default()).unwrap();
    assert_ne!(bounds(&dragged, "timeline-clip-0").origin.x, initial_x);
    assert_eq!(
        bounds(&dragged, "timeline-clip-0").origin.x,
        bounds(&dragged, "timeline-secondary-clip-0").origin.x
    );
    click(&app, "timeline-secondary-extend");
    let edited = app.render(Default::default()).unwrap();
    let edited_size = find(&edited, "timeline-clip-0").style.size;
    pan(GesturePhase::Cancelled, 80.0);
    let cancelled = app.render(Default::default()).unwrap();
    assert_eq!(bounds(&cancelled, "timeline-clip-0").origin.x, initial_x);
    assert_eq!(find(&cancelled, "timeline-clip-0").style.size, edited_size);
}

#[test]
fn resize_regions_share_semantic_and_keyboard_edits_and_reject_nonfinite_values() {
    use argui::accessibility::{Role, SemanticAction, SemanticValue};
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::custom-timeline");
    let before = app.render(Default::default()).unwrap();
    let handle = find(&before, "timeline-resize-0");
    assert_eq!(handle.semantics.as_ref().unwrap().role, Role::Slider);
    assert!(
        handle
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
    let initial = bounds(&before, "timeline-clip-0").size.width;
    dispatch(
        &app,
        "timeline-resize-0",
        UiEventKind::SemanticAction {
            action: SemanticAction::Increment,
            value: None,
        },
    );
    let after = app.render(Default::default()).unwrap();
    assert_eq!(bounds(&after, "timeline-clip-0").size.width, initial + 20.0);
    assert_eq!(
        bounds(&after, "timeline-secondary-clip-0").size.width,
        initial + 20.0
    );
    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        dispatch(
            &app,
            "timeline-resize-0",
            UiEventKind::SemanticAction {
                action: SemanticAction::SetValue,
                value: Some(SemanticValue::Number {
                    value,
                    minimum: None,
                    maximum: None,
                    step: None,
                }),
            },
        );
    }
    let after = app.render(Default::default()).unwrap();
    assert_eq!(bounds(&after, "timeline-clip-0").size.width, initial + 20.0);
    dispatch(
        &app,
        "timeline-resize-0",
        UiEventKind::KeyInput(argui::core::KeyInput {
            key: argui::core::Key::ArrowLeft,
            state: argui::core::KeyState::Pressed,
            modifiers: Default::default(),
            repeat: false,
            text: None,
        }),
    );
    let after = app.render(Default::default()).unwrap();
    assert_eq!(
        bounds(&after, "timeline-secondary-clip-0").size.width,
        initial
    );
    let clip = bounds(&after, "timeline-clip-0");
    let handle = bounds(&after, "timeline-resize-0");
    assert_eq!(
        handle.origin.x + handle.size.width,
        clip.origin.x + clip.size.width
    );
}
