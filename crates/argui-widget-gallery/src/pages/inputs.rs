use argui::{
    animation::{Duration, Keyframe, Keyframes},
    core::{Color, Transform2D},
    paint::{CornerRadii, QuadStyle},
    ui::{
        CaretAlign, CaretAnimation, CaretFrame, CaretHeight, CaretPrimitive, CaretStyle,
        CaretVisual, Element,
    },
    widgets::{Input, InputKind, TablerIcon, WidgetAssets, WidgetTheme},
};

use super::preview;
use crate::app::WidgetGallery;

pub(super) fn render(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
) -> Element {
    Element::column([
        controlled_fields(gallery, theme, assets),
        caret_playground(theme),
    ])
    .gap(28.0)
}

fn controlled_fields(
    gallery: &WidgetGallery,
    theme: &WidgetTheme,
    assets: &WidgetAssets,
) -> Element {
    let search = Input::new(
        "input-search-demo",
        &gallery.input_search,
        "Search the GPU graph…",
        theme.input(),
    )
    .kind(InputKind::Search)
    .label("GPU graph search")
    .leading(assets.icon(TablerIcon::Search, 16.0), 38.0)
    .build();
    preview(
        "Controlled fields",
        "Edit your profile, try a search, or correct the sample email. The last two fields demonstrate locked values.",
        Element::column([
            Input::new("name", &gallery.name, "Full name", theme.input())
                .label("Full name")
                .build(),
            Input::new("email", &gallery.email, "Email", theme.input())
                .label("Email address")
                .description("Used only for this local preview")
                .build(),
            search,
            Input::new("invalid", &gallery.invalid_email, "Email", theme.input())
                .label("Invalid field")
                .invalid(
                    !gallery
                        .invalid_email
                        .split_once('@')
                        .is_some_and(|(name, domain)| !name.is_empty() && domain.contains('.')),
                )
                .build(),
            Input::new("readonly", "Read-only value", "", theme.input())
                .read_only(true)
                .build(),
            Input::new("input-disabled", "Disabled", "", theme.input())
                .enabled(false)
                .build(),
        ])
        .gap(12.0),
        theme,
    )
}

fn caret_playground(theme: &WidgetTheme) -> Element {
    preview(
        "Caret playground",
        "Each caret is composed from public GPU primitives and generic keyframes; focus a field to run it.",
        Element::column([
            Input::new(
                "caret-standard",
                "Standard blinking bar",
                "Standard caret",
                theme.input(),
            )
            .read_only(true)
            .build(),
            Input::new(
                "caret-dot",
                "A dot with stepped colors",
                "Dot caret",
                with_caret(theme, colored_dot()),
            )
            .read_only(true)
            .build(),
            Input::new(
                "caret-ellipsis",
                "Three continuously animated dots",
                "Ellipsis caret",
                with_caret(theme, animated_ellipsis()),
            )
            .read_only(true)
            .build(),
        ])
        .gap(12.0),
        theme,
    )
}

fn with_caret(theme: &WidgetTheme, caret: CaretStyle) -> argui::widgets::InputStyle {
    theme.input().caret(caret)
}

fn colored_dot() -> CaretStyle {
    let dot = CaretPrimitive::new(
        6.0,
        CaretHeight::Pixels(6.0),
        QuadStyle::solid(Color::WHITE).radius(CornerRadii::all(999.0)),
    )
    .align(CaretAlign::End)
    .offset(1.0, -2.0);
    let cyan = Color::srgb(0.10, 0.82, 1.0);
    let pink = Color::srgb(1.0, 0.20, 0.58);
    let frames = Keyframes::new([
        Keyframe::new(0.0, CaretFrame::new(1.0, cyan)).hold(),
        Keyframe::new(0.24, CaretFrame::new(1.0, cyan)),
        Keyframe::new(0.25, CaretFrame::new(0.0, cyan)).hold(),
        Keyframe::new(0.49, CaretFrame::new(0.0, cyan)),
        Keyframe::new(0.5, CaretFrame::new(1.0, pink)).hold(),
        Keyframe::new(0.74, CaretFrame::new(1.0, pink)),
        Keyframe::new(0.75, CaretFrame::new(0.0, pink)).hold(),
        Keyframe::new(1.0, CaretFrame::new(0.0, pink)),
    ])
    .expect("the dot caret keyframes must cover a sorted 0..=1 range");
    CaretStyle::new(CaretVisual::new([dot])).animated(
        CaretAnimation::new(frames, Duration::from_millis(1_200))
            .expect("the dot caret duration must be non-zero"),
    )
}

fn animated_ellipsis() -> CaretStyle {
    let dots = (0..3).map(|index| {
        CaretPrimitive::new(
            4.0,
            CaretHeight::Pixels(4.0),
            QuadStyle::solid(Color::WHITE).radius(CornerRadii::all(999.0)),
        )
        .align(CaretAlign::End)
        .offset(index as f32 * 6.0, -2.0)
    });
    let frames = Keyframes::new([
        Keyframe::new(
            0.0,
            CaretFrame::new(1.0, Color::srgb(0.10, 0.82, 1.0))
                .transform(Transform2D::IDENTITY.scale(0.82, 0.82)),
        ),
        Keyframe::new(
            0.34,
            CaretFrame::new(1.0, Color::srgb(0.58, 0.25, 1.0))
                .transform(Transform2D::IDENTITY.scale(1.15, 1.15)),
        ),
        Keyframe::new(
            0.68,
            CaretFrame::new(1.0, Color::srgb(1.0, 0.24, 0.42))
                .transform(Transform2D::IDENTITY.scale(0.92, 0.92)),
        ),
        Keyframe::new(
            1.0,
            CaretFrame::new(1.0, Color::srgb(0.10, 0.82, 1.0))
                .transform(Transform2D::IDENTITY.scale(0.82, 0.82)),
        ),
    ])
    .expect("the ellipsis caret keyframes must cover a sorted 0..=1 range");
    CaretStyle::new(CaretVisual::new(dots)).animated(
        CaretAnimation::new(frames, Duration::from_millis(1_600))
            .expect("the ellipsis caret duration must be non-zero"),
    )
}
