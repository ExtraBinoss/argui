//! Desktop application services for the QuickJS gallery.

mod windows;

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Sender},
    },
    time::Duration,
};

use argui_core::Color;
use argui_platform::{
    AppIcon, ApplicationConfig, ApplicationId, ApplicationIdentity, CloseBehavior, GlobalShortcut,
    GlobalShortcutState, IconSet, PlatformEvent, TrayAction, TrayConfig, TrayEvent, TrayItemId,
    TrayMenuItem, TrayPointerButton, WindowConfig, WindowKey,
};
use argui_runtime::{
    AppCommand, AppEvent, AppModel, AppUpdate, NativeHostApplicationRequest, RuntimeEvent,
    ViewUpdate, WindowEnvironment, WindowRuntimeEvent,
};
use argui_ui::{DesktopBackdrop, Element, Interaction, Sides, UiEventKind, percent};
use serde_json::{Value, json};

use crate::services::{ServiceOutcome, ServiceRegistry, ServiceResponse};

/// Gallery application policy for tray and global shortcut activations.
pub struct GalleryApp {
    message: String,
    events: Sender<ServiceResponse>,
    appearance: Arc<Mutex<CompanionAppearance>>,
}

#[derive(Clone, Copy, Default)]
pub(crate) struct CompanionAppearance {
    transparent: bool,
    backdrop: bool,
    whole_window: bool,
    open: bool,
}

impl GalleryApp {
    /// Creates the gallery model with the channel used for companion-window events.
    /// `events` delivers messages to the active QuickJS actor.
    pub fn new(events: Sender<ServiceResponse>) -> Self {
        Self::with_appearance(events, Arc::new(Mutex::new(CompanionAppearance::default())))
    }

    /// Shares creation-time companion appearance with native service handlers.
    /// `events` routes clicks to JavaScript and `appearance` stores window options.
    pub(crate) fn with_appearance(
        events: Sender<ServiceResponse>,
        appearance: Arc<Mutex<CompanionAppearance>>,
    ) -> Self {
        Self {
            message: "Waiting for the main window".into(),
            events,
            appearance,
        }
    }
}

impl AppModel for GalleryApp {
    fn view(&self, window: &WindowKey, _environment: WindowEnvironment) -> Option<Element> {
        (window.as_str() == "companion").then(|| {
            let appearance = *self
                .appearance
                .lock()
                .unwrap_or_else(|poison| poison.into_inner());
            let content = Element::text(format!(
                "Companion window\n\nMessage from main: {}\n\nClick here to reply to the gallery.",
                self.message,
            ));
            let backdrop = DesktopBackdrop::new(
                Color::srgba(0.13, 0.17, 0.27, 0.45),
                Color::srgb(0.13, 0.17, 0.27),
            );
            let content = if appearance.backdrop && !appearance.whole_window {
                Element::container([content])
                    .width(percent(0.8))
                    .height(percent(0.6))
                    .padding(Sides::length(18.0))
                    .desktop_backdrop(backdrop)
            } else if appearance.transparent && !appearance.backdrop {
                Element::container([content])
                    .width(percent(0.8))
                    .height(percent(0.6))
                    .padding(Sides::length(18.0))
                    .background(Color::srgba(0.13, 0.17, 0.27, 0.82))
            } else {
                content
            };
            let root = Element::container([content])
                .keyed("companion-root")
                .width(percent(1.0))
                .height(percent(1.0))
                .interaction(Interaction::default());
            if appearance.backdrop && appearance.whole_window {
                root.desktop_backdrop(backdrop)
            } else if appearance.backdrop || appearance.transparent {
                root
            } else {
                root.background(Color::srgb(0.13, 0.17, 0.27))
            }
        })
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Tray(TrayEvent::Click {
                button: TrayPointerButton::Primary,
                ..
            })
            | AppEvent::Tray(TrayEvent::Action {
                action: TrayAction::Custom(_),
                ..
            }) => AppUpdate::none().command(AppCommand::FocusWindow(WindowKey::main())),
            AppEvent::GlobalShortcut(event) if event.state == GlobalShortcutState::Pressed => {
                AppUpdate::none().command(AppCommand::FocusWindow(WindowKey::main()))
            }
            AppEvent::HostMessage { window, message } if window.as_str() == "companion" => {
                self.message = message.clone();
                AppUpdate::none().window(window.clone(), ViewUpdate::Rebuild)
            }
            AppEvent::Ui { window, event }
                if window.as_str() == "companion"
                    && matches!(event.kind, UiEventKind::Click(_)) =>
            {
                let _ = self.events.send(ServiceResponse {
                    session: u32::MAX,
                    window: "main".into(),
                    request_id: 0,
                    outcome: ServiceOutcome::Event(json!({
                        "type":"window", "window":"companion",
                        "message":format!("Companion received: {}", self.message),
                    })),
                });
                AppUpdate::none()
            }
            AppEvent::Window {
                window,
                event: PlatformEvent::Closed,
            } if window.as_str() == "companion" => {
                self.appearance
                    .lock()
                    .unwrap_or_else(|poison| poison.into_inner())
                    .open = false;
                AppUpdate::none()
            }
            AppEvent::Window {
                window,
                event: PlatformEvent::PreferencesChanged(preferences),
            } if window.as_str() == "main" => {
                let scheme = match preferences.color_scheme.value {
                    argui_core::ColorScheme::Light => "light",
                    argui_core::ColorScheme::Dark => "dark",
                };
                let _ = self.events.send(ServiceResponse {
                    session: u32::MAX,
                    window: "main".into(),
                    request_id: 0,
                    outcome: ServiceOutcome::Event(
                        json!({ "type": "systemScheme", "scheme": scheme }),
                    ),
                });
                AppUpdate::none()
            }
            _ => AppUpdate::none(),
        }
    }
}

