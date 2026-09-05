use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase};
use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_ui::{
    CollisionPolicy, DismissPolicy, Element, FloatingPlacement, Placement, PortalTarget, Role,
    UiEvent, UiEventKind, UiTree, WindowLayer,
};
use argui_widgets::{Popover, PopoverAction, PopoverBehavior, shadcn};

fn event(key: Option<&str>, kind: UiEventKind) -> UiEvent {
    let tree = UiTree::new(Element::container([]));
    UiEvent::new(tree.node_ids()[0], key.map(str::to_owned), kind)
}

#[test]
fn animated_popover_keeps_exit_visuals_but_releases_focus() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(argui_core::ColorScheme::Dark);
    let mut presence = argui_widgets::Presence::default();
    presence.set_open(true, true);
    presence.set_open(false, false);
    let build = |presence: &argui_widgets::Presence| {
        Popover::new(
            "animated",
            "Animated",
            false,
            Element::text("Open"),
            Element::text("Content").interaction(argui_ui::Interaction::default().focusable(true)),
        )
        .presence(presence)
        .trap_focus(true)
        .build(theme)
    };
    let closing = build(&presence);
    assert_eq!(closing.children.len(), 2);
    let content = &closing.children[1];
    assert!(content.focus_scope.is_none());
    assert!(content.semantic_hidden);
    assert!(!content.children[0].interaction.as_ref().unwrap().focusable);
    assert_eq!(
        content.portal.as_ref().unwrap().dismiss,
        DismissPolicy::Manual
    );
    presence.advance(argui_animation::Duration::from_millis(100));
    assert_eq!(build(&presence).children.len(), 1);
}

#[test]
fn outside_pointer_only_dismisses_the_addressed_popover() {
    let open = PopoverBehavior::new("menu", "Menu", true);
    assert_eq!(
        open.action(&event(
            Some("other::content"),
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Pressed,
                Point::default()
            ),)
        )),
        None
    );
}

#[test]
fn popover_behavior_toggles_and_dismisses() {
    let closed = PopoverBehavior::new("menu", "Menu", false);
    assert_eq!(
        closed.action(&event(
            Some("menu"),
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(PopoverAction::Toggle)
    );
    let open = PopoverBehavior::new("menu", "Menu", true);
    assert_eq!(
        open.action(&event(
            Some("menu::content"),
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Pressed,
                Point::default(),
            )),
        )),
        Some(PopoverAction::Close)
    );
    assert_eq!(
        open.action(&event(
            Some("child"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            })
        )),
        Some(PopoverAction::Close)
    );
    assert_eq!(
        closed.action(&event(
            Some("other"),
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        None
    );
    assert_eq!(
        closed.action(&event(
            Some("menu"),
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Pressed,
                Point::default(),
            )),
        )),
        None
    );
    assert_eq!(
        open.action(&event(
            Some("menu"),
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        )),
        Some(PopoverAction::Toggle)
    );
    assert_eq!(
        open.action(&event(
            Some("child"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Enter,
                state: KeyState::Released,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            })
        )),
        None
    );
}

