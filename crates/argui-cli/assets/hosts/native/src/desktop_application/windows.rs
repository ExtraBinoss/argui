//! Companion-window creation and live native window services.

use std::sync::{
    Arc, Mutex,
    mpsc::{self, Sender},
};
use std::time::Duration;

use argui_core::{BackdropMaterial, Point, Rect, Size};
use argui_platform::{
    CloseBehavior, WindowBackend, WindowConfig, WindowInputRegion, WindowKey, WindowLevel,
    WindowSpec,
};
use argui_runtime::{NativeHostApplicationRequest, NativeWindowInfo};
use serde_json::{Value, json};

use super::{CompanionAppearance, dispatch};
use crate::services::{ServiceOutcome, ServiceRegistry};

/// Registers companion-window creation, messaging, inspection, and mutation.
/// `registry` owns the services, `sender` reaches the native UI thread, and
/// `appearance` shares creation-time fallback settings with the gallery model.
pub(super) fn register_window_services(
    registry: &ServiceRegistry,
    sender: Sender<NativeHostApplicationRequest>,
    appearance: &Arc<Mutex<CompanionAppearance>>,
) {
    let open_sender = sender.clone();
    let open_appearance = Arc::clone(appearance);
    registry.register("windows", "openCompanion", move |payload| {
        let (spec, requested) = match companion_window(&payload) {
            Ok(value) => value,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let previous = *open_appearance
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if !previous.open {
            *open_appearance
                .lock()
                .unwrap_or_else(|poison| poison.into_inner()) = requested;
        }
        let outcome = dispatch(&open_sender, |reply| {
            NativeHostApplicationRequest::OpenWindow(spec, reply)
        });
        let mut appearance = open_appearance
            .lock()
            .unwrap_or_else(|poison| poison.into_inner());
        if matches!(outcome, ServiceOutcome::Ok(_)) {
            appearance.open = true;
        } else if !previous.open {
            *appearance = previous;
        }
        outcome
    });
    let message_sender = sender.clone();
    registry.register("windows", "sendMessage", move |payload| {
        let Some(message) = payload.get("message").and_then(Value::as_str) else {
            return ServiceOutcome::Error("windows.sendMessage requires message: string".into());
        };
        if message.len() > 4096 {
            return ServiceOutcome::Error("window message exceeds 4096 bytes".into());
        }
        dispatch(&message_sender, |reply| {
            NativeHostApplicationRequest::SendWindowMessage(
                WindowKey::new("companion"),
                message.into(),
                reply,
            )
        })
    });
    let info_sender = sender.clone();
    registry.register("windows", "getInfo", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let (reply, result) = mpsc::channel();
        if info_sender
            .send(NativeHostApplicationRequest::GetWindowInfo(key, reply))
            .is_err()
        {
            return ServiceOutcome::Error("native application runtime has stopped".into());
        }
        match result.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(info)) => ServiceOutcome::Ok(window_info_json(info)),
            Ok(Err(error)) => ServiceOutcome::Error(error),
            Err(_) => ServiceOutcome::Error("window information request timed out".into()),
        }
    });
    let monitor_sender = sender.clone();
    registry.register("windows", "getMonitors", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let (reply, result) = mpsc::channel();
        if monitor_sender
            .send(NativeHostApplicationRequest::GetMonitors(key, reply))
            .is_err()
        {
            return ServiceOutcome::Error("native application runtime has stopped".into());
        }
        match result.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(monitors)) => ServiceOutcome::Ok(json!(
                monitors
                    .iter()
                    .map(|monitor| json!({
                        "name": monitor.name, "x": monitor.x, "y": monitor.y,
                        "width": monitor.width, "height": monitor.height,
                        "scaleFactor": monitor.scale_factor, "primary": monitor.primary,
                    }))
                    .collect::<Vec<_>>()
            )),
            Ok(Err(error)) => ServiceOutcome::Error(error),
            Err(_) => ServiceOutcome::Error("monitor information request timed out".into()),
        }
    });
    let input_sender = sender.clone();
    registry.register("windows", "setInputRegion", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let region = match parse_input_region(&payload) {
            Ok(region) => region,
            Err(error) => return ServiceOutcome::Error(error),
        };
        dispatch(&input_sender, |reply| {
            NativeHostApplicationRequest::SetWindowInputRegion(key, region, reply)
        })
    });
    let level_sender = sender.clone();
    registry.register("windows", "setLevel", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let level = match payload.get("level").and_then(Value::as_str) {
            Some("bottom") => WindowLevel::AlwaysOnBottom,
            Some("normal") => WindowLevel::Normal,
            Some("top") => WindowLevel::AlwaysOnTop,
            _ => {
                return ServiceOutcome::Error("window level must be bottom, normal, or top".into());
            }
        };
        dispatch(&level_sender, |reply| {
            NativeHostApplicationRequest::SetWindowLevel(key, level, reply)
        })
    });
    let show_sender = sender.clone();
    registry.register("windows", "show", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        dispatch(&show_sender, |reply| {
            NativeHostApplicationRequest::ShowWindow(key, reply)
        })
    });
    let close_sender = sender.clone();
    registry.register("windows", "close", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        dispatch(&close_sender, |reply| {
            NativeHostApplicationRequest::CloseWindow(key, reply)
        })
    });
    let title_sender = sender.clone();
    registry.register("windows", "setTitle", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let Some(title) = payload
            .get("title")
            .and_then(Value::as_str)
            .filter(|title| title.len() <= 256)
        else {
            return ServiceOutcome::Error(
                "window title must be a string of at most 256 bytes".into(),
            );
        };
        dispatch(&title_sender, |reply| {
            NativeHostApplicationRequest::SetWindowTitle(key, title.into(), reply)
        })
    });
    let size_sender = sender.clone();
    registry.register("windows", "setSize", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let (Some(width), Some(height)) = (
            payload.get("width").and_then(Value::as_f64),
            payload.get("height").and_then(Value::as_f64),
        ) else {
            return ServiceOutcome::Error("window size requires numeric width and height".into());
        };
        dispatch(&size_sender, |reply| {
            NativeHostApplicationRequest::SetWindowSize(key, width, height, reply)
        })
    });
    let position_sender = sender.clone();
    registry.register("windows", "setPosition", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let (Some(x), Some(y)) = (
            payload.get("x").and_then(Value::as_f64),
            payload.get("y").and_then(Value::as_f64),
        ) else {
            return ServiceOutcome::Error("window position requires numeric x and y".into());
        };
        dispatch(&position_sender, |reply| {
            NativeHostApplicationRequest::SetWindowPosition(key, x, y, reply)
        })
    });
    registry.register("windows", "setDecorations", move |payload| {
        let key = match window_key(&payload) {
            Ok(key) => key,
            Err(error) => return ServiceOutcome::Error(error),
        };
        let Some(decorations) = payload.get("decorations").and_then(Value::as_bool) else {
            return ServiceOutcome::Error("window decorations must be a boolean".into());
        };
        dispatch(&sender, |reply| {
            NativeHostApplicationRequest::SetWindowDecorations(key, decorations, reply)
        })
    });
}

