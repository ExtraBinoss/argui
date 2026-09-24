use argui_core::{
    Affine2D, Point, PointerButton, PointerEvent, PointerId, PointerKind, PointerPhase, Rect, Size,
};
use argui_paint::ClipChain;
use argui_schema::{NativeElementInput, NativeEventValue, SchemaError, SchemaValue, builtin};
use argui_ui::{
    CursorIcon, EventFilter, EventHandler, EventHandlerId, EventOwnerId, EventType, FocusPolicy,
    GestureDelivery, GesturePhase, HitRegion, HitShape, Sides, UiEventKind, UiTree, UserSelect,
};

#[test]
fn touch_area_declares_read_only_state_and_rejects_assignments() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::TOUCH_AREA).unwrap();
    for property in [
        builtin::HAS_HOVER,
        builtin::PRESSED,
        builtin::MOUSE_X,
        builtin::MOUSE_Y,
        builtin::PRESSED_X,
        builtin::PRESSED_Y,
    ] {
        assert!(
            schema
                .properties
                .iter()
                .any(|entry| entry.id == property && entry.read_only)
        );
    }
    let error = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new().property(builtin::PRESSED, SchemaValue::Bool(true)),
        )
        .unwrap_err();
    assert!(matches!(error, SchemaError::ReadOnlyProperty(_)));
}

#[test]
fn touch_area_keeps_visual_style_out_of_its_schema_and_binds_pointer_events() {
    let registry = builtin::registry().unwrap();
    let schema = registry.schema(builtin::TOUCH_AREA).unwrap();
    assert!(
        !schema
            .properties
            .iter()
            .any(|entry| entry.name.as_str() == "background")
    );
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 1));
    let area = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new()
                .property(builtin::ENABLED, SchemaValue::Bool(false))
                .property(
                    builtin::MOUSE_CURSOR,
                    SchemaValue::String("crosshair".into()),
                )
                .event(NativeEventValue::new(builtin::MOVED, handler))
                .event(NativeEventValue::new(builtin::DOUBLE_CLICK, handler)),
        )
        .unwrap();
    let interaction = area.interaction.as_ref().unwrap();
    assert!(!interaction.enabled);
    assert_eq!(interaction.cursor, CursorIcon::Crosshair);
    assert!(interaction.capture_on_press);
    assert_eq!(area.user_select, UserSelect::None);
    assert!(area.paint.quad.background.is_none());
    assert!(area.event_listeners.iter().any(|listener| {
        listener.event == EventType::PointerMove
            && listener.options.filter == EventFilter::PressedPointerMove
    }));
    assert!(area.event_listeners.iter().any(|listener| {
        listener.event == EventType::Click && listener.options.filter == EventFilter::DoubleClick
    }));
}

