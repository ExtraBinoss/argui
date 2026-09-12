use argui::platform::WindowKey;
use argui_devtools::DockMode;
use argui_web_demo::{application_config, devtools_app, renderer_config};

#[test]
fn browser_application_configuration_uses_the_showcase_identity_and_default_window() {
    let config = application_config().unwrap();

    assert_eq!(config.identity.id.as_str(), "dev.argui.state");
    assert_eq!(config.identity.display_name, "Argui state showcase");
    assert!(config.identity.icons.is_empty());
    assert_eq!(config.windows.len(), 1);
    assert_eq!(config.windows[0].key, WindowKey::main());
    assert_eq!(config.windows[0].window.title, "Argui state showcase");
    assert_eq!(config.windows[0].window.width, 960.0);
    assert_eq!(config.windows[0].window.height, 640.0);
    assert!(config.validate().is_ok());
}

#[test]
fn browser_renderer_configuration_includes_devtools_effects() {
    let config = renderer_config().unwrap();

    assert!(!config.profiling);
    assert!(config.effects.definitions().len() >= 2);
}

#[test]
fn showcase_is_wrapped_in_the_default_single_window_devtools_app() {
    let app = devtools_app();

    assert_eq!(app.dock_mode(), DockMode::Bottom);
    assert_eq!(app.tools_window(), &WindowKey::main());
}
