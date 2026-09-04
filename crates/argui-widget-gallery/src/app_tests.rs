use argui::{
    core::{Key, KeyInput, KeyState, Modifiers, Point, Rect, Size},
    layout::LayoutEngine,
    runtime::{Context, LayoutBounds, Render, ViewUpdate},
    text::TextEngine,
    theme::ThemeMode,
    ui::{GestureEvent, GestureKind, GesturePhase, UiEvent, UiEventKind, UiTree},
};

use super::{PRIMARIES, WidgetGallery};
use crate::navigation::Page;

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(argui::ui::Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}

fn pressed(key: Key, modifiers: Modifiers) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key,
        state: KeyState::Pressed,
        modifiers,
        repeat: false,
        text: None,
    })
}

fn dispatch(gallery: &mut WidgetGallery, event: &UiEvent) -> ViewUpdate {
    let mut cx = Context::default();
    Render::event(gallery, event, &mut cx);
    cx.view_update()
}

fn keyed(root: &argui::ui::Element, key: &str) -> bool {
    root.key.as_deref() == Some(key) || root.children.iter().any(|child| keyed(child, key))
}

#[test]
fn every_gallery_route_builds_real_widget_content() {
    let mut gallery = WidgetGallery::default();
    for page in Page::ALL {
        gallery.page = page;
        let root = Render::render(&mut gallery, &mut Context::default());
        assert!(keyed(&root, "gallery-root"));
        assert!(keyed(&root, "gallery-search"));
    }
    assert_eq!(gallery.image_assets().len(), 1);
    assert_eq!(gallery.vector_assets().len(), 33);
}

#[test]
fn gallery_document_text_remains_selectable_under_its_interactive_window_root() {
    let mut gallery = WidgetGallery::default();
    let root = Render::render(&mut gallery, &mut Context::default());
    let mut tree = UiTree::new(root);

    let button = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("demo-button"))
        .unwrap();
    assert_eq!(
        tree.resolved_user_select(button),
        argui::ui::UserSelect::None
    );

    tree.select_all_document_text();
    let selected = tree.selected_document_text().unwrap();
    assert!(selected.contains("Button"));
    assert!(selected.contains("Actions with variants"));
}

