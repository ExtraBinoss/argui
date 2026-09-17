mod advanced;
mod implicit;
mod keyframes;
mod physics;

use argui::{
    animation::{
        Direction, Duration, Easing, FillMode, Inertia, InertiaConfig, Interpolate, Iterations,
        Keyframe, Keyframes, Spring, SpringConfig, Timeline, Timing, curves,
    },
    core::{Color, Transform2D},
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    ui::{
        AlignItems, Axes, Element, EventType, FlexWrap, Overflow, Sides, UiEventKind, length,
        percent,
    },
    widgets::{Button, WidgetTheme, shadcn},
};

pub(crate) struct MotionDemo {
    spring: Spring<f32>,
    pub(super) spring_value: f32,
    inertia: Inertia,
    pub(super) inertia_value: f32,
    timeline: Timeline<f32>,
    alternate: Timeline<f32>,
    pub(super) timeline_value: f32,
    pub(super) alternate_value: f32,
    timeline_from: f32,
    pub(super) target: f32,
    pending: bool,
    pub(super) reduced_motion: bool,
}

impl Default for MotionDemo {
    fn default() -> Self {
        Self {
            spring: spring(0.0, 0.0),
            spring_value: 0.0,
            inertia: inertia(0.0),
            inertia_value: 0.0,
            timeline: timeline(),
            alternate: alternate_timeline(),
            timeline_value: 0.0,
            alternate_value: 0.0,
            timeline_from: 0.0,
            target: 1.0,
            pending: true,
            reduced_motion: false,
        }
    }
}

impl MotionDemo {
    fn replay(&mut self) {
        self.target = if self.target > 0.5 { 0.0 } else { 1.0 };
        self.timeline_from = self.timeline_value;
        if self.reduced_motion {
            self.snap();
        } else {
            self.pending = true;
        }
    }

    fn snap(&mut self) {
        self.pending = false;
        self.spring = spring(self.target, self.target);
        self.spring_value = self.target;
        self.timeline_value = self.target;
        self.alternate_value = self.target;
        self.inertia_value = self.target * 150.0;
        self.inertia = inertia(self.inertia_value);
    }

    fn content(&self, theme: &WidgetTheme) -> Element {
        let controls = Element::row([
            Button::new("motion-replay", "Run all animations", theme.button()).build(),
            super::super::app::text(
                if self.reduced_motion {
                    "Reduced motion: every example snaps to its destination"
                } else {
                    "20 live examples · interrupt at any time to test retargeting"
                },
                13.0,
                theme.muted_foreground,
                450,
            ),
        ])
        .gap(12.0)
        .flex_wrap(FlexWrap::Wrap)
        .align_items(AlignItems::CENTER);

        Element::column([
            controls,
            section(
                "Implicit animation",
                "Change the target; Argui retains the presented value and handles the transition.",
                implicit::cards(self, theme),
                theme,
            ),
            section(
                "Keyframes & orchestration",
                "Typed values, holds, steps, alternate playback and stagger share one clock.",
                keyframes::cards(self, theme),
                theme,
            ),
            section(
                "Physics",
                "Analytical springs and bounded inertia remain stable across uneven frames.",
                physics::cards(self, theme),
                theme,
            ),
            section(
                "Composition & rendering",
                "Perceptual color, additive tracks and custom curves reach the compositor directly.",
                advanced::cards(self, theme),
                theme,
            ),
        ])
        .gap(28.0)
    }
}

impl Render for MotionDemo {
    fn wants_animation_frame(&self) -> bool {
        !self.reduced_motion
            && (self.pending
                || self.spring.is_active()
                || self.inertia.is_active()
                || self.timeline.needs_frame()
                || self.alternate.needs_frame())
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        let mut changed = false;
        if self.pending {
            self.pending = false;
            self.spring.retarget(self.target);
            let velocity = if self.target > 0.5 { 720.0 } else { -720.0 };
            self.inertia.launch(self.inertia_value, velocity);
            self.timeline = timeline();
            self.timeline.restart(frame.now);
            self.alternate = alternate_timeline();
            self.alternate.restart(frame.now);
            changed = true;
        }
        let elapsed = frame.elapsed.min(Duration::from_millis(34));
        if self.spring.advance(elapsed) {
            self.spring_value = self.spring.value();
            changed = true;
        }
        if self.inertia.advance(elapsed) {
            self.inertia_value = self.inertia.value();
            changed = true;
        }
        if let Some(progress) = self.timeline.sample(frame.now).value {
            let next = self.timeline_from.interpolate(self.target, progress);
            changed |= next != self.timeline_value;
            self.timeline_value = next;
        }
        if let Some(progress) = self.alternate.sample(frame.now).value {
            changed |= progress != self.alternate_value;
            self.alternate_value = progress;
        }
        if changed {
            cx.notify();
        }
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let reduced_motion = cx.environment().reduced_motion;
        if self.reduced_motion != reduced_motion {
            self.reduced_motion = reduced_motion;
            if reduced_motion {
                self.snap();
            }
        }
        let themes = shadcn(cx.environment());
        let theme = themes.resolve(cx.environment().color_scheme);
        super::preview(
            "Animation laboratory",
            "From one-line implicit transitions to keyframes, orchestration, physics and custom curves. Every card uses the public Argui API.",
            self.content(theme),
            theme,
        )
        .on(cx.listener(EventType::Click, |demo, event, cx| {
            if event.target_key() == Some("motion-replay")
                && matches!(event.kind, UiEventKind::Click(_))
            {
                demo.replay();
                cx.notify();
            }
        }))
    }
}

