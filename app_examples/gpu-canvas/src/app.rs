use std::sync::Arc;

use argui::{
    animation::Frame,
    core::{Color, Key, KeyState, ScrollDelta},
    paint::{Border, CornerRadii, Filter, LayerMask, LayerStyle},
    runtime::{Context, Render},
    text::{TextColor, TextStyle, TextWrap},
    ui::{
        Axes, CursorIcon, Element, EventType, FocusPolicy, GestureCapture, GestureDelivery,
        GestureKind, GesturePhase, GestureSet, GpuCanvasSpec, Interaction, JustifyContent,
        Overflow, PanGesture, PinchGesture, Position, Role, Semantics, Sides, UiEvent, UiEventKind,
        auto, length, percent,
    },
    widgets::{Button, WidgetTheme, shadcn},
};

use crate::state::SharedLab;

/// Argui application shell surrounding the retained GPU canvas.
pub(crate) struct GpuCanvasLab {
    canvas: argui::paint::GpuCanvasId,
    shared: Arc<SharedLab>,
}

impl GpuCanvasLab {
    /// Creates the UI model for `canvas` and its shared scene state.
    pub(crate) fn new(canvas: argui::paint::GpuCanvasId, shared: Arc<SharedLab>) -> Self {
        Self { canvas, shared }
    }

    /// Applies one toolbar action selected by its stable element key.
    fn click(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        match event.target_key() {
            Some("pause") => self.shared.update(|state| {
                state.toggle_paused();
            }),
            Some("reset") => self.shared.update(crate::state::LabState::reset_view),
            Some("zoom-in") => self.shared.update(|state| state.zoom_by(1.2)),
            Some("zoom-out") => self.shared.update(|state| state.zoom_by(1.0 / 1.2)),
            Some("error") => self.shared.update(|state| {
                state.toggle_error();
            }),
            _ => return,
        }
        cx.notify();
    }

    /// Applies pointer pan and pinch gestures emitted by the canvas leaf.
    fn gesture(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.target_key() != Some("lab-canvas") {
            return;
        }
        let UiEventKind::Gesture(gesture) = event.kind else {
            return;
        };
        if gesture.phase != GesturePhase::Changed {
            return;
        }
        match gesture.kind {
            GestureKind::Pan { delta, .. } => {
                self.shared.update(|state| state.pan_by(delta.x, delta.y));
            }
            GestureKind::Pinch { scale } => {
                self.shared.update(|state| state.zoom_by(scale));
            }
            GestureKind::Tap { .. } | GestureKind::Rotation { .. } => return,
        }
        cx.notify();
    }

    /// Converts wheel movement over the canvas into bounded zoom changes.
    fn wheel(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.target_key() != Some("lab-canvas") {
            return;
        }
        let UiEventKind::Wheel { delta, .. } = event.kind else {
            return;
        };
        let y = match delta {
            ScrollDelta::Lines(point) | ScrollDelta::Pixels(point) => point.y,
        };
        self.shared
            .update(|state| state.zoom_by((-y * 0.015).exp()));
        let _ = event.prevent_default();
        cx.notify();
    }

    /// Provides keyboard alternatives for every canvas navigation action.
    fn key(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        if event.target_key() != Some("lab-canvas") {
            return;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return;
        };
        if input.state != KeyState::Pressed {
            return;
        }
        match &input.key {
            Key::ArrowLeft => self.shared.update(|state| state.pan_by(-16.0, 0.0)),
            Key::ArrowRight => self.shared.update(|state| state.pan_by(16.0, 0.0)),
            Key::ArrowUp => self.shared.update(|state| state.pan_by(0.0, -16.0)),
            Key::ArrowDown => self.shared.update(|state| state.pan_by(0.0, 16.0)),
            Key::Character(value) if value == "+" || value == "=" => {
                self.shared.update(|state| state.zoom_by(1.2));
            }
            Key::Character(value) if value == "-" => {
                self.shared.update(|state| state.zoom_by(1.0 / 1.2));
            }
            Key::Character(value) if value == "0" => {
                self.shared.update(crate::state::LabState::reset_view);
            }
            Key::Character(value) if value == " " => self.shared.update(|state| {
                state.toggle_paused();
            }),
            _ => return,
        }
        let _ = event.prevent_default();
        cx.notify();
    }

