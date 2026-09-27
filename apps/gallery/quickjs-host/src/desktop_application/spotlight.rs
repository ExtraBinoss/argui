//! X11 screen spotlight used to exercise desktop input regions end to end.

use std::sync::{
    Arc, Mutex,
    mpsc::{self, Sender},
};
use std::time::Duration;

use argui_core::{Color, Point, PointerButton, PointerEvent, PointerPhase, Rect, Size};
use argui_platform::{
    CloseBehavior, WindowBackend, WindowConfig, WindowInputRegion, WindowKey, WindowLevel,
    WindowSpec,
};
use argui_runtime::{AppCommand, AppUpdate, NativeHostApplicationRequest, ViewUpdate};
use argui_ui::{Element, Sides, auto, length, percent};
use serde_json::Value;

use super::dispatch;
use crate::services::{ServiceOutcome, ServiceRegistry};

/// Visual and input geometry shared by the X11 spotlight window.
#[derive(Default)]
pub(crate) struct SpotlightState {
    pub open: bool,
    hole: Option<Rect>,
    drag: Option<(Point, Rect)>,
}

impl SpotlightState {
    /// Builds the transparent window content from the same hole sent to X11.
    pub(super) fn view(&self) -> Element {
        let hole = self
            .hole
            .unwrap_or(Rect::new(Point::new(0.0, 0.0), Size::new(0.0, 0.0)));
        let x = hole.origin.x;
        let y = hole.origin.y;
        let right = x + hole.size.width;
        let bottom = y + hole.size.height;
        let dim = Color::srgba(0.015, 0.025, 0.045, 0.72);
        let top = mask(
            Sides {
                left: length(0.0),
                right: length(0.0),
                top: length(0.0),
                bottom: auto(),
            },
            None,
            Some(y),
            dim,
        );
        let left = mask(
            Sides {
                left: length(0.0),
                right: auto(),
                top: length(y),
                bottom: auto(),
            },
            Some(x),
            Some(hole.size.height),
            dim,
        );
        let right_side = mask(
            Sides {
                left: length(right),
                right: length(0.0),
                top: length(y),
                bottom: auto(),
            },
            None,
            Some(hole.size.height),
            dim,
        );
        let bottom_side = mask(
            Sides {
                left: length(0.0),
                right: length(0.0),
                top: length(bottom),
                bottom: length(0.0),
            },
            None,
            None,
            dim,
        );
        Element::container([top, left, right_side, bottom_side])
            .keyed("spotlight-root")
            .width(percent(1.0))
            .height(percent(1.0))
    }

    /// Handles a native mouse sample and synchronizes the X11 hole on release.
    /// `pointer` uses Argui UI logical coordinates.
    pub(super) fn pointer(&mut self, pointer: PointerEvent) -> AppUpdate {
        let key = WindowKey::new("spotlight");
        match pointer.phase {
            PointerPhase::Pressed if pointer.button == Some(PointerButton::Secondary) => {
                AppUpdate::none().command(AppCommand::CloseWindow(key))
            }
            PointerPhase::Pressed if pointer.button == Some(PointerButton::Primary) => {
                if let Some(hole) = self.hole {
                    self.drag = Some((pointer.position, hole));
                    AppUpdate::none().command(AppCommand::SetWindowInputRegion {
                        window: key,
                        region: WindowInputRegion::Full,
                    })
                } else {
                    AppUpdate::none()
                }
            }
            PointerPhase::Moved => {
                if let Some((origin, _)) = self.drag {
                    self.hole = Some(selection(origin, pointer.position));
                    AppUpdate::none().window(key, ViewUpdate::Rebuild)
                } else {
                    AppUpdate::none()
                }
            }
            PointerPhase::Released | PointerPhase::Cancelled => {
                let Some((origin, previous)) = self.drag.take() else {
                    return AppUpdate::none();
                };
                let selected = selection(origin, pointer.position);
                let hole = if selected.size.width >= 24.0 && selected.size.height >= 24.0 {
                    selected
                } else {
                    previous
                };
                self.hole = Some(hole);
                AppUpdate::none()
                    .window(key.clone(), ViewUpdate::Rebuild)
                    .command(AppCommand::SetWindowInputRegion {
                        window: key,
                        region: WindowInputRegion::Exclude(hole),
                    })
            }
            _ => AppUpdate::none(),
        }
    }
}

/// Creates one absolute, translucent side of the spotlight mask.
/// `inset` anchors the side; optional dimensions constrain its free axes.
fn mask(
    inset: Sides<argui_ui::LengthPercentageAuto>,
    width: Option<f32>,
    height: Option<f32>,
    color: Color,
) -> Element {
    let mut element = Element::container([]).absolute(inset).background(color);
    if let Some(width) = width {
        element = element.width(length(width.max(0.0)));
    }
    if let Some(height) = height {
        element = element.height(length(height.max(0.0)));
    }
    element
}

