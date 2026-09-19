use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use argui::{
    paint::{Border, CornerRadii, PaintStyle, QuadStyle},
    platform::WindowKey,
    render::{DamageMode, DamageTracking, RenderProfile},
    runtime::{AppCommand, Context, Render},
    ui::{
        AlignItems, Axes, Element, FlexWrap, FloatingPlacement, JustifyContent, Overflow,
        Placement, Sides, length, percent,
    },
    widgets::{Button, Popover, Tab, Tabs, WidgetTheme, shadcn},
};

use crate::app::text;

const SAMPLE_LIMIT: usize = 120;

/// Shared renderer measurements consumed by the damage-control showcase.
pub(crate) type DamageTelemetryHandle = Rc<RefCell<DamageTelemetry>>;

#[derive(Clone, Copy, Debug, Default)]
struct DamageSample {
    mode: DamageMode,
    regions: usize,
    damaged_pixels: u64,
    viewport_pixels: u64,
    retained_bytes: u64,
    cpu_time: Duration,
    gpu_time: Option<Duration>,
}

#[derive(Clone, Copy, Debug, Default)]
struct DamageMetrics {
    samples: usize,
    mode: DamageMode,
    regions: usize,
    damaged_pixels: u64,
    viewport_pixels: u64,
    retained_bytes: u64,
    average_ratio: f64,
    average_cpu_ms: f64,
    average_gpu_ms: Option<f64>,
}

/// Bounded rolling history of renderer profiles for the live comparison page.
#[derive(Debug, Default)]
pub(crate) struct DamageTelemetry {
    samples: VecDeque<DamageSample>,
}

impl DamageTelemetry {
    /// Creates an empty handle suitable for sharing with the runtime callback.
    #[must_use]
    pub(crate) fn handle() -> DamageTelemetryHandle {
        Rc::new(RefCell::new(Self::default()))
    }

    /// Records the relevant counters from one completed renderer frame.
    ///
    /// `profile` is the renderer profile emitted after presentation.
    pub(crate) fn record(&mut self, profile: &RenderProfile) {
        self.samples.push_back(DamageSample {
            mode: profile.damage.mode,
            regions: profile.damage.regions,
            damaged_pixels: profile.damage.damaged_pixels,
            viewport_pixels: profile.viewport_pixels,
            retained_bytes: profile.damage.retained_bytes,
            cpu_time: profile.cpu_time,
            gpu_time: profile.gpu.as_ref().map(|gpu| gpu.total),
        });
        if self.samples.len() > SAMPLE_LIMIT {
            self.samples.pop_front();
        }
    }

    /// Clears all samples so two renderer modes start with independent history.
    pub(crate) fn clear(&mut self) {
        self.samples.clear();
    }