/// Parses the companion's creation-time visual options from a TSX request.
/// `payload` may omit every option to use the gallery defaults.
///
/// # Errors
/// Returns an error for invalid dimensions, titles, or backdrop scope.
fn companion_window(payload: &Value) -> Result<(WindowSpec, CompanionAppearance), String> {
    let title = payload
        .get("title")
        .map(|value| {
            value
                .as_str()
                .filter(|title| title.len() <= 256)
                .ok_or("companion title must be a string of at most 256 bytes")
        })
        .transpose()?
        .unwrap_or("Argui Gallery Companion");
    let width = payload
        .get("width")
        .map(|value| value.as_f64().ok_or("companion width must be numeric"))
        .transpose()?
        .unwrap_or(480.0);
    let height = payload
        .get("height")
        .map(|value| value.as_f64().ok_or("companion height must be numeric"))
        .transpose()?
        .unwrap_or(260.0);
    if !width.is_finite()
        || !height.is_finite()
        || !(1.0..=16384.0).contains(&width)
        || !(1.0..=16384.0).contains(&height)
    {
        return Err("companion width and height must be from 1 to 16384".into());
    }
    let decorations = payload
        .get("decorations")
        .map(|value| value.as_bool().ok_or("decorations must be a boolean"))
        .transpose()?
        .unwrap_or(true);
    let transparent = payload
        .get("transparent")
        .map(|value| value.as_bool().ok_or("transparent must be a boolean"))
        .transpose()?
        .unwrap_or(false);
    let backdrop = payload
        .get("backdrop")
        .map(|value| value.as_bool().ok_or("backdrop must be a boolean"))
        .transpose()?
        .unwrap_or(false);
    let whole_window = match payload
        .get("backdropScope")
        .map(|value| value.as_str().ok_or("backdropScope must be a string"))
        .transpose()?
        .unwrap_or("window")
    {
        "window" => true,
        "panel" => false,
        _ => return Err("backdropScope must be window or panel".into()),
    };
    let spec = WindowSpec::new(
        WindowKey::new("companion"),
        WindowConfig {
            title: title.into(),
            width,
            height,
            decorations,
            transparent: transparent || backdrop,
            desktop_backdrop: backdrop.then_some(BackdropMaterial::Glass),
            close_behavior: CloseBehavior::CloseWindow,
            ..WindowConfig::default()
        },
    );
    Ok((
        spec,
        CompanionAppearance {
            transparent,
            backdrop,
            whole_window,
            open: false,
        },
    ))
}

