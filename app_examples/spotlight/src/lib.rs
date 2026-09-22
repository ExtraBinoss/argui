//! Native and WebAssembly entry points for the Argui Spotlight example.

#[cfg(target_os = "linux")]
use std::{fs::OpenOptions, io::Write, path::PathBuf};

#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use argui::platform::{
    GlobalShortcut, TrayAction, TrayConfig, TrayItemId, TrayMenuItem, WindowKey,
};
use argui::{
    core::{BackdropMaterial, ColorScheme},
    platform::{
        AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, CloseBehavior, IconSet,
        PreferenceOverrides, WindowConfig, WindowLevel,
    },
    render::RendererConfig,
    runtime::{RuntimeEvent, WindowRuntimeEvent, run_application_with_text_engine},
    text::TextEngine,
};
#[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
use argui_showcase::spotlight::SPOTLIGHT_SHORTCUT_ID;
use argui_showcase::spotlight::{
    SPOTLIGHT_WINDOW_HEIGHT, SPOTLIGHT_WINDOW_WIDTH, SpotlightShowcase,
};

const NOTO_SANS: &[u8] =
    include_bytes!("../../../crates/argui-web-demo/assets/fonts/NotoSans-Regular.ttf");
const APP_ICON: &[u8] = include_bytes!("../../../crates/argui/examples/assets/astra-icon-256.png");
#[cfg(target_os = "linux")]
const DEVELOPMENT_DESKTOP_MARKER: &str = "X-Argui-Development=true";
#[cfg(target_os = "linux")]
const LEGACY_DEVELOPMENT_DESKTOP_ENTRY: &str = "[Desktop Entry]\nType=Application\nName=Argui Spotlight\nExec=argui-example-spotlight\nNoDisplay=true\n";
#[cfg(target_os = "linux")]
struct DevelopmentDesktopEntry {
    path: Option<PathBuf>,
    contents: String,
}

#[cfg(target_os = "linux")]
impl Drop for DevelopmentDesktopEntry {
    fn drop(&mut self) {
        let Some(path) = self.path.take() else {
            return;
        };
        if std::fs::read_to_string(&path).is_ok_and(|contents| contents == self.contents) {
            let _ = std::fs::remove_file(path);
        }
    }
}

/// Builds a valid desktop entry whose executable points at the current binary.
#[cfg(target_os = "linux")]
fn development_desktop_entry() -> std::io::Result<String> {
    let executable = std::env::current_exe()?;
    let executable = executable.to_string_lossy();
    let executable = executable
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('`', "\\`")
        .replace('$', "\\$")
        .replace('%', "%%");
    Ok(format!(
        "[Desktop Entry]\nType=Application\nName=Argui Spotlight\nExec=\"{executable}\"\nNoDisplay=true\n{DEVELOPMENT_DESKTOP_MARKER}\n"
    ))
}

/// Returns the per-user data directory used for a development desktop entry.
#[cfg(target_os = "linux")]
fn user_data_home() -> Option<PathBuf> {
    std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .or_else(|| {
            std::env::var_os("HOME")
                .map(PathBuf::from)
                .filter(|path| path.is_absolute())
                .map(|path| path.join(".local/share"))
        })
}

/// Makes the uninstalled example discoverable by the Wayland shortcuts portal.
///
/// An existing `dev.argui.spotlight.desktop` file not created by this example
/// is preserved. Stale example entries are refreshed with the current absolute
/// executable path. A file created by this process is removed on clean shutdown
/// if its contents are unchanged.
///
/// # Errors
/// Returns an I/O error when the applications directory or entry cannot be
/// created or written.
#[cfg(target_os = "linux")]
fn install_development_desktop_entry() -> std::io::Result<DevelopmentDesktopEntry> {
    let contents = development_desktop_entry()?;
    let Some(data_home) = user_data_home() else {
        return Ok(DevelopmentDesktopEntry {
            path: None,
            contents,
        });
    };
    let applications = data_home.join("applications");
    std::fs::create_dir_all(&applications)?;
    let path = applications.join("dev.argui.spotlight.desktop");
    if let Ok(existing) = std::fs::read_to_string(&path) {
        let owned_by_example = existing == LEGACY_DEVELOPMENT_DESKTOP_ENTRY
            || existing
                .lines()
                .any(|line| line == DEVELOPMENT_DESKTOP_MARKER);
        if !owned_by_example {
            return Ok(DevelopmentDesktopEntry {
                path: None,
                contents,
            });
        }
        std::fs::remove_file(&path)?;
    }
    let mut file = match OpenOptions::new().write(true).create_new(true).open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
            return Ok(DevelopmentDesktopEntry {
                path: None,
                contents,
            });
        }
        Err(error) => return Err(error),
    };
    if let Err(error) = file.write_all(contents.as_bytes()) {
        let _ = std::fs::remove_file(&path);
        return Err(error);
    }
    Ok(DevelopmentDesktopEntry {
        path: Some(path),
        contents,
    })
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(
    inline_js = "export function renderer_state(state) { window.dispatchEvent(new CustomEvent('argui:renderer-state', { detail: { state } })); }"
)]
extern "C" {
    /// Notifies the browser host that renderer initialization completed or failed.
    fn renderer_state(state: &str);
}

