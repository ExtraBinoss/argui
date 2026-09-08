use argui::{
    core::{Point, PointerEvent, PointerId, PointerPhase},
    runtime::Entity,
    ui::{
        ClickEvent, Element, GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEventKind,
        UiTree, length,
    },
};
use argui_widget_gallery::WidgetGallery;

#[path = "app/interaction.rs"]
mod interaction;

#[test]
fn sidebar_fast_hover_has_no_trail_and_keeps_the_active_page_highlighted() {
    let gallery = Entity::new(WidgetGallery::default());
    let mut tree = UiTree::new(gallery.render());
    let output = argui::layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui::text::TextEngine::new(),
            argui::core::Size::new(1220.0, 900.0),
        )
        .unwrap();
    let entries: Vec<_> = [
        "nav::button",
        "nav::input",
        "nav::textarea",
        "nav::checkbox",
    ]
    .into_iter()
    .map(|key| {
        let index = tree
            .node_ids()
            .iter()
            .position(|node| tree.key(*node) == Some(key))
            .unwrap();
        let node = tree.node_ids()[index];
        let region = output
            .hit_regions
            .iter()
            .find(|region| region.node == node)
            .unwrap();
        (
            node,
            index,
            Point::new(region.bounds.origin.x + 10.0, region.bounds.origin.y + 10.0),
            tree.resolved_quad(node, tree.element_at(index).unwrap())
                .background,
        )
    })
    .collect();
    // Cross several buttons without advancing a frame or waiting for a tween.
    for hovered in [1, 2, 3, 2, 1] {
        tree.pointer_moved(entries[hovered].2, &output.hit_regions);
        for (index, (node, element, _, resting)) in entries.iter().enumerate() {
            let background = tree
                .resolved_quad(*node, tree.element_at(*element).unwrap())
                .background;
            if index == hovered {
                assert_ne!(&background, resting);
            } else {
                assert_eq!(
                    &background, resting,
                    "sidebar row {index} left a hover trail"
                );
            }
        }
    }
    tree.pointer_moved(Point::new(1200.0, 890.0), &output.hit_regions);
    for (node, index, _, resting) in entries {
        assert_eq!(
            tree.resolved_quad(node, tree.element_at(index).unwrap())
                .background,
            resting
        );
    }
}

fn has_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key) || element.children.iter().any(|child| has_key(child, key))
}

fn find_key<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| find_key(child, key))
}

#[test]
fn gallery_app_builds_a_searchable_public_root() {
    let gallery = Entity::new(WidgetGallery::default());
    let root = gallery.render();
    assert!(has_key(&root, "gallery-root"));
    assert!(has_key(&root, "gallery-search"));
}

#[test]
fn any_element_can_drive_a_constrained_resize_and_double_click_reset() {
    let gallery = Entity::new(WidgetGallery::default());
    dispatch(
        &gallery,
        "nav::textarea",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let pan = |phase, total| {
        UiEventKind::Gesture(GestureEvent {
            target: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
            pointer: PointerId::MOUSE,
            phase,
            kind: GestureKind::Pan {
                position: total,
                delta: total,
                total,
                velocity: Point::default(),
            },
            delivery: GestureDelivery::FrameCoalesced,
        })
    };
    dispatch(
        &gallery,
        "notes-resize",
        pan(GesturePhase::Started, Point::default()),
    );
    dispatch(
        &gallery,
        "notes-resize",
        pan(GesturePhase::Changed, Point::new(100.0, 50.0)),
    );

    let resized = gallery.render();
    let panel = find_key(&resized, "notes-resizable").unwrap();
    assert_eq!(panel.style.size.width, length(620.0));
    assert_eq!(panel.style.size.height, length(220.0));

    dispatch(
        &gallery,
        "notes-resize",
        pan(GesturePhase::Changed, Point::new(1_000.0, 1_000.0)),
    );
    let maximum = gallery.render();
    let panel = find_key(&maximum, "notes-resizable").unwrap();
    assert_eq!(panel.style.size.width, length(760.0));
    assert_eq!(panel.style.size.height, length(480.0));

    dispatch(
        &gallery,
        "notes-resize",
        pan(GesturePhase::Changed, Point::new(-1_000.0, -1_000.0)),
    );
    dispatch(
        &gallery,
        "notes-resize",
        pan(GesturePhase::Cancelled, Point::new(-1_000.0, -1_000.0)),
    );
    let minimum = gallery.render();
    let panel = find_key(&minimum, "notes-resizable").unwrap();
    assert_eq!(panel.style.size.width, length(280.0));
    assert_eq!(panel.style.size.height, length(120.0));

    dispatch(
        &gallery,
        "notes-resize",
        UiEventKind::Click(ClickEvent::pointer(
            PointerEvent::mouse(PointerPhase::Released, Point::default()),
            2,
        )),
    );
    let reset = gallery.render();
    let panel = find_key(&reset, "notes-resizable").unwrap();
    assert_eq!(panel.style.size.width, length(520.0));
    assert_eq!(panel.style.size.height, length(170.0));
}

fn dispatch(gallery: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(gallery.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap();
    let kind = match kind {
        UiEventKind::Gesture(mut gesture) => {
            gesture.target = target;
            UiEventKind::Gesture(gesture)
        }
        kind => kind,
    };
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            gallery.dispatch_event(&event);
        }
    }
}
