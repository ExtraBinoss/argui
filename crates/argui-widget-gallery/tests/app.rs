use argui::{
    core::{Insets, Point, PointerEvent, PointerId, PointerPhase, Rect, Size},
    runtime::{Entity, LayoutSnapshot, WindowEnvironment},
    ui::{
        ClickEvent, Element, GestureDelivery, GestureEvent, GestureKind, GesturePhase, UiEventKind,
        UiTree, length,
    },
};
use argui_widget_gallery::WidgetGallery;

#[path = "app/desktop_backdrop.rs"]
mod desktop_backdrop;
#[path = "app/interaction.rs"]
mod interaction;
#[path = "app/navigation.rs"]
mod navigation;

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
    let entries: Vec<_> = ["nav::button", "nav::badge", "nav::card", "nav::alert"]
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

/// Finds the laid-out bounds for an element identified by its stable UI key.
///
/// * `tree` — retained UI tree containing the keyed element.
/// * `output` — layout result produced for `tree`.
/// * `key` — stable key of the element whose bounds are requested.
///
/// # Panics
///
/// Panics when the tree does not contain `key` or the layout output omits it.
fn layout_bounds_for_key(tree: &UiTree, output: &argui::layout::LayoutOutput, key: &str) -> Rect {
    output
        .nodes
        .iter()
        .find(|node| tree.key(node.node) == Some(key))
        .unwrap_or_else(|| panic!("layout node with key {key:?} is missing"))
        .bounds
}

#[test]
fn gallery_app_builds_a_searchable_public_root() {
    let gallery = Entity::new(WidgetGallery::default());
    let root = gallery.render();
    assert!(has_key(&root, "gallery-root"));
    assert!(has_key(&root, "gallery-search"));
}

#[test]
fn gallery_root_respects_native_safe_area_insets() {
    let gallery = Entity::new(WidgetGallery::default());
    let root = gallery.render_in(WindowEnvironment {
        safe_area_insets: Insets::new(24.0, 8.0, 34.0, 6.0),
        ..WindowEnvironment::default()
    });

    assert_eq!(root.style.padding.top, length(24.0));
    assert_eq!(root.style.padding.right, length(8.0));
    assert_eq!(root.style.padding.bottom, length(34.0));
    assert_eq!(root.style.padding.left, length(6.0));
    assert!(has_key(&root, "gallery-root"));
    let tree = UiTree::new(root);
    assert!(
        tree.resolved_quad(tree.node_ids()[0], tree.root())
            .background
            .is_some()
    );
}

#[test]
fn compact_gallery_layout_keeps_its_header_and_content_inside_native_safe_area() {
    let gallery = Entity::new(WidgetGallery::default());
    let mount = gallery.mount().unwrap();
    resize_gallery(&mount, 390.0, 844.0);
    let insets = Insets::new(44.0, 8.0, 24.0, 6.0);
    let mut tree = UiTree::new(
        mount
            .render(WindowEnvironment {
                safe_area_insets: insets,
                ..WindowEnvironment::default()
            })
            .unwrap(),
    );
    let output = argui::layout::LayoutEngine::new()
        .compute(
            &mut tree,
            &mut argui::text::TextEngine::new(),
            Size::new(390.0, 844.0),
        )
        .unwrap();

    let gallery_bounds = layout_bounds_for_key(&tree, &output, "gallery-root");
    let topbar_bounds = layout_bounds_for_key(&tree, &output, "gallery-topbar");
    let mobile_header_bounds = layout_bounds_for_key(&tree, &output, "gallery-mobile-header");
    let content_bounds = layout_bounds_for_key(&tree, &output, "gallery-content-scroll");
    let safe_right = 390.0 - insets.right;
    let safe_bottom = 844.0 - insets.bottom;

    assert_eq!(gallery_bounds.origin, Point::new(insets.left, insets.top));
    assert!(gallery_bounds.origin.x + gallery_bounds.size.width <= safe_right);
    assert!(gallery_bounds.origin.y + gallery_bounds.size.height <= safe_bottom);
    assert_eq!(topbar_bounds.origin.y, insets.top);
    assert!(mobile_header_bounds.origin.y >= topbar_bounds.origin.y + topbar_bounds.size.height);
    assert!(content_bounds.origin.y + content_bounds.size.height <= safe_bottom);
}

