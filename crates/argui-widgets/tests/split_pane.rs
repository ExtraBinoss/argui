use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, PointerId};
use argui_ui::{
    Dimension, Element, EventHandler, EventHandlerId, EventOwnerId, FlexDirection, GestureCapture,
    GestureDelivery, GestureEvent, GestureKind, GesturePhase, HandlerValue, Orientation, PanAxis,
    Role, UiEvent, UiEventKind, UserSelect, ValueHandler,
};
use argui_widgets::{SplitAxis, SplitPane, shadcn};

fn event(key: &str, kind: UiEventKind) -> UiEvent {
    let tree = argui_ui::UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], Some(key.into()), kind)
}

fn click(key: &str, count: u8) -> UiEvent {
    event(
        key,
        UiEventKind::Click(argui_ui::ClickEvent::pointer(
            argui_core::PointerEvent::mouse(argui_core::PointerPhase::Released, Point::default()),
            count,
        )),
    )
}

fn key(key: &str, input: Key, modifiers: Modifiers, state: KeyState) -> UiEvent {
    event(
        key,
        UiEventKind::KeyInput(KeyInput {
            key: input,
            state,
            modifiers,
            repeat: false,
            text: None,
        }),
    )
}

fn pan(key: &str, phase: GesturePhase, total: Point) -> UiEvent {
    let tree = argui_ui::UiTree::new(Element::container([]));
    UiEvent::new(
        tree.node_ids()[0],
        Some(key.into()),
        UiEventKind::Gesture(GestureEvent {
            target: tree.node_ids()[0],
            pointer: PointerId::MOUSE,
            phase,
            kind: GestureKind::Pan {
                position: total,
                delta: total,
                total,
                velocity: Point::default(),
            },
            delivery: GestureDelivery::Immediate,
        }),
    )
}

#[test]
fn constructor_clamps_size_and_rejects_invalid_bounds() {
    let high = SplitPane::new("pane", SplitAxis::Horizontal, 150.0, 20.0, 100.0);
    assert_eq!(high.size, 100.0);
    assert_eq!(high.effective_size(200.0, 20.0), 100.0);
    assert_eq!(high.effective_size(20.0, 20.0), 0.0);

    let low = SplitPane::new("pane", SplitAxis::Vertical, -5.0, 20.0, 100.0);
    assert_eq!(low.size, 20.0);

    let constructors: [fn() -> SplitPane; 4] = [
        || SplitPane::new("bad", SplitAxis::Horizontal, 10.0, -1.0, 100.0),
        || SplitPane::new("bad", SplitAxis::Horizontal, 10.0, 100.0, 20.0),
        || SplitPane::new("bad", SplitAxis::Horizontal, f32::NAN, 0.0, 100.0),
        || SplitPane::new("bad", SplitAxis::Horizontal, 10.0, f32::INFINITY, 100.0),
    ];
    for constructor in constructors {
        assert!(std::panic::catch_unwind(constructor).is_err());
    }
}

#[test]
fn updates_ignore_unrelated_events_and_clamp_gesture_preferences() {
    let mut pane = SplitPane::new("divider", SplitAxis::Horizontal, 50.0, 20.0, 100.0);
    assert!(!pane.update(&click("other", 2)));
    assert!(!pane.update(&click("divider", 1)));
    assert_eq!(pane.size, 50.0);
    assert!(!pane.update(&event(
        "divider",
        UiEventKind::Gesture(GestureEvent {
            target: argui_ui::UiTree::new(Element::container([])).node_ids()[0],
            pointer: PointerId::MOUSE,
            phase: GesturePhase::Started,
            kind: GestureKind::Tap {
                position: Point::default(),
            },
            delivery: GestureDelivery::Immediate,
        }),
    )));

    assert!(pane.update(&pan(
        "divider",
        GesturePhase::Started,
        Point::new(15.0, 4.0)
    )));
    assert_eq!(pane.size, 65.0);
    assert!(pane.update(&pan(
        "divider",
        GesturePhase::Changed,
        Point::new(100.0, 0.0)
    )));
    assert_eq!(pane.size, 100.0);
    assert!(!pane.update(&pan(
        "divider",
        GesturePhase::Cancelled,
        Point::new(-50.0, 0.0)
    )));
    assert_eq!(pane.size, 100.0);
    assert!(!pane.update(&pan(
        "divider",
        GesturePhase::Changed,
        Point::new(-50.0, 0.0)
    )));
    assert_eq!(pane.size, 100.0);

    assert!(pane.update(&click("divider", 2)));
    assert_eq!(pane.size, 50.0);
    assert!(!pane.update(&pan("divider", GesturePhase::Started, Point::new(0.0, 0.0))));
    assert!(!pane.update(&pan("divider", GesturePhase::Ended, Point::new(0.0, 0.0))));
    assert_eq!(pane.size, 50.0);
}

