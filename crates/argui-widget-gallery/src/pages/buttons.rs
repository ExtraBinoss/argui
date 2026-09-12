use argui::{
    core::{Color, Rect, Transform2D},
    paint::{CornerRadii, LayerMask, LayerStyle, Shadow},
    ui::{Element, FlexWrap, property},
    widgets::{Button, WidgetTheme},
};
use argui_effects::{ANIMATED_GRADIENT_ID, AnimatedGradient};

use super::{preview, text};
use crate::app::WidgetGallery;

pub(super) fn render(gallery: &WidgetGallery, theme: &WidgetTheme, spinner: Element) -> Element {
    Element::column([
        preview(
            "Variants",
            "Hover for a tooltip, then click any available button to update the count below.",
            variants(theme, spinner),
            theme,
        ),
        preview(
            "Depth and GPU hover",
            "Shadows, transforms and typed WGSL parameters use the same retained hover transition.",
            expressive_buttons(theme),
            theme,
        ),
        text(
            format!("Button activations: {}", gallery.clicks),
            13.0,
            theme.muted_foreground,
            500,
        ),
    ])
    .gap(8.0)
}

fn variants(theme: &WidgetTheme, spinner: Element) -> Element {
    Element::row([
        Button::new("demo-button", "Primary", theme.button()).build(),
        Button::new("secondary", "Secondary", theme.secondary_button()).build(),
        Button::new("outline", "Outline", theme.outline_button()).build(),
        Button::new("ghost", "Ghost", theme.ghost_button()).build(),
        Button::new("danger", "Delete", theme.destructive_button()).build(),
        Button::new("disabled", "Disabled", theme.outline_button())
            .enabled(false)
            .build(),
        Button::new("loading", "Loading", theme.button())
            .loading(spinner)
            .build(),
    ])
    .flex_wrap(FlexWrap::Wrap)
    .gap(10.0)
}

fn expressive_buttons(theme: &WidgetTheme) -> Element {
    Element::row([
        elevated_button(theme),
        lift_button(theme),
        shader_button(theme),
    ])
    .flex_wrap(FlexWrap::Wrap)
    .gap(14.0)
}

fn elevated_button(theme: &WidgetTheme) -> Element {
    Button::new("button-elevated", "Elevated", theme.secondary_button())
        .build()
        .layer(button_layer().shadow(Shadow::drop(
            [0.0, 5.0],
            12.0,
            Color::srgba(0.0, 0.0, 0.0, 0.24),
        )))
}

fn lift_button(theme: &WidgetTheme) -> Element {
    let mut style = theme.outline_button();
    style.hovered = style
        .hovered
        .set(
            property::Transform,
            Transform2D::IDENTITY.translate(0.0, -3.0),
        )
        .set(property::shadow_offset(0), [0.0, 8.0])
        .set(property::shadow_blur(0), 16.0);
    style.pressed = style.pressed.set(
        property::Transform,
        Transform2D::IDENTITY.translate(0.0, 1.0),
    );
    Button::new("button-lift", "Lift on hover", style)
        .build()
        .layer(button_layer().shadow(Shadow::drop(
            [0.0, 2.0],
            5.0,
            Color::srgba(0.0, 0.0, 0.0, 0.22),
        )))
}

fn shader_button(theme: &WidgetTheme) -> Element {
    let mut style = theme.button();
    style.hovered = style.hovered.set(
        property::effect_f32(ANIMATED_GRADIENT_ID, "intensity"),
        0.82,
    );
    style.pressed = style.pressed.set(
        property::effect_f32(ANIMATED_GRADIENT_ID, "intensity"),
        0.48,
    );
    Button::new("button-shader", "GPU shader hover", style)
        .build()
        .layer(
            button_layer().filter(
                AnimatedGradient::new(0.4)
                    .frequency(1.15)
                    .intensity(0.0)
                    .filter(),
            ),
        )
}

fn button_layer() -> LayerStyle {
    LayerStyle::new(Rect::default()).mask(LayerMask::Rounded(CornerRadii::all(7.0)))
}
