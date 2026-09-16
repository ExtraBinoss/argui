use std::sync::Arc;

use argui::{
    animation::Frame,
    core::{Color, Key, KeyState, ScrollDelta},
    paint::{Border, CornerRadii, Filter, LayerMask, LayerStyle},
    runtime::{Context, Render},
    text::{TextColor, TextStyle, TextWrap},
    ui::{
        Axes, CursorIcon, Element, EventType, FocusPolicy, GestureCapture, GestureDelivery,
        GestureKind, GesturePhase, GestureSet, GpuCanvasSpec, HitTestStyle, Interaction, Overflow,
        PanGesture, PinchGesture, PointerEvents, Position, Role, Semantics, Sides, UiEvent,
        UiEventKind, auto, length, percent,
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
        let UiEventKind::Wheel { delta, .. } = event.kind else {
            return;
        };
        let notches = match delta {
            ScrollDelta::Lines(point) => point.y,
            ScrollDelta::Pixels(point) => point.y / 100.0,
        };
        let zoom = (notches * 0.18).clamp(-0.6, 0.6).exp();
        self.shared
            .update(|state| state.zoom_by(zoom));
        let _ = event.prevent_default();
        cx.notify();
    }

    /// Provides keyboard alternatives for every canvas navigation action.
    fn key(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
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
        Element::column([
            Element::row([
                Element::column([
                    text("GPU Canvas Lab", 24.0, theme.foreground, 750),
                    text(
                        "App-owned WGPU passes inside an Argui-managed frame",
                        12.0,
                        theme.muted_foreground,
                        500,
                    ),
                ])
                .gap(1.0)
                .grow(1.0),
                badge("NEW GPU CANVAS API", Color::srgb(0.2, 0.78, 1.0), theme),
            ])
            .width(percent(1.0))
            .gap(12.0),
            Element::row([
                text(
                    "Explore the scene",
                    11.0,
                    theme.muted_foreground,
                    650,
                )
                .grow(1.0),
                button("zoom-out", "Zoom −"),
                button("zoom-in", "Zoom +"),
                button("reset", "Reset view"),
                button(
                    "pause",
                    if state.paused() {
                        "Resume animation"
                    } else {
                        "Pause animation"
                    },
                ),
                button(
                    "error",
                    if state.force_error() {
                        "Recover canvas"
                    } else {
                        "Test recovery"
                    },
                ),
            ])
            .width(percent(1.0))
            .padding(Sides::length(8.0))
            .gap(7.0)
            .background(theme.card)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(10.0)),
        ])
        .width(percent(1.0))
        .gap(10.0)
    }

    /// Builds the clipped canvas viewport and a normal Argui overlay above it.
    fn canvas_panel(&self, theme: &WidgetTheme, cx: &mut Context<Self>) -> Element {
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
        .on(cx.listener(EventType::Gesture, Self::gesture))
        .on(cx.listener(EventType::Wheel, Self::wheel))
        .on(cx.listener(EventType::Key, Self::key))
        .layer(
            LayerStyle::new(Default::default())
                .filter(Filter::Brightness(1.02))
                .mask(LayerMask::Rounded(CornerRadii::all(14.0))),
        );
        let overlay = Element::column([
            text(
                if state.paused() {
                    "PAUSED · CAMERA STILL INTERACTIVE"
                } else {
                    "LIVE · COMPUTE + RENDER"
                },
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
        .hit_test(HitTestStyle::default().pointer_events(PointerEvents::None))
        .semantic_hidden(true);
        let help = text(
            "Drag to pan  ·  Scroll to zoom  ·  Pinch on touch  ·  Select canvas for keyboard",
            11.0,
            Color::WHITE,
            600,
        )
        .padding(Sides::length(9.0))
        .background(Color::BLACK.with_alpha(0.58))
        .border(Border::all(1.0, Color::WHITE.with_alpha(0.18)))
        .radius(CornerRadii::all(9.0))
        .position(Position::Absolute)
        .absolute(Sides {
            left: length(14.0),
            top: auto(),
            right: auto(),
            bottom: length(14.0),
        })
        .z_index(2)
        .hit_test(HitTestStyle::default().pointer_events(PointerEvents::None))
        .semantic_hidden(true);
        Element::container([canvas, overlay, help])
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
            "Live API inspector",
            Element::column([
                metric("Revision", state.revision().to_string(), theme),
                metric("Renderer calls", callbacks.to_string(), theme),
                metric("Zoom", format!("{:.2}×", state.zoom_factor()), theme),
                metric(
                    "Pan",
                    format!("{:.0}, {:.0}", state.pan_offset()[0], state.pan_offset()[1]),
                    theme,
                ),
                divider(theme),
                text("WHAT THIS PROVES", 10.0, theme.muted_foreground, 750),
                proof(
                    "1",
                    "Device context",
                    "The factory receives Device, limits and target format.",
                    theme,
                ),
                proof(
                    "2",
                    "App GPU work",
                    "The callback writes the Queue and encodes compute + render passes.",
                    theme,
                ),
                proof(
                    "3",
                    "Argui lifecycle",
                    "Argui owns texture retention, submission and presentation.",
                    theme,
                ),
                divider(theme),
                text("NEGOTIATED CONTEXT", 10.0, theme.muted_foreground, 750),
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
            ])
            .gap(9.0),
            theme,
        )
        .width(length(280.0))
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
                self.canvas_panel(theme, cx)
                    .grow(1.0)
                    .min_width(length(0.0)),
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
    }

    /// Returns whether the running scene needs another animation frame.
    fn wants_animation_frame(&self) -> bool {
        let state = self.shared.state();
        !state.paused() || state.view_is_settling()
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

/// Builds a compact accent badge for the active GPU-canvas API.
fn badge(label: &str, accent: Color, theme: &WidgetTheme) -> Element {
    Element::row([
        Element::container([])
            .width(length(7.0))
            .height(length(7.0))
            .background(accent)
            .radius(CornerRadii::all(4.0)),
        text(label, 10.0, theme.foreground, 750),
    ])
    .padding(Sides::length(8.0))
    .gap(7.0)
    .background(theme.card)
    .border(Border::all(1.0, accent.with_alpha(0.34)))
    .radius(CornerRadii::all(9.0))
}

/// Builds one numbered API-boundary explanation for the live inspector.
fn proof(
    number: &str,
    title: &str,
    description: &str,
    theme: &WidgetTheme,
) -> Element {
    Element::row([
        text(
            number,
            10.0,
            Color::srgb(0.32, 0.82, 1.0),
            800,
        )
        .padding(Sides::length(5.0))
        .background(Color::srgb(0.08, 0.3, 0.4))
        .radius(CornerRadii::all(6.0)),
        Element::column([
            text(title, 11.0, theme.foreground, 700),
            text(description, 10.0, theme.muted_foreground, 450),
        ])
        .gap(1.0)
        .min_width(length(0.0))
        .grow(1.0),
    ])
    .width(percent(1.0))
    .min_width(length(0.0))
    .gap(8.0)
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