    /// Summarizes the current rolling history for presentation.
    fn metrics(&self) -> DamageMetrics {
        let Some(latest) = self.samples.back().copied() else {
            return DamageMetrics::default();
        };
        let count = self.samples.len() as f64;
        let average_ratio = self
            .samples
            .iter()
            .map(|sample| {
                if sample.viewport_pixels == 0 {
                    0.0
                } else {
                    sample.damaged_pixels as f64 / sample.viewport_pixels as f64
                }
            })
            .sum::<f64>()
            / count;
        let average_cpu_ms = self
            .samples
            .iter()
            .map(|sample| sample.cpu_time.as_secs_f64() * 1_000.0)
            .sum::<f64>()
            / count;
        let (gpu_total, gpu_count) = self
            .samples
            .iter()
            .filter_map(|sample| sample.gpu_time)
            .fold((0.0, 0_usize), |(total, count), duration| {
                (total + duration.as_secs_f64() * 1_000.0, count + 1)
            });
        let average_gpu_ms = (gpu_count > 0).then(|| gpu_total / gpu_count as f64);
        DamageMetrics {
            samples: self.samples.len(),
            mode: latest.mode,
            regions: latest.regions,
            damaged_pixels: latest.damaged_pixels,
            viewport_pixels: latest.viewport_pixels,
            retained_bytes: latest.retained_bytes,
            average_ratio,
            average_cpu_ms,
            average_gpu_ms,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ComparisonMode {
    #[default]
    Auto,
    Off,
}

impl ComparisonMode {
    /// Returns the renderer configuration represented by this comparison mode.
    const fn tracking(self) -> DamageTracking {
        match self {
            Self::Auto => DamageTracking::enabled(),
            Self::Off => DamageTracking::disabled(),
        }
    }
}

/// Interactive page comparing full-frame and adaptive damage rendering.
pub(crate) struct DamageControlDemo {
    mode: ComparisonMode,
    telemetry: DamageTelemetryHandle,
    displayed: DamageMetrics,
    phase: f32,
    frames: u64,
    running: bool,
    reduced_motion: bool,
    blur_open: bool,
}

impl DamageControlDemo {
    /// Creates the page with adaptive damage rendering selected.
    ///
    /// `telemetry` receives profiles from the gallery runtime callback.
    #[must_use]
    pub(crate) fn new(telemetry: DamageTelemetryHandle) -> Self {
        Self {
            mode: ComparisonMode::Auto,
            telemetry,
            displayed: DamageMetrics::default(),
            phase: 0.0,
            frames: 0,
            running: true,
            reduced_motion: false,
            blur_open: false,
        }
    }

    /// Returns the damage configuration currently selected by the page.
    pub(crate) const fn tracking(&self) -> DamageTracking {
        self.mode.tracking()
    }

    /// Clears stale measurements when the page becomes active.
    pub(crate) fn activate(&mut self) {
        self.telemetry.borrow_mut().clear();
        self.displayed = DamageMetrics::default();
        self.frames = 0;
    }

    /// Closes transient overlays when the gallery navigates away from this page.
    pub(crate) fn deactivate(&mut self) {
        self.blur_open = false;
    }

    /// Selects a renderer mode and starts a fresh measurement window.
    ///
    /// `mode` identifies the selected tab and `cx` queues the runtime command.
    fn set_mode(&mut self, mode: ComparisonMode, cx: &mut Context<Self>) {
        if self.mode == mode {
            return;
        }
        self.mode = mode;
        self.activate();
        cx.command(AppCommand::SetDamageTracking {
            window: WindowKey::main(),
            tracking: mode.tracking(),
        });
        cx.notify();
        cx.request_animation_frame();
    }

    /// Advances the controlled workload by one visible step.
    fn step(&mut self, amount: f32) {
        self.phase = (self.phase + amount).rem_euclid(1.0);
    }

    /// Builds the animated workload whose changed bounds drive damage tracking.
    fn workload(&self, theme: &WidgetTheme, blur_popover: Element) -> Element {
        let travel = triangle_wave(self.phase) * 560.0;
        let orb = Element::container([])
            .keyed("damage-control-orb")
            .width(length(54.0))
            .height(length(54.0))
            .background(theme.primary)
            .border(Border::all(6.0, theme.primary.with_alpha(0.22)))
            .radius(CornerRadii::all(999.0))
            .margin(Sides {
                left: length(travel),
                right: length(0.0),
                top: length(0.0),
                bottom: length(0.0),
            });
        let markers = Element::row((0..9).map(|index| {
            Element::container([])
                .width(length(if index % 2 == 0 { 34.0 } else { 18.0 }))
                .height(length(4.0))
                .background(theme.border)
                .radius(CornerRadii::all(999.0))
        }))
        .width(percent(1.0))
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .align_items(AlignItems::CENTER);
        Element::column([
            Element::row([
                text("CONTROLLED WORKLOAD", 11.0, theme.muted_foreground, 700),
                Element::row([
                    text(
                        "One moving element · identical in both modes",
                        12.0,
                        theme.muted_foreground,
                        450,
                    ),
                    blur_popover,
                ])
                .gap(10.0)
                .align_items(AlignItems::CENTER)
                .flex_wrap(FlexWrap::Wrap),
            ])
            .width(percent(1.0))
            .justify_content(JustifyContent::SPACE_BETWEEN)
            .gap(12.0)
            .flex_wrap(FlexWrap::Wrap),
            Element::column([orb, markers])
                .width(length(650.0))
                .max_width(percent(1.0))
                .gap(18.0),
        ])
        .width(percent(1.0))
        .height(length(190.0))
        .padding(Sides::length(22.0))
        .justify_content(JustifyContent::SPACE_BETWEEN)
        .overflow(Axes {
            x: Overflow::Hidden,
            y: Overflow::Hidden,
        })
        .background(theme.muted.with_alpha(0.55))
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(14.0))
    }

    /// Builds the live metric cards for the latest rolling sample window.
    fn metrics(&self, theme: &WidgetTheme) -> Element {
        let metrics = self.displayed;
        let ready = metrics.samples > 0;
        let ratio = if metrics.viewport_pixels == 0 {
            0.0
        } else {
            metrics.damaged_pixels as f64 / metrics.viewport_pixels as f64
        };
        let cards = [
            metric_card(
                "GPU / frame",
                when_ready(
                    ready,
                    metrics
                        .average_gpu_ms
                        .map_or_else(|| "Unavailable".into(), |value| format!("{value:.3} ms")),
                    "Waiting…",
                ),
                "Rolling GPU timestamp",
                theme,
            ),
            metric_card(
                "CPU encode",
                when_ready(
                    ready,
                    format!("{:.3} ms", metrics.average_cpu_ms),
                    "Waiting…",
                ),
                "Rolling renderer average",
                theme,
            ),
            metric_card(
                "Pixels repainted",
                when_ready(ready, format!("{:.1}%", ratio * 100.0), "Waiting…"),
                format_pixels(metrics.damaged_pixels, metrics.viewport_pixels),
                theme,
            ),
            metric_card(
                "Damage regions",
                when_ready(ready, metrics.regions.to_string(), "Waiting…"),
                when_ready(
                    ready,
                    format!(
                        "{} · {} samples",
                        damage_mode(metrics.mode),
                        metrics.samples
                    ),
                    "Collecting presented frames",
                ),
                theme,
            ),
        ];
        let retained = metrics.retained_bytes as f64 / (1024.0 * 1024.0);
        Element::column([
            Element::row(cards)
                .width(percent(1.0))
                .gap(10.0)
                .flex_wrap(FlexWrap::Wrap),
            Element::column([
                Element::row([
                    text("Rolling repaint average", 12.0, theme.muted_foreground, 550)
                        .min_width(length(0.0))
                        .shrink(1.0),
                    text(
                        when_ready(
                            ready,
                            format!(
                                "{:.1}% · retained {:.1} MiB",
                                metrics.average_ratio * 100.0,
                                retained
                            ),
                            "Waiting for samples",
                        ),
                        12.0,
                        theme.muted_foreground,
                        550,
                    )
                    .min_width(length(0.0))
                    .max_width(percent(1.0))
                    .shrink(1.0),
                ])
                .width(percent(1.0))
                .justify_content(JustifyContent::SPACE_BETWEEN)
                .gap(8.0)
                .flex_wrap(FlexWrap::Wrap),
                Element::container([Element::container([])
                    .width(percent(metrics.average_ratio.clamp(0.0, 1.0) as f32))
                    .height(percent(1.0))
                    .background(theme.primary)
                    .radius(CornerRadii::all(999.0))])
                .width(percent(1.0))
                .height(length(7.0))
                .background(theme.border.with_alpha(0.65))
                .overflow(Axes {
                    x: Overflow::Hidden,
                    y: Overflow::Hidden,
                })
                .radius(CornerRadii::all(999.0)),
            ])
            .keyed("damage-control-ratio")
            .width(percent(1.0))
            .gap(8.0)
            .padding(Sides::length(14.0))
            .background(theme.muted.with_alpha(0.4))
            .radius(CornerRadii::all(10.0)),
        ])
        .gap(12.0)
    }
}

impl Render for DamageControlDemo {
    fn wants_animation_frame(&self) -> bool {
        self.running && !self.reduced_motion
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        if !self.wants_animation_frame() {
            return;
        }
        let elapsed = frame.elapsed.as_secs_f64().min(0.05) as f32;
        self.step(elapsed / 3.2);
        self.frames = self.frames.wrapping_add(1);
        if self.frames.is_multiple_of(8) {
            self.displayed = self.telemetry.borrow().metrics();
        }
        cx.notify();
        cx.request_animation_frame();
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.reduced_motion = cx.environment().reduced_motion;
        if self.wants_animation_frame() {
            cx.request_animation_frame();
        }
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        let auto_panel = text(
            "Adaptive mode repaints only the changed tiles, then reuses retained pixels.",
            13.0,
            theme.muted_foreground,
            450,
        );
        let off_panel = text(
            "Off forces a complete surface render on every animation frame.",
            13.0,
            theme.muted_foreground,
            450,
        );
        let mode = cx.value_event_handler(|demo, selected, _, cx| {
            demo.set_mode(
                if selected == 0 {
                    ComparisonMode::Auto
                } else {
                    ComparisonMode::Off
                },
                cx,
            );
        });
        let pause = cx.callback(|demo| demo.running = !demo.running);
        let step = cx.callback(|demo| demo.step(0.08));
        let reset = cx.callback(|demo| demo.activate());
        let blur_popover = Popover::new(
            "damage-control-blur",
            "Inspect backdrop blur",
            self.blur_open,
            Button::new(
                "damage-control-blur",
                if self.blur_open {
                    "Close blur"
                } else {
                    "Open blur"
                },
                theme.outline_button(),
            )
            .build(),
            Element::column([
                text("Backdrop blur is active", 15.0, theme.foreground, 650),
                text(
                    "The moving workload continues behind this translucent surface.",
                    13.0,
                    theme.muted_foreground,
                    450,
                ),
                text(
                    "Backdrop-dependent effects require full composition today, even while Auto is selected.",
                    12.0,
                    theme.muted_foreground,
                    450,
                ),
            ])
            .gap(10.0),
        )
        .placement(FloatingPlacement::new(Placement::BottomEnd))
        .size(286.0, 220.0)
        .paint(PaintStyle::new(
            QuadStyle::solid(theme.popover.with_alpha(0.68))
                .border(Border::all(1.0, theme.popover_border))
                .radius(CornerRadii::all(12.0)),
        ))
        .radius(12.0)
        .backdrop_blur(14.0)
        .on_open_change(cx.value_callback(|demo, open| {
            demo.blur_open = open;
            demo.activate();
        }))
        .build(theme);
        super::preview(
            "Same scene, real renderer modes",
            "Switch modes while the workload runs. The cards come from RenderProfile after each presented frame, not from estimated UI state.",
            Element::column([
                Tabs::new(
                    "damage-control-mode",
                    [Tab::new("Auto · adaptive", auto_panel), Tab::new("Off · full frame", off_panel)],
                    usize::from(self.mode == ComparisonMode::Off),
                )
                .on_select(mode)
                .build(theme),
                self.workload(theme, blur_popover),
                self.metrics(theme),
                Element::row([
                    Button::new(
                        "damage-control-pause",
                        if self.running { "Pause" } else { "Run" },
                        theme.outline_button(),
                    )
                    .enabled(!self.reduced_motion)
                    .on_click(pause)
                    .build(),
                    Button::new("damage-control-step", "Step", theme.outline_button())
                        .on_click(step)
                        .build(),
                    Button::new("damage-control-reset", "Reset samples", theme.ghost_button())
                        .on_click(reset)
                        .build(),
                ])
                .gap(8.0)
                .flex_wrap(FlexWrap::Wrap),
                text(
                    "GPU time uses timestamp queries when supported; it is intentionally shown as unavailable on backends without them. Pixel ratio is the most portable comparison.",
                    12.0,
                    theme.muted_foreground,
                    400,
                ),
            ])
            .keyed("damage-control-demo")
            .width(percent(1.0))
            .gap(16.0),
            theme,
        )
    }
}

/// Chooses a measured value once renderer samples are available.
fn when_ready(ready: bool, value: String, pending: &str) -> String {
    if ready { value } else { pending.into() }
}

/// Builds one compact renderer-metric card.
fn metric_card(
    label: &str,
    value: String,
    detail: impl Into<String>,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        text(label, 11.0, theme.muted_foreground, 650),
        text(value, 22.0, theme.foreground, 720),
        text(detail, 11.0, theme.muted_foreground, 450),
    ])
    .width(length(190.0))
    .min_width(length(150.0))
    .grow(1.0)
    .padding(Sides::length(14.0))
    .gap(4.0)
    .background(theme.card)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(10.0))
}

/// Formats the latest damaged and viewport pixel counts compactly.
fn format_pixels(damaged: u64, viewport: u64) -> String {
    format!(
        "{:.2}M / {:.2}M px",
        damaged as f64 / 1_000_000.0,
        viewport as f64 / 1_000_000.0
    )
}

/// Returns a concise label for one renderer damage decision.
const fn damage_mode(mode: DamageMode) -> &'static str {
    match mode {
        DamageMode::Full => "Full",
        DamageMode::Seed => "Seed",
        DamageMode::Partial => "Partial",
        DamageMode::Reused => "Reused",
    }
}

/// Converts a repeating linear phase into a smooth back-and-forth position.
fn triangle_wave(phase: f32) -> f32 {
    let phase = phase.rem_euclid(1.0);
    if phase < 0.5 {
        phase * 2.0
    } else {
        (1.0 - phase) * 2.0
    }
}
