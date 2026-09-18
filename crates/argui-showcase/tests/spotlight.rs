use argui_core::{ColorScheme, Key, KeyInput, KeyState, Modifiers, Transform2D};
use argui_platform::{
    GlobalShortcutEvent, GlobalShortcutId, GlobalShortcutState, PlatformEvent, WindowBackend,
    WindowKey, WindowLevel,
};
use argui_runtime::{AppCommand, AppEvent, AppModel, ViewUpdate, WindowEnvironment};
use argui_ui::{
    Display, Element, FocusTarget, InitialFocus, Overflow, UiEvent, UiEventKind, UiTree,
};

use argui_showcase::spotlight::{SPOTLIGHT_SHORTCUT_ID, SpotlightShowcase};

fn ui_event(key: &str, kind: UiEventKind) -> AppEvent {
    let tree = UiTree::new(Element::container([]));
    AppEvent::Ui {
        window: WindowKey::main(),
        event: UiEvent::new(tree.node_id_at(0).unwrap(), Some(key.into()), kind),
    }
}

fn platform_key_event(key: Key) -> AppEvent {
    AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::Keyboard(KeyInput {
            key,
            state: KeyState::Pressed,
            modifiers: Modifiers::default(),
            repeat: false,
            text: None,
        }),
    }
}

fn texts(element: &Element, output: &mut Vec<String>) {
    if let argui_ui::ElementKind::Text { content, .. } = &element.kind {
        output.push(content.as_str().to_owned());
    }
    for child in &element.children {
        texts(child, output);
    }
}

fn keyed_element<'a>(element: &'a Element, key: &str) -> Option<&'a Element> {
    if element.key.as_deref() == Some(key) {
        return Some(element);
    }
    element
        .children
        .iter()
        .find_map(|child| keyed_element(child, key))
}

