use argui::{
    animation::{
        CubicBezier, Duration, Easing, FillMode, Interpolate, Keyframe, Keyframes, Spring,
        SpringConfig, Timeline, Timing,
    },
    core::{Color, Transform2D, TransformOrigin},
    paint::{Border, CornerRadii},
    runtime::{Context, Render},
    ui::{AlignItems, Axes, Element, EventType, Overflow, Sides, UiEventKind, length, percent},
    widgets::{Button, WidgetTheme, shadcn},
};

pub(crate) struct MotionDemo {
    spring: Spring<f32>,
    spring_value: f32,
    timeline: Timeline<f32>,
    timeline_value: f32,
    timeline_from: f32,
    target: f32,
    pending: bool,
    reduced_motion: bool,
}

impl Default for MotionDemo {
    fn default() -> Self {
        Self {
            spring: spring(0.0, 0.0),
            spring_value: 0.0,
            timeline: timeline(),
            timeline_value: 0.0,
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
    }

    fn stage(&self, label: &str, detail: &str, art: Element, theme: &WidgetTheme) -> Element {
        Element::column([
            Element::column([
                super::super::app::text(label, 16.0, theme.foreground, 650),
                super::super::app::text(detail, 13.0, theme.muted_foreground, 400),
            ])
            .gap(2.0),
            Element::row([art])
                .keyed(format!(
                    "motion-stage-{}",
                    label.to_lowercase().replace(' ', "-")
                ))
                .width(percent(1.0))
                .height(length(104.0))
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
        .gap(10.0)
        .width(percent(1.0))
    }

    fn content(&self, theme: &WidgetTheme) -> Element {
        let spring = self.spring_value;
        let tween = self.timeline_value;
        let spring_square = Element::container([])
            .keyed("motion-spring-square")
            .width(length(58.0))
            .height(length(58.0))
            .background(theme.primary)
            .radius(CornerRadii::all(10.0 + spring.clamp(0.0, 1.0) * 18.0))
            .transform(
                Transform2D::IDENTITY
                    .translate(spring * 150.0, 0.0)
                    .rotate(spring * 1.15),
            )
            .transform_origin(TransformOrigin::CENTER);
        let morph_color =
            Color::srgb(0.18, 0.48, 0.98).interpolate(Color::srgb(0.94, 0.28, 0.58), tween);
        let morph = Element::container([])
            .keyed("motion-morph")
            .width(length(212.0))
            .height(length(76.0))
            .background(morph_color)
            .radius(CornerRadii::all(8.0 + tween * 30.0))
            .transform(
                Transform2D::IDENTITY
                    .scale((62.0 + tween * 150.0) / 212.0, (48.0 + tween * 28.0) / 76.0),
            )
            .transform_origin(TransformOrigin::CENTER);
        let transforms = Element::row([
            square(
                "motion-translate",
                theme.primary,
                Transform2D::IDENTITY.translate(tween * 34.0, 0.0),
            ),
            square(
                "motion-rotate",
                Color::srgb(0.55, 0.30, 0.96),
                Transform2D::IDENTITY.rotate(tween * std::f32::consts::PI),
            ),
            square(
                "motion-scale",
                Color::srgb(0.12, 0.72, 0.62),
                Transform2D::IDENTITY.scale(0.62 + tween * 0.5, 0.62 + tween * 0.5),
            ),
        ])
        .gap(24.0)
        .align_items(AlignItems::CENTER);
        Element::column([
            Button::new("motion-replay", "Run all animations", theme.button()).build(),
            self.stage(
                "Spring physics",
                "Retargeting keeps velocity, including the overshoot.",
                spring_square,
                theme,
            ),
            self.stage(
                "Resize, radius and Oklab color",
                "Layout and paint values share one typed timeline.",
                morph,
                theme,
            ),
            self.stage(
                "Transform composition",
                "Translate, rotate and scale remain independent.",
                transforms,
                theme,
            ),
        ])
        .gap(18.0)
    }
}

impl Render for MotionDemo {
    fn wants_animation_frame(&self) -> bool {
        !self.reduced_motion
            && (self.pending || self.spring.is_active() || self.timeline.needs_frame())
    }

    fn animation_frame(&mut self, frame: argui::animation::Frame, cx: &mut Context<Self>) {
        let mut changed = false;
        if self.pending {
            self.pending = false;
            self.spring.retarget(self.target);
            self.timeline = timeline();
            self.timeline.restart(frame.now);
            changed = true;
        }
        let elapsed = frame.elapsed.min(Duration::from_millis(34));
        if self.spring.advance(elapsed) {
            self.spring_value = self.spring.value();
            changed = true;
        }
        if let Some(progress) = self.timeline.sample(frame.now).value {
            let next = self.timeline_from.interpolate(self.target, progress);
            changed |= next != self.timeline_value;
            self.timeline_value = next;
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
            "One replay combines spring physics, layout interpolation, Oklab color and composed transforms. Reduced motion snaps every track to its destination.",
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

fn square(key: &str, color: Color, transform: Transform2D) -> Element {
    Element::container([])
        .keyed(key)
        .width(length(48.0))
        .height(length(48.0))
        .background(color)
        .radius(CornerRadii::all(10.0))
        .transform(transform)
        .transform_origin(TransformOrigin::CENTER)
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

fn timeline() -> Timeline<f32> {
    let easing = Easing::CubicBezier(
        CubicBezier::new(0.22, 1.0, 0.36, 1.0).expect("the motion curve is valid"),
    );
    Timeline::new(
        Keyframes::new([
            Keyframe::new(0.0, 0.0).easing(easing),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("motion keyframes are ordered"),
        Timing::new(Duration::from_millis(850)).fill(FillMode::Forwards),
    )
    .expect("the motion timeline is valid")
}
