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
    assert!(
        ApplicationConfig::new(identity(), WindowConfig::default())
            .validate()
            .is_ok()
    );

    let config = ApplicationConfig::new(identity(), WindowConfig::default())
        .with_window(WindowSpec::new(WindowKey::main(), WindowConfig::default()));
    assert!(config.validate().is_err());

    let config = ApplicationConfig::new(identity(), WindowConfig::default())
        .with_window(WindowSpec::new(WindowKey::new(""), WindowConfig::default()));
    assert!(config.validate().is_err());
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

#[test]
fn invalid_configuration_diagnostics_identify_the_offending_window_or_tray() {
    use argui_platform::{ApplicationConfigError, PreferenceOverrides};
    let config = ApplicationConfig::new(identity(), WindowConfig::default()).with_preferences(
        PreferenceOverrides {
            color_scheme: Some(argui_core::ColorScheme::Dark),
            ..Default::default()
        },
    );
    assert_eq!(
        config.preferences.color_scheme,
        Some(argui_core::ColorScheme::Dark)
    );
    assert_eq!(
        ApplicationConfigError::EmptyWindowKey.to_string(),
        "window keys cannot be empty"
    );
    assert_eq!(
        ApplicationConfigError::DuplicateWindowKey(WindowKey::new("settings")).to_string(),
        "duplicate window key: settings"
    );
    assert_eq!(
        ApplicationConfigError::InvalidTray("duplicate action".into()).to_string(),
        "duplicate action"
    );
}
