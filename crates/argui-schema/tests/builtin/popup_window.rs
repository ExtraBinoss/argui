use argui_core::{Point, Rect, Size};
use argui_schema::{
    NativeElementInput, NativeEventValue, NativeSlotValue, SchemaError, SchemaValue, builtin,
};
use argui_ui::{
    DismissPolicy, Element, EventHandler, EventHandlerId, EventOwnerId, EventType,
    FocusContainment, InitialFocus, Placement, PortalTarget, ViewportPlacement, WindowLayer,
    WritingDirection,
};

#[test]
fn anchored_popup_uses_portal_and_focus_policies_without_paint() {
    let registry = builtin::registry().unwrap();
    let handler = EventHandler::from_identity(EventHandlerId::new(EventOwnerId(42), 0));
    let popup = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::ANCHOR, SchemaValue::String("trigger".into()))
                .property(builtin::PLACEMENT, SchemaValue::String("top_end".into()))
                .property(builtin::WINDOW_LAYER, SchemaValue::String("modal".into()))
                .property(
                    builtin::DISMISS_POLICY,
                    SchemaValue::String("outside_pointer_or_escape".into()),
                )
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("modal".into()),
                )
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::text("authored content")],
                ))
                .event(NativeEventValue::new(builtin::DISMISS, handler)),
        )
        .unwrap();
    let portal = popup.portal.as_ref().unwrap();
    assert_eq!(portal.layer, WindowLayer::Modal);
    assert_eq!(portal.dismiss, DismissPolicy::OutsidePointerOrEscape);
    assert!(matches!(
        &portal.target,
        PortalTarget::Anchor(anchor)
            if anchor.key == "trigger" && anchor.placement.preferred == Placement::TopEnd
    ));
    let focus = popup.focus_scope.as_ref().unwrap();
    assert_eq!(focus.containment, FocusContainment::Modal);
    assert_eq!(focus.initial, Some(InitialFocus::First));
    assert!(focus.restore);
    assert_eq!(popup.children.len(), 1);
    assert!(popup.paint.quad.background.is_none());
    assert_eq!(popup.event_listeners.len(), 2);
    assert!(
        popup
            .event_listeners
            .iter()
            .any(|event| event.event == EventType::Dismiss)
    );
    assert!(
        popup
            .event_listeners
            .iter()
            .any(|event| event.event == EventType::PointerOutside)
    );
}

#[test]
fn viewport_popup_is_centered_and_hidden_popup_has_no_portal() {
    let registry = builtin::registry().unwrap();
    let centered = registry
        .construct(builtin::POPUP_WINDOW, &NativeElementInput::new())
        .unwrap();
    assert!(matches!(
        &centered.portal.as_ref().unwrap().target,
        PortalTarget::Viewport(ViewportPlacement::Positioned { .. })
    ));
    let hidden = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new().property(builtin::VISIBLE, SchemaValue::Bool(false)),
        )
        .unwrap();
    assert!(hidden.portal.is_none());
    assert!(hidden.focus_scope.is_none());
    assert!(hidden.event_listeners.is_empty());
}

#[test]
fn popup_rejects_unknown_placement_for_its_target_kind() {
    let registry = builtin::registry().unwrap();
    let error = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::PLACEMENT, SchemaValue::String("right_end".into())),
        )
        .unwrap_err();
    assert!(matches!(error, SchemaError::Adapter(_)));
}

#[test]
fn anchored_popup_fits_near_a_window_edge() {
    let registry = builtin::registry().unwrap();
    let popup = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::ANCHOR, SchemaValue::String("trigger".into()))
                .property(builtin::PLACEMENT, SchemaValue::String("right_end".into())),
        )
        .unwrap();
    let PortalTarget::Anchor(anchor) = &popup.portal.as_ref().unwrap().target else {
        panic!("anchor portal expected");
    };
    let placed = anchor.placement.place(
        Rect::new(Point::default(), Size::new(300.0, 200.0)),
        Rect::new(Point::new(270.0, 150.0), Size::new(20.0, 25.0)),
        Size::new(120.0, 90.0),
        WritingDirection::Ltr,
    );
    assert!(placed.bounds.origin.x >= 0.0);
    assert!(placed.bounds.origin.y >= 0.0);
    assert!(placed.bounds.origin.x + placed.bounds.size.width <= 300.0);
    assert!(placed.bounds.origin.y + placed.bounds.size.height <= 200.0);
}

#[test]
fn popup_rejects_invalid_policies_and_preserves_manual_focus_choices() {
    let registry = builtin::registry().unwrap();
    for (property, value) in [
        (builtin::WINDOW_LAYER, "above_all"),
        (builtin::DISMISS_POLICY, "click_anywhere"),
        (builtin::FOCUS_CONTAINMENT, "locked"),
        (builtin::PLACEMENT, "outside"),
    ] {
        let error = registry
            .construct(
                builtin::POPUP_WINDOW,
                &NativeElementInput::new().property(property, SchemaValue::String(value.into())),
            )
            .unwrap_err();
        assert!(matches!(error, SchemaError::Adapter(_)), "{value}");
    }
    let manual = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::ANCHOR, SchemaValue::String("trigger".into()))
                .property(
                    builtin::WINDOW_LAYER,
                    SchemaValue::String("floating".into()),
                )
                .property(
                    builtin::DISMISS_POLICY,
                    SchemaValue::String("manual".into()),
                )
                .property(
                    builtin::FOCUS_CONTAINMENT,
                    SchemaValue::String("trap".into()),
                )
                .property(builtin::INITIAL_FOCUS, SchemaValue::String(String::new()))
                .property(builtin::RESTORE_FOCUS, SchemaValue::Bool(false)),
        )
        .unwrap();
    let portal = manual.portal.as_ref().unwrap();
    assert_eq!(portal.layer, WindowLayer::Floating);
    assert_eq!(portal.dismiss, DismissPolicy::Manual);
    let focus = manual.focus_scope.as_ref().unwrap();
    assert_eq!(focus.containment, FocusContainment::Trap);
    assert_eq!(focus.initial, None);
    assert!(!focus.restore);
}
