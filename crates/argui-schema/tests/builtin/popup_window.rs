use argui_core::{Point, Rect, Size};
use argui_layout::LayoutEngine;
use argui_schema::{
    NativeElementInput, NativeEventValue, NativeSlotValue, SchemaError, SchemaValue, builtin,
};
use argui_ui::{
    DismissPolicy, Element, EventHandler, EventHandlerId, EventOwnerId, EventType,
    FocusContainment, InitialFocus, Interaction, Placement, PortalTarget, ViewportPlacement,
    VisualState, WindowLayer, WritingDirection, length, percent,
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
fn modal_popup_blocks_hover_behind_it_and_keeps_child_interaction() {
    let registry = builtin::registry().unwrap();
    let popup = registry
        .construct(
            builtin::POPUP_WINDOW,
            &NativeElementInput::new()
                .property(builtin::PLACEMENT, SchemaValue::String("fill".into()))
                .property(builtin::WINDOW_LAYER, SchemaValue::String("modal".into()))
                .property(builtin::WIDTH, SchemaValue::Dimension(percent(1.0)))
                .property(builtin::HEIGHT, SchemaValue::Dimension(percent(1.0)))
                .slot(NativeSlotValue::new(
                    builtin::CHILDREN,
                    [Element::container([])
                        .keyed("dialog-button")
                        .width(length(40.0))
                        .height(length(30.0))
                        .interaction(Interaction::default())],
                )),
        )
        .unwrap()
        .keyed("modal");
    let underlay = Element::container([])
        .keyed("underlay")
        .width(length(200.0))
        .height(length(120.0))
        .interaction(Interaction::default());
    let mut ui = argui_ui::UiTree::new(Element::container([underlay, popup]));
    let output = LayoutEngine::new()
        .compute(
            &mut ui,
            &mut argui_text::TextEngine::from_embedded_fonts(
                [],
                "sans-serif",
                "serif",
                "monospace",
            ),
            Size::new(200.0, 120.0),
        )
        .unwrap();
    let node = |key| {
        ui.node_ids()
            .iter()
            .copied()
            .find(|id| ui.key(*id) == Some(key))
            .unwrap()
    };
    let underlay = node("underlay");
    let modal = node("modal");
    let button = node("dialog-button");
    assert_eq!(output.hit_regions.last().map(|hit| hit.node), Some(button));
    ui.pointer_moved(Point::new(180.0, 100.0), &output.hit_regions);
    assert!(ui.visual_states(modal).contains(VisualState::Hovered));
    assert!(!ui.visual_states(underlay).contains(VisualState::Hovered));
    ui.pointer_moved(Point::new(20.0, 15.0), &output.hit_regions);
    assert!(ui.visual_states(button).contains(VisualState::Hovered));
    assert!(!ui.visual_states(underlay).contains(VisualState::Hovered));
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