/// Twelve raw moves produce twelve pointer-move callbacks but one coalesced drag callback per frame.
#[test]
fn touch_area_drag_coalesces_pointer_samples_per_frame() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 1));
    let area = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new()
                .event(NativeEventValue::new(builtin::MOVED, handler))
                .event(NativeEventValue::new(builtin::DRAG_X, handler)),
        )
        .unwrap();
    assert_eq!(
        area.interaction
            .as_ref()
            .unwrap()
            .gestures
            .pan
            .unwrap()
            .delivery,
        GestureDelivery::FrameCoalesced
    );
    let mut tree = UiTree::new(area);
    let node = tree.node_ids()[0];
    let region = HitRegion {
        node,
        bounds: Rect::new(Point::new(0.0, 0.0), Size::new(100.0, 100.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides {
            left: 0.0,
            right: 0.0,
            top: 0.0,
            bottom: 0.0,
        },
        enabled: true,
        focus_policy: FocusPolicy::None,
        cursor: CursorIcon::Default,
        gestures: tree
            .element_for(node)
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .gestures,
        window_drag: None,
    };
    tree.pointer_event(
        PointerEvent {
            button: Some(PointerButton::Primary),
            buttons: 1,
            ..PointerEvent::mouse(PointerPhase::Pressed, Point::new(10.0, 10.0))
        },
        std::slice::from_ref(&region),
    );
    let mut immediate_moves = 0;
    let mut immediate_drags = 0;
    for sample in 1..=12 {
        let update = tree.pointer_event(
            PointerEvent {
                buttons: 1,
                timestamp: std::time::Duration::from_millis(sample),
                ..PointerEvent::mouse(PointerPhase::Moved, Point::new(10.0 + sample as f32, 10.0))
            },
            std::slice::from_ref(&region),
        );
        immediate_moves += update
            .events
            .iter()
            .filter(|event| event.kind.event_type() == EventType::PointerMove)
            .count();
        immediate_drags += update
            .events
            .iter()
            .filter(|event| matches!(&event.kind, UiEventKind::Gesture(gesture) if gesture.phase == GesturePhase::Changed))
            .count();
    }
    assert_eq!(immediate_moves, 12);
    assert_eq!(immediate_drags, 0);
    let flushed = tree.flush_gesture_frame();
    assert_eq!(
        flushed
            .events
            .iter()
            .filter(|event| matches!(&event.kind, UiEventKind::Gesture(gesture) if gesture.phase == GesturePhase::Changed))
            .count(),
        1
    );
}

/// A tap on a passive touch area activates, while a drag can be owned by a scroll ancestor.
#[test]
fn passive_touch_area_keeps_parent_scroll_available_and_cancels_drag_activation() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(1), 1));
    let area = registry
        .construct(
            builtin::TOUCH_AREA,
            &NativeElementInput::new()
                .event(NativeEventValue::new(builtin::CLICK, handler))
                .event(NativeEventValue::new(builtin::POINTER_DOWN, handler))
                .event(NativeEventValue::new(builtin::POINTER_UP, handler)),
        )
        .unwrap();
    assert!(!area.interaction.as_ref().unwrap().capture_on_press);
    let mut tree = UiTree::new(area);
    let node = tree.node_ids()[0];
    let region = HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(100.0, 100.0)),
        transform: Affine2D::IDENTITY,
        clips: ClipChain::default(),
        shape: HitShape::Bounds,
        slop: Sides::default(),
        enabled: true,
        focus_policy: FocusPolicy::None,
        cursor: CursorIcon::Default,
        gestures: tree
            .element_for(node)
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .gestures,
        window_drag: None,
    };
    let pointer = PointerId::new(77);
    let contact = |phase, x| PointerEvent {
        id: pointer,
        kind: PointerKind::Touch,
        button: Some(PointerButton::Primary),
        buttons: u16::from(matches!(phase, PointerPhase::Pressed | PointerPhase::Moved)),
        ..PointerEvent::mouse(phase, Point::new(x, 20.0))
    };
    for (end_x, expected_click) in [(20.0, true), (60.0, false)] {
        let down = tree.pointer_event(
            contact(PointerPhase::Pressed, 20.0),
            std::slice::from_ref(&region),
        );
        let pressed = down.events.iter().find(|event| {
            matches!(
                event.kind,
                UiEventKind::Pointer(PointerEvent {
                    phase: PointerPhase::Pressed,
                    ..
                })
            )
        });
        tree.pointer_press_default(pointer, pressed, std::slice::from_ref(&region));
        assert!(!tree.pointer_captured(pointer));
        if end_x != 20.0 {
            tree.pointer_event(
                contact(PointerPhase::Moved, end_x),
                std::slice::from_ref(&region),
            );
        }
        let up = tree.pointer_event(
            contact(PointerPhase::Released, end_x),
            std::slice::from_ref(&region),
        );
        assert_eq!(
            up.events
                .iter()
                .any(|event| matches!(event.kind, UiEventKind::Click(_))),
            expected_click
        );
    }
}