/// Builds the shared window configuration and native lifecycle integrations.
///
/// Browser builds omit tray and global-shortcut requests because those services
/// are owned by the desktop operating system.
///
/// # Errors
/// Returns an error when the stable application ID or embedded icon is invalid.
pub fn application_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.spotlight")?,
        "Argui Spotlight",
        IconSet::single(AppIcon::from_png(APP_ICON)?),
    );
    let config = ApplicationConfig::new(
        identity,
        WindowConfig {
            title: "Argui Spotlight".into(),
            width: SPOTLIGHT_WINDOW_WIDTH,
            height: SPOTLIGHT_WINDOW_HEIGHT,
            decorations: false,
            resizable: true,
            transparent: true,
            desktop_backdrop: Some(BackdropMaterial::Glass),
            native_shadow: cfg!(any(target_os = "windows", target_os = "macos")),
            level: WindowLevel::Normal,
            close_behavior: CloseBehavior::Hide,
            ..WindowConfig::default()
        },
    )
    .with_preferences(PreferenceOverrides {
        color_scheme: Some(ColorScheme::Light),
        ..PreferenceOverrides::default()
    });
    #[cfg(any(target_os = "linux", target_os = "windows", target_os = "macos"))]
    let config = config
        .with_global_shortcut(GlobalShortcut::new(
            SPOTLIGHT_SHORTCUT_ID,
            "CmdOrCtrl+Space",
        ))
        .with_tray(TrayConfig {
            tooltip: Some("Argui Spotlight — Cmd/Ctrl+Space".into()),
            menu: vec![
                TrayMenuItem::Action {
                    id: TrayItemId::new("show"),
                    label: "Show Spotlight".into(),
                    enabled: true,
                    action: TrayAction::FocusWindow(WindowKey::main()),
                },
                TrayMenuItem::Separator,
                TrayMenuItem::Action {
                    id: TrayItemId::new("quit"),
                    label: "Quit".into(),
                    enabled: true,
                    action: TrayAction::Quit,
                },
            ],
            ..TrayConfig::default()
        });
    Ok(config)
}

/// Starts the Spotlight application on the current native or browser target.
///
/// # Errors
/// Returns identity, icon, renderer, or platform startup failures.
pub fn launch() -> Result<(), Box<dyn std::error::Error>> {
    #[cfg(target_os = "linux")]
    let _development_desktop_entry = install_development_desktop_entry().map_err(|error| {
        format!("could not prepare the Wayland development desktop entry: {error}")
    })?;
    let text = TextEngine::from_embedded_fonts([NOTO_SANS], "Noto Sans", "Noto Sans", "Noto Sans");
    run_application_with_text_engine(
        application_config()?,
        RendererConfig::default(),
        text,
        SpotlightShowcase::default(),
        handle_event,
    )?;
    Ok(())
}

/// Publishes browser readiness and keeps recoverable native integration failures visible.
fn handle_event(event: RuntimeEvent) {
    #[cfg(target_arch = "wasm32")]
    if matches!(
        &event,
        RuntimeEvent::RendererReady
            | RuntimeEvent::Window {
                event: WindowRuntimeEvent::RendererReady,
                ..
            }
    ) {
        renderer_state("ready");
    }
    let error = match event {
        RuntimeEvent::RendererFailed(message)
        | RuntimeEvent::LayoutFailed(message)
        | RuntimeEvent::CommandFailed(message)
        | RuntimeEvent::GlobalShortcutsFailed(message)
        | RuntimeEvent::GlobalShortcutsUnavailable(message)
        | RuntimeEvent::TrayFailed(message)
        | RuntimeEvent::TrayUnavailable(message) => Some(message),
        RuntimeEvent::Window {
            event:
                WindowRuntimeEvent::RendererFailed(message) | WindowRuntimeEvent::LayoutFailed(message),
            ..
        } => Some(message),
        _ => None,
    };
    if let Some(error) = error {
        eprintln!("{error}");
        #[cfg(target_arch = "wasm32")]
        renderer_state("error");
    }
}

/// Starts the browser build when the WebAssembly module loads.
///
/// # Errors
/// Returns a JavaScript error when application startup fails.
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    std::panic::set_hook(Box::new(|info| {
        renderer_state("error");
        eprintln!("{info}");
    }));
    launch().map_err(|error| wasm_bindgen::JsValue::from_str(&error.to_string()))
}