/// Creates the gallery's native identity, visible tray icon, and close-to-tray policy.
///
/// # Errors
/// Returns an error if the application identifier or icon cannot be encoded.
pub(crate) fn gallery_config() -> Result<ApplicationConfig, Box<dyn std::error::Error>> {
    let identity = ApplicationIdentity::new(
        ApplicationId::new("dev.argui.gallery")?,
        "Argui Gallery",
        IconSet::single(gallery_icon()?),
    );
    let tray = TrayConfig {
        tooltip: Some("Argui Gallery".into()),
        title: Some("Argui Gallery".into()),
        menu: complete_menu(Vec::new()),
        show_menu_on_left_click: false,
        ..TrayConfig::default()
    };
    Ok(ApplicationConfig::new(
        identity,
        WindowConfig {
            title: "Argui Gallery / QuickJS".into(),
            close_behavior: CloseBehavior::Hide,
            ..WindowConfig::default()
        },
    )
    .with_tray(tray))
}

/// Draws a compact blue gallery mark used by both the native window and tray.
///
/// # Errors
/// Returns an error if the generated RGBA pixels cannot be encoded as PNG.
fn gallery_icon() -> Result<AppIcon, argui_platform::AppIconError> {
    let mut pixels = Vec::with_capacity(32 * 32 * 4);
    for y in 0_i32..32 {
        for x in 0_i32..32 {
            let dx = x - 16;
            let dy = y - 16;
            let inside = dx * dx + dy * dy < 225;
            let mark = ((x - 16).abs() <= (24 - y) / 2
                && (8..26).contains(&y)
                && ((x - 16).abs() >= (21 - y) / 2 || (17..20).contains(&y)))
                || ((x - 16).abs() < 2 && (6..9).contains(&y));
            let rgba = if !inside {
                [0, 0, 0, 0]
            } else if mark {
                [255, 255, 255, 255]
            } else {
                [45, 110, 225, 255]
            };
            pixels.extend_from_slice(&rgba);
        }
    }
    AppIcon::from_rgba8(32, 32, pixels)
}

/// Appends reliable show and quit commands to custom tray items.
/// `items` are application commands, retained in their original order.
fn complete_menu(mut items: Vec<TrayMenuItem>) -> Vec<TrayMenuItem> {
    if !items.is_empty() {
        items.push(TrayMenuItem::Separator);
    }
    items.push(TrayMenuItem::Action {
        id: TrayItemId::new("argui-show"),
        label: "Show gallery".into(),
        enabled: true,
        action: TrayAction::FocusWindow(WindowKey::main()),
    });
    items.push(TrayMenuItem::Action {
        id: TrayItemId::new("argui-quit"),
        label: "Quit gallery".into(),
        enabled: true,
        action: TrayAction::Quit,
    });
    items
}

#[derive(Clone)]
struct TrayState {
    config: TrayConfig,
    enabled: bool,
    preferred_close_behavior: CloseBehavior,
}

