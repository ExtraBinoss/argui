use super::*;

#[test]
fn live_button_color_matches_the_swatch_without_advancing_the_animation_clock() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::color-picker");
    let mut tree = UiTree::new(gallery.render());
    tree.advance_animations(argui::animation::Time::ZERO);
    for value in ["#FF0000FF", "#00FF00FF", "#0000FF80"] {
        dispatch(
            &gallery,
            "gallery-color::field::0",
            UiEventKind::TextChanged(value.into()),
        );
        tree.update(gallery.render());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some("color-preview"))
            .unwrap();
        let swatch = keyed(tree.root(), "color-preview-swatch").unwrap();
        assert_eq!(
            tree.resolved_quad(node, tree.element_for(node).unwrap())
                .background,
            swatch.paint.quad.background
        );
    }
}

#[test]
fn color_picker_updates_its_preview_switches_formats_and_survives_navigation() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "nav::color-picker");
    dispatch(
        &gallery,
        "gallery-color::field::0",
        UiEventKind::TextChanged("#8040ff80".into()),
    );
    assert!(contains_text(&gallery.render(), "#8040FF80"));
    click(&gallery, "gallery-color::format::RGB");
    dispatch(
        &gallery,
        "gallery-color::field::0",
        UiEventKind::TextChanged("255".into()),
    );
    let root = gallery.render();
    let Some(argui::paint::Fill::Solid(color)) = keyed(&root, "color-preview-swatch")
        .unwrap()
        .paint
        .quad
        .background
    else {
        panic!("preview color");
    };
    assert_eq!(color.to_srgba8(), [255, 64, 255, 128]);
    assert_eq!(
        keyed(&root, "color-preview").unwrap().paint.quad.background,
        Some(argui::paint::Fill::Solid(color))
    );
    click(&gallery, "color-enabled");
    assert!(
        !keyed(&gallery.render(), "gallery-color::pad")
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .enabled
    );
    click(&gallery, "color-enabled");
    click(&gallery, "nav::button");
    click(&gallery, "nav::color-picker");
    assert!(contains_text(&gallery.render(), "#FF40FF80"));
}

#[test]
fn mounted_gallery_delivers_pad_bounds_to_the_lazy_page() {
    use argui::{
        core::{Point, Rect},
        runtime::{LayoutBounds, LayoutSnapshot, Render},
        ui::{GestureDelivery, GestureEvent, GestureKind, GesturePhase},
    };
    let gallery = Entity::new(WidgetGallery::default()).mount().unwrap();
    let deliver = |key: &str, kind: UiEventKind| {
        let mut tree = UiTree::new(gallery.render(Default::default()).unwrap());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|id| tree.key(*id) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(node, kind) {
            if event.should_dispatch() {
                gallery.dispatch_event(&event).unwrap();
            }
        }
    };
    deliver(
        "nav::color-picker",
        UiEventKind::Click(ClickEvent::accessibility()),
    );
    let tree = UiTree::new(gallery.render(Default::default()).unwrap());
    let node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|id| tree.key(*id) == Some("gallery-color::pad"))
        .unwrap();
    gallery
        .update(|app, cx| {
            app.layout_changed(
                &LayoutSnapshot {
                    viewport: Rect::new(Point::default(), Size::new(800.0, 600.0)),
                    nodes: vec![LayoutBounds {
                        node,
                        key: Some("gallery-color::pad".into()),
                        retained_identity: None,
                        bounds: Rect::new(Point::new(100.0, 100.0), Size::new(200.0, 100.0)),
                    }],
                },
                cx,
            )
        })
        .unwrap();
    deliver(
        "gallery-color::pad",
        UiEventKind::Gesture(GestureEvent {
            target: node,
            pointer: argui::core::PointerId::MOUSE,
            phase: GesturePhase::Started,
            delivery: GestureDelivery::Immediate,
            kind: GestureKind::Pan {
                position: Point::new(200.0, 150.0),
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
        }),
    );
    let root = gallery.render(Default::default()).unwrap();
    let Some(argui::paint::Fill::Solid(color)) = keyed(&root, "color-preview-swatch")
        .unwrap()
        .paint
        .quad
        .background
    else {
        panic!("swatch")
    };
    let [r, g, b, _] = color.to_srgba();
    assert!((r.max(g).max(b) - 0.5).abs() < 0.001);
    assert!((r.min(g).min(b) - 0.25).abs() < 0.001);
    assert_eq!(
        keyed(&root, "color-preview").unwrap().paint.quad.background,
        Some(argui::paint::Fill::Solid(color))
    );
}
