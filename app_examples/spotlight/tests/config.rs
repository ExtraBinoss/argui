use argui::{
    core::{BackdropMaterial, ColorScheme},
    platform::{CloseBehavior, WindowKey, WindowLevel},
};
use argui_showcase::spotlight::SPOTLIGHT_SHORTCUT_ID;

#[test]
fn native_config_keeps_the_launcher_alive_while_hidden() {
    let config = argui_example_spotlight::application_config().unwrap();
    config.validate().unwrap();
    let main = config
        .windows
        .iter()
        .find(|window| window.key == WindowKey::main())
        .unwrap();
    assert_eq!(main.window.close_behavior, CloseBehavior::Hide);
    assert_eq!(main.window.level, WindowLevel::Normal);
    assert_eq!(main.window.desktop_backdrop, Some(BackdropMaterial::Glass));
    assert!(main.window.transparent);
    assert_eq!(config.preferences.color_scheme, Some(ColorScheme::Light));
    assert_eq!(
        config.global_shortcuts[0].id.as_str(),
        SPOTLIGHT_SHORTCUT_ID
    );
    assert_eq!(config.global_shortcuts[0].accelerator, "CmdOrCtrl+Space");
    assert!(config.tray.is_some());
}
