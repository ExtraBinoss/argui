use argui_platform::{TrayAction, TrayConfig, TrayItemId, TrayMenuItem, WindowKey};

#[test]
fn tray_items_expose_only_stable_actionable_ids() {
    let item = TrayMenuItem::Action {
        id: TrayItemId::new("show"),
        label: "Show".into(),
        enabled: true,
        action: TrayAction::ShowWindow(WindowKey::main()),
    };
    assert_eq!(item.id().unwrap().as_str(), "show");
    let checked = TrayMenuItem::Check {
        id: TrayItemId::new("checked"),
        label: "Checked".into(),
        enabled: true,
        checked: true,
        action: TrayAction::Custom("checked".into()),
    };
    assert_eq!(checked.id().unwrap().as_str(), "checked");
    let submenu = TrayMenuItem::Submenu {
        id: TrayItemId::new("more"),
        label: "More".into(),
        enabled: true,
        items: Vec::new(),
    };
    assert_eq!(submenu.id().unwrap().as_str(), "more");
    assert!(TrayMenuItem::Separator.id().is_none());
}

#[test]
fn tray_configs_reject_duplicate_ids_across_submenus() {
    let action = || TrayMenuItem::Action {
        id: TrayItemId::new("show"),
        label: "Show".into(),
        enabled: true,
        action: TrayAction::ShowWindow(WindowKey::main()),
    };
    let config = TrayConfig {
        menu: vec![
            action(),
            TrayMenuItem::Submenu {
                id: TrayItemId::new("more"),
                label: "More".into(),
                enabled: true,
                items: vec![action()],
            },
        ],
        ..TrayConfig::default()
    };
    assert!(config.validate().is_err());
}

#[test]
fn tray_configs_accept_unique_nested_items_and_reject_empty_ids() {
    let valid = TrayConfig {
        menu: vec![TrayMenuItem::Submenu {
            id: TrayItemId::new("more"),
            label: "More".into(),
            enabled: true,
            items: vec![TrayMenuItem::Check {
                id: TrayItemId::new("enabled"),
                label: "Enabled".into(),
                enabled: true,
                checked: false,
                action: TrayAction::Custom("enabled".into()),
            }],
        }],
        ..TrayConfig::default()
    };
    assert!(valid.validate().is_ok());

    let invalid = TrayConfig {
        menu: vec![TrayMenuItem::Action {
            id: TrayItemId::new(""),
            label: "Invalid".into(),
            enabled: true,
            action: TrayAction::Quit,
        }],
        ..TrayConfig::default()
    };
    assert!(invalid.validate().is_err());
}
