use argui::{
    core::{
        Key, KeyInput, KeyState, Modifiers, Point, PointerButton, PointerEvent, PointerPhase, Rect,
        Size,
    },
    layout::LayoutEngine,
    runtime::{Context, Entity, LayoutBounds, LayoutSnapshot, Render},
    text::TextEngine,
    ui::{
        ClickEvent, Element, ElementKind, GestureDelivery, GestureEvent, GestureKind, GesturePhase,
        UiEventKind, UiTree,
    },
    widgets::SelectionHost,
};
use argui_widget_gallery::WidgetGallery;

struct RoutedWindow<A: Render> {
    app: Entity<A>,
}

impl<A: Render> RoutedWindow<A> {
    fn new(app: A) -> Self {
        Self {
            app: Entity::new(app),
        }
    }
}

impl<A: Render> Render for RoutedWindow<A> {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let root = self.app.render_in(cx.environment());
        cx.route_events_to(self.app.erase());
        root
    }
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

fn contains_text(element: &Element, value: &str) -> bool {
    matches!(&element.kind, ElementKind::Text { content, .. } if content.as_str().contains(value))
        || element
            .children
            .iter()
            .any(|child| contains_text(child, value))
}

fn dispatch(app: &Entity<WidgetGallery>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .unwrap_or_else(|| panic!("missing event target {key}"));
    let kind = match kind {
        UiEventKind::Gesture(mut gesture) => {
            gesture.target = target;
            UiEventKind::Gesture(gesture)
        }
        kind => kind,
    };
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn click(app: &Entity<WidgetGallery>, key: &str) {
    dispatch(app, key, UiEventKind::Click(ClickEvent::accessibility()));
}

fn pointer_click<A: Render>(app: &Entity<A>, key: &str) {
    let mut tree = UiTree::new(app.render());
    let mut text = TextEngine::new();
    let output = LayoutEngine::new()
        .compute(&mut tree, &mut text, Size::new(1_254.0, 707.0))
        .unwrap();
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
                button: Some(PointerButton::Primary),
                buttons,
                ..PointerEvent::mouse(phase, point)
            },
            &output.hit_regions,
        );
        for event in update.events {
            if event.should_dispatch() {
                app.dispatch_event(&event);
            }
        }
    }
}

fn key(key: Key, modifiers: Modifiers) -> UiEventKind {
    UiEventKind::KeyInput(KeyInput {
        key,
        state: KeyState::Pressed,
        modifiers,
        repeat: false,
        text: None,
    })
}

#[test]
fn search_keyboard_theme_and_primary_are_observable_through_the_public_tree() {
    let app = Entity::new(WidgetGallery::default());
    dispatch(
        &app,
        "gallery-search",
        UiEventKind::TextChanged("slide".into()),
    );
    let filtered = app.render();
    assert!(find_key(&filtered, "nav::slider").is_some());
    assert!(find_key(&filtered, "nav::button").is_none());

    for navigation in [
        Key::ArrowDown,
        Key::ArrowUp,
        Key::End,
        Key::Home,
        Key::Enter,
    ] {
        dispatch(
            &app,
            "gallery-search",
            key(navigation, Modifiers::default()),
        );
    }
    assert!(contains_text(&app.render(), "Slider"));

    dispatch(
        &app,
        "gallery-search",
        UiEventKind::TextChanged("no matching page".into()),
    );
    dispatch(
        &app,
        "gallery-search",
        key(Key::ArrowDown, Modifiers::default()),
    );
    dispatch(
        &app,
        "gallery-search",
        UiEventKind::TextChanged(String::new()),
    );

    click(&app, "theme-mode");
    click(&app, "theme-mode");
    click(&app, "theme-mode");
    for index in 0..6 {
        click(&app, &format!("primary::{index}"));
    }
    assert!(find_key(&app.render(), "gallery-root").is_some());
}

#[test]
fn application_shortcuts_and_control_protocols_drive_real_widget_state() {
    let app = Entity::new(WidgetGallery::default());
    let command = Modifiers {
        control: true,
        ..Modifiers::default()
    };
    dispatch(
        &app,
        "gallery-root",
        key(Key::Character("k".into()), command),
    );
    dispatch(
        &app,
        "gallery-root",
        key(
            Key::Character("l".into()),
            Modifiers {
                control: true,
                shift: true,
                ..Modifiers::default()
            },
        ),
    );
    dispatch(
        &app,
        "gallery-root",
        key(Key::Character("x".into()), command),
    );

    click(&app, "demo-button");
    assert!(contains_text(&app.render(), "Primary activations: 1"));

    click(&app, "nav::checkbox");
    click(&app, "accepted");
    click(&app, "nav::switch");
    click(&app, "notifications");
    click(&app, "nav::radio-group");
    click(&app, "quality::option::2");
    click(&app, "nav::tabs");
    click(&app, "demo-tabs::tab::2");
    assert!(find_key(&app.render(), "demo-tabs::panel::2").is_some());
}