/// Returns the rectangle drawn between two pointer positions.
/// `start` and `end` are in UI logical coordinates.
fn selection(start: Point, end: Point) -> Rect {
    let start_x = start.x.max(0.0);
    let start_y = start.y.max(0.0);
    let end_x = end.x.max(0.0);
    let end_y = end.y.max(0.0);
    let left = start_x.min(end_x);
    let top = start_y.min(end_y);
    Rect::new(
        Point::new(left, top),
        Size::new((start_x - end_x).abs(), (start_y - end_y).abs()),
    )
}

/// Registers a gallery demonstration that opens one monitor-sized X11 overlay.
/// `registry` stores the TSX service, `sender` reaches the UI thread, and
/// `state` provides the exact rectangle used for paint and native input.
pub(super) fn register_spotlight_service(
    registry: &ServiceRegistry,
    sender: Sender<NativeHostApplicationRequest>,
    state: &Arc<Mutex<SpotlightState>>,
) {
    let state = Arc::clone(state);
    registry.register("windows", "openSpotlight", move |_payload: Value| {
        let (reply, result) = mpsc::channel();
        if sender.send(NativeHostApplicationRequest::GetWindowInfo(WindowKey::main(), reply)).is_err() {
            return ServiceOutcome::Error("native application runtime has stopped".into());
        }
        let info = match result.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(info)) => info,
            Ok(Err(error)) => return ServiceOutcome::Error(error),
            Err(_) => return ServiceOutcome::Error("window information request timed out".into()),
        };
        if info.capabilities.backend != WindowBackend::X11 || !info.capabilities.input_regions {
            return ServiceOutcome::Unsupported("screen spotlight requires X11 input regions and absolute window positioning; this backend cannot provide them".into());
        }
        if !info.capabilities.transparent_compositing {
            return ServiceOutcome::Unsupported("screen spotlight requires an active X11 compositor for transparent windows".into());
        }
        let (reply, result) = mpsc::channel();
        if sender.send(NativeHostApplicationRequest::GetMonitors(WindowKey::main(), reply)).is_err() {
            return ServiceOutcome::Error("native application runtime has stopped".into());
        }
        let monitors = match result.recv_timeout(Duration::from_secs(30)) {
            Ok(Ok(monitors)) => monitors,
            Ok(Err(error)) => return ServiceOutcome::Error(error),
            Err(_) => return ServiceOutcome::Error("monitor information request timed out".into()),
        };
        let Some(monitor) = monitors.iter().find(|monitor| monitor.primary).or_else(|| monitors.first()) else {
            return ServiceOutcome::Error("no native monitor is available".into());
        };
        let width = f64::from(monitor.width) / monitor.scale_factor;
        let height = f64::from(monitor.height) / monitor.scale_factor;
        let ui_width = width / f64::from(info.ui_zoom_factor);
        let ui_height = height / f64::from(info.ui_zoom_factor);
        let hole = Rect::new(
            Point::new((ui_width * 0.16) as f32, (ui_height * 0.12) as f32),
            Size::new((ui_width * 0.48) as f32, (ui_height * 0.40) as f32),
        );
        {
            let mut state = state.lock().unwrap_or_else(|poison| poison.into_inner());
            state.hole = Some(hole);
            state.drag = None;
        }
        let mut spec = WindowSpec::new(WindowKey::new("spotlight"), WindowConfig {
            title: "Argui screen spotlight".into(),
            width, height,
            physical_position: Some((monitor.x, monitor.y)),
            decorations: false, resizable: false, transparent: true,
            level: WindowLevel::AlwaysOnTop,
            close_behavior: CloseBehavior::CloseWindow,
            ..WindowConfig::default()
        });
        spec.visible = false;
        let outcome = dispatch(&sender, |reply| NativeHostApplicationRequest::OpenWindow(spec, reply));
        if !matches!(outcome, ServiceOutcome::Ok(_)) { return outcome; }
        let outcome = dispatch(&sender, |reply| NativeHostApplicationRequest::SetWindowInputRegion(
            WindowKey::new("spotlight"), WindowInputRegion::Exclude(hole), reply,
        ));
        if !matches!(outcome, ServiceOutcome::Ok(_)) {
            let _ = dispatch(&sender, |reply| NativeHostApplicationRequest::CloseWindow(WindowKey::new("spotlight"), reply));
            return outcome;
        }
        let outcome = dispatch(&sender, |reply| NativeHostApplicationRequest::ShowWindow(WindowKey::new("spotlight"), reply));
        if matches!(outcome, ServiceOutcome::Ok(_)) {
            state.lock().unwrap_or_else(|poison| poison.into_inner()).open = true;
        }
        outcome
    });
}
