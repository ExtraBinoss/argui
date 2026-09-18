use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, GlobalShortcut, IconSet, TrayAction,
    TrayConfig, TrayItemId, TrayMenuItem, UiZoomConfig, WindowConfig, WindowKey, WindowSpec,
};

fn identity() -> ApplicationIdentity {
    ApplicationIdentity::new(
        ApplicationId::new("dev.argui.test").unwrap(),
        "Test",
        IconSet::new(),
    )
}

#[test]
fn application_ui_zoom_is_enabled_by_default_and_can_be_disabled() {
    let default = ApplicationConfig::new(identity(), WindowConfig::default());
    assert!(default.ui_zoom.enabled);

    let disabled = default.with_ui_zoom(UiZoomConfig::disabled());
    assert!(!disabled.ui_zoom.enabled);
    assert!(disabled.validate().is_ok());
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
    assert_eq!(
        ApplicationConfigError::InvalidGlobalShortcut("duplicate shortcut".into()).to_string(),
        "duplicate shortcut"
    );
}

#[test]
fn application_configs_validate_global_shortcuts() {
    let valid = ApplicationConfig::new(identity(), WindowConfig::default())
        .with_global_shortcut(GlobalShortcut::new("spotlight", "CmdOrCtrl+Space"));
    assert_eq!(valid.global_shortcuts.len(), 1);
    assert!(valid.validate().is_ok());

    for shortcuts in [
        vec![GlobalShortcut::new("", "Ctrl+Space")],
        vec![GlobalShortcut::new("open", "  ")],
        vec![
            GlobalShortcut::new("open", "Ctrl+Space"),
            GlobalShortcut::new("open", "Ctrl+KeyK"),
        ],
        vec![
            GlobalShortcut::new("open", "Ctrl+Space"),
            GlobalShortcut::new("search", "ctrl+space"),
        ],
    ] {
        let mut invalid = ApplicationConfig::new(identity(), WindowConfig::default());
        invalid.global_shortcuts = shortcuts;
        assert!(invalid.validate().is_err());
    }
}
