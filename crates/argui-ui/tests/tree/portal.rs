use argui_core::{
    Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase, Rect, Size,
};
use argui_ui::{
    DismissPolicy, Element, EventHandlerId, EventListener, EventOwnerId, EventType, FocusPolicy,
    FocusRequest, FocusScope, GestureSet, HitRegion, InitialFocus, Interaction, OverlaySurface,
    UiEventKind, UiTree, ViewportPlacement, WindowLayer,
};

#[test]
fn nested_portals_inherit_preferences_but_keep_their_physical_owner() {
    let mut ui = UiTree::new(Element::column([
        Element::column([
            Element::container([])
                .portal(WindowLayer::Popover)
                .keyed("inherited"),
            Element::container([])
                .portal(WindowLayer::Popover)
                .portal_surface(OverlaySurface::InWindow)
                .keyed("internal"),
        ])
        .portal(WindowLayer::Popover)
        .portal_surface(OverlaySurface::PreferNative)
        .keyed("outer"),
        Element::container([])
            .portal(WindowLayer::Popover)
            .keyed("sibling"),
    ]));
    let node = |key| {
        *ui.node_ids()
            .iter()
            .find(|id| ui.key(**id) == Some(key))
            .unwrap()
    };
    let outer = node("outer");
    let inherited = node("inherited");
    let internal = node("internal");
    let sibling = node("sibling");
    assert_eq!(
        ui.portal_surface_preference(inherited),
        OverlaySurface::PreferNative
    );
    assert_eq!(
        ui.portal_surface_preference(internal),
        OverlaySurface::InWindow
    );
    assert_eq!(
        ui.portal_surface_preference(sibling),
        OverlaySurface::InWindow
    );
    let bounds = Rect::new(Point::new(300.0, -40.0), Size::new(220.0, 150.0));
    assert!(ui.set_native_portal(outer, Some(bounds)));
    assert!(!ui.set_native_portal(outer, Some(bounds)));
    assert_eq!(ui.native_portal_owner(inherited), Some(outer));
    assert_eq!(ui.native_portal_owner(internal), Some(outer));
    assert_eq!(ui.native_portal_owner(sibling), None);
    assert!(ui.set_native_portal(inherited, Some(bounds)));
    assert_eq!(ui.native_portal_owner(inherited), Some(inherited));
    assert!(ui.set_native_portal(inherited, None));
    assert_eq!(ui.native_portal_owner(inherited), Some(outer));
    assert!(ui.set_native_portal(outer, None));
    assert_eq!(ui.native_portal_owner(internal), None);
    assert!(!ui.set_native_portal(sibling, Some(Rect::default())));
    assert!(!ui.set_native_portal(ui.node_ids()[0], Some(bounds)));
    for invalid in [
        Rect::new(Point::new(f32::NAN, 0.0), bounds.size),
        Rect::new(Point::new(0.0, f32::INFINITY), bounds.size),
        Rect::new(Point::default(), Size::new(f32::NAN, 2.0)),
        Rect::new(Point::default(), Size::new(2.0, f32::INFINITY)),
        Rect::new(Point::default(), Size::new(-2.0, 1.0)),
    ] {
        assert!(!ui.set_native_portal(sibling, Some(invalid)));
    }
    ui.set_native_portal(outer, Some(bounds));
    ui.update(Element::column([]));
    assert!(!ui.set_native_portal(outer, Some(bounds)));
    assert_eq!(ui.native_portal_bounds(outer), None);
    assert_eq!(ui.native_portal_owner(outer), None);
    assert_eq!(
        ui.portal_surface_preference(outer),
        OverlaySurface::InWindow
    );
}

#[test]
fn escape_and_outside_pointer_choose_only_the_topmost_eligible_portal() {
    let dismiss_listener = |slot| {
        EventListener::new(
            EventType::Dismiss,
            EventHandlerId::new(EventOwnerId(8), slot),
        )
    };
    let outside_listener = EventListener::new(
        EventType::PointerOutside,
        EventHandlerId::new(EventOwnerId(8), 3),
    );
    let mut ui = UiTree::new(Element::column([
        Element::container([])
            .keyed("escape")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::Escape)
            .on(dismiss_listener(0)),
        Element::container([])
            .keyed("combined")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointerOrEscape)
            .on(dismiss_listener(1))
            .on(outside_listener),
        Element::container([])
            .keyed("manual")
            .portal(WindowLayer::Floating)
            .on(dismiss_listener(2)),
    ]));
    let combined = ui.node_id_at(2).unwrap();
    let key = KeyInput {
        key: Key::Escape,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
    };

    let dismiss = ui.keyboard_default(&key, &[]);
    assert!(
        dismiss.events.iter().any(|event| {
            event.target == combined && event.kind == UiEventKind::DismissRequested
        })
    );
    assert!(
        ui.keyboard_default(
            &KeyInput {
                repeat: true,
                ..key
            },
            &[]
        )
        .is_empty()
    );

    let outside = ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(500.0, 500.0)),
        &[],
    );
    assert!(outside.events.iter().any(|event| {
        event.target == combined && matches!(event.kind, UiEventKind::PointerOutside(_))
    }));
}

