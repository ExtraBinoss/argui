use argui_animation::{Duration, Frame, Time};
use argui_core::{Point, Rect, Size};
use argui_runtime::{Context, LayoutBounds, LayoutSnapshot, Render, ViewUpdate};
use argui_ui::{Element, UiEvent, UiEventKind, UiTree};

#[derive(Default)]
struct Counter(u32);

impl Render for Counter {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text(self.0.to_string())
    }

    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.kind == UiEventKind::Clicked {
            self.0 += 1;
            cx.notify();
        }
    }
}

#[test]
fn apps_rebuild_only_when_their_state_changes() {
    let mut app = Counter::default();
    let tree = UiTree::new(Element::container([]));
    let moved = UiEvent {
        target: tree.node_id_at(0).unwrap(),
        key: Some("increment".into()),
        kind: UiEventKind::PointerEntered,
    };
    let clicked = UiEvent {
        kind: UiEventKind::Clicked,
        ..moved.clone()
    };

    let mut cx = Context::default();
    app.event(&moved, &mut cx);
    assert_eq!(cx.view_update(), ViewUpdate::None);
    let mut cx = Context::default();
    app.event(&clicked, &mut cx);
    assert_eq!(cx.view_update(), ViewUpdate::Rebuild);
    assert!(
        matches!(&app.render(&mut Context::default()).kind, argui_ui::ElementKind::Text { content, .. } if content == "1")
    );
    assert!(!Render::wants_animation_frame(&app));
    let mut cx = Context::default();
    Render::animation_frame(
        &mut app,
        Frame {
            now: Time::ZERO,
            elapsed: Duration::ZERO,
        },
        &mut cx,
    );
    assert_eq!(cx.view_update(), ViewUpdate::None);
}

#[test]
fn layout_snapshots_expose_viewport_and_keyed_logical_bounds() {
    let tree = UiTree::new(Element::container([]).keyed("anchor"));
    let node = tree.node_id_at(0).unwrap();
    let bounds = Rect::new(Point::new(20.0, 30.0), Size::new(80.0, 40.0));
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), Size::new(640.0, 480.0)),
        nodes: vec![LayoutBounds {
            node,
            key: Some("anchor".into()),
            bounds,
        }],
    };

    assert_eq!(snapshot.bounds("anchor"), Some(bounds));
    assert_eq!(snapshot.bounds("missing"), None);
    assert_eq!(snapshot.viewport_size(), Size::new(640.0, 480.0));
    let mut app = Counter::default();
    let mut cx = Context::default();
    app.layout_changed(&snapshot, &mut cx);
    assert_eq!(cx.view_update(), ViewUpdate::None);
}
