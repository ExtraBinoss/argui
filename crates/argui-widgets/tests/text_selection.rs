use argui_core::{ColorScheme, Point, Rect, Size};
use argui_runtime::{Context, Entity, Render, WindowEnvironment};
use argui_ui::{Element, ElementKind, SelectionCapabilities, UiEventKind, UiTree, UserSelect};
use argui_widgets::{TablerIcon, TextSelectionToolbar, WidgetAssets, shadcn};
#[path = "text_selection/host.rs"]
mod host;

struct Editor;

impl Render for Editor {
    fn render(&mut self, _cx: &mut Context<Self>) -> Element {
        Element::text("selectable editor")
    }
}

fn dispatch<A: Render>(app: &Entity<A>, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree.node_ids()[0];
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn dispatch_key<A: Render>(app: &Entity<A>, key: &str, kind: UiEventKind) {
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some(key))
        .expect("selection toolbar command should be present");
    for event in tree.event_deliveries(target, kind) {
        if event.should_dispatch() {
            app.dispatch_event(&event);
        }
    }
}

fn has_key(element: &Element, key: &str) -> bool {
    element.key.as_deref() == Some(key) || element.children.iter().any(|child| has_key(child, key))
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

#[test]
fn touch_selection_toolbar_is_a_clamped_non_selectable_widget() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Light);
    let toolbar = TextSelectionToolbar::new(
        "copy",
        Rect::new(Point::new(2.0, 2.0), Size::new(20.0, 20.0)),
        Size::new(200.0, 120.0),
        SelectionCapabilities {
            copy: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert!(matches!(toolbar.kind, ElementKind::Container));
    assert_eq!(toolbar.user_select, UserSelect::None);
    assert!(!toolbar.interaction.as_ref().unwrap().focusable);
    assert_eq!(toolbar.children.len(), 2);
    assert_eq!(
        toolbar.portal.as_ref().unwrap().dismiss,
        argui_ui::DismissPolicy::OutsidePointer
    );
    assert!(!toolbar.layer.as_ref().unwrap().backdrop_filters.is_empty());
}

#[test]
fn selection_toolbar_dismisses_on_outside_pointer_without_consuming_it() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("argui::selection-menu"))
        .unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::new(0.0, 0.0),
        )),
    );
    for event in &events {
        if event.should_dispatch() {
            app.dispatch_event(event);
        }
        assert!(!event.default_prevented());
    }
    host::finish_exit(&app);
    assert!(!has_key(&app.render(), "argui::selection-menu"));
}

#[test]
fn selection_toolbar_finishes_its_exit_before_unmounting() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(140_000_000),
                elapsed: argui_animation::Duration::from_millis(140),
            },
            cx,
        )
    });
    dispatch_key(
        &app,
        "argui::selection-menu",
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::default(),
        )),
    );
    assert!(has_key(&app.render(), "argui::selection-menu"));
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(240_000_000),
                elapsed: argui_animation::Duration::from_millis(100),
            },
            cx,
        )
    });
    assert!(!has_key(&app.render(), "argui::selection-menu"));
}

#[test]
fn editable_toolbar_keeps_unavailable_commands_visible_but_disabled() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Dark);
    let toolbar = TextSelectionToolbar::new(
        "editor",
        Rect::default(),
        Size::new(500.0, 300.0),
        SelectionCapabilities {
            editable: true,
            paste: true,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);

    assert_eq!(toolbar.children.len(), 4);
    assert!(!toolbar.children[0].interaction.as_ref().unwrap().enabled);
    assert!(!toolbar.children[1].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[2].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[3].interaction.as_ref().unwrap().enabled);
}

#[test]
fn selection_host_presents_and_removes_the_public_touch_toolbar() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    assert!(matches!(app.render().kind, ElementKind::Container));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: true,
        },
    );
    host::finish_exit(&app);
    assert!(matches!(app.render().kind, ElementKind::Text { .. }));
}

#[test]
fn context_menu_requires_a_command_and_public_toolbar_commands_dismiss_it() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities::default(),
        },
    );
    assert!(!has_key(&app.render(), "argui::selection-menu::copy"));

    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
        },
    );
    assert!(has_key(&app.render(), "argui::selection-menu::copy"));

    dispatch_key(
        &app,
        "argui::selection-menu::copy",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    host::finish_exit(&app);
    assert!(!has_key(&app.render(), "argui::selection-menu::copy"));
}

