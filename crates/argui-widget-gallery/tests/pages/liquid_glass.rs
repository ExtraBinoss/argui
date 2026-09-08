#[path = "liquid_glass/controls.rs"]
mod controls;

use argui::{
    paint::Filter,
    runtime::{Entity, Mount},
    ui::{ClickEvent, Element, UiEventKind, UiTree},
};
use argui_widget_gallery::WidgetGallery;

fn find<'a>(root: &'a Element, key: &str) -> Option<&'a Element> {
    if root.key.as_deref() == Some(key) {
        Some(root)
    } else {
        root.children.iter().find_map(|child| find(child, key))
    }
}
fn click(app: &Mount<WidgetGallery>, key: &str) {
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some(key))
        .unwrap();
    for event in tree.event_deliveries(node, UiEventKind::Click(ClickEvent::accessibility())) {
        if event.should_dispatch() {
            app.dispatch_event(&event).unwrap();
        }
    }
}
fn filter(app: &Mount<WidgetGallery>) -> Option<Filter> {
    find(
        &app.render(Default::default()).unwrap(),
        "liquid-glass-pane",
    )
    .unwrap()
    .layer
    .as_ref()
    .and_then(|layer| layer.backdrop_filters.first().cloned())
}
#[test]
fn gallery_glass_controls_update_real_filter_and_can_disable_it() {
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::effects");
    let Filter::Effect(initial) = filter(&app).unwrap() else {
        panic!()
    };
    assert_eq!(initial.id, argui_effects::LIQUID_GLASS_ID);
    for control in ["glass-octaves", "glass-seed", "glass-tint-blue"] {
        let before = filter(&app);
        click(&app, control);
        assert_ne!(filter(&app), before, "{control}");
    }
    click(&app, "glass-enable");
    assert!(filter(&app).is_none());
    click(&app, "glass-enable");
    assert!(filter(&app).is_some());
    click(&app, "nav::button");
    assert!(
        find(
            &app.render(Default::default()).unwrap(),
            "liquid-glass-pane"
        )
        .is_none()
    );
    click(&app, "nav::effects");
    assert!(filter(&app).is_some());
}

#[test]
fn drag_uses_actual_stage_width_and_stays_inside_after_resize() {
    use argui::{
        core::{Point, PointerId, Rect, Size},
        runtime::{LayoutBounds, LayoutSnapshot, Render},
        ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase, length},
    };
    let app = Entity::new(WidgetGallery::default()).mount().unwrap();
    click(&app, "nav::effects");
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some("liquid-glass-pane"))
        .unwrap();
    let layout = |width| LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(width, 500.0)),
        nodes: vec![LayoutBounds {
            node,
            key: Some("liquid-glass-stage".into()),
            bounds: Rect::new(Point::default(), Size::new(width, 360.0)),
        }],
    };
    app.update(|gallery, cx| gallery.layout_changed(&layout(600.0), cx))
        .unwrap();
    for phase in [GesturePhase::Started, GesturePhase::Changed] {
        for event in tree.event_deliveries(
            node,
            UiEventKind::Gesture(GestureEvent {
                target: node,
                pointer: PointerId::new(0),
                phase,
                delivery: GestureDelivery::Immediate,
                kind: GestureKind::Pan {
                    position: Point::default(),
                    delta: Point::default(),
                    total: Point::new(1000.0, 1000.0),
                    velocity: Point::default(),
                },
            }),
        ) {
            if event.should_dispatch() {
                app.dispatch_event(&event).unwrap();
            }
        }
    }
    let root = app.render(Default::default()).unwrap();
    let pane = find(&root, "liquid-glass-pane").unwrap();
    assert_eq!(pane.style.inset.left, length(370.0));
    assert_eq!(pane.style.inset.top, length(210.0));
    app.update(|gallery, cx| gallery.layout_changed(&layout(300.0), cx))
        .unwrap();
    assert_eq!(
        find(
            &app.render(Default::default()).unwrap(),
            "liquid-glass-pane"
        )
        .unwrap()
        .style
        .inset
        .left,
        length(70.0)
    );
    click(&app, "glass-recenter");
    let root = app.render(Default::default()).unwrap();
    let pane = find(&root, "liquid-glass-pane").unwrap();
    assert_eq!(pane.style.inset.left, length(35.0));
    assert_eq!(pane.style.inset.top, length(105.0));
}