#[test]
fn gallery_scroll_viewport_ends_inside_the_window() {
    let mut gallery = WidgetGallery {
        page: Page::Composition,
        ..WidgetGallery::default()
    };
    let root = Render::render(&mut gallery, &mut Context::default());
    let images = Render::image_assets(&gallery);
    let vectors = Render::vector_assets(&gallery);
    let mut tree = UiTree::new(root);
    let scroll_node = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("gallery-content-scroll"))
        .unwrap();
    let mut layout = LayoutEngine::new();
    layout.set_assets(&images, &vectors);
    let mut text =
        TextEngine::from_embedded_fonts([crate::NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");

    let output = layout
        .compute(&mut tree, &mut text, Size::new(1_220.0, 780.0))
        .unwrap();
    let scroll = output
        .scroll_regions
        .iter()
        .find(|region| region.node == scroll_node)
        .unwrap();
    let bottom = scroll.bounds.origin.y + scroll.bounds.size.height;
    assert!(scroll.bounds.origin.y >= 64.0);
    assert!(bottom <= 780.0);
    if let Some(scrollbar) = scroll
        .scrollbar
        .as_ref()
        .and_then(|scrollbar| scrollbar.vertical.as_ref())
    {
        assert!(scrollbar.track.origin.y + scrollbar.track.size.height <= 780.0);
    }
}

#[test]
fn controlled_values_navigation_theme_and_search_update_state() {
    let mut gallery = WidgetGallery::default();
    for (key, value) in [
        ("gallery-search", "slide"),
        ("name", "Grace Hopper"),
        ("email", "grace@example.com"),
        ("notes", "Compiler pioneer"),
    ] {
        assert_eq!(
            dispatch(
                &mut gallery,
                &event(key, UiEventKind::TextChanged(value.into()))
            ),
            ViewUpdate::Rebuild
        );
    }
    assert_eq!(gallery.search, "slide");
    assert_eq!(gallery.name, "Grace Hopper");
    assert_eq!(gallery.email, "grace@example.com");
    assert_eq!(gallery.notes, "Compiler pioneer");

    dispatch(&mut gallery, &event("nav::tabs", UiEventKind::Clicked));
    assert_eq!(gallery.page, Page::Tabs);
    dispatch(&mut gallery, &event("theme-mode", UiEventKind::Clicked));
    assert_eq!(gallery.theme_mode, ThemeMode::Light);
    dispatch(&mut gallery, &event("primary::4", UiEventKind::Clicked));
    assert_eq!(gallery.primary, 4);
    assert_eq!(gallery.theme_request().primary, Some(PRIMARIES[4]));
}

#[test]
fn widget_events_are_decoded_by_their_public_control_protocols() {
    let mut gallery = WidgetGallery::default();
    for key in [
        "demo-button",
        "composition-circle",
        "accepted",
        "notifications",
    ] {
        dispatch(&mut gallery, &event(key, UiEventKind::Clicked));
    }
    assert_eq!(gallery.clicks, 1);
    assert_eq!(gallery.composition_hits, 1);
    assert!(!gallery.accepted);
    assert!(!gallery.notifications);

    dispatch(
        &mut gallery,
        &event("quality::option::2", UiEventKind::Clicked),
    );
    dispatch(
        &mut gallery,
        &event("demo-tabs::tab::2", UiEventKind::Clicked),
    );
    assert_eq!((gallery.radio, gallery.tab), (2, 2));

    dispatch(&mut gallery, &event("backend", UiEventKind::Clicked));
    assert!(gallery.select_open);
    dispatch(
        &mut gallery,
        &event("backend", pressed(Key::ArrowDown, Modifiers::default())),
    );
    assert_eq!(gallery.select_highlight, 1);
    dispatch(
        &mut gallery,
        &event("backend::option::1", UiEventKind::Clicked),
    );
    assert_eq!(gallery.select_selected, Some(1));
    assert!(!gallery.select_open);

    dispatch(
        &mut gallery,
        &event("demo-dialog::trigger", UiEventKind::Clicked),
    );
    assert!(gallery.dialog_open);
    dispatch(
        &mut gallery,
        &event(
            "demo-dialog::panel",
            pressed(Key::Escape, Modifiers::default()),
        ),
    );
    assert!(!gallery.dialog_open);
}

#[test]
fn shortcuts_slider_layout_and_resize_use_engine_requests() {
    let mut gallery = WidgetGallery::default();
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    dispatch(
        &mut gallery,
        &event("gallery-root", pressed(Key::Character("k".into()), command)),
    );
    let theme_shortcut = Modifiers {
        control: true,
        shift: true,
        ..Modifiers::default()
    };
    dispatch(
        &mut gallery,
        &event(
            "gallery-root",
            pressed(Key::Character("l".into()), theme_shortcut),
        ),
    );
    assert_eq!(gallery.theme_mode, ThemeMode::Light);

    let node = UiTree::new(argui::ui::Element::container([])).node_ids()[0];
    Render::layout_changed(
        &mut gallery,
        &argui::runtime::LayoutSnapshot {
            viewport: Rect::default(),
            nodes: vec![
                LayoutBounds {
                    node,
                    key: Some("property-slider::track".into()),
                    bounds: Rect::new(Point::default(), Size::new(200.0, 28.0)),
                },
                LayoutBounds {
                    node,
                    key: Some("plain-slider::track".into()),
                    bounds: Rect::new(Point::default(), Size::new(200.0, 28.0)),
                },
            ],
        },
        &mut Context::default(),
    );
    dispatch(
        &mut gallery,
        &event(
            "property-slider",
            pressed(Key::ArrowRight, Modifiers::default()),
        ),
    );
    assert_eq!(gallery.slider, 65.0);

    let range_pan = |phase, position| {
        event(
            "property-slider",
            UiEventKind::Gesture(GestureEvent {
                target: node,
                phase,
                kind: GestureKind::Pan {
                    position,
                    delta: Point::default(),
                    total: Point::default(),
                    velocity: Point::default(),
                },
            }),
        )
    };
    dispatch(
        &mut gallery,
        &range_pan(GesturePhase::Started, Point::new(150.0, 14.0)),
    );
    assert_eq!(gallery.slider, 75.0);
    dispatch(
        &mut gallery,
        &range_pan(GesturePhase::Changed, Point::new(40.0, 14.0)),
    );
    assert_eq!(gallery.slider, 20.0);
    dispatch(
        &mut gallery,
        &event(
            "plain-slider",
            UiEventKind::Gesture(GestureEvent {
                target: node,
                phase: GesturePhase::Started,
                kind: GestureKind::Pan {
                    position: Point::new(130.0, 14.0),
                    delta: Point::default(),
                    total: Point::default(),
                    velocity: Point::default(),
                },
            }),
        ),
    );
    assert_eq!(gallery.plain_slider, 65.0);

    let resize = |phase, total| {
        event(
            "notes-resize",
            UiEventKind::Gesture(GestureEvent {
                target: node,
                phase,
                kind: GestureKind::Pan {
                    position: total,
                    delta: total,
                    total,
                    velocity: Point::default(),
                },
            }),
        )
    };
    dispatch(
        &mut gallery,
        &resize(GesturePhase::Started, Point::default()),
    );
    dispatch(
        &mut gallery,
        &resize(GesturePhase::Changed, Point::new(40.0, 30.0)),
    );
    assert_eq!(gallery.editor_size.size(), Size::new(560.0, 200.0));
}

#[test]
fn property_slider_supports_direct_edit_cancel_and_reset() {
    let mut gallery = WidgetGallery::default();
    dispatch(
        &mut gallery,
        &event("property-slider::edit", UiEventKind::Clicked),
    );
    assert!(gallery.slider_editing);
    dispatch(
        &mut gallery,
        &event(
            "property-slider::input",
            UiEventKind::TextChanged("83".into()),
        ),
    );
    dispatch(
        &mut gallery,
        &event(
            "property-slider::input",
            UiEventKind::Submitted("83".into()),
        ),
    );
    assert_eq!(gallery.slider, 83.0);
    assert!(!gallery.slider_editing);

    dispatch(
        &mut gallery,
        &event("property-slider::edit", UiEventKind::Clicked),
    );
    dispatch(
        &mut gallery,
        &event(
            "property-slider::input",
            pressed(Key::Escape, Modifiers::default()),
        ),
    );
    assert!(!gallery.slider_editing);
    assert_eq!(gallery.slider, 83.0);

    dispatch(
        &mut gallery,
        &event("property-slider::reset", UiEventKind::Clicked),
    );
    assert_eq!(gallery.slider, 50.0);
}
