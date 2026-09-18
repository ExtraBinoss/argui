use argui_platform::{
    ApplicationConfig, ApplicationId, ApplicationIdentity, IconSet, PlatformEvent, WindowConfig,
    WindowKey,
};
use argui_runtime::{AppCommand, AppEvent, AppModel, AppUpdate, WindowEnvironment};
use argui_ui::Element;

struct WaylandVisibilityApp;

impl AppModel for WaylandVisibilityApp {
    fn view(&self, key: &WindowKey, _: WindowEnvironment) -> Option<Element> {
        (key == &WindowKey::main()).then(|| Element::text("Wayland visibility test"))
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Window {
                window,
                event: PlatformEvent::Opened { .. },
            } => AppUpdate::none().command(AppCommand::HideWindow(window.clone())),
            AppEvent::Window {
                event: PlatformEvent::VisibilityChanged(false),
                ..
            } => AppUpdate::none().command(AppCommand::Quit),
            _ => AppUpdate::none(),
        }
    }
}

/// Runs a Winit-backed Wayland child that hides its window before quitting.
pub fn run_child() {
    let config = ApplicationConfig::new(
        ApplicationIdentity::new(
            ApplicationId::new("dev.argui.visibility-test").unwrap(),
            "Wayland visibility integration",
            IconSet::default(),
        ),
        WindowConfig {
            title: "Argui Wayland visibility test".into(),
            ..Default::default()
        },
    );
    argui_runtime::run_application(config, Default::default(), WaylandVisibilityApp, |_| {})
        .unwrap();
}

/// Launches the visibility child while keeping Winit on the private Wayland display.
pub fn verify() {
    if std::env::var_os("WAYLAND_DISPLAY").is_none() {
        return;
    }
    let status = std::process::Command::new(std::env::current_exe().unwrap())
        .env("ARGUI_WAYLAND_VISIBILITY_CHILD", "1")
        // Argui uses this variable only to avoid selecting the optional GTK host;
        // Winit still selects Wayland because the test display has no DISPLAY.
        .env("GDK_BACKEND", "x11")
        .status()
        .unwrap();
    assert!(status.success(), "Wayland visibility child failed");
}
