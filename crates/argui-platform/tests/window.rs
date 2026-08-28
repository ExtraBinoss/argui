use argui_platform::{
    AppIcon, ApplicationId, ApplicationIdentity, IconSet, WindowConfig, WindowKey, WindowSpec,
};

#[test]
fn default_window_is_a_decorated_resizable_surface() {
    let config = WindowConfig::default();

    assert_eq!(config.title, "Argui");
    assert!(config.decorations);
    assert!(config.resizable);
    assert!(!config.transparent);
    assert!(config.append_to_document);
    let _attributes = config.into_attributes();
}

#[test]
fn window_attributes_inherit_identity_and_accept_native_icons() {
    let identity_without_icon = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.window").unwrap(),
        "Identity title",
        IconSet::new(),
    );
    let _inherited = WindowConfig {
        title: String::new(),
        ..WindowConfig::default()
    }
    .into_attributes_with_identity(&identity_without_icon);

    let icon = AppIcon::from_rgba8(2, 2, vec![255; 2 * 2 * 4]).unwrap();
    let identity_with_icon = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.icon-window").unwrap(),
        "Fallback title",
        IconSet::new().with(icon),
    );
    let _explicit = WindowConfig {
        title: "Explicit title".into(),
        ..WindowConfig::default()
    }
    .into_attributes_with_identity(&identity_with_icon);

    let spec = WindowSpec::new(WindowKey::new("secondary"), WindowConfig::default());
    assert_eq!(spec.key.as_str(), "secondary");
    assert!(spec.visible);
}
