use argui_core::{Color, ColorInterpolation, ColorScheme};
#[cfg(any(feature = "button", feature = "input", feature = "textarea"))]
use argui_paint::{Border, PaintStyle};
use argui_paint::{CornerRadii, Filter, LayerMask, LayerStyle, QuadStyle, Shadow};
#[cfg(any(feature = "button", feature = "input", feature = "textarea"))]
use argui_text::TextStyle;
use argui_theme::Theme;
use argui_ui::{ScrollbarPartStyle, ScrollbarStyle, Sides};

#[cfg(feature = "button")]
use crate::ButtonStyle;
#[cfg(any(feature = "input", feature = "textarea"))]
use crate::InputStyle;
#[cfg(feature = "button")]
use argui_ui::Dimension;
#[cfg(any(feature = "input", feature = "textarea"))]
use argui_ui::{CaretHeight, CaretPrimitive, CaretStyle, CaretVisual};

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetTheme {
    pub background: Color,
    pub card: Color,
    pub popover: Color,
    /// Floating-surface outline, independent from ordinary control borders.
    pub popover_border: Color,
    pub foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub secondary: Color,
    pub destructive: Color,
    pub destructive_foreground: Color,
    pub border: Color,
    pub input_border: Color,
    pub ring: Color,
    pub overlay_blur: f32,
    /// Shared floating-panel elevation. Clear this vector to disable shadows.
    pub overlay_shadows: Vec<Shadow>,
    pub dialog_backdrop: Color,
    pub dialog_backdrop_blur: f32,
    pub scrollbar: ScrollbarStyle,
}

#[must_use]
pub fn shadcn(primary: Color) -> Theme<WidgetTheme> {
    Theme::new(
        widgets(ColorScheme::Light, primary),
        widgets(ColorScheme::Dark, primary),
    )
}

fn widgets(scheme: ColorScheme, primary: Color) -> WidgetTheme {
    let (background, card, foreground, muted, muted_foreground, border, destructive) = match scheme
    {
        ColorScheme::Light => (
            Color::WHITE,
            Color::WHITE,
            Color::from_srgb8(9, 9, 11),
            Color::from_srgb8(244, 244, 245),
            Color::from_srgb8(113, 113, 122),
            Color::from_srgb8(228, 228, 231),
            Color::from_srgb8(239, 68, 68),
        ),
        ColorScheme::Dark => (
            Color::from_srgb8(9, 9, 11),
            Color::from_srgb8(9, 9, 11),
            Color::from_srgb8(250, 250, 250),
            Color::from_srgb8(39, 39, 42),
            Color::from_srgb8(161, 161, 170),
            Color::from_srgb8(39, 39, 42),
            Color::from_srgb8(127, 29, 29),
        ),
    };
    let primary_foreground = contrasting(primary);
    let (popover, popover_border, shadow_alpha, backdrop_alpha) = match scheme {
        ColorScheme::Light => (Color::WHITE, Color::BLACK.with_alpha(0.10), 0.18, 0.32),
        ColorScheme::Dark => (
            Color::from_srgb8(24, 24, 27),
            Color::WHITE.with_alpha(0.08),
            0.55,
            0.62,
        ),
    };
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::TRANSPARENT)),
        ScrollbarPartStyle::new(
            QuadStyle::solid(mix(muted_foreground, card, 0.18)).radius(CornerRadii::all(999.0)),
        ),
    )
    .width(8.0)
    .insets(Sides {
        left: 4.0,
        right: 4.0,
        top: 4.0,
        bottom: 4.0,
    })
    .min_thumb(24.0);
    WidgetTheme {
        background,
        card,
        popover,
        popover_border,
        foreground,
        muted,
        muted_foreground,
        primary,
        primary_foreground,
        secondary: muted,
        destructive,
        destructive_foreground: Color::WHITE,
        border,
        input_border: border,
        ring: primary,
        overlay_blur: 12.0,
        overlay_shadows: vec![Shadow::drop(
            [0.0, 6.0],
            14.0,
            Color::BLACK.with_alpha(shadow_alpha),
        )],
        dialog_backdrop: Color::BLACK.with_alpha(backdrop_alpha),
        dialog_backdrop_blur: 4.0,
        scrollbar,
    }
}