#[test]
fn topmost_manual_portal_shields_lower_portal_dismissal() {
    let lower = Element::container([])
        .portal(WindowLayer::Popover)
        .portal_dismiss(DismissPolicy::OutsidePointerOrEscape)
        .on(EventListener::new(
            EventType::Dismiss,
            EventHandlerId::new(EventOwnerId(9), 0),
        ))
        .on(EventListener::new(
            EventType::PointerOutside,
            EventHandlerId::new(EventOwnerId(9), 1),
        ));
    let manual = Element::container([]).portal(WindowLayer::Modal);
    let mut ui = UiTree::new(Element::container([lower, manual]));
    let escape = KeyInput {
        key: Key::Escape,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
    };

    assert!(ui.keyboard_default(&escape, &[]).events.is_empty());
    let outside = ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(500.0, 500.0)),
        &[],
    );
    assert!(
        !outside
            .events
            .iter()
            .any(|event| { matches!(event.kind, UiEventKind::PointerOutside(_)) })
    );
}

#[test]
fn click_inside_child_portal_does_not_dismiss_its_parent() {
    let child = Element::container([])
        .portal(WindowLayer::Popover)
        .portal_dismiss(DismissPolicy::OutsidePointerOrEscape);
    let parent = Element::container([child])
        .portal(WindowLayer::Popover)
        .portal_dismiss(DismissPolicy::OutsidePointerOrEscape)
        .on(EventListener::new(
            EventType::PointerOutside,
            EventHandlerId::new(EventOwnerId(10), 0),
        ));
    let mut ui = UiTree::new(Element::container([parent]));
    let child_node = ui.node_id_at(2).unwrap();
    let regions = [HitRegion {
        node: child_node,
        bounds: Rect::new(Point::default(), Size::new(40.0, 30.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: Default::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: Default::default(),
        enabled: true,
        focus_policy: FocusPolicy::None,
        cursor: argui_ui::CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    }];
    let pressed = ui.pointer_event(
        PointerEvent::mouse(PointerPhase::Pressed, Point::new(10.0, 10.0)),
        &regions,
    );
    assert!(
        !pressed
            .events
            .iter()
            .any(|event| { matches!(event.kind, UiEventKind::PointerOutside(_)) })
    );
}

#[test]
fn removing_a_focused_portal_restores_its_trigger() {
    let view = |open| {
        let mut children = vec![
            Element::container([])
                .keyed("trigger")
                .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop)),
        ];
        if open {
            children.push(
                Element::container([Element::container([])
                    .keyed("inside")
                    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))])
                .keyed("popup")
                .viewport_portal(WindowLayer::Modal, ViewportPlacement::centered())
                .focus_scope(FocusScope::trapped(InitialFocus::First)),
            );
        }
        Element::column(children)
    };
    let region = |node, x| HitRegion {
        node,
        bounds: Rect::new(Point::new(x, 0.0), Size::new(40.0, 30.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: Default::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: Default::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: argui_ui::CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    };

    let mut ui = UiTree::new(view(false));
    let trigger = ui.node_id_at(1).unwrap();
    ui.sync_focus(
        &[region(trigger, 0.0)],
        Some(FocusRequest::Focus(trigger.into())),
    );
    ui.update(view(true));
    let inside = ui.node_id_at(3).unwrap();
    ui.sync_focus(&[region(trigger, 0.0), region(inside, 50.0)], None);
    assert_eq!(ui.focused_node(), Some(inside));

    ui.update(view(false));
    ui.sync_focus(&[region(trigger, 0.0)], None);
    assert_eq!(ui.focused_node(), Some(trigger));
}

#[test]
fn escape_and_outside_closure_restore_trigger_focus() {
    let view = |open| {
        let mut children = vec![
            Element::container([])
                .keyed("trigger")
                .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop)),
        ];
        if open {
            children.push(
                Element::container([Element::container([])
                    .keyed("inside")
                    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))])
                .keyed("popup")
                .viewport_portal(WindowLayer::Popover, ViewportPlacement::centered())
                .portal_dismiss(DismissPolicy::OutsidePointerOrEscape)
                .focus_scope(FocusScope::trapped(InitialFocus::First))
                .on(EventListener::new(
                    EventType::Dismiss,
                    EventHandlerId::new(EventOwnerId(11), 0),
                ))
                .on(EventListener::new(
                    EventType::PointerOutside,
                    EventHandlerId::new(EventOwnerId(11), 1),
                )),
            );
        }
        Element::container(children)
    };
    let region = |node| HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(40.0, 30.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: Default::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: Default::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: argui_ui::CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    };
    let key = KeyInput {
        key: Key::Escape,
        state: KeyState::Pressed,
        modifiers: Modifiers::default(),
        repeat: false,
        text: None,
    };

    for escape in [true, false] {
        let mut ui = UiTree::new(view(false));
        let trigger = ui.node_id_at(1).unwrap();
        ui.sync_focus(
            &[region(trigger)],
            Some(FocusRequest::Focus(trigger.into())),
        );
        ui.update(view(true));
        let popup = ui.node_id_at(2).unwrap();
        let inside = ui.node_id_at(3).unwrap();
        ui.sync_focus(&[region(trigger), region(inside)], None);
        assert_eq!(ui.focused_node(), Some(inside));

        let dismissal = if escape {
            ui.keyboard_default(&key, &[])
        } else {
            ui.pointer_event(
                PointerEvent::mouse(PointerPhase::Pressed, Point::new(500.0, 500.0)),
                &[],
            )
        };
        assert!(dismissal.events.iter().any(|event| {
            event.target == popup
                && if escape {
                    event.kind == UiEventKind::DismissRequested
                } else {
                    matches!(event.kind, UiEventKind::PointerOutside(_))
                }
        }));
        ui.update(view(false));
        ui.sync_focus(&[region(trigger)], None);
        assert_eq!(ui.focused_node(), Some(trigger));
    }
}

