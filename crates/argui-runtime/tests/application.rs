use argui_platform::{WindowConfig, WindowKey, WindowSpec};
use argui_runtime::{AppCommand, AppUpdate, ViewUpdate};

#[test]
fn app_updates_coalesce_each_window_to_its_strongest_invalidation() {
    let main = WindowKey::main();
    let auxiliary = WindowKey::new("auxiliary");
    let update = AppUpdate::none()
        .window(main.clone(), ViewUpdate::Paint)
        .window(auxiliary, ViewUpdate::Paint)
        .window(main.clone(), ViewUpdate::Rebuild)
        .window(main.clone(), ViewUpdate::None);
    assert_eq!(update.windows.len(), 2);
    assert_eq!(update.windows[0].window, main);
    assert_eq!(update.windows[0].update, ViewUpdate::Rebuild);
}

#[test]
fn app_updates_keep_commands_and_explicit_tray_changes() {
    let spec = WindowSpec::new(WindowKey::new("settings"), WindowConfig::default());
    let update = AppUpdate::none()
        .command(AppCommand::OpenWindow(spec.clone()))
        .tray_changed();
    assert_eq!(update.commands, vec![AppCommand::OpenWindow(spec)]);
    assert!(update.tray_changed);
}