/// Wraps one live example in the shared animation-gallery card treatment.
pub(super) fn card(
    key: &'static str,
    label: &'static str,
    detail: &'static str,
    art: Element,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        Element::column([
            super::super::app::text(label, 15.0, theme.foreground, 650),
            super::super::app::text(detail, 12.0, theme.muted_foreground, 400),
        ])
        .gap(2.0),
        Element::row([art])
            .keyed(format!("motion-stage-{key}"))
            .width(percent(1.0))
            .height(length(112.0))
            .padding(Sides::length(14.0))
            .align_items(AlignItems::CENTER)
            .background(theme.muted)
            .border(Border::all(1.0, theme.border))
            .radius(CornerRadii::all(12.0))
            .overflow(Axes {
                x: Overflow::Hidden,
                y: Overflow::Hidden,
            }),
    ])
    .keyed(format!("motion-card-{key}"))
    .width(length(300.0))
    .grow(1.0)
    .gap(10.0)
    .padding(Sides::length(14.0))
    .background(theme.card)
    .border(Border::all(1.0, theme.border))
    .radius(CornerRadii::all(14.0))
}

/// Groups related animation cards below a titled introduction.
fn section(
    title: &'static str,
    detail: &'static str,
    cards: Vec<Element>,
    theme: &WidgetTheme,
) -> Element {
    Element::column([
        Element::column([
            super::super::app::text(title, 19.0, theme.foreground, 720),
            super::super::app::text(detail, 13.0, theme.muted_foreground, 400),
        ])
        .gap(3.0),
        Element::row(cards)
            .width(percent(1.0))
            .gap(12.0)
            .flex_wrap(FlexWrap::Wrap),
    ])
    .gap(12.0)
}

/// Builds the shared transformed square used by several motion examples.
pub(super) fn square(key: &str, color: Color, transform: Transform2D) -> Element {
    Element::container([])
        .keyed(key)
        .width(length(48.0))
        .height(length(48.0))
        .background(color)
        .radius(CornerRadii::all(10.0))
        .transform(transform)
}

fn spring(value: f32, target: f32) -> Spring<f32> {
    Spring::new(
        value,
        target,
        0.0,
        SpringConfig {
            stiffness: 165.0,
            damping: 13.0,
            rest_speed: 0.002,
            rest_delta: 0.002,
            ..SpringConfig::default()
        },
    )
    .expect("the gallery spring is physical")
}

/// Creates the bounded inertia simulation used by the gallery.
fn inertia(value: f32) -> Inertia {
    Inertia::new(
        value,
        0.0,
        InertiaConfig {
            bounds: Some((0.0, 150.0)),
            ..InertiaConfig::default()
        },
    )
    .expect("the gallery inertia is bounded by valid values")
}

fn timeline() -> Timeline<f32> {
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 0.0).easing(curves::EASE_OUT),
            Keyframe::new(0.42, 0.68).easing(curves::EASE_IN_OUT),
            Keyframe::new(0.72, 0.86).easing(curves::DECELERATE),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("motion keyframes are ordered"),
        Timing::new(Duration::from_millis(850)).fill(FillMode::Forwards),
    )
    .expect("the motion timeline is valid")
}

/// Creates the two-pass timeline used to demonstrate alternate direction.
fn alternate_timeline() -> Timeline<f32> {
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 0.0).easing(Easing::curve(|value| value * value)),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("alternate keyframes are ordered"),
        Timing::new(Duration::from_millis(430))
            .iterations(Iterations::Finite(2.0))
            .direction(Direction::Alternate)
            .fill(FillMode::Forwards),
    )
    .expect("the alternate timeline is valid")
}