#[test]
fn open_popover_uses_the_window_popover_layer() {
    let themes = shadcn(argui_core::Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let tree = Popover::new(
        "menu",
        "Menu",
        true,
        Element::text("Open"),
        Element::text("Content"),
    )
    .trap_focus(true)
    .build(theme);
    let content = &tree.children[1];
    let portal = content.portal.as_ref().unwrap();

    assert_eq!(portal.layer, WindowLayer::Popover);
    assert_eq!(portal.dismiss, DismissPolicy::OutsidePointer);
    assert!(matches!(portal.target, PortalTarget::Anchor(_)));
    assert!(content.focus_scope.as_ref().unwrap().traps());
    assert_eq!(content.semantics.as_ref().unwrap().role, Role::Group);
}

#[test]
fn popover_surface_options_are_applied_without_trapping_focus() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Light);
    let tree = Popover::new(
        "options",
        "Options",
        true,
        Element::text("Open"),
        Element::text("Content"),
    )
    .placement(FloatingPlacement::new(Placement::TopEnd).collision(CollisionPolicy::FIT_VIEWPORT))
    .paint(PaintStyle::new(QuadStyle::solid(Color::BLACK)))
    .size(240.0, 180.0)
    .padding(20.0)
    .radius(14.0)
    .backdrop_blur(0.0)
    .build(theme);
    let content = &tree.children[1];
    let PortalTarget::Anchor(anchor) = &content.portal.as_ref().unwrap().target else {
        panic!("popover content must remain anchored");
    };

    assert_eq!(anchor.placement.preferred, Placement::TopEnd);
    assert!(!content.focus_scope.as_ref().unwrap().traps());
    assert!(content.layer.is_none());
    assert_eq!(
        content.style.padding.left,
        argui_ui::LengthPercentage::length(20.0)
    );
}

#[test]
fn closed_popover_mounts_only_trigger_and_reports_collapsed_state() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Light);
    let tree = Popover::new(
        "closed",
        "Closed menu",
        false,
        Element::text("Open"),
        Element::text("Content"),
    )
    .build(theme);
    assert_eq!(tree.children.len(), 1);
    let trigger = &tree.children[0];
    assert_eq!(trigger.key.as_deref(), Some("closed"));
    assert_eq!(
        trigger.semantics.as_ref().unwrap().state.expanded,
        Some(false)
    );
    assert!(trigger.interaction.as_ref().unwrap().focusable);
}

#[test]
fn popover_clamps_negative_surface_values_and_avoids_zero_blur_layer() {
    let themes = shadcn(Color::srgb(0.2, 0.5, 0.9));
    let theme = themes.resolve(argui_core::ColorScheme::Dark);
    let tree = Popover::new(
        "clamped",
        "Clamped menu",
        true,
        Element::text("Open"),
        Element::text("Content"),
    )
    .size(-10.0, -20.0)
    .padding(-4.0)
    .radius(-8.0)
    .backdrop_blur(-1.0)
    .build(theme);
    let content = &tree.children[1];
    assert_eq!(content.style.size.width, argui_ui::Dimension::length(0.0));
    assert_eq!(
        content.style.max_size.height,
        argui_ui::LengthPercentageAuto::length(0.0)
    );
    assert_eq!(content.style.padding, argui_ui::Sides::length(0.0));
    assert_eq!(content.paint.quad.radii, argui_paint::CornerRadii::all(0.0));
    assert!(content.layer.is_none());
    assert_eq!(
        content.portal.as_ref().unwrap().dismiss,
        DismissPolicy::OutsidePointer
    );
}

#[test]
fn popover_behavior_only_toggles_trigger_and_ignores_non_escape_close_events() {
    let closed = PopoverBehavior::new("menu", "Menu", false);
    assert_eq!(
        closed.action(&event(
            Some("menu::content"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        )),
        None
    );
    let open = PopoverBehavior::new("menu", "Menu", true);
    assert_eq!(
        open.action(&event(
            Some("menu::content"),
            UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
        )),
        None
    );
    assert_eq!(
        open.action(&event(
            Some("child"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Escape,
                state: KeyState::Released,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        )),
        None
    );
    assert_eq!(
        open.action(&event(
            Some("child"),
            UiEventKind::KeyInput(KeyInput {
                key: Key::Enter,
                state: KeyState::Pressed,
                modifiers: Modifiers::default(),
                repeat: false,
                text: None,
            }),
        )),
        None
    );
    assert_eq!(
        open.action(&event(
            Some("menu::content"),
            UiEventKind::PointerOutside(PointerEvent::mouse(
                PointerPhase::Released,
                Point::default(),
            )),
        )),
        Some(PopoverAction::Close)
    );
}
