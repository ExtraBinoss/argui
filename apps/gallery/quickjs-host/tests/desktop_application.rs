#![cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]

use std::sync::mpsc;

use argui_gallery_quickjs::{GalleryApp, ServiceOutcome};
use argui_platform::{
    GlobalShortcutEvent, GlobalShortcutId, GlobalShortcutState, TrayAction, TrayEvent, TrayItemId,
    WindowKey,
};
use argui_runtime::{AppCommand, AppEvent, AppModel, ViewUpdate};
use argui_ui::{ClickEvent, Element, ElementKind, UiEvent, UiEventKind, UiTree};

#[test]
fn companion_receives_main_message_and_returns_click_to_quickjs() {
    let (sender, responses) = mpsc::channel();
    let mut app = GalleryApp::new(sender);
    let main = WindowKey::main();
    let companion = WindowKey::new("companion");
    assert!(app.view(&main, Default::default()).is_none());
    let update = app.update(&AppEvent::HostMessage {
        window: companion.clone(),
        message: "A message from main".into(),
    });
    assert_eq!(update.windows[0].window, companion);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
    let view = app.view(&companion, Default::default()).unwrap();
    assert!(
        matches!(&view.children[0].kind, ElementKind::Text { content, .. } if content.as_str().contains("A message from main"))
    );
    let node = UiTree::new(Element::container([])).node_ids()[0];
    app.update(&AppEvent::Ui {
        window: companion,
        event: UiEvent::new(node, None, UiEventKind::Click(ClickEvent::accessibility())),
    });
    let response = responses.try_recv().unwrap();
    assert!(matches!(response.outcome, ServiceOutcome::Event(_)));
    assert_eq!(
        response.json()["value"],
        serde_json::json!({
            "type":"window", "window":"companion",
            "message":"Companion received: A message from main",
        })
    );
}

#[test]
fn tray_action_and_global_shortcut_both_reopen_the_main_window() {
    let (sender, _) = mpsc::channel();
    let mut app = GalleryApp::new(sender);
    let focus = vec![AppCommand::FocusWindow(WindowKey::main())];
    let tray = app.update(&AppEvent::Tray(TrayEvent::Action {
        id: TrayItemId::new("open-services"),
        action: TrayAction::Custom("open-services".into()),
    }));
    assert_eq!(tray.commands, focus);
    let shortcut = app.update(&AppEvent::GlobalShortcut(GlobalShortcutEvent {
        id: GlobalShortcutId::new("wake"),
        state: GlobalShortcutState::Pressed,
        activation_token: None,
    }));
    assert_eq!(shortcut.commands, focus);
    let release = app.update(&AppEvent::GlobalShortcut(GlobalShortcutEvent {
        id: GlobalShortcutId::new("wake"),
        state: GlobalShortcutState::Released,
        activation_token: None,
    }));
    assert!(release.commands.is_empty());
}
