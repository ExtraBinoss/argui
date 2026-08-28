use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, TrayAction, TrayConfig,
    TrayItemId, TrayMenuItem, WindowConfig, WindowKey, WindowSpec,
};

fn identity() -> ApplicationIdentity {
    ApplicationIdentity::new(
        ApplicationId::new("dev.argui.test").unwrap(),
        "Test",
        IconSet::new(),
    )
}

#[test]
fn application_configs_reject_duplicate_or_empty_window_keys() {
    let config = ApplicationConfig::new(identity(), WindowConfig::default())
        .with_window(WindowSpec::new(WindowKey::main(), WindowConfig::default()));
    assert!(config.validate().is_err());

    let config = ApplicationConfig::new(identity(), WindowConfig::default())
        .with_window(WindowSpec::new(WindowKey::new(""), WindowConfig::default()));
    assert!(config.validate().is_err());
}

#[test]
fn legacy_windows_receive_a_deterministic_development_identity() {
    let config = ApplicationConfig::legacy(WindowConfig {
        title: "Legacy".into(),
        ..WindowConfig::default()
    });
    assert_eq!(config.identity.display_name, "Legacy");
    assert_eq!(config.identity.id.as_str(), "dev.argui.application");
    assert!(config.validate().is_ok());
}

#[test]
fn application_configs_validate_their_optional_tray() {
    let valid = ApplicationConfig::new(identity(), WindowConfig::default()).with_tray(TrayConfig {
        menu: vec![TrayMenuItem::Action {
            id: TrayItemId::new("quit"),
            label: "Quit".into(),
            enabled: true,
            action: TrayAction::Quit,
        }],
        ..TrayConfig::default()
    });
    assert!(valid.validate().is_ok());

    let invalid =
        ApplicationConfig::new(identity(), WindowConfig::default()).with_tray(TrayConfig {
            menu: vec![TrayMenuItem::Action {
                id: TrayItemId::new(""),
                label: "Invalid".into(),
                enabled: true,
                action: TrayAction::Quit,
            }],
            ..TrayConfig::default()
        });
    assert!(invalid.validate().is_err());
}