/// Registers TSX-callable tray, close-policy, focus, and global-shortcut services.
/// `registry` owns the handlers; `sender` posts requests to the UI thread; `tray`
/// contains the initial visible icon and menu.
pub(crate) fn register_application_services(
    registry: &ServiceRegistry,
    sender: Sender<NativeHostApplicationRequest>,
    tray: Option<TrayConfig>,
) -> Arc<Mutex<CompanionAppearance>> {
    let appearance = Arc::new(Mutex::new(CompanionAppearance::default()));
    let state = Arc::new(Mutex::new(TrayState {
        config: tray.expect("desktop gallery config has a tray"),
        enabled: true,
        preferred_close_behavior: CloseBehavior::Hide,
    }));
    let menu_state = Arc::clone(&state);
    let menu_sender = sender.clone();
    registry.register("menus", "set", move |payload| {
        let Some(items) = payload.get("items").and_then(Value::as_array) else {
            return ServiceOutcome::Error("menus.set requires an items array".into());
        };
        let custom = match parse_menu_items(items, 0) {
            Ok(custom) => custom,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let mut state = menu_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        let mut config = state.config.clone();
        config.menu = complete_menu(custom);
        if let Err(error) = config.validate() {
            return ServiceOutcome::Error(error.to_string());
        }
        if state.enabled {
            let outcome = dispatch(&menu_sender, |reply| {
                NativeHostApplicationRequest::SetTray(Some(config.clone()), reply)
            });
            if !matches!(outcome, ServiceOutcome::Ok(_)) {
                return outcome;
            }
        }
        state.config = config;
        ServiceOutcome::Ok(Value::Null)
    });
    let tray_state = Arc::clone(&state);
    let tray_sender = sender.clone();
    registry.register("tray", "setEnabled", move |payload| {
        let Some(enabled) = payload.get("enabled").and_then(Value::as_bool) else {
            return ServiceOutcome::Error("tray.setEnabled requires enabled: boolean".into());
        };
        let mut state = tray_state
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if state.enabled == enabled {
            return ServiceOutcome::Ok(Value::Null);
        }
        if !enabled {
            let outcome = dispatch(&tray_sender, |reply| {
                NativeHostApplicationRequest::SetCloseBehavior(CloseBehavior::Quit, reply)
            });
            if !matches!(outcome, ServiceOutcome::Ok(_)) {
                return outcome;
            }
        }
        let config = enabled.then(|| state.config.clone());
        let outcome = dispatch(&tray_sender, |reply| {
            NativeHostApplicationRequest::SetTray(config, reply)
        });
        if !matches!(outcome, ServiceOutcome::Ok(_)) {
            return outcome;
        }
        if enabled {
            let behavior = state.preferred_close_behavior;
            let outcome = dispatch(&tray_sender, |reply| {
                NativeHostApplicationRequest::SetCloseBehavior(behavior, reply)
            });
            if !matches!(outcome, ServiceOutcome::Ok(_)) {
                return outcome;
            }
        }
        state.enabled = enabled;
        ServiceOutcome::Ok(Value::Null)
    });
    let close_sender = sender.clone();
    let close_state = Arc::clone(&state);
    registry.register("windows", "setCloseBehavior", move |payload| {
        let behavior = match payload.get("behavior").and_then(Value::as_str) {
            Some("hide") => CloseBehavior::Hide,
            Some("quit") => CloseBehavior::Quit,
            _ => return ServiceOutcome::Error("close behavior must be 'hide' or 'quit'".into()),
        };
        let outcome = dispatch(&close_sender, |reply| {
            NativeHostApplicationRequest::SetCloseBehavior(behavior, reply)
        });
        if matches!(outcome, ServiceOutcome::Ok(_)) {
            close_state
                .lock()
                .unwrap_or_else(|poison| poison.into_inner())
                .preferred_close_behavior = behavior;
        }
        outcome
    });
    let focus_sender = sender.clone();
    registry.register("windows", "focus", move |_| {
        dispatch(&focus_sender, NativeHostApplicationRequest::FocusWindow)
    });
    let shortcut_sender = sender.clone();
    registry.register("shortcuts", "set", move |payload| {
        let Some(items) = payload.get("shortcuts").and_then(Value::as_array) else {
            return ServiceOutcome::Error("shortcuts.set requires a shortcuts array".into());
        };
        let shortcuts = match items
            .iter()
            .map(|item| {
                let id = item
                    .get("id")
                    .and_then(Value::as_str)
                    .ok_or("shortcut id must be a string")?;
                let accelerator = item
                    .get("accelerator")
                    .and_then(Value::as_str)
                    .ok_or("shortcut accelerator must be a string")?;
                Ok::<_, &str>(GlobalShortcut::new(id, accelerator))
            })
            .collect::<Result<Vec<_>, _>>()
        {
            Ok(shortcuts) => shortcuts,
            Err(error) => return ServiceOutcome::Error(error.into()),
        };
        dispatch(&shortcut_sender, |reply| {
            NativeHostApplicationRequest::SetGlobalShortcuts(shortcuts, reply)
        })
    });
    windows::register_window_services(registry, sender, &appearance);
    appearance
}

/// Parses a bounded list of translated menu labels and associated actions.
/// `items` are JSON menu descriptions; `depth` prevents unbounded nesting.
///
/// # Errors
/// Returns a field or nesting validation error.
fn parse_menu_items(items: &[Value], depth: usize) -> Result<Vec<TrayMenuItem>, String> {
    if depth > 8 || items.len() > 100 {
        return Err("tray menu exceeds its depth or item limit".into());
    }
    items
        .iter()
        .map(|item| {
            if item.get("separator").and_then(Value::as_bool) == Some(true) {
                return Ok(TrayMenuItem::Separator);
            }
            let id = item
                .get("id")
                .and_then(Value::as_str)
                .filter(|id| !id.is_empty())
                .ok_or("menu item id must be a nonempty string")?;
            let label = item
                .get("label")
                .and_then(Value::as_str)
                .filter(|label| !label.is_empty())
                .ok_or("menu item label must be a nonempty string")?
                .to_owned();
            let enabled = item.get("enabled").and_then(Value::as_bool).unwrap_or(true);
            if let Some(children) = item.get("children") {
                let children = children
                    .as_array()
                    .ok_or("menu children must be an array")?;
                return Ok(TrayMenuItem::Submenu {
                    id: TrayItemId::new(id),
                    label,
                    enabled,
                    items: parse_menu_items(children, depth + 1)?,
                });
            }
            let action = match item
                .get("action")
                .and_then(Value::as_str)
                .unwrap_or("custom")
            {
                "custom" => TrayAction::Custom(id.into()),
                "focus" => TrayAction::FocusWindow(WindowKey::main()),
                "hide" => TrayAction::HideWindow(WindowKey::main()),
                "toggle" => TrayAction::ToggleWindow(WindowKey::main()),
                "quit" => TrayAction::Quit,
                _ => return Err("menu action must be custom, focus, hide, toggle, or quit".into()),
            };
            Ok(TrayMenuItem::Action {
                id: TrayItemId::new(id),
                label,
                enabled,
                action,
            })
        })
        .collect()
}

/// Sends one native request and converts its completion to a service result.
/// `sender` reaches the UI thread and `build` inserts the reply channel.
fn dispatch(
    sender: &Sender<NativeHostApplicationRequest>,
    build: impl FnOnce(Sender<Result<(), String>>) -> NativeHostApplicationRequest,
) -> ServiceOutcome {
    let (reply, result) = mpsc::channel();
    if sender.send(build(reply)).is_err() {
        return ServiceOutcome::Error("native application runtime has stopped".into());
    }
    match result.recv_timeout(Duration::from_secs(30)) {
        Ok(Ok(())) => ServiceOutcome::Ok(Value::Null),
        Ok(Err(error)) => ServiceOutcome::Error(error),
        Err(_) => ServiceOutcome::Error("native application request timed out".into()),
    }
}

/// Converts tray and shortcut activations and asynchronous failures to TSX events.
/// `event` is emitted by the native application runtime; unrelated events return `None`.
pub(crate) fn runtime_service_event(event: &RuntimeEvent) -> Option<ServiceResponse> {
    let value = match event {
        RuntimeEvent::Tray(TrayEvent::Action {
            id,
            action: TrayAction::Custom(_),
        }) => {
            json!({"type":"menu", "id":id.as_str()})
        }
        RuntimeEvent::GlobalShortcut(event) => json!({
            "type":"shortcut", "id":event.id.as_str(),
            "state": if event.state == GlobalShortcutState::Pressed { "pressed" } else { "released" },
        }),
        RuntimeEvent::GlobalShortcutsFailed(message) => {
            json!({"type":"shortcutError", "message":message})
        }
        RuntimeEvent::TrayFailed(message) => json!({"type":"trayError", "message":message}),
        RuntimeEvent::Window {
            window,
            event: WindowRuntimeEvent::DesktopBackdropUnavailable(message),
        } => {
            json!({"type":"windowAppearanceError", "window":window.as_str(), "message":message})
        }
        _ => return None,
    };
    Some(ServiceResponse {
        session: u32::MAX,
        window: "main".into(),
        request_id: 0,
        outcome: ServiceOutcome::Event(value),
    })
}