    /// Builds the top toolbar with mouse, touch and keyboard-accessible actions.
    fn toolbar(&self, theme: &WidgetTheme) -> Element {
        let state = self.shared.state();
        let button = |key, label| Button::new(key, label, theme.outline_button()).build();
        Element::row([
            Element::column([
                text("GPU Canvas Lab", 22.0, theme.foreground, 700),
                text(
                    "Argui shell · custom WGPU compute + render passes",
                    11.0,
                    theme.muted_foreground,
                    500,
                ),
            ])
            .gap(1.0)
            .grow(1.0),
            button("zoom-out", "Zoom −"),
            button("zoom-in", "Zoom +"),
            button("reset", "Reset"),
            button("pause", if state.paused() { "Resume" } else { "Pause" }),
            button(
                "error",
                if state.force_error() {
                    "Recover canvas"
                } else {
                    "Simulate error"
                },
            ),
        ])
        .width(percent(1.0))
        .gap(8.0)
        .justify_content(JustifyContent::SPACE_BETWEEN)
    }

    /// Builds the clipped canvas viewport and a normal Argui overlay above it.
    fn canvas_panel(&self, theme: &WidgetTheme) -> Element {
        let state = self.shared.state();
        let gestures = GestureSet::EMPTY
            .pan(
                PanGesture::default()
                    .immediate()
                    .capture(GestureCapture::OnPress)
                    .delivery(GestureDelivery::FrameCoalesced),
            )
            .pinch(PinchGesture::default().delivery(GestureDelivery::FrameCoalesced));
        let canvas = Element::gpu_canvas(
            GpuCanvasSpec::new(self.canvas)
                .content_revision(state.revision())
                .resolution_scale(1.0),
        )
        .keyed("lab-canvas")
        .width(percent(1.0))
        .height(percent(1.0))
        .min_height(length(320.0))
        .radius(CornerRadii::all(14.0))
        .interaction(
            Interaction::default()
                .focus_policy(FocusPolicy::TabStop)
                .cursor(CursorIcon::Grab)
                .gestures(gestures),
        )
        .semantics(
            Semantics::new(Role::Image)
                .label("Interactive GPU particle canvas")
                .description(
                    "Drag or use arrow keys to pan. Wheel, pinch, plus and minus change zoom. Space pauses animation and zero resets the view.",
                ),
        )
        .layer(
            LayerStyle::new(Default::default())
                .filter(Filter::Brightness(1.02))
                .mask(LayerMask::Rounded(CornerRadii::all(14.0))),
        );
        let overlay = Element::column([
            text(
                if state.paused() { "PAUSED" } else { "LIVE" },
                11.0,
                Color::WHITE,
                800,
            ),
            text(
                format!(
                    "zoom {:.2}× · pan {:.0}, {:.0}",
                    state.zoom_factor(),
                    state.pan_offset()[0],
                    state.pan_offset()[1]
                ),
                12.0,
                Color::WHITE,
                600,
            ),
        ])
        .gap(2.0)
        .padding(Sides::length(10.0))
        .background(Color::BLACK.with_alpha(0.58))
        .border(Border::all(1.0, Color::WHITE.with_alpha(0.18)))
        .radius(CornerRadii::all(9.0))
        .position(Position::Absolute)
        .absolute(Sides {
            left: length(14.0),
            top: length(14.0),
            right: auto(),
            bottom: auto(),
        })
        .z_index(2)
        .semantic_hidden(true);
        Element::container([canvas, overlay])
            .position(Position::Relative)
            .width(percent(1.0))
            .height(percent(1.0))
            .min_width(length(0.0))
            .min_height(length(320.0))
            .grow(1.0)
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            })
            .background(theme.card)
            .radius(CornerRadii::all(14.0))
    }

    /// Builds the Argui inspector from retained scene and renderer event state.
    fn inspector(&self, theme: &WidgetTheme) -> Element {
        let state = self.shared.state();
        let (capability, diagnostic, callbacks) = self.shared.status();
        panel(
            "Canvas inspector",
            Element::column([
                metric("Revision", state.revision().to_string(), theme),
                metric("GPU callbacks", callbacks.to_string(), theme),
                metric("Zoom", format!("{:.2}×", state.zoom_factor()), theme),
                metric(
                    "Pan",
                    format!("{:.0}, {:.0}", state.pan_offset()[0], state.pan_offset()[1]),
                    theme,
                ),
                divider(theme),
                text("CAPABILITIES", 10.0, theme.muted_foreground, 700),
                text(capability, 11.0, theme.foreground, 450),
                divider(theme),
                text("LAST GPU CANVAS EVENT", 10.0, theme.muted_foreground, 700),
                text(
                    diagnostic,
                    11.0,
                    if state.force_error() {
                        Color::srgb(1.0, 0.45, 0.45)
                    } else {
                        theme.foreground
                    },
                    500,
                ),
                divider(theme),
                text(
                    "Controls: drag / arrows · wheel / pinch / ± · 0 reset · Space pause",
                    11.0,
                    theme.muted_foreground,
                    450,
                ),
            ])
            .gap(9.0),
            theme,
        )
        .width(length(260.0))
        .shrink(0.0)
    }
}

