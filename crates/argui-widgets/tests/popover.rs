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
