use argui::{
    animation::{Duration, curves},
    core::{Color, Rect, Transform2D},
    paint::{Border, CornerRadii, LayerStyle, Shadow},
    ui::{AlignItems, Element, Sides, length},
    widgets::{AnimatedContainer, AnimatedOpacity, WidgetTheme},
};

use super::{MotionDemo, card};

/// Builds the seven implicit-animation gallery cards for the current state.
pub(super) fn cards(demo: &MotionDemo, theme: &WidgetTheme) -> Vec<Element> {
    let expanded = demo.target > 0.5;
    let primary = theme.primary;
    let accent = Color::srgb(0.93, 0.32, 0.56);
    let fade = AnimatedOpacity::new(
        "motion-implicit-opacity",
        if expanded { 1.0 } else { 0.12 },
        Element::container([])
            .width(length(76.0))
            .height(length(76.0))
            .background(primary)
            .radius(CornerRadii::all(22.0)),
    )
    .duration(Duration::from_millis(260))
    .curve(curves::EASE_OUT)
    .build();

    let size = AnimatedContainer::new("motion-implicit-size", [])
        .width(length(if expanded { 190.0 } else { 72.0 }))
        .height(length(if expanded { 76.0 } else { 46.0 }))
        .background(primary)
        .radius(CornerRadii::all(18.0))
        .duration(Duration::from_millis(420))
        .curve(curves::EMPHASIZED)
        .build();

    let color_radius = AnimatedContainer::new("motion-implicit-color", [])
        .width(length(92.0))
        .height(length(72.0))
        .background(if expanded { accent } else { primary })
        .radius(CornerRadii::all(if expanded { 36.0 } else { 8.0 }))
        .duration(Duration::from_millis(380))
        .curve(curves::STANDARD)
        .build();

    let dots = Element::row((0..3).map(|index| {
        Element::container([])
            .width(length(12.0))
            .height(length(12.0))
            .background(if index == 1 { accent } else { primary })
            .radius(CornerRadii::all(999.0))
    }))
    .align_items(AlignItems::CENTER);
    let spacing = AnimatedContainer::from_element(dots.keyed("motion-implicit-spacing"))
        .padding(Sides::length(if expanded { 20.0 } else { 6.0 }))
        .gap(if expanded { 24.0 } else { 5.0 })
        .background(theme.card)
        .border(Border::all(1.0, theme.border))
        .radius(CornerRadii::all(999.0))
        .duration(Duration::from_millis(360))
        .curve(curves::EASE_IN_OUT)
        .build();

    let transform = AnimatedContainer::new("motion-implicit-transform", [])
        .width(length(62.0))
        .height(length(62.0))
        .background(primary)
        .radius(CornerRadii::all(14.0))
        .transform(if expanded {
            Transform2D::IDENTITY
                .translate(130.0, 0.0)
                .rotate(std::f32::consts::PI * 0.75)
                .scale(1.12, 1.12)
        } else {
            Transform2D::IDENTITY
        })
        .duration(Duration::from_millis(520))
        .curve(curves::BACK_OUT)
        .build();

    let layer = LayerStyle::new(Rect::default()).shadow(
        Shadow::drop(
            [0.0, if expanded { 12.0 } else { 2.0 }],
            if expanded { 26.0 } else { 4.0 },
            primary.with_alpha(if expanded { 0.42 } else { 0.12 }),
        )
        .spread(if expanded { 4.0 } else { 0.0 }),
    );
    let depth = AnimatedContainer::new("motion-implicit-depth", [])
        .width(length(112.0))
        .height(length(66.0))
        .background(theme.card)
        .border(Border::all(if expanded { 3.0 } else { 1.0 }, primary))
        .radius(CornerRadii::all(if expanded { 24.0 } else { 10.0 }))
        .layer(layer)
        .duration(Duration::from_millis(420))
        .curve(curves::DECELERATE)
        .build();

    let interrupted = AnimatedContainer::new("motion-implicit-retarget", [])
        .width(length(54.0))
        .height(length(54.0))
        .background(accent)
        .radius(CornerRadii::all(16.0))
        .transform(Transform2D::IDENTITY.translate(if expanded { 145.0 } else { 0.0 }, 0.0))
        .duration(Duration::from_millis(700))
        .curve(curves::EASE_IN_OUT)
        .build();

    vec![
        card(
            "implicit-opacity",
            "AnimatedOpacity",
            "One target fades the complete child subtree.",
            fade,
            theme,
        ),
        card(
            "implicit-size",
            "Animated size",
            "Compatible layout dimensions interpolate automatically.",
            size,
            theme,
        ),
        card(
            "implicit-color",
            "Color & radius",
            "Oklab color and four radii share one transition.",
            color_radius,
            theme,
        ),
        card(
            "implicit-spacing",
            "Padding & gap",
            "Layout spacing changes without manual frame math.",
            spacing,
            theme,
        ),
        card(
            "implicit-transform",
            "Composed transform",
            "Translate, rotate and scale remain compositor-friendly.",
            transform,
            theme,
        ),
        card(
            "implicit-depth",
            "Border & shadow",
            "Paint and layer properties stay synchronized.",
            depth,
            theme,
        ),
        card(
            "implicit-retarget",
            "Interruptible retargeting",
            "Replay mid-flight: motion continues from the visible value.",
            interrupted,
            theme,
        ),
    ]
}