/// Resolves the target window key from a service payload.
/// `payload.window` defaults to the main window.
///
/// # Errors
/// Returns an error for an empty or non-string window key.
fn window_key(payload: &Value) -> Result<WindowKey, String> {
    match payload.get("window") {
        None => Ok(WindowKey::main()),
        Some(Value::String(value)) if !value.is_empty() => Ok(WindowKey::new(value)),
        _ => Err("window must be a nonempty string".into()),
    }
}

/// Converts native logical window information to the JavaScript service response.
/// `info` contains the latest title, dimensions, visibility, and appearance state.
fn window_info_json(info: NativeWindowInfo) -> Value {
    let backend = match info.capabilities.backend {
        WindowBackend::Windows => "windows",
        WindowBackend::MacOs => "macos",
        WindowBackend::X11 => "x11",
        WindowBackend::Wayland => "wayland",
        WindowBackend::Android => "android",
        WindowBackend::Ios => "ios",
        WindowBackend::Web => "web",
        WindowBackend::Other => "other",
    };
    json!({
        "window": info.window.as_str(), "title": info.title,
        "width": info.width, "height": info.height, "visible": info.visible,
        "x": info.x, "y": info.y,
        "decorations": info.decorations, "transparent": info.transparent,
        "backdrop": info.backdrop.is_some(),
        "backdropAvailable": info.backdrop_available,
        "scaleFactor": info.scale_factor, "uiZoomFactor": info.ui_zoom_factor,
        "capabilities": {
            "backend": backend,
            "absolutePosition": info.capabilities.absolute_position,
            "windowLevel": info.capabilities.window_level,
            "mousePassthrough": info.capabilities.mouse_passthrough,
            "inputRegions": info.capabilities.input_regions,
            "transparentCompositing": info.capabilities.transparent_compositing,
            "nativeShadow": info.capabilities.native_shadow,
        },
    })
}

/// Parses one TSX input policy with a rectangular hole in UI coordinates.
/// `payload` contains `mode` and, for exclusion, a finite nonnegative `rect`.
///
/// # Errors
/// Returns a validation message for missing or invalid fields.
fn parse_input_region(payload: &Value) -> Result<WindowInputRegion, String> {
    match payload.get("mode").and_then(Value::as_str) {
        Some("full") => Ok(WindowInputRegion::Full),
        Some("passThrough") => Ok(WindowInputRegion::PassThrough),
        Some("exclude") => {
            let rect = payload
                .get("rect")
                .ok_or("exclude input region requires rect")?;
            let field = |name| -> Result<f32, String> {
                let value = rect
                    .get(name)
                    .and_then(Value::as_f64)
                    .ok_or_else(|| format!("input region {name} must be numeric"))?;
                if !value.is_finite() || value.abs() > 100_000.0 {
                    return Err(format!(
                        "input region {name} is outside the supported range"
                    ));
                }
                Ok(value as f32)
            };
            let (x, y, width, height) =
                (field("x")?, field("y")?, field("width")?, field("height")?);
            if width < 0.0 || height < 0.0 {
                return Err("input region width and height must be nonnegative".into());
            }
            Ok(WindowInputRegion::Exclude(Rect::new(
                Point::new(x, y),
                Size::new(width, height),
            )))
        }
        _ => Err("input region mode must be full, passThrough, or exclude".into()),
    }
}
