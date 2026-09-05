use argui_animation::{Duration, Frame, Time};
use argui_core::ColorScheme;
use argui_platform::{PlatformEvent, WindowBackend, WindowKey};
use argui_runtime::{AppCommand, AppEvent, AppModel, ViewUpdate, WindowEnvironment};
use argui_ui::{Element, FocusTarget, InitialFocus, Overflow, UiEvent, UiEventKind, UiTree};

use argui_showcase::spotlight::SpotlightShowcase;

fn ui_event(key: &str, kind: UiEventKind) -> AppEvent {
    let tree = UiTree::new(Element::container([]));
    AppEvent::Ui {
        window: WindowKey::main(),
        event: UiEvent::new(tree.node_id_at(0).unwrap(), Some(key.into()), kind),
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

#[test]
fn spotlight_starts_with_search_focus_and_a_native_drag_region() {
    let app = SpotlightShowcase::default();
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
    assert_eq!(panel.style.overflow.x, Overflow::Hidden);
    assert_eq!(panel.style.overflow.y, Overflow::Hidden);
    assert!(panel.layer.is_none());
    let panel_border = panel.paint.quad.border.unwrap();
    assert_eq!(panel_border.widths.as_array(), [1.0; 4]);
    assert_eq!(panel.paint.quad.radii.as_array(), [16.0; 4]);
    let title_border = panel.children[0].paint.quad.border.unwrap();
    assert_eq!(title_border.widths.as_array(), [0.0, 0.0, 0.0, 1.0]);
    assert_eq!(
        scope.initial,
        Some(InitialFocus::Target(FocusTarget::Key(
            "spotlight-search".into()
        )))
    );
    assert!(
        root.children[0].children[0]
            .interaction
            .as_ref()
            .unwrap()
            .window_drag
            .is_some()
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
            "close",
            UiEventKind::Click(argui_ui::ClickEvent::accessibility())
        ))
        .commands,
        [AppCommand::Quit]
    );
}

#[test]
fn passthrough_restores_itself_after_two_seconds() {
    let mut app = SpotlightShowcase::default();
    let enabled = app.update(&ui_event(
        "passthrough",
        UiEventKind::Click(argui_ui::ClickEvent::accessibility()),
    ));
    assert_eq!(
        enabled.commands,
        [AppCommand::SetWindowMousePassthrough {
            window: WindowKey::main(),
            passthrough: true,
        }]
    );
    assert!(app.wants_animation_frame(&WindowKey::main()));
    assert!(
        app.animation_frame(
            &WindowKey::main(),
            Frame {
                now: Time::ZERO,
                elapsed: Duration::from_secs(1),
            },
        )
        .commands
        .is_empty()
    );
    let disabled = app.animation_frame(
        &WindowKey::main(),
        Frame {
            now: Time::from_nanos(2_000_000_000),
            elapsed: Duration::from_secs(1),
        },
    );
    assert_eq!(
        disabled.commands,
        [AppCommand::SetWindowMousePassthrough {
            window: WindowKey::main(),
            passthrough: false,
        }]
    );
    assert!(!app.wants_animation_frame(&WindowKey::main()));
}

#[test]
fn backend_capabilities_are_rendered_without_claiming_wayland_support() {
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
    let root = app
        .view(&WindowKey::main(), WindowEnvironment::default())
        .unwrap();
    let mut labels = Vec::new();
    texts(&root, &mut labels);
    assert!(
        labels
            .iter()
            .any(|label| label == "Linux · Wayland · always on top unavailable")
    );

    app.update(&AppEvent::Window {
        window: WindowKey::main(),
        event: PlatformEvent::Opened {
            width: 720,
            height: 460,
            scale_factor: 1.0,
            capabilities: WindowBackend::Windows.capabilities(),
        },
    });
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
    assert!(
        app.view(&WindowKey::new("secondary"), WindowEnvironment::default())
            .is_none()
    );
}
