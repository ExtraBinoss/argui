use super::*;
use argui::{
    core::{Color, Key, KeyInput, KeyState},
    paint::Fill,
    runtime::WindowEnvironment,
    ui::DesktopBackdropState,
};

fn click(gallery: &Entity<WidgetGallery>, key: &str) {
    dispatch(
        gallery,
        key,
        UiEventKind::Click(ClickEvent::accessibility()),
    );
}

#[test]
fn sidebar_glass_is_optional_and_uses_the_public_backdrop_api() {
    let gallery = Entity::new(WidgetGallery::default());
    assert!(
        find_key(&gallery.render(), "gallery-sidebar")
            .unwrap()
            .desktop_backdrop
            .is_none()
    );
    click(&gallery, "gallery-appearance");
    click(&gallery, "sidebar-glass");
    let root = gallery.render();
    let sidebar = find_key(&root, "gallery-sidebar").unwrap();
    let backdrop = sidebar.desktop_backdrop.unwrap();
    assert_eq!(backdrop.fallback.to_srgba()[3], 1.0);
    assert!((backdrop.tint.to_srgba()[3] - 0.78).abs() < 0.001);
    assert!(backdrop.inactive_tint.to_srgba()[3] > backdrop.tint.to_srgba()[3]);
    assert_eq!(
        root.paint.quad.background,
        Some(Fill::Solid(Color::TRANSPARENT))
    );
    assert!(
        matches!(find_key(&root, "gallery-content-scroll").unwrap().paint.quad.background, Some(Fill::Solid(color)) if color.to_srgba()[3] == 1.0)
    );
    let mut tree = UiTree::new(root);
    let layout = argui::layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui::text::TextEngine::new(),
            argui::core::Size::new(1600.0, 1000.0),
        )
        .unwrap();
    let content = layout
        .nodes
        .iter()
        .find(|node| tree.key(node.node) == Some("gallery-content-scroll"))
        .unwrap();
    assert_eq!(content.bounds.origin.x + content.bounds.size.width, 1600.0);
    click(&gallery, "sidebar-fallback");
    let root = gallery.render();
    let backdrop = find_key(&root, "gallery-sidebar")
        .unwrap()
        .desktop_backdrop
        .unwrap();
    assert_eq!(backdrop.fallback.to_srgba()[3], backdrop.tint.to_srgba()[3]);
    click(&gallery, "sidebar-glass");
    assert!(
        find_key(&gallery.render(), "gallery-sidebar")
            .unwrap()
            .desktop_backdrop
            .is_none()
    );
    click(&gallery, "gallery-appearance");
    assert!(!has_key(&gallery.render(), "gallery-appearance::content"));
}

#[test]
fn settings_sliders_change_only_background_colors_and_dismiss_cleanly() {
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "gallery-appearance");
    click(&gallery, "sidebar-glass");
    let before = gallery.render();
    for key in [
        "sidebar-opacity",
        "sidebar-tint",
        "sidebar-inactive-opacity",
    ] {
        dispatch(
            &gallery,
            key,
            UiEventKind::KeyInput(KeyInput {
                key: Key::ArrowRight,
                state: KeyState::Pressed,
                text: None,
                modifiers: Default::default(),
                repeat: false,
            }),
        );
    }
    let after = gallery.render();
    let old = find_key(&before, "gallery-sidebar").unwrap();
    let new = find_key(&after, "gallery-sidebar").unwrap();
    assert_eq!(old.style, new.style);
    assert_ne!(old.desktop_backdrop, new.desktop_backdrop);
    let backdrop = new.desktop_backdrop.unwrap();
    assert!(
        (backdrop
            .color(DesktopBackdropState {
                available: true,
                focused: true
            })
            .to_srgba()[3]
            - 0.79)
            .abs()
            < 0.001
    );
    assert!((backdrop.inactive_tint.to_srgba()[3] - 0.93).abs() < 0.001);
    let native = gallery.render_in(WindowEnvironment {
        desktop_backdrop_available: true,
        ..Default::default()
    });
    assert!(format!("{native:?}").contains("Native desktop blur is available"));
    dispatch(
        &gallery,
        "gallery-appearance::content",
        UiEventKind::DismissRequested,
    );
    assert!(!has_key(&gallery.render(), "gallery-appearance::content"));
}

