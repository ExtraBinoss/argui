#![cfg(all(feature = "tray", target_os = "linux"))]
use argui_platform::{
    ApplicationId, IconSet, NativeTray, TrayAction, TrayConfig, TrayEvent, TrayItemId, TrayMenuItem,
};
use std::{
    process::Command,
    sync::{Arc, mpsc},
    time::Duration,
};

fn bus(arguments: &[&str]) -> String {
    let output = Command::new("busctl")
        .args(["--user", "--no-pager", "--no-legend", "--timeout=5"])
        .arg("--")
        .args(arguments)
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn native_tray_keeps_nested_menu_actions_and_rejects_invalid_updates() {
    if std::env::var_os("ARGUI_NATIVE_TESTS").is_none() {
        return;
    }
    assert_eq!(std::env::var("ARGUI_HIDDEN_DISPLAY").as_deref(), Ok("1"));
    assert!(
        std::path::Path::new(&std::env::var("XDG_RUNTIME_DIR").unwrap())
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .starts_with("argui-display.")
    );
    let id = ApplicationId::new("dev.argui.traytest").unwrap();
    let icons = IconSet::new();
    let (sender, receiver) = mpsc::channel();
    let handler = Arc::new(move |event| {
        sender.send(event).unwrap();
    });
    let invalid = TrayConfig {
        menu: vec![TrayMenuItem::Action {
            id: TrayItemId::new(""),
            label: "Invalid".into(),
            enabled: true,
            action: TrayAction::Quit,
        }],
        ..Default::default()
    };
    assert!(NativeTray::new(&id, invalid.clone(), &icons, handler.clone()).is_err());
    let config = TrayConfig {
        menu: vec![
            TrayMenuItem::Action {
                id: TrayItemId::new("open"),
                label: "Open panel".into(),
                enabled: true,
                action: TrayAction::Custom("open".into()),
            },
            TrayMenuItem::Separator,
            TrayMenuItem::Submenu {
                id: TrayItemId::new("options"),
                label: "Options".into(),
                enabled: true,
                items: vec![TrayMenuItem::Check {
                    id: TrayItemId::new("pinned"),
                    label: "Pinned".into(),
                    enabled: true,
                    checked: true,
                    action: TrayAction::Custom("pin".into()),
                }],
            },
        ],
        ..Default::default()
    };
    let mut tray = NativeTray::new(&id, config.clone(), &icons, handler).unwrap();
    let prefix = format!("org.kde.StatusNotifierItem-{}-", std::process::id());
    let listing = bus(&["list"]);
    let name = listing
        .lines()
        .filter_map(|line| line.split_whitespace().next())
        .find(|name| name.starts_with(&prefix))
        .unwrap();
    let layout = bus(&[
        "call",
        name,
        "/MenuBar",
        "com.canonical.dbusmenu",
        "GetLayout",
        "iias",
        "0",
        "-1",
        "0",
    ]);
    for label in ["Open panel", "Options", "Pinned"] {
        assert!(layout.contains(label), "{layout}");
    }
    bus(&[
        "call",
        name,
        "/MenuBar",
        "com.canonical.dbusmenu",
        "Event",
        "isvu",
        "1",
        "clicked",
        "i",
        "0",
        "0",
    ]);
    assert_eq!(
        receiver.recv_timeout(Duration::from_secs(3)).unwrap(),
        TrayEvent::Action {
            id: TrayItemId::new("open"),
            action: TrayAction::Custom("open".into()),
        }
    );
    assert!(tray.sync(invalid, &icons).is_err());
    tray.sync(
        TrayConfig {
            title: Some("Updated panel".into()),
            visible: false,
            ..config
        },
        &icons,
    )
    .unwrap();
    assert!(
        bus(&[
            "get-property",
            name,
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
            "Title"
        ])
        .contains("Updated panel")
    );
    assert!(
        bus(&[
            "get-property",
            name,
            "/StatusNotifierItem",
            "org.kde.StatusNotifierItem",
            "Status"
        ])
        .contains("Passive")
    );
}
