use argui_core::{Color, ColorScheme};
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_theme::Theme;
use argui_ui::{
    CaretHeight, CaretPrimitive, CaretStyle, CaretVisual, Dimension, ScrollbarPartStyle,
    ScrollbarStyle, Sides,
};

use crate::{ButtonStyle, InputStyle};

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetTheme {
    pub background: Color,
    pub card: Color,
    pub popover: Color,
    pub foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub secondary: Color,
    pub destructive: Color,
    pub destructive_foreground: Color,
    pub border: Color,
    pub ring: Color,
    pub button: ButtonStyle,
    pub secondary_button: ButtonStyle,
    pub outline_button: ButtonStyle,
    pub ghost_button: ButtonStyle,
    pub destructive_button: ButtonStyle,
    pub input: InputStyle,
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
    let (background, card, foreground, muted, muted_foreground, border) = match scheme {
        ColorScheme::Light => (
            Color::rgb(0.985, 0.985, 0.99),
            Color::WHITE,
            Color::rgb(0.035, 0.035, 0.045),
            Color::rgb(0.955, 0.955, 0.965),
            Color::rgb(0.40, 0.40, 0.46),
            Color::rgb(0.875, 0.875, 0.895),
        ),
        ColorScheme::Dark => (
            Color::rgb(0.035, 0.035, 0.045),
            Color::rgb(0.055, 0.055, 0.07),
            Color::rgb(0.97, 0.97, 0.98),
            Color::rgb(0.105, 0.105, 0.13),
            Color::rgb(0.62, 0.62, 0.68),
            Color::rgb(0.16, 0.16, 0.20),
        ),
    };
    let primary_foreground = contrasting(primary);
    let destructive = Color::rgb(0.86, 0.20, 0.24);
    let text = TextStyle {
        font_size: 15.0,
        line_height: 20.0,
        color: foreground,
        ..TextStyle::default()
    };
    let button = button_style(
        primary,
        primary,
        primary_foreground,
        foreground,
        text.clone(),
    );
    let secondary_button = button_style(muted, border, foreground, foreground, text.clone());
    let outline_button = button_style(card, border, foreground, primary, text.clone());
    let ghost_button = button_style(
        Color::TRANSPARENT,
        Color::TRANSPARENT,
        foreground,
        primary,
        text.clone(),
    );
    let destructive_button = button_style(
        destructive,
        destructive,
        Color::WHITE,
        foreground,
        text.clone(),
    );
    let mut input = InputStyle::new(PaintStyle::new(quad(card, border)), text);
    input.hovered = quad(card, mix(border, foreground, 0.28)).into();
    input.focused = quad(card, primary).border(Border::all(1.5, primary)).into();
    input.placeholder.color = muted_foreground;
    input.selection = with_alpha(primary, 0.28);
    input.caret = CaretStyle::default();
    input.caret.visual = CaretVisual::new([CaretPrimitive::new(
        1.5,
        CaretHeight::Line,
        QuadStyle::solid(primary),
    )]);
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
        popover: card,
        foreground,
        muted,
        muted_foreground,
        primary,
        primary_foreground,
        secondary: muted,
        destructive,
        destructive_foreground: Color::WHITE,
        border,
        ring: primary,
        button,
        secondary_button,
        outline_button,
        ghost_button,
        destructive_button,
        input,
        scrollbar,
    }
}

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
        mix(background, active_mix, 0.10),
        mix(border, active_mix, 0.24),
    );
    let mut style = ButtonStyle::new(PaintStyle::new(resting.clone()), text)
        .hovered(active.clone())
        .pressed(active.opacity(0.80));
    style.layout.size.height = Dimension::length(36.0);
    style.layout.padding = argui_ui::sides(14.0, 0.0);
    style
}

fn quad(background: Color, border: Color) -> QuadStyle {
    QuadStyle::solid(background)
        .border(Border::all(1.0, border))
        .radius(CornerRadii::all(7.0))
}

fn contrasting(color: Color) -> Color {
    let [red, green, blue, _] = color.as_array();
    if red * 0.299 + green * 0.587 + blue * 0.114 > 0.62 {
        Color::rgb(0.04, 0.04, 0.05)
    } else {
        Color::WHITE
    }
}

fn mix(left: Color, right: Color, amount: f32) -> Color {
    let left = left.as_array();
    let right = right.as_array();
    Color::rgba(
        left[0] + (right[0] - left[0]) * amount,
        left[1] + (right[1] - left[1]) * amount,
        left[2] + (right[2] - left[2]) * amount,
        left[3] + (right[3] - left[3]) * amount,
    )
}

fn with_alpha(color: Color, alpha: f32) -> Color {
    let [red, green, blue, _] = color.as_array();
    Color::rgba(red, green, blue, alpha)
}