#[test]
fn toolbar_clamps_narrow_viewports_and_uses_icon_only_buttons() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Light);
    let icons = WidgetAssets::tabler_subset(
        theme.foreground,
        [
            TablerIcon::Cut,
            TablerIcon::Copy,
            TablerIcon::Paste,
            TablerIcon::SelectAll,
        ],
    );
    let toolbar = TextSelectionToolbar::new(
        "narrow",
        Rect::new(Point::new(0.0, 0.0), Size::new(20.0, 12.0)),
        Size::new(100.0, 80.0),
        SelectionCapabilities {
            editable: true,
            cut: true,
            copy: true,
            paste: true,
            select_all: true,
        },
    )
    .icons(&icons)
    .build(theme);
    assert_eq!(toolbar.children.len(), 4);
    assert_eq!(toolbar.style.size.width, argui_ui::Dimension::length(84.0));
    assert_eq!(toolbar.style.size.height, argui_ui::Dimension::length(40.0));
    assert_eq!(
        toolbar.style.inset.left,
        argui_ui::LengthPercentageAuto::length(8.0)
    );
    assert_eq!(
        toolbar.style.inset.top,
        argui_ui::LengthPercentageAuto::length(20.0)
    );
    assert!(
        toolbar
            .children
            .iter()
            .all(|button| button.children.len() == 1)
    );
    assert!(
        toolbar
            .children
            .iter()
            .all(|button| matches!(button.children[0].kind, ElementKind::Vector { .. }))
    );
}

#[test]
fn toolbar_filters_edit_commands_when_not_editable_and_clamps_right_edge() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Dark);
    let toolbar = TextSelectionToolbar::new(
        "readonly",
        Rect::new(Point::new(480.0, 120.0), Size::new(10.0, 10.0)),
        Size::new(500.0, 300.0),
        SelectionCapabilities {
            cut: true,
            paste: true,
            copy: false,
            select_all: true,
            ..SelectionCapabilities::default()
        },
    )
    .build(theme);
    assert_eq!(toolbar.children.len(), 2);
    assert_eq!(toolbar.style.size.width, argui_ui::Dimension::length(224.0));
    assert_eq!(
        toolbar.style.inset.left,
        argui_ui::LengthPercentageAuto::length(268.0)
    );
    assert_eq!(
        toolbar.style.inset.top,
        argui_ui::LengthPercentageAuto::length(72.0)
    );
    assert!(!toolbar.children[0].interaction.as_ref().unwrap().enabled);
    assert!(toolbar.children[1].interaction.as_ref().unwrap().enabled);
}

#[test]
fn toolbar_wide_icons_use_leading_content_instead_of_icon_only_mode() {
    let palette = shadcn(argui_core::Color::srgb(0.2, 0.4, 0.8));
    let theme = palette.resolve(ColorScheme::Light);
    let icons = WidgetAssets::tabler_subset(
        theme.foreground,
        [
            TablerIcon::Cut,
            TablerIcon::Copy,
            TablerIcon::Paste,
            TablerIcon::SelectAll,
        ],
    );
    let toolbar = TextSelectionToolbar::new(
        "wide",
        Rect::new(Point::new(250.0, 150.0), Size::new(20.0, 20.0)),
        Size::new(600.0, 400.0),
        SelectionCapabilities {
            editable: true,
            cut: true,
            copy: true,
            paste: true,
            select_all: true,
        },
    )
    .icons(&icons)
    .build(theme);
    assert_eq!(toolbar.children.len(), 4);
    assert_eq!(toolbar.style.size.width, argui_ui::Dimension::length(448.0));
    assert!(
        toolbar
            .children
            .iter()
            .all(|button| button.children.len() == 2)
    );
    assert!(matches!(
        toolbar.children[0].children[0].kind,
        ElementKind::Vector { .. }
    ));
}