#[test]
fn sidebar_navigation_works_through_layout_hit_testing_and_pointer_routing() {
    let app = Entity::new(RoutedWindow::new(SelectionHost::new(
        WidgetGallery::default(),
    )));
    pointer_click(&app, "nav::slider");

    let rendered = app.render();
    assert!(find_key(&rendered, "property-slider").is_some());

    pointer_click(&app, "nav::select");
    pointer_click(&app, "backend");
    assert!(find_key(&app.render(), "backend::list").is_some());
}

#[test]
fn select_and_dialog_follow_their_complete_public_event_protocols() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::select");
    click(&app, "backend");
    assert!(find_key(&app.render(), "backend::list").is_some());
    for navigation in [Key::ArrowDown, Key::ArrowUp, Key::Home, Key::End] {
        dispatch(&app, "backend", key(navigation, Modifiers::default()));
    }
    click(&app, "backend::option::1");
    assert!(find_key(&app.render(), "backend::list").is_none());
    assert!(contains_text(&app.render(), "DirectX 12"));

    click(&app, "backend");
    dispatch(&app, "backend", key(Key::Escape, Modifiers::default()));
    assert!(find_key(&app.render(), "backend::list").is_none());

    click(&app, "nav::dialog");
    click(&app, "demo-dialog::trigger");
    assert!(find_key(&app.render(), "demo-dialog::panel").is_some());
    dispatch(
        &app,
        "demo-dialog::panel",
        key(Key::Escape, Modifiers::default()),
    );
    assert!(find_key(&app.render(), "demo-dialog::panel").is_none());
    click(&app, "demo-dialog::trigger");
    click(&app, "demo-dialog::close");
    assert!(find_key(&app.render(), "demo-dialog::panel").is_none());
}

fn update_layout(app: &Entity<WidgetGallery>) {
    let root = app.render();
    let tree = UiTree::new(root);
    let slider = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("property-slider::track"))
        .unwrap();
    let plain = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("plain-slider::track"))
        .unwrap();
    app.update(|gallery, cx| {
        gallery.layout_changed(
            &LayoutSnapshot {
                viewport: Rect::new(Point::default(), Size::new(800.0, 600.0)),
                nodes: vec![
                    LayoutBounds {
                        node: slider,
                        key: Some("property-slider::track".into()),
                        bounds: Rect::new(Point::default(), Size::new(200.0, 28.0)),
                    },
                    LayoutBounds {
                        node: plain,
                        key: Some("plain-slider::track".into()),
                        bounds: Rect::new(Point::default(), Size::new(200.0, 28.0)),
                    },
                ],
            },
            cx,
        );
    });
}

fn pan(position: Point, phase: GesturePhase) -> UiEventKind {
    UiEventKind::Gesture(GestureEvent {
        target: UiTree::new(Element::container([])).node_id_at(0).unwrap(),
        pointer: argui::core::PointerId::MOUSE,
        phase,
        kind: GestureKind::Pan {
            position,
            delta: Point::default(),
            total: Point::default(),
            velocity: Point::default(),
        },
        delivery: GestureDelivery::Immediate,
    })
}

#[test]
fn composed_and_plain_sliders_share_layout_pointer_and_keyboard_behavior() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::slider");
    update_layout(&app);
    dispatch(
        &app,
        "property-slider",
        key(Key::ArrowRight, Modifiers::default()),
    );
    dispatch(
        &app,
        "property-slider",
        pan(Point::new(150.0, 14.0), GesturePhase::Started),
    );
    dispatch(
        &app,
        "property-slider",
        pan(Point::new(40.0, 14.0), GesturePhase::Changed),
    );
    assert!(contains_text(&app.render(), "20%"));

    dispatch(
        &app,
        "plain-slider",
        pan(Point::new(130.0, 14.0), GesturePhase::Started),
    );
    dispatch(&app, "plain-slider", key(Key::End, Modifiers::default()));
    assert!(find_key(&app.render(), "plain-slider").is_some());
}

#[test]
fn editable_property_accepts_arithmetic_and_keeps_invalid_edits_safe() {
    let app = Entity::new(WidgetGallery::default());
    click(&app, "nav::composition");

    for (expression, expected) in [
        ("(20 + 5) * 3", "75%"),
        ("100 / 4 + 6 % 4", "27%"),
        ("-10 + +30", "20%"),
        ("200", "100%"),
    ] {
        click(&app, "property-slider::edit");
        dispatch(
            &app,
            "property-slider::input",
            UiEventKind::TextChanged(expression.into()),
        );
        dispatch(
            &app,
            "property-slider::input",
            UiEventKind::Submitted(expression.into()),
        );
        assert!(contains_text(&app.render(), expected), "missing {expected}");
    }

    click(&app, "property-slider::edit");
    dispatch(
        &app,
        "property-slider::input",
        UiEventKind::TextChanged("not a number".into()),
    );
    dispatch(&app, "property-slider::input", UiEventKind::Blurred);
    assert!(contains_text(&app.render(), "100%"));

    click(&app, "property-slider::edit");
    dispatch(
        &app,
        "property-slider::input",
        key(Key::Escape, Modifiers::default()),
    );
    click(&app, "property-slider::reset");
    assert!(contains_text(&app.render(), "50%"));
}
