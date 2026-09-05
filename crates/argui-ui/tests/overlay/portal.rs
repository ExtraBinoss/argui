use argui_ui::{
    AnchorPortal, DismissPolicy, Element, FloatingPlacement, Placement, Portal, PortalTarget,
    Position, ViewportPlacement, WindowLayer,
};

#[test]
fn portal_builders_prepare_absolute_layout_and_keep_the_target() {
    let anchored = Element::container([])
        .anchored_portal(
            WindowLayer::Popover,
            "trigger",
            FloatingPlacement::new(Placement::BottomEnd),
        )
        .portal_dismiss(DismissPolicy::OutsidePointer);
    assert_eq!(anchored.style.position, Position::Absolute);
    assert!(anchored.style.inset.left.is_auto());
    assert!(anchored.style.inset.right.is_auto());
    assert!(anchored.style.inset.top.is_auto());
    assert!(anchored.style.inset.bottom.is_auto());
    let portal = anchored.portal.as_ref().expect("anchored portal");
    assert_eq!(portal.layer, WindowLayer::Popover);
    assert_eq!(portal.dismiss, DismissPolicy::OutsidePointer);
    assert_eq!(
        portal.target,
        PortalTarget::Anchor(AnchorPortal::new(
            "trigger",
            FloatingPlacement::new(Placement::BottomEnd),
        ))
    );

    let viewport = Element::container([])
        .viewport_portal(WindowLayer::Modal, ViewportPlacement::fill().margin(12.0));
    assert_eq!(viewport.style.position, Position::Absolute);
    assert!(matches!(
        viewport.portal.as_ref().map(|portal| &portal.target),
        Some(PortalTarget::Viewport(ViewportPlacement::Fill { margin }))
            if (*margin - 12.0).abs() < f32::EPSILON
    ));
}

#[test]
fn portal_layer_updates_do_not_discard_dismissal_or_target() {
    let portal = Element::container([])
        .portal(WindowLayer::Floating)
        .portal_dismiss(DismissPolicy::OutsidePointer);
    let relayered = portal.clone().portal(WindowLayer::Debug);
    let state = relayered.portal.as_ref().expect("layout portal");
    assert_eq!(state.layer, WindowLayer::Debug);
    assert_eq!(state.target, PortalTarget::Layout);
    assert_eq!(state.dismiss, DismissPolicy::OutsidePointer);
    assert_eq!(relayered.style.position, Position::Absolute);

    let untouched = Element::container([]).portal_dismiss(DismissPolicy::OutsidePointer);
    assert!(untouched.portal.is_none());

    let manual = Portal::new(WindowLayer::Content, PortalTarget::Layout);
    assert_eq!(manual.dismiss, DismissPolicy::Manual);
    assert_eq!(
        manual.dismiss(DismissPolicy::OutsidePointer).dismiss,
        DismissPolicy::OutsidePointer
    );
    assert!(WindowLayer::Modal > WindowLayer::Popover);
}
