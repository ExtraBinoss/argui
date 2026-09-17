use argui::{
    animation::{Composition, Contribution, Curve, Interpolate, compose},
    core::{Color, Transform2D},
    paint::{CornerRadii, LayerStyle, Shadow},
    ui::{AlignItems, Element, length},
    widgets::WidgetTheme,
};

use super::{MotionDemo, card, square};

/// Builds the three curve, composition and rendering gallery cards.
pub(super) fn cards(demo: &MotionDemo, theme: &WidgetTheme) -> Vec<Element> {
    let progress = demo.timeline_value.clamp(0.0, 1.0);
    let mut contributions = [
        Contribution::replace(progress * 88.0, 0, 0),
        Contribution {
            value: demo.alternate_value * 42.0,
            composition: Composition::Add,
            priority: 10,
            order: 1,
            completed_iterations: 0,
        },
    ];
    let composed = compose(0.0, &mut contributions);
    let transforms = Element::row([
        square(
            "motion-translate",
            theme.primary,
            Transform2D::IDENTITY.translate(composed, 0.0),
        ),
        square(
            "motion-rotate",
            Color::srgb(0.55, 0.30, 0.96),
            Transform2D::IDENTITY.rotate(progress * std::f32::consts::PI),
        ),
        square(
            "motion-scale",
            Color::srgb(0.12, 0.72, 0.62),
            Transform2D::IDENTITY.scale(0.62 + progress * 0.5, 0.62 + progress * 0.5),
        ),
    ])
    .gap(14.0)
    .align_items(AlignItems::CENTER);

    let custom = Anticipate.sample(progress).clamp(-0.2, 1.2);
    let custom_curve = Element::container([])
        .width(length(54.0))
        .height(length(54.0))
        .background(Color::srgb(0.93, 0.32, 0.56))
        .radius(CornerRadii::all(16.0))
        .transform(
            Transform2D::IDENTITY
                .translate(custom * 138.0, 0.0)
                .rotate(custom * 1.4),
        );

    let glow_color = theme
        .primary
        .interpolate(Color::srgb(0.93, 0.32, 0.56), progress);
    let compositor = Element::container([])
        .width(length(118.0))
        .height(length(64.0))
        .background(glow_color)
        .radius(CornerRadii::all(14.0 + progress * 18.0))
        .opacity(0.35 + progress * 0.65)
        .layer(LayerStyle::new(Default::default()).shadow(Shadow::glow(
            8.0 + progress * 26.0,
            glow_color.with_alpha(0.25 + progress * 0.35),
        )))
        .transform(Transform2D::IDENTITY.scale(0.82 + progress * 0.18, 0.82 + progress * 0.18));

    vec![
        card(
            "composition",
            "Additive composition",
            "Replace and additive tracks resolve by stable priority.",
            transforms,
            theme,
        ),
        card(
            "custom-curve",
            "User-defined Curve",
            "A tiny trait implementation adds anticipation and overshoot.",
            custom_curve,
            theme,
        ),
        card(
            "compositor",
            "Layer & compositor",
            "Opacity, transform, Oklab color and glow stay synchronized.",
            compositor,
            theme,
        ),
    ]
}

struct Anticipate;

impl Curve for Anticipate {
    fn sample(&self, progress: f32) -> f32 {
        let progress = progress.clamp(0.0, 1.0);
        let tension = 1.7;
        progress * progress * ((tension + 1.0) * progress - tension)
    }
}