#[test]
fn selection_host_handles_empty_selection_escape_and_custom_menu_key() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor).menu_key("custom-menu"));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some(String::new()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    let rendered = app.render();
    assert!(has_key(&rendered, "custom-menu"));
    assert!(has_key(&rendered, "custom-menu::select-all"));
    assert!(
        !find_key(&rendered, "custom-menu::copy")
            .unwrap()
            .interaction
            .as_ref()
            .unwrap()
            .enabled
    );

    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(140_000_000),
                elapsed: argui_animation::Duration::from_millis(140),
            },
            cx,
        )
    });

    dispatch(
        &app,
        UiEventKind::KeyInput(argui_core::KeyInput {
            key: argui_core::Key::Escape,
            state: argui_core::KeyState::Pressed,
            modifiers: argui_core::Modifiers::default(),
            repeat: false,
            text: None,
        }),
    );
    assert!(has_key(&app.render(), "custom-menu"));
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(100_000_000),
                elapsed: argui_animation::Duration::from_millis(100),
            },
            cx,
        )
    });
    assert!(!has_key(&app.render(), "custom-menu"));

    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: None,
            touch: true,
            dragging: false,
        },
    );
    assert!(!has_key(&app.render(), "custom-menu"));
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: false,
            dragging: false,
        },
    );
    assert!(!has_key(&app.render(), "custom-menu"));
}

#[test]
fn selection_host_ignores_unaddressed_close_events_and_handles_each_context_command() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    for capabilities in [
        SelectionCapabilities {
            cut: true,
            ..SelectionCapabilities::default()
        },
        SelectionCapabilities {
            paste: true,
            ..SelectionCapabilities::default()
        },
        SelectionCapabilities {
            select_all: true,
            ..SelectionCapabilities::default()
        },
    ] {
        dispatch(
            &app,
            UiEventKind::ContextMenu {
                position: Point::new(20.0, 24.0),
                capabilities,
            },
        );
    }
    app.update(|host, cx| {
        host.animation_frame(
            argui_animation::Frame {
                now: argui_animation::Time::from_nanos(140_000_000),
                elapsed: argui_animation::Duration::from_millis(140),
            },
            cx,
        )
    });

    dispatch(
        &app,
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::default(),
        )),
    );
    dispatch_key(
        &app,
        "argui::selection-menu",
        UiEventKind::PointerOutside(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::default(),
        )),
    );
    dispatch_key(
        &app,
        "argui::selection-menu",
        UiEventKind::KeyInput(argui_core::KeyInput {
            key: argui_core::Key::Escape,
            state: argui_core::KeyState::Pressed,
            modifiers: argui_core::Modifiers::default(),
            repeat: false,
            text: None,
        }),
    );
    dispatch_key(
        &app,
        "argui::selection-menu",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(has_key(&app.render(), "argui::selection-menu"));
}

#[test]
fn selection_host_reduced_motion_mounts_immediately_and_global_command_uses_no_target() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    let environment = WindowEnvironment {
        reduced_motion: true,
        ..WindowEnvironment::default()
    };
    let _ = app.render_in(environment);
    dispatch(
        &app,
        UiEventKind::DocumentSelectionChanged {
            text: Some("selected".into()),
            bounds: Some(Rect::new(Point::new(40.0, 60.0), Size::new(50.0, 18.0))),
            touch: true,
            dragging: false,
        },
    );
    let rendered = app.render_in(environment);
    assert!(has_key(&rendered, "argui::selection-menu"));
    assert!(!app.read(Render::wants_animation_frame));

    dispatch_key(
        &app,
        "argui::selection-menu::copy",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    );
    assert!(!has_key(&app.render(), "argui::selection-menu"));
}

#[test]
fn selection_host_prevents_pointer_default_for_command_press_and_keeps_menu_open() {
    let app = Entity::new(argui_widgets::SelectionHost::new(Editor));
    dispatch(
        &app,
        UiEventKind::ContextMenu {
            position: Point::new(20.0, 24.0),
            capabilities: SelectionCapabilities {
                copy: true,
                ..SelectionCapabilities::default()
            },
        },
    );
    let mut tree = UiTree::new(app.render());
    let target = tree
        .node_ids()
        .iter()
        .copied()
        .find(|node| tree.key(*node) == Some("argui::selection-menu::copy"))
        .unwrap();
    let events = tree.event_deliveries(
        target,
        UiEventKind::Pointer(argui_core::PointerEvent::mouse(
            argui_core::PointerPhase::Pressed,
            Point::new(22.0, 25.0),
        )),
    );
    assert!(!events.is_empty());
    for event in &events {
        if event.should_dispatch() {
            app.dispatch_event(event);
        }
    }
    assert!(events.iter().all(argui_ui::UiEvent::default_prevented));
    assert!(has_key(&app.render(), "argui::selection-menu::copy"));
}