#[test]
fn nested_popup_outside_close_restores_parent_then_trigger() {
    let view = |parent_open: bool, child_open: bool| {
        let trigger = Element::container([])
            .keyed("trigger")
            .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop));
        if !parent_open {
            return Element::container([trigger]);
        }
        let mut parent_children = vec![
            Element::container([])
                .keyed("parent-item")
                .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop)),
        ];
        if child_open {
            parent_children.push(
                Element::container([Element::container([])
                    .keyed("child-item")
                    .interaction(Interaction::default().focus_policy(FocusPolicy::TabStop))])
                .keyed("child-popup")
                .portal(WindowLayer::Popover)
                .portal_dismiss(DismissPolicy::OutsidePointer)
                .focus_scope(FocusScope::trapped(InitialFocus::First))
                .on(EventListener::new(
                    EventType::PointerOutside,
                    EventHandlerId::new(EventOwnerId(12), 1),
                )),
            );
        }
        let parent = Element::container(parent_children)
            .keyed("parent-popup")
            .portal(WindowLayer::Popover)
            .portal_dismiss(DismissPolicy::OutsidePointer)
            .focus_scope(FocusScope::trapped(InitialFocus::First))
            .on(EventListener::new(
                EventType::PointerOutside,
                EventHandlerId::new(EventOwnerId(12), 0),
            ));
        Element::container([trigger, parent])
    };
    let region = |node| HitRegion {
        node,
        bounds: Rect::new(Point::default(), Size::new(40.0, 30.0)),
        transform: argui_core::Affine2D::IDENTITY,
        clips: Default::default(),
        shape: argui_ui::HitShape::Bounds,
        slop: Default::default(),
        enabled: true,
        focus_policy: FocusPolicy::TabStop,
        cursor: argui_ui::CursorIcon::Auto,
        gestures: GestureSet::EMPTY,
        window_drag: None,
    };
    let node = |ui: &UiTree, key: &str| {
        *ui.node_ids()
            .iter()
            .find(|node| ui.key(**node) == Some(key))
            .unwrap()
    };
    let outside = || PointerEvent::mouse(PointerPhase::Pressed, Point::new(500.0, 500.0));

    let mut ui = UiTree::new(view(false, false));
    let trigger = node(&ui, "trigger");
    ui.sync_focus(
        &[region(trigger)],
        Some(FocusRequest::Focus(trigger.into())),
    );
    ui.update(view(true, false));
    let parent = node(&ui, "parent-popup");
    let parent_item = node(&ui, "parent-item");
    ui.sync_focus(&[region(trigger), region(parent_item)], None);
    assert_eq!(ui.focused_node(), Some(parent_item));

    ui.update(view(true, true));
    let child = node(&ui, "child-popup");
    let child_item = node(&ui, "child-item");
    ui.sync_focus(
        &[region(trigger), region(parent_item), region(child_item)],
        None,
    );
    assert_eq!(ui.focused_node(), Some(child_item));
    let first = ui.pointer_event(outside(), &[]);
    assert!(first.events.iter().any(|event| {
        event.target == child && matches!(event.kind, UiEventKind::PointerOutside(_))
    }));
    assert!(!first.events.iter().any(|event| {
        event.target == parent && matches!(event.kind, UiEventKind::PointerOutside(_))
    }));

    ui.update(view(true, false));
    ui.sync_focus(&[region(trigger), region(parent_item)], None);
    assert_eq!(ui.focused_node(), Some(parent_item));
    let second = ui.pointer_event(outside(), &[]);
    assert!(second.events.iter().any(|event| {
        event.target == parent && matches!(event.kind, UiEventKind::PointerOutside(_))
    }));
    ui.update(view(false, false));
    ui.sync_focus(&[region(trigger)], None);
    assert_eq!(ui.focused_node(), Some(trigger));
}