impl Render for GpuCanvasLab {
    /// Renders the complete toolbar, canvas, overlay and inspector shell.
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let environment = cx.environment();
        let themes = shadcn(&environment);
        let theme = themes.resolve(environment.color_scheme);
        Element::column([
            self.toolbar(theme),
            Element::row([
                self.canvas_panel(theme).grow(1.0).min_width(length(0.0)),
                self.inspector(theme),
            ])
            .width(percent(1.0))
            .height(percent(1.0))
            .min_height(length(0.0))
            .gap(14.0),
        ])
        .width(percent(1.0))
        .height(percent(1.0))
        .min_width(length(0.0))
        .min_height(length(0.0))
        .padding(Sides::length(18.0))
        .gap(14.0)
        .background(theme.background)
        .on(cx.listener(EventType::Click, Self::click))
        .on(cx.listener(EventType::Gesture, Self::gesture))
        .on(cx.listener(EventType::Wheel, Self::wheel))
        .on(cx.listener(EventType::Key, Self::key))
    }

    /// Returns whether the running scene needs another animation frame.
    fn wants_animation_frame(&self) -> bool {
        !self.shared.state().paused()
    }

    /// Advances the scene from `frame` timing and notifies `cx` when it changed.
    fn animation_frame(&mut self, frame: Frame, cx: &mut Context<Self>) {
        let seconds = frame.elapsed.as_secs_f64() as f32;
        let mut changed = false;
        self.shared.update(|state| changed = state.advance(seconds));
        if changed {
            cx.notify();
        }
    }
}

/// Builds a bordered inspector panel with a visible heading.
fn panel(title: &str, content: Element, theme: &WidgetTheme) -> Element {
    Element::column([text(title, 15.0, theme.foreground, 700), content])
        .padding(Sides::length(14.0))
        .gap(12.0)
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(12.0))
}

/// Builds one compact name/value inspector row.
fn metric(label: &str, value: String, theme: &WidgetTheme) -> Element {
    Element::row([
        text(label, 11.0, theme.muted_foreground, 500).grow(1.0),
        text(value, 11.0, theme.foreground, 650),
    ])
    .gap(8.0)
}

/// Builds a one-pixel separator using the active widget theme.
fn divider(theme: &WidgetTheme) -> Element {
    Element::container([])
        .width(percent(1.0))
        .height(length(1.0))
        .background(theme.border)
}

/// Builds consistently wrapped application text.
fn text(value: impl Into<String>, size: f32, color: TextColor, weight: u16) -> Element {
    Element::text(value.into()).text_style(TextStyle {
        font_size: size,
        line_height: size * 1.35,
        color,
        weight,
        wrap: TextWrap::Word,
        ..TextStyle::default()
    })
}
