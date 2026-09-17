use std::sync::OnceLock;

use argui::{
    animation::{
        CueId, Duration, Easing, Interpolate, Keyframe, Keyframes, Schedule, StepPosition, Steps,
        curves,
    },
    core::{Color, Transform2D},
    paint::CornerRadii,
    ui::{AlignItems, Element, length},
    widgets::WidgetTheme,
};

use super::{MotionDemo, card, square};

/// Builds the six keyframe and orchestration gallery cards.
pub(super) fn cards(demo: &MotionDemo, theme: &WidgetTheme) -> Vec<Element> {
    let progress = demo.timeline_value.clamp(0.0, 1.0);
    let alternate = demo.alternate_value.clamp(0.0, 1.0);
    let accent = Color::srgb(0.93, 0.32, 0.56);
    let morph_color = Color::srgb(0.18, 0.48, 0.98).interpolate(accent, progress);
    let morph = Element::container([])
        .keyed("motion-morph")
        .width(length(212.0))
        .height(length(76.0))
        .background(morph_color)
        .radius(CornerRadii::all(8.0 + progress * 30.0))
        .transform(Transform2D::IDENTITY.scale(
            (62.0 + progress * 150.0) / 212.0,
            (48.0 + progress * 28.0) / 76.0,
        ));

    let multi_stop = Element::row((0..4).map(|index| {
        let local = ((progress * 4.0) - index as f32).clamp(0.0, 1.0);
        Element::container([])
            .width(length(32.0))
            .height(length(32.0 + local * 44.0))
            .background(theme.primary.interpolate(accent, local))
            .radius(CornerRadii::all(8.0 + local * 12.0))
    }))
    .gap(10.0)
    .align_items(AlignItems::CENTER);

    let held = held_keyframes().sample(progress);
    let hold = Element::row([
        square(
            "motion-hold-left",
            theme.primary.with_alpha(1.0 - held),
            Transform2D::IDENTITY.translate(held * 110.0, 0.0),
        ),
        Element::container([])
            .width(length(8.0))
            .height(length(62.0))
            .background(theme.border)
            .radius(CornerRadii::all(999.0)),
    ])
    .gap(10.0)
    .align_items(AlignItems::CENTER);

    let stepped =
        Easing::Steps(Steps::new(5, StepPosition::JumpEnd).expect("five steps form a valid curve"))
            .sample(progress);
    let steps = Element::row((0..5).map(|index| {
        let active = stepped * 5.0 > index as f32;
        Element::container([])
            .width(length(24.0))
            .height(length(24.0 + index as f32 * 9.0))
            .background(if active { theme.primary } else { theme.border })
            .radius(CornerRadii::all(7.0))
    }))
    .gap(8.0)
    .align_items(AlignItems::CENTER);

    let alternate_art = Element::row([
        square(
            "motion-alternate-a",
            theme.primary,
            Transform2D::IDENTITY.translate(alternate * 112.0, 0.0),
        ),
        square(
            "motion-alternate-b",
            accent,
            Transform2D::IDENTITY.translate((1.0 - alternate) * 24.0, 0.0),
        ),
    ])
    .gap(10.0);

    let elapsed = Duration::from_nanos((progress * 1_000_000_000.0) as u64);
    let stagger = Element::row((0..5).map(|index| {
        let cue = stagger_schedule()
            .cue(CueId::from_index(index))
            .expect("the gallery schedule contains five cues");
        let local = cue.progress(elapsed);
        Element::container([])
            .width(length(26.0))
            .height(length(26.0))
            .background(theme.primary.interpolate(accent, local))
            .radius(CornerRadii::all(13.0))
            .transform(Transform2D::IDENTITY.translate(0.0, -local * 34.0))
    }))
    .gap(9.0)
    .align_items(AlignItems::CENTER);

    vec![
        card(
            "typed-morph",
            "Typed multi-property timeline",
            "One f32 timeline drives size, radius and Oklab color.",
            morph,
            theme,
        ),
        card(
            "multi-stop",
            "Multiple keyframes",
            "Four phases reveal distinct intermediate poses.",
            multi_stop,
            theme,
        ),
        card(
            "holds",
            "Held keyframes",
            "The value waits, then jumps without polling while idle.",
            hold,
            theme,
        ),
        card(
            "steps",
            "Step easing",
            "Discrete progress is useful for meters and sprite-like motion.",
            steps,
            theme,
        ),
        card(
            "alternate",
            "Alternate direction",
            "Two iterations reverse automatically on the same timeline.",
            alternate_art,
            theme,
        ),
        card(
            "stagger",
            "Stagger schedule",
            "Five cues overlap with deterministic phase offsets.",
            stagger,
            theme,
        ),
    ]
}

/// Returns the lazily constructed keyframes used by the hold example.
fn held_keyframes() -> &'static Keyframes<f32> {
    static FRAMES: OnceLock<Keyframes<f32>> = OnceLock::new();
    FRAMES.get_or_init(|| {
        Keyframes::new([
            Keyframe::new(0.0, 0.0).hold(),
            Keyframe::new(0.58, 0.0).easing(curves::BACK_OUT),
            Keyframe::new(1.0, 1.0),
        ])
        .expect("held gallery keyframes cover the normalized interval")
    })
}

/// Returns the lazily constructed five-item stagger schedule.
fn stagger_schedule() -> &'static Schedule {
    static SCHEDULE: OnceLock<Schedule> = OnceLock::new();
    SCHEDULE.get_or_init(|| {
        Schedule::stagger(5, Duration::from_millis(520), Duration::from_millis(110))
    })
}