#[test]
fn keyboard_resize_uses_axis_direction_shift_step_and_edges() {
    let mut horizontal = SplitPane::new("h", SplitAxis::Horizontal, 50.0, 20.0, 100.0);
    assert!(horizontal.update(&key(
        "h",
        Key::ArrowRight,
        Modifiers::default(),
        KeyState::Pressed,
    )));
    assert_eq!(horizontal.size, 60.0);
    assert!(horizontal.update(&key(
        "h",
        Key::ArrowLeft,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        KeyState::Pressed,
    )));
    assert_eq!(horizontal.size, 59.0);
    assert!(!horizontal.update(&key(
        "h",
        Key::ArrowDown,
        Modifiers::default(),
        KeyState::Pressed,
    )));
    assert!(!horizontal.update(&key(
        "h",
        Key::ArrowRight,
        Modifiers::default(),
        KeyState::Released,
    )));
    assert!(horizontal.update(&key(
        "h",
        Key::Home,
        Modifiers::default(),
        KeyState::Pressed,
    )));
    assert_eq!(horizontal.size, 20.0);
    assert!(horizontal.update(&key("h", Key::End, Modifiers::default(), KeyState::Pressed,)));
    assert_eq!(horizontal.size, 100.0);

    let mut trailing = SplitPane::new("v", SplitAxis::Vertical, 50.0, 20.0, 100.0).trailing(true);
    assert!(trailing.update(&key(
        "v",
        Key::ArrowUp,
        Modifiers::default(),
        KeyState::Pressed,
    )));
    assert_eq!(trailing.size, 60.0);
    assert!(trailing.update(&key(
        "v",
        Key::ArrowDown,
        Modifiers {
            shift: true,
            ..Modifiers::default()
        },
        KeyState::Pressed,
    )));
    assert_eq!(trailing.size, 59.0);
    assert!(!trailing.update(&key(
        "v",
        Key::Other,
        Modifiers::default(),
        KeyState::Pressed,
    )));
}

#[test]
fn trailing_vertical_pan_moves_from_the_opposite_edge() {
    let mut pane = SplitPane::new("v", SplitAxis::Vertical, 50.0, 20.0, 100.0).trailing(true);
    assert!(pane.update(&pan("v", GesturePhase::Started, Point::new(4.0, 10.0))));
    assert_eq!(pane.size, 40.0);
    assert!(pane.update(&pan("v", GesturePhase::Ended, Point::new(0.0, -10.0))));
    assert_eq!(pane.size, 60.0);
}

#[test]
fn controlled_split_delivers_changed_values_without_an_ended_rollback() {
    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let handler = ValueHandler::from_handler(EventHandler::from_identity(EventHandlerId::new(
        EventOwnerId(1),
        0,
    )));
    let pane = SplitPane::new("v", SplitAxis::Vertical, 100.0, 20.0, 200.0)
        .trailing(true)
        .on_change(handler);
    let mut tree = argui_ui::UiTree::new(pane.separator(theme));
    let target = tree.node_ids()[0];
    let gesture = |phase, delta| {
        UiEventKind::Gesture(GestureEvent {
            target,
            pointer: PointerId::MOUSE,
            phase,
            kind: GestureKind::Pan {
                position: Point::default(),
                delta,
                total: delta,
                velocity: Point::default(),
            },
            delivery: GestureDelivery::FrameCoalesced,
        })
    };

    let changed = tree.event_deliveries(
        target,
        gesture(GesturePhase::Changed, Point::new(0.0, -70.0)),
    );
    assert!(matches!(
        changed[0].handler_value(),
        Some(HandlerValue::Number(170.0))
    ));
    assert!(
        tree.event_deliveries(target, gesture(GesturePhase::Ended, Point::default()))
            .is_empty()
    );
}

#[test]
fn separator_and_layout_expose_axis_accessibility_and_fixed_side() {
    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let horizontal = SplitPane::new("horizontal", SplitAxis::Horizontal, 120.0, 40.0, 200.0);
    let separator = horizontal.separator(theme);
    assert_eq!(separator.key.as_deref(), Some("horizontal"));
    assert_eq!(separator.style.size.width, Dimension::length(6.0));
    assert_eq!(separator.style.size.height, Dimension::percent(1.0));
    assert_eq!(separator.user_select, UserSelect::None);
    assert_eq!(
        separator.interaction.as_ref().unwrap().cursor,
        argui_ui::CursorIcon::EwResize
    );
    let gesture = separator
        .interaction
        .as_ref()
        .unwrap()
        .gestures
        .pan
        .unwrap();
    assert_eq!(gesture.axis, PanAxis::Horizontal);
    assert_eq!(gesture.capture, GestureCapture::OnPress);
    assert_eq!(gesture.delivery, GestureDelivery::FrameCoalesced);
    let semantics = separator.semantics.as_ref().unwrap();
    assert_eq!(semantics.role, Role::Separator);
    assert_eq!(semantics.orientation, Some(Orientation::Vertical));
    assert_eq!(
        semantics.value,
        Some(argui_ui::SemanticValue::Number {
            value: 120.0,
            minimum: Some(40.0),
            maximum: Some(200.0),
            step: Some(10.0),
        })
    );

    let root = horizontal.build(
        Element::text("first"),
        separator,
        Element::text("second"),
        180.0,
        20.0,
    );
    assert_eq!(root.style.flex_direction, FlexDirection::Row);
    assert_eq!(root.children[0].style.size.width, Dimension::length(120.0));
    assert_eq!(root.children[2].style.flex_grow, 1.0);

    let vertical =
        SplitPane::new("vertical", SplitAxis::Vertical, 80.0, 20.0, 100.0).trailing(true);
    let vertical_separator = vertical.separator(theme);
    assert_eq!(vertical_separator.style.size.width, Dimension::percent(1.0));
    assert_eq!(vertical_separator.style.size.height, Dimension::length(6.0));
    assert_eq!(
        vertical_separator.interaction.as_ref().unwrap().cursor,
        argui_ui::CursorIcon::NsResize
    );
    assert_eq!(
        vertical_separator.semantics.as_ref().unwrap().orientation,
        Some(Orientation::Horizontal)
    );
    let vertical_root = vertical.build(
        Element::text("first"),
        vertical_separator,
        Element::text("second"),
        150.0,
        20.0,
    );
    assert_eq!(vertical_root.style.flex_direction, FlexDirection::Column);
    assert_eq!(vertical_root.children[0].style.flex_grow, 1.0);
    assert_eq!(
        vertical_root.children[2].style.size.height,
        Dimension::length(80.0)
    );
}
