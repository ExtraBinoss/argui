use argui_core::{Point, PointerButton, PointerEvent, PointerPhase, Size};
use argui_layout::{LayoutEngine, LayoutOutput};
use argui_ui::{
    Element, EventHandlerId, EventListener, EventOwnerId, EventType, Interaction, ResizeAxis,
    ResizeHandle, UiEventKind, UiTree, length, percent,
};

fn root(axis: ResizeAxis, trailing: bool) -> Element {
    let pane = Element::container([Element::text("native sizing")])
        .keyed("pane")
        .width(length(200.))
        .height(length(200.))
        .shrink(0.);
    let mut interaction = Interaction::default().capture_on_press(true);
    interaction.resize = Some(ResizeHandle {
        target: "pane".into(),
        axis,
        trailing,
        minimum: 100.,
        maximum: 400.,
    });
    let handle = Element::container([])
        .keyed("handle")
        .width(length(8.))
        .height(length(8.))
        .interaction(interaction)
        .on(EventListener::new(
            EventType::ResizeCommit,
            EventHandlerId::new(EventOwnerId(1), 1),
        ));
    let sibling = Element::container([])
        .keyed("sibling")
        .grow(1.)
        .min_width(length(0.))
        .min_height(length(0.));
    let children = if trailing {
        vec![sibling, handle, pane]
    } else {
        vec![pane, handle, sibling]
    };
    let root = if axis == ResizeAxis::Horizontal {
        Element::row(children)
    } else {
        Element::column(children)
    };
    root.width(percent(1.)).height(percent(1.))
}

fn measured(ui: &UiTree, output: &LayoutOutput, key: &str) -> argui_core::Rect {
    output
        .nodes
        .iter()
        .find(|node| ui.key(node.node) == Some(key))
        .unwrap()
        .bounds
}

fn pointer(phase: PointerPhase, position: Point) -> PointerEvent {
    PointerEvent {
        button: matches!(phase, PointerPhase::Pressed | PointerPhase::Released)
            .then_some(PointerButton::Primary),
        buttons: u16::from(matches!(phase, PointerPhase::Pressed | PointerPhase::Moved)),
        ..PointerEvent::mouse(phase, position)
    }
}

#[test]
fn native_resize_updates_cached_geometry_without_reconciling_the_producer_tree() {
    for axis in [ResizeAxis::Horizontal, ResizeAxis::Vertical] {
        for trailing in [false, true] {
            let mut ui = UiTree::new(root(axis, trailing));
            let mut engine = LayoutEngine::new();
            let mut text = super::text_engine();
            let viewport = Size::new(800., 800.);
            let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
            let bounds = measured(&ui, &output, "handle");
            let origin = Point::new(bounds.origin.x + 4., bounds.origin.y + 4.);
            let revision = ui.revision();
            let authored = ui.root().clone();
            ui.pointer_event(pointer(PointerPhase::Pressed, origin), &output.hit_regions);
            let delta = if trailing { -100. } else { 100. };
            let position = if axis == ResizeAxis::Horizontal {
                Point::new(origin.x + delta, origin.y)
            } else {
                Point::new(origin.x, origin.y + delta)
            };
            let update =
                ui.pointer_event(pointer(PointerPhase::Moved, position), &output.hit_regions);
            assert!(update.layout_changed && update.events.is_empty());
            output = engine.compute(&mut ui, &mut text, viewport).unwrap();
            let size = measured(&ui, &output, "pane").size;
            let sibling = measured(&ui, &output, "sibling").size;
            assert_eq!(
                if axis == ResizeAxis::Horizontal {
                    size.width
                } else {
                    size.height
                },
                300.
            );
            assert_eq!(
                if axis == ResizeAxis::Horizontal {
                    sibling.width
                } else {
                    sibling.height
                },
                492.
            );
            assert_eq!(ui.revision(), revision);
            assert!(ui.root().ptr_eq(&authored));
            let released = ui.pointer_event(
                pointer(PointerPhase::Released, position),
                &output.hit_regions,
            );
            assert!(
                released
                    .events
                    .iter()
                    .any(|event| event.kind == UiEventKind::ResizeCommitted { value: 300. })
            );
        }
    }
}

#[test]
fn native_resize_cancel_restores_the_retained_taffy_style() {
    let mut ui = UiTree::new(root(ResizeAxis::Horizontal, false));
    let mut engine = LayoutEngine::new();
    let mut text = super::text_engine();
    let viewport = Size::new(800., 800.);
    let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    ui.pointer_event(
        pointer(PointerPhase::Pressed, Point::new(204., 4.)),
        &output.hit_regions,
    );
    ui.pointer_event(
        pointer(PointerPhase::Moved, Point::new(304., 4.)),
        &output.hit_regions,
    );
    output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(measured(&ui, &output, "pane").size.width, 300.);
    let update = ui.pointer_event(
        pointer(PointerPhase::Cancelled, Point::new(304., 4.)),
        &output.hit_regions,
    );
    assert!(update.events.is_empty());
    output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(measured(&ui, &output, "pane").size.width, 200.);
    assert_eq!(measured(&ui, &output, "sibling").size.width, 592.);
}

#[test]
fn native_resize_uses_measured_percentage_dimensions_and_authored_resizes_release_override() {
    let mut authored = root(ResizeAxis::Horizontal, false);
    authored.children[0].style.size.width = percent(0.25);
    let mut ui = UiTree::new(authored.clone());
    let mut engine = LayoutEngine::new();
    let mut text = super::text_engine();
    let viewport = Size::new(800., 800.);
    let mut output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    ui.pointer_event(
        pointer(PointerPhase::Pressed, Point::new(204., 4.)),
        &output.hit_regions,
    );
    ui.pointer_event(
        pointer(PointerPhase::Released, Point::new(254., 4.)),
        &output.hit_regions,
    );
    output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(measured(&ui, &output, "pane").size.width, 250.);
    authored.children[0].style.size.width = length(250.);
    ui.update(authored.clone());
    output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(measured(&ui, &output, "pane").size.width, 250.);
    authored.children[0].style.size.width = length(310.);
    ui.update(authored);
    output = engine.compute(&mut ui, &mut text, viewport).unwrap();
    assert_eq!(measured(&ui, &output, "pane").size.width, 310.);
}
