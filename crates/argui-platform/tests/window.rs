use argui_platform::{
    AppIcon, ApplicationId, ApplicationIdentity, IconSet, WindowBackend, WindowConfig, WindowKey,
    WindowLevel, WindowSpec,
};

#[test]
fn default_window_is_a_decorated_resizable_surface() {
    let config = WindowConfig::default();

    assert_eq!(config.title, "Argui");
    assert!(config.decorations);
    assert!(config.resizable);
    assert!(!config.transparent);
    assert_eq!(config.desktop_backdrop, None);
    assert!(!config.native_shadow);
    assert_eq!(config.level, WindowLevel::Normal);
    assert!(config.append_to_document);
    assert_eq!(config.focus_on_launch, cfg!(not(target_arch = "wasm32")));
    assert_eq!(config.safe_area_insets, None);
    let attributes = config.into_attributes();
    assert_eq!(attributes.active, cfg!(not(target_arch = "wasm32")));
}

#[test]
fn initial_focus_is_configurable() {
    let attributes = WindowConfig::default()
        .with_focus_on_launch(false)
        .into_attributes();

    assert!(!attributes.active);
}

#[test]
fn window_safe_area_override_uses_logical_insets() {
    let insets = argui_platform::Insets::new(24.0, 0.0, 30.0, 0.0);
    let config = WindowConfig::default().with_safe_area_insets(insets);

    assert_eq!(config.safe_area_insets, Some(insets));
    let _attributes = config.into_attributes();
}

#[test]
fn desktop_materials_opt_into_window_transparency() {
    assert_eq!(
        argui_core::BackdropMaterial::default(),
        argui_core::BackdropMaterial::Glass
    );
    for material in [
        argui_core::BackdropMaterial::Glass,
        argui_core::BackdropMaterial::Sidebar,
        argui_core::BackdropMaterial::Header,
    ] {
        let attributes = WindowConfig {
            desktop_backdrop: Some(material),
            ..WindowConfig::default()
        }
        .into_attributes();
        assert!(attributes.transparent);
    }
}

#[test]
fn capabilities_expose_real_backend_limits() {
    let wayland = WindowBackend::Wayland.capabilities();
    assert!(wayland.native_drag);
    assert!(!wayland.native_shadow);
    assert!(wayland.mouse_passthrough);
    assert!(!wayland.window_level);

    for backend in [
        WindowBackend::Windows,
        WindowBackend::MacOs,
        WindowBackend::X11,
    ] {
        let capabilities = backend.capabilities();
        assert!(capabilities.native_drag);
        assert!(capabilities.minimize);
        assert!(capabilities.maximize);
        assert!(capabilities.window_level);
        assert!(capabilities.mouse_passthrough);
    }
    assert!(WindowBackend::Windows.capabilities().native_shadow);
    assert!(WindowBackend::MacOs.capabilities().native_shadow);

    for backend in [WindowBackend::Android, WindowBackend::Ios] {
        let capabilities = backend.capabilities();
        assert_eq!(capabilities.backend, backend);
        assert!(!capabilities.native_drag);
        assert!(!capabilities.native_shadow);
        assert!(!capabilities.minimize);
        assert!(!capabilities.maximize);
        assert!(!capabilities.window_level);
        assert!(!capabilities.mouse_passthrough);
    }

    assert_eq!(
        WindowBackend::Web.capabilities(),
        argui_platform::WindowCapabilities {
            backend: WindowBackend::Web,
            native_drag: false,
            native_shadow: false,
            minimize: false,
            maximize: false,
            window_level: false,
            mouse_passthrough: false,
        }
    );
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
