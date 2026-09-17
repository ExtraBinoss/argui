use argui::{
    core::Transform2D,
    paint::CornerRadii,
    ui::{Element, length},
    widgets::WidgetTheme,
};

use super::{MotionDemo, card};

/// Builds the four physics-driven animation gallery cards.
pub(super) fn cards(demo: &MotionDemo, theme: &WidgetTheme) -> Vec<Element> {
    let spring = demo.spring_value;
    let clamped = spring.clamp(0.0, 1.0);
    let spring_square = Element::container([])
        .keyed("motion-spring-square")
        .width(length(58.0))
        .height(length(58.0))
        .background(theme.primary)
        .radius(CornerRadii::all(10.0 + clamped * 18.0))
        .transform(
            Transform2D::IDENTITY
                .translate(spring * 150.0, 0.0)
                .rotate(spring * 1.15),
        );

    let inertia = Element::container([])
        .keyed("motion-inertia")
        .width(length(42.0))
        .height(length(42.0))
        .background(theme.primary)
        .radius(CornerRadii::all(999.0))
        .transform(Transform2D::IDENTITY.translate(demo.inertia_value, 0.0));

    let tail = (spring - clamped).abs().min(0.35);
    let velocity = Element::row((0..3).map(|index| {
        let lag = index as f32 * 12.0;
        Element::container([])
            .width(length(38.0 - index as f32 * 6.0))
            .height(length(38.0 - index as f32 * 6.0))
            .background(theme.primary.with_alpha(1.0 - index as f32 * 0.25))
            .radius(CornerRadii::all(999.0))
            .transform(
                Transform2D::IDENTITY
                    .translate(clamped * 112.0 - lag - tail * index as f32 * 34.0, 0.0),
            )
    }));

    let squash = (1.0 - (spring - clamped).abs() * 0.9).clamp(0.72, 1.18);
    let stretch = (2.0 - squash).clamp(0.82, 1.28);
    let squash_art = Element::container([])
        .width(length(82.0))
        .height(length(58.0))
        .background(theme.primary)
        .radius(CornerRadii::all(22.0))
        .transform(
            Transform2D::IDENTITY
                .translate(clamped * 108.0, 0.0)
                .scale(stretch, squash),
        );

    vec![
        card(
            "spring",
            "Spring physics",
            "Analytical motion overshoots and settles exactly.",
            spring_square,
            theme,
        ),
        card(
            "inertia",
            "Bounded inertia",
            "Decay crosses a bound and hands velocity to a bounce spring.",
            inertia,
            theme,
        ),
        card(
            "velocity",
            "Velocity-preserving retarget",
            "Interrupt the run: the new spring keeps current momentum.",
            velocity,
            theme,
        ),
        card(
            "squash",
            "Squash & stretch",
            "Physical overshoot becomes a restrained secondary deformation.",
            squash_art,
            theme,
        ),
    ]
}
