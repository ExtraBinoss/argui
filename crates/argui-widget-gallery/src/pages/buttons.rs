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
            "Every button shares keyboard, pointer, touch and accessibility behavior.",
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
            format!("Primary activations: {}", gallery.clicks),
            13.0,
            theme.muted_foreground,
            500,
        ),
    ])
    .gap(8.0)
}

fn variants(theme: &WidgetTheme, spinner: Element) -> Element {
    Element::row([
        Button::new("demo-button", "Primary", theme.button.clone()).build(),
        Button::new("secondary", "Secondary", theme.secondary_button.clone()).build(),
        Button::new("outline", "Outline", theme.outline_button.clone()).build(),
        Button::new("ghost", "Ghost", theme.ghost_button.clone()).build(),
        Button::new("danger", "Delete", theme.destructive_button.clone()).build(),
        Button::new("disabled", "Disabled", theme.outline_button.clone())
            .enabled(false)
            .build(),
        Button::new("loading", "Loading", theme.button.clone())
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
    Button::new(
        "button-elevated",
        "Elevated",
        theme.secondary_button.clone(),
    )
    .build()
    .layer(button_layer().shadow(Shadow::drop(
        [0.0, 5.0],
        12.0,
        Color::srgba(0.0, 0.0, 0.0, 0.24),
    )))
}

fn lift_button(theme: &WidgetTheme) -> Element {
    let mut style = theme.outline_button.clone();
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
    let mut style = theme.button.clone();
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

#[cfg(test)]
mod tests {
    use argui::animation::Time;
    use argui::core::{Affine2D, ColorScheme, Point, Size};
    use argui::paint::{ClipChain, EffectValue, Filter};
    use argui::ui::{CursorIcon, GestureSet, HitRegion, UiTree};
    use argui::widgets::shadcn;

    use super::*;

    #[test]
    fn shader_button_targets_the_registered_effect_on_hover() {
        let themes = shadcn(Color::srgb(0.1, 0.45, 0.91));
        let element = shader_button(themes.resolve(ColorScheme::Dark));
        let mut tree = UiTree::new(element.clone());
        let node = tree.node_ids()[0];
        tree.pointer_moved(
            Point::new(5.0, 5.0),
            &[HitRegion {
                node,
                bounds: Rect::new(Point::default(), Size::new(120.0, 36.0)),
                transform: Affine2D::IDENTITY,
                clips: ClipChain::default(),
                shape: argui::ui::HitShape::Bounds,
                slop: argui::ui::HitTestStyle::default().slop,
                enabled: true,
                focusable: true,
                cursor: CursorIcon::Pointer,
                gestures: GestureSet::NONE,
                window_drag: None,
            }],
        );
        tree.advance_animations(Time::from_nanos(1));
        tree.advance_animations(Time::from_nanos(200_000_001));
        let layer = element.layer.as_ref().unwrap();
        let resolved = tree.resolved_layer(node, &element, layer);
        let Filter::Effect(effect) = &resolved.filters[0] else {
            panic!("expected the animated-gradient effect");
        };
        assert_eq!(effect.id, ANIMATED_GRADIENT_ID);
        assert!(effect.parameters.iter().any(|parameter| {
            parameter.name == "intensity" && parameter.value == EffectValue::F32(0.82)
        }));
    }
}