#[test]
fn changing_theme_with_glass_updates_navigation_text() {
    use argui::{animation::Time, core::ColorScheme};
    let gallery = Entity::new(WidgetGallery::default());
    click(&gallery, "gallery-appearance");
    click(&gallery, "sidebar-glass");
    click(&gallery, "gallery-appearance");
    let mut tree = UiTree::new(gallery.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        ..Default::default()
    }));
    let mut engine = argui::layout::LayoutEngine::new();
    let mut text = argui::text::TextEngine::new();
    let size = argui::core::Size::new(1220.0, 900.0);
    let mut layout = engine.compute(&mut tree, &mut text, size).unwrap();
    tree.advance_animations(Time::ZERO);
    tree.update(gallery.render_in(WindowEnvironment {
        color_scheme: ColorScheme::Light,
        ..Default::default()
    }));
    tree.advance_animations(Time::from_nanos(0));
    engine.repaint(&tree, &mut layout);
    tree.advance_animations(Time::from_nanos(2_000_000_000));
    engine.repaint(&tree, &mut layout);
    let expected = argui::widgets::shadcn(WindowEnvironment::default().primary)
        .resolve(ColorScheme::Light)
        .foreground;
    let (index, style) = tree
        .node_ids()
        .iter()
        .enumerate()
        .find_map(|(index, _)| match &tree.element_at(index)?.kind {
            argui::ui::ElementKind::Text { content, style } if content.as_str() == "Alert" => {
                Some((index, style))
            }
            _ => None,
        })
        .unwrap();
    assert_eq!(
        tree.resolved_text_color(tree.node_ids()[index], style.color),
        expected
    );
    let block = layout
        .nodes
        .iter()
        .find(|node| node.node == tree.node_ids()[index])
        .unwrap()
        .text_index
        .unwrap();
    assert_eq!(layout.text.blocks()[block].style.color, expected);
}

#[test]
fn fallback_sliders_accept_track_clicks_and_drag_after_layout() {
    use argui::{
        core::{Rect, Size},
        runtime::{LayoutBounds, LayoutSnapshot, Mount},
    };
    fn send(mount: &Mount<WidgetGallery>, key: &str, kind: UiEventKind) {
        let mut tree = UiTree::new(mount.render(WindowEnvironment::default()).unwrap());
        let node = tree
            .node_ids()
            .iter()
            .copied()
            .find(|node| tree.key(*node) == Some(key))
            .unwrap();
        for event in tree.event_deliveries(node, kind) {
            if event.should_dispatch() {
                mount.dispatch_event(&event).unwrap();
            }
        }
    }
    let gallery = Entity::new(WidgetGallery::default());
    let mount = gallery.mount().unwrap();
    for key in ["gallery-appearance", "sidebar-glass", "sidebar-fallback"] {
        send(&mount, key, UiEventKind::Click(ClickEvent::accessibility()));
    }
    let mut tree = UiTree::new(mount.render(WindowEnvironment::default()).unwrap());
    let size = Size::new(1220.0, 900.0);
    let layout = argui::layout::LayoutEngine::new()
        .compute(&mut tree, &mut argui::text::TextEngine::new(), size)
        .unwrap();
    let snapshot = LayoutSnapshot {
        viewport: Rect::new(Point::default(), size),
        nodes: layout
            .nodes
            .iter()
            .map(|node| LayoutBounds {
                node: node.node,
                key: tree.key(node.node).map(str::to_owned),
                bounds: node.bounds,
            })
            .collect(),
    };
    mount.layout_changed(&snapshot).unwrap();
    for (key, ratio) in [
        ("sidebar-opacity", 0.25),
        ("sidebar-tint", 0.60),
        ("sidebar-inactive-opacity", 0.85),
    ] {
        let track = snapshot.bounds(&format!("{key}::track")).unwrap();
        let position = Point::new(
            track.origin.x + track.size.width * ratio,
            track.origin.y + track.size.height / 2.0,
        );
        send(
            &mount,
            key,
            UiEventKind::Gesture(GestureEvent {
                target: tree.node_ids()[0],
                pointer: PointerId::MOUSE,
                phase: GesturePhase::Ended,
                kind: GestureKind::Tap { position },
                delivery: GestureDelivery::Immediate,
            }),
        );
    }
    let root = mount.render(WindowEnvironment::default()).unwrap();
    let backdrop = find_key(&root, "gallery-sidebar")
        .unwrap()
        .desktop_backdrop
        .unwrap();
    assert!((backdrop.fallback.to_srgba()[3] - 0.25).abs() < 0.01);
    assert!(
        (backdrop
            .color(DesktopBackdropState {
                available: false,
                focused: false
            })
            .to_srgba()[3]
            - 0.85)
            .abs()
            < 0.01
    );
    let track = snapshot.bounds("sidebar-opacity::track").unwrap();
    let position = Point::new(track.origin.x + track.size.width * 0.45, track.origin.y);
    send(
        &mount,
        "sidebar-opacity",
        UiEventKind::Gesture(GestureEvent {
            target: tree.node_ids()[0],
            pointer: PointerId::MOUSE,
            phase: GesturePhase::Started,
            kind: GestureKind::Pan {
                position,
                delta: Point::default(),
                total: Point::default(),
                velocity: Point::default(),
            },
            delivery: GestureDelivery::FrameCoalesced,
        }),
    );
    let root = mount.render(WindowEnvironment::default()).unwrap();
    assert!(
        (find_key(&root, "gallery-sidebar")
            .unwrap()
            .desktop_backdrop
            .unwrap()
            .fallback
            .to_srgba()[3]
            - 0.45)
            .abs()
            < 0.01
    );
}
