use argui_core::{Color, ColorScheme, Point, PointerId};
use argui_ui::{
    ClickEvent, Element, GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEvent,
    UiEventKind, UiTree,
};
use argui_widgets::{Drawer, DrawerAction, shadcn};

#[test]
fn only_handle_gestures_drag_or_dismiss_and_cancellation_returns_home() {
    let mut drawer = Drawer::new(
        "d",
        "Drawer",
        true,
        Element::text("Open"),
        Element::text("Content"),
    );
    let node = UiTree::new(Element::container([])).node_ids()[0];
    for (phase, total, velocity, expected) in [
        (GesturePhase::Started, 20.0, 0.0, DrawerAction::Drag(20.0)),
        (GesturePhase::Changed, -20.0, 0.0, DrawerAction::Drag(0.0)),
        (
            GesturePhase::Cancelled,
            120.0,
            900.0,
            DrawerAction::Drag(0.0),
        ),
        (GesturePhase::Ended, 10.0, 0.0, DrawerAction::Drag(0.0)),
        (GesturePhase::Ended, 100.0, 0.0, DrawerAction::Close),
        (GesturePhase::Ended, 20.0, 700.0, DrawerAction::Close),
    ] {
        let gesture = UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: PointerId::new(1),
            phase,
            delivery: GestureDelivery::Immediate,
            kind: GestureKind::Pan {
                position: Point::default(),
                delta: Point::default(),
                total: Point::new(0.0, total),
                velocity: Point::new(0.0, velocity),
            },
        });
        assert_eq!(
            drawer.action(&UiEvent::new(
                node,
                Some("d::handle".into()),
                gesture.clone()
            )),
            Some(expected)
        );
        assert_eq!(
            drawer.action(&UiEvent::new(node, Some("content".into()), gesture)),
            None
        );
    }
    drawer.offset = 30.0;
    let root = drawer
        .clone()
        .build(shadcn(Color::BLACK).resolve(ColorScheme::Light));
    assert_eq!(root.children[1].children[1].transform.translation.y, 30.0);
    assert_eq!(
        drawer.action(&UiEvent::new(
            node,
            Some("d::close".into()),
            UiEventKind::Click(ClickEvent::accessibility())
        )),
        Some(DrawerAction::Close)
    );
    drawer.sheet.open = false;
    assert_eq!(
        drawer.action(&UiEvent::new(
            node,
            Some("d::trigger".into()),
            UiEventKind::Click(ClickEvent::accessibility())
        )),
        Some(DrawerAction::Open)
    );
    assert_eq!(
        drawer
            .build(shadcn(Color::BLACK).resolve(ColorScheme::Dark))
            .children
            .len(),
        1
    );
}
