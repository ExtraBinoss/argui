use argui_core::{Key, KeyInput, KeyState, Modifiers, Point, PointerEvent, PointerPhase};
use argui_paint::{Color, PaintStyle, QuadStyle};
use argui_ui::{
    CollisionPolicy, DismissPolicy, Element, FloatingPlacement, Placement, PortalTarget, Role,
    UiEvent, UiEventKind, UiTree, WindowLayer,
};
use argui_widgets::{Popover, PopoverAction, PopoverBehavior, shadcn};

#[test]
fn popover_accepts_a_complete_effect_layer_without_overwriting_it() {
    use argui_paint::{EffectId, EffectInstance, Filter, LayerStyle};
    let themes = shadcn(Color::WHITE);
    let layer = LayerStyle::new(Default::default())
        .backdrop(Filter::Blur(8.0))
        .filter(Filter::Effect(EffectInstance::new(
            EffectId::new("test.popover"),
            [("strength", argui_paint::EffectValue::F32(0.5))],
        )));
    let root = Popover::new(
        "custom",
        "Custom",
        true,
        Element::text("Open"),
        Element::text("Content"),
    )
    .layer(layer.clone())
    .build(themes.resolve(argui_core::ColorScheme::Dark));
    assert_eq!(root.children[1].layer.as_deref(), Some(&layer));
}

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
            Element::text("Content").interaction(
                argui_ui::Interaction::default().focus_policy(argui_ui::FocusPolicy::TabStop),
            ),
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
    assert!(
        !content.children[0]
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
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
    let layer = content.layer.as_ref().unwrap();
    assert!(layer.backdrop_filters.is_empty());
    assert_eq!(layer.shadows, theme.overlay_shadows);
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
    assert!(
        trigger
            .interaction
            .as_ref()
            .unwrap()
            .focus_policy
            .is_focusable()
    );
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
    let layer = content.layer.as_ref().unwrap();
    assert!(layer.backdrop_filters.is_empty());
    assert_eq!(layer.shadows, theme.overlay_shadows);
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

#[test]
fn panel_text_padding_and_nested_popovers_are_inside_click_targets() {
    use argui_core::Size;
    use argui_ui::{EventHandlerId, EventListener, EventOwnerId, EventType, length};
    let themes = shadcn(Color::BLACK);
    let theme = themes.resolve(argui_core::ColorScheme::Light);
    let build = |nested| {
        Popover::new(
            "parent",
            "Parent",
            true,
            Element::text("Open"),
            Element::column([
                Element::text("Panel title").keyed("title"),
                Element::text("Panel description").keyed("description"),
                Element::text("Unavailable option")
                    .keyed("disabled")
                    .interaction(argui_ui::Interaction::default().enabled(false)),
                Popover::new(
                    "child",
                    "Child",
                    nested,
                    Element::text("More settings").height(length(30.0)),
                    Element::text("Nested description").keyed("nested-description"),
                )
                .size(180.0, 200.0)
                .placement(FloatingPlacement::new(Placement::RightStart))
                .build(theme),
            ])
            .gap(12.0),
        )
        .build(theme)
        .on(EventListener::new(
            EventType::PointerOutside,
            EventHandlerId::new(EventOwnerId(1), 0),
        ))
    };
    let mut tree = UiTree::new(build(false));
    let mut engine = argui_layout::LayoutEngine::new();
    let mut text = argui_text::TextEngine::from_embedded_fonts(
        [include_bytes!("../../argui-web-demo/assets/fonts/NotoSans-Regular.ttf").as_slice()],
        "Noto Sans",
        "Noto Sans",
        "Noto Sans",
    );
    for nested in [false, true] {
        tree.update(build(nested));
        let layout = engine
            .compute(&mut tree, &mut text, Size::new(900.0, 600.0))
            .unwrap();
        let bounds = |key| {
            layout
                .nodes
                .iter()
                .find(|node| tree.key(node.node) == Some(key))
                .unwrap()
                .bounds
        };
        let targets = if nested {
            vec![
                (bounds("nested-description"), None),
                (bounds("child::content"), None),
                (bounds("title"), Some("child::content")),
            ]
        } else {
            vec![
                (bounds("title"), None),
                (bounds("description"), None),
                (bounds("disabled"), None),
                (bounds("parent::content"), None),
            ]
        };
        // The panel's top-left inset exercises padding, outside any child hit region.
        for (rect, expected) in targets {
            let point = Point::new(rect.origin.x + 2.0, rect.origin.y + 2.0);
            let update = tree.pointer_event(
                PointerEvent::mouse(PointerPhase::Pressed, point),
                &layout.hit_regions,
            );
            let outside = update
                .events
                .iter()
                .find(|event| matches!(event.kind, UiEventKind::PointerOutside(_)));
            assert_eq!(
                outside.and_then(UiEvent::target_key),
                expected,
                "{point:?}, nested={nested}"
            );
            tree.pointer_event(
                PointerEvent::mouse(PointerPhase::Released, point),
                &layout.hit_regions,
            );
        }
        let update = tree.pointer_event(
            PointerEvent::mouse(PointerPhase::Pressed, Point::new(850.0, 550.0)),
            &layout.hit_regions,
        );
        let outside = update
            .events
            .iter()
            .find(|event| matches!(event.kind, UiEventKind::PointerOutside(_)))
            .unwrap();
        assert_eq!(
            outside.target_key(),
            Some(if nested {
                "child::content"
            } else {
                "parent::content"
            })
        );
    }
}

#[test]
fn surface_preference_is_optional_and_os_dismissal_targets_only_its_panel() {
    let palette = shadcn(Color::WHITE);
    let theme = palette.resolve(argui_core::ColorScheme::Light);
    let build = || {
        Popover::new(
            "native",
            "Settings",
            true,
            Element::text("Open"),
            Element::text("Panel"),
        )
    };
    assert_eq!(
        build().build(theme).children[1]
            .portal
            .as_ref()
            .unwrap()
            .surface,
        None
    );
    let native = build()
        .surface(argui_ui::OverlaySurface::PreferNative)
        .build(theme);
    assert_eq!(
        native.children[1].portal.as_ref().unwrap().surface,
        Some(argui_ui::OverlaySurface::PreferNative)
    );
    let behavior = PopoverBehavior::new("native", "Settings", true);
    assert_eq!(
        behavior.action(&event(
            Some("other::content"),
            UiEventKind::DismissRequested
        )),
        None
    );
    assert_eq!(
        behavior.action(&event(
            Some("native::content"),
            UiEventKind::DismissRequested
        )),
        Some(PopoverAction::Close)
    );
    assert_eq!(
        UiEventKind::DismissRequested.event_type(),
        argui_ui::EventType::Dismiss
    );
}