#[test]
fn spotlight_starts_with_search_focus_and_a_clickable_header() {
    let app = SpotlightShowcase::default();
    assert!(!app.vector_assets().is_empty());
    let root = app
        .view(
            &WindowKey::main(),
            WindowEnvironment {
                color_scheme: ColorScheme::Dark,
                ..WindowEnvironment::default()
            },
        )
        .unwrap();
    let scope = root.children[0].focus_scope.as_ref().unwrap();
    let panel = &root.children[0];
    assert_eq!(panel.key.as_deref(), Some("spotlight-panel"));
    assert!(panel.desktop_backdrop.is_some());
    assert_eq!(panel.style.overflow.x, Overflow::Hidden);
    assert_eq!(panel.style.overflow.y, Overflow::Hidden);
    assert!(panel.layer.is_none());
    let panel_border = panel.paint.quad.border.unwrap();
    assert_eq!(panel_border.widths.as_array(), [1.0; 4]);
    assert_eq!(panel.paint.quad.radii.as_array(), [18.0; 4]);
    let title_border = panel.children[0].paint.quad.border.unwrap();
    assert_eq!(title_border.widths.as_array(), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(
        scope.initial,
        Some(InitialFocus::Target(FocusTarget::Key(
            "spotlight-search".into()
        )))
    );
    assert!(root.children[0].children[0].interaction.is_none());
    let result = keyed_element(&root, "spotlight-result::command-palette").unwrap();
    assert!(
        result
            .interaction
            .as_ref()
            .is_some_and(|value| value.enabled)
    );
    assert_eq!(
        result.semantics.as_ref().unwrap().role,
        argui_ui::Role::Button
    );
    assert_eq!(result.transform, Transform2D::IDENTITY);
}

#[test]
fn result_icons_use_a_centering_flex_container() {
    let app = SpotlightShowcase::default();
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let icon = keyed_element(&root, "spotlight-result-icon::Open command palette").unwrap();
    assert_eq!(icon.style.display, Display::Flex);
    assert_eq!(icon.style.align_items, Some(argui_ui::AlignItems::CENTER));
    assert_eq!(
        icon.style.justify_content,
        Some(argui_ui::JustifyContent::CENTER)
    );
}

#[test]
fn search_is_controlled_and_filters_the_visible_results() {
    let mut app = SpotlightShowcase::default();
    let update = app.update(&ui_event(
        "spotlight-search",
        UiEventKind::TextChanged("gpu".into()),
    ));
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(labels.iter().any(|label| label == "Inspect GPU frame"));
    assert!(!labels.iter().any(|label| label == "Open command palette"));
}

#[test]
fn results_support_arrow_navigation_enter_and_pointer_activation() {
    let mut app = SpotlightShowcase::default();
    let update = app.update(&platform_key_event(Key::ArrowDown));
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    app.update(&platform_key_event(Key::Enter));
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(
        labels
            .iter()
            .any(|label| label == "Activated · Browse recent files")
    );

    app.update(&ui_event(
        "spotlight-result::color-theme",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ));
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(
        labels
            .iter()
            .any(|label| label == "Activated · Switch color theme")
    );

    let mut app = SpotlightShowcase::default();
    app.update(&platform_key_event(Key::ArrowUp));
    app.update(&platform_key_event(Key::Enter));
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(
        labels
            .iter()
            .any(|label| label == "Activated · Inspect GPU frame")
    );
}

#[test]
fn window_controls_emit_typed_commands() {
    let mut app = SpotlightShowcase::default();
    assert_eq!(
        app.update(&ui_event(
            "minimize",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))
        .commands,
        [AppCommand::MinimizeWindow(WindowKey::main())]
    );
    assert_eq!(
        app.update(&ui_event(
            "hide",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))
        .commands,
        [AppCommand::HideWindow(WindowKey::main())]
    );
}

#[test]
fn global_shortcut_wakes_and_focuses_the_spotlight_window() {
    let mut app = SpotlightShowcase::default();
    let update = app.update(&AppEvent::GlobalShortcut(GlobalShortcutEvent {
        id: GlobalShortcutId::new(SPOTLIGHT_SHORTCUT_ID),
        state: GlobalShortcutState::Pressed,
        activation_token: Some("portal-token".into()),
    }));
    assert_eq!(
        update.commands,
        [AppCommand::FocusWindow(WindowKey::main())]
    );
    assert!(
        app.update(&AppEvent::GlobalShortcut(GlobalShortcutEvent {
            id: GlobalShortcutId::new(SPOTLIGHT_SHORTCUT_ID),
            state: GlobalShortcutState::Released,
            activation_token: None,
        }))
        .commands
        .is_empty()
    );
}

#[test]
fn backend_capabilities_control_window_level_and_backend_label() {
    let mut app = SpotlightShowcase::default();
    let update = app.update(&AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::Opened {
            width: 720,
            height: 460,
            scale_factor: 1.0,
            capabilities: WindowBackend::Wayland.capabilities(),
        },
    });
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    assert!(update.commands.is_empty());
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(labels.iter().any(|label| label == "Linux · Wayland"));

    let update = app.update(&AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::Opened {
            width: 720,
            height: 460,
            scale_factor: 1.0,
            capabilities: WindowBackend::Windows.capabilities(),
        },
    });
    assert_eq!(
        update.commands,
        [AppCommand::SetWindowLevel {
            window: WindowKey::main(),
            level: WindowLevel::AlwaysOnTop,
        }]
    );
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(
        labels
            .iter()
            .any(|label| label == "Windows · always on top")
    );

    app.update(&AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::Opened {
            width: 720,
            height: 460,
            scale_factor: 1.0,
            capabilities: WindowBackend::Web.capabilities(),
        },
    });
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(labels.iter().any(|label| label == "Web preview"));
    assert!(!labels.iter().any(|label| label == "Minimize"));
    assert!(!labels.iter().any(|label| label == "Close"));
    assert!(!labels.iter().any(|label| label == "Pass 2s"));
    assert!(
        app.view(&WindowKey::new("secondary"), WindowEnvironment::default())
            .is_none()
    );
}