#[cfg(feature = "button")]
fn button_style(
    background: Color,
    border: Color,
    foreground: Color,
    active_mix: Color,
    mut text: TextStyle,
) -> ButtonStyle {
    text.color = foreground;
    text.weight = 600;
    let resting = quad(background, border);
    let active = quad(
        mix(background, active_mix, 0.16),
        mix(border, active_mix, 0.36),
    );
    let mut style = ButtonStyle::new(PaintStyle::new(resting.clone()), text)
        .hovered(active.clone())
        .pressed(active.opacity(0.80));
    style.layout.size.height = Dimension::length(36.0);
    style.layout.padding = argui_ui::sides(14.0, 0.0);
    style
}

#[cfg(any(feature = "button", feature = "input", feature = "textarea"))]
fn quad(background: Color, border: Color) -> QuadStyle {
    QuadStyle::solid(background)
        .border(Border::all(1.0, border))
        .radius(CornerRadii::all(7.0))
}

fn contrasting(color: Color) -> Color {
    let dark = Color::from_srgb8(9, 9, 11);
    if color.contrast_ratio(Color::WHITE) >= color.contrast_ratio(dark) {
        Color::WHITE
    } else {
        dark
    }
}

fn mix(left: Color, right: Color, amount: f32) -> Color {
    left.mix(right, amount, ColorInterpolation::Oklab)
}

#[cfg(any(
    feature = "button",
    feature = "input",
    feature = "textarea",
    feature = "tabs",
    feature = "checkbox",
    feature = "switch",
    feature = "radio-group"
))]
pub(crate) fn instant_hover(
    mut transition: argui_ui::StyleTransition,
    hover: argui_ui::StateSelector,
) -> argui_ui::StyleTransition {
    for direction in [
        argui_ui::TransitionDirection::Enter(hover.into()),
        argui_ui::TransitionDirection::Exit(hover.into()),
    ] {
        transition = transition.rule(
            argui_ui::TransitionRule::new(argui_ui::Transition::tween(
                argui_animation::Tween::new(argui_animation::Duration::ZERO),
            ))
            .direction(direction),
        );
    }
    transition
}

impl WidgetTheme {
    /// Rounded floating surface with independent backdrop blur and elevation.
    #[must_use]
    pub fn overlay_layer(&self, radius: f32, blur: f32) -> LayerStyle {
        let mut layer = LayerStyle::new(Default::default())
            .mask(LayerMask::Rounded(CornerRadii::all(radius.max(0.0))));
        layer.shadows.clone_from(&self.overlay_shadows);
        if blur > 0.0 {
            layer = layer.backdrop(Filter::Blur(blur));
        }
        layer
    }

    #[cfg(any(feature = "button", feature = "input", feature = "textarea"))]
    fn text(&self) -> TextStyle {
        TextStyle {
            font_size: 15.0,
            line_height: 20.0,
            color: self.foreground,
            ..TextStyle::default()
        }
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn button(&self) -> ButtonStyle {
        button_style(
            self.primary,
            self.primary,
            self.primary_foreground,
            self.foreground,
            self.text(),
        )
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn secondary_button(&self) -> ButtonStyle {
        button_style(
            self.muted,
            self.border,
            self.foreground,
            self.foreground,
            self.text(),
        )
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn outline_button(&self) -> ButtonStyle {
        button_style(
            self.card,
            self.border,
            self.foreground,
            self.primary,
            self.text(),
        )
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn ghost_button(&self) -> ButtonStyle {
        button_style(
            Color::TRANSPARENT,
            Color::TRANSPARENT,
            self.foreground,
            self.primary,
            self.text(),
        )
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn destructive_button(&self) -> ButtonStyle {
        button_style(
            self.destructive,
            self.destructive,
            self.destructive_foreground,
            self.foreground,
            self.text(),
        )
    }
    #[cfg(any(feature = "input", feature = "textarea"))]
    #[must_use]
    pub fn input(&self) -> InputStyle {
        let mut input = InputStyle::new(PaintStyle::new(quad(self.card, self.border)), self.text());
        input.hovered = quad(self.card, mix(self.border, self.foreground, 0.28)).into();
        input.focused = quad(self.card, self.primary).into();
        input.placeholder.color = self.muted_foreground;
        input.selection = self.primary.with_alpha(0.28);
        input.caret = CaretStyle::default();
        input.caret.visual = CaretVisual::new([CaretPrimitive::new(
            1.5,
            CaretHeight::Line,
            QuadStyle::solid(self.primary),
        )]);
        input
    }
}