#[test]
fn narrow_gallery_uses_a_horizontal_header_and_compact_content() {
    let gallery = Entity::new(WidgetGallery::default());
    let mount = gallery.mount().unwrap();
    assert!(has_key(
        &mount.render(WindowEnvironment::default()).unwrap(),
        "gallery-sidebar"
    ));

    resize_gallery(&mount, 390.0, 844.0);
    let compact = mount.render(WindowEnvironment::default()).unwrap();
    assert!(!has_key(&compact, "gallery-sidebar"));
    let topbar = find_key(&compact, "gallery-topbar").unwrap();
    assert_eq!(topbar.style.padding.right, length(14.0));
    let navigation = find_key(&compact, "gallery-mobile-navigation").unwrap();
    assert_eq!(navigation.style.overflow.x, argui::ui::Overflow::Auto);
    assert_eq!(navigation.style.overflow.y, argui::ui::Overflow::Hidden);
    assert_eq!(
        navigation.scroll.as_ref().map(|config| config.axes),
        Some(argui::ui::ScrollAxes::Horizontal)
    );
    assert!(has_key(navigation, "nav::button"));
    assert!(has_key(navigation, "theme-mode"));
    let content = find_key(&compact, "gallery-content-scroll").unwrap();
    assert_eq!(content.style.padding.left, length(14.0));
    assert_eq!(content.style.padding.right, length(14.0));
    assert_eq!(content.style.overflow.x, argui::ui::Overflow::Auto);

    resize_gallery(&mount, 900.0, 700.0);
    let desktop = mount.render(WindowEnvironment::default()).unwrap();
    assert!(has_key(&desktop, "gallery-sidebar"));
    assert!(!has_key(&desktop, "gallery-mobile-navigation"));
    let topbar = find_key(&desktop, "gallery-topbar").unwrap();
    assert_eq!(topbar.style.padding.right, length(126.0));
}

fn resize_gallery(mount: &argui::runtime::Mount<WidgetGallery>, width: f32, height: f32) {
    mount
        .layout_changed(&LayoutSnapshot {
            viewport: argui::core::Rect::new(
                Point::default(),
                argui::core::Size::new(width, height),
            ),
            nodes: Vec::new(),
        })
        .unwrap();
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

fn pointer_click<A: argui::runtime::Render>(app: &argui::runtime::Mount<A>, key: &str) {
    let mut tree = UiTree::new(app.render(Default::default()).unwrap());
    let mut text = argui::text::TextEngine::new();
    let mut engine = argui::layout::LayoutEngine::new();
    let mut output = engine
        .compute(&mut tree, &mut text, argui::core::Size::new(1_254.0, 707.0))
        .unwrap();
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap_or_else(|| panic!("missing pointer target {key}"));
    let mut ancestor = tree.parent_of(target);
    while let Some(parent) = ancestor {
        if let Some(region) = output
            .scroll_regions
            .iter()
            .find(|region| region.node == parent)
        {
            let bounds = output
                .nodes
                .iter()
                .find(|node| node.node == target)
                .unwrap()
                .bounds;
            let delta = if bounds.origin.y < region.clip.origin.y {
                bounds.origin.y - region.clip.origin.y
            } else {
                (bounds.origin.y + bounds.size.height
                    - region.clip.origin.y
                    - region.clip.size.height)
                    .max(0.0)
            };
            let mut offset = tree.scroll_offset(parent);
            offset.y = (offset.y + delta).clamp(0.0, region.max_offset.y);
            tree.set_scroll_offset(parent, offset);
            engine.apply_scroll(&tree, &mut output).unwrap();
        }
        ancestor = tree.parent_of(parent);
    }
    let region = output
        .hit_regions
        .iter()
        .find(|region| tree.key(region.node) == Some(key))
        .unwrap_or_else(|| panic!("missing hit region {key}"));
    let point = Point::new(
        region.bounds.origin.x + region.bounds.size.width * 0.5,
        region.bounds.origin.y + region.bounds.size.height * 0.5,
    );
    for (phase, buttons) in [(PointerPhase::Pressed, 1), (PointerPhase::Released, 0)] {
        let update = tree.pointer_event(
            PointerEvent {
                button: Some(argui::core::PointerButton::Primary),
                buttons,
                ..PointerEvent::mouse(phase, point)
            },
            &output.hit_regions,
        );
        for event in update.events {
            if event.should_dispatch() {
                app.dispatch_event(&event).unwrap();
            }
        }
    }
}
