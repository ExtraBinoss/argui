use argui_core::{Color, ColorScheme};
use argui_paint::{Border, CornerRadii, PaintStyle, QuadStyle};
use argui_text::TextStyle;
use argui_ui::{ButtonStyle, ScrollbarPartStyle, ScrollbarStyle, TextInputStyle};

use crate::Theme;

#[derive(Clone, Debug, PartialEq)]
pub struct WidgetTheme {
    pub background: Color,
    pub card: Color,
    pub foreground: Color,
    pub muted: Color,
    pub muted_foreground: Color,
    pub primary: Color,
    pub primary_foreground: Color,
    pub border: Color,
    pub button: ButtonStyle,
    pub outline_button: ButtonStyle,
    pub text_input: TextInputStyle,
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
    let text = TextStyle {
        font_size: 15.0,
        line_height: 20.0,
        color: foreground,
        ..TextStyle::default()
    };
    let primary_quad = quad(primary, primary);
    let primary_active = quad(mix(primary, foreground, 0.12), primary);
    let button = ButtonStyle::new(
        PaintStyle::new(primary_quad.clone()),
        TextStyle {
            color: primary_foreground,
            weight: 600,
            ..text.clone()
        },
    )
    .hovered(primary_active.clone())
    .pressed(primary_active.opacity(0.82))
    .focused(primary_quad.border(Border::all(2.0, mix(primary, foreground, 0.35))));
    let outline = quad(card, border);
    let outline_active = quad(muted, primary);
    let outline_button = ButtonStyle::new(PaintStyle::new(outline.clone()), text.clone())
        .hovered(outline_active.clone())
        .pressed(outline_active.opacity(0.78))
        .focused(outline.border(Border::all(2.0, primary)));
    let mut text_input = TextInputStyle::new(PaintStyle::new(quad(card, border)), text);
    text_input.hovered = quad(card, mix(border, foreground, 0.28)).into();
    text_input.focused = quad(card, primary).border(Border::all(1.5, primary)).into();
    text_input.placeholder.color = muted_foreground;
    text_input.selection = with_alpha(primary, 0.28);
    text_input.caret = primary;
    let scrollbar = ScrollbarStyle::new(
        ScrollbarPartStyle::new(QuadStyle::solid(Color::TRANSPARENT)),
        ScrollbarPartStyle::new(
            QuadStyle::solid(mix(muted_foreground, card, 0.18)).radius(CornerRadii::all(999.0)),
        ),
    )
    .width(8.0)
    .insets(argui_ui::Edges::all(4.0))
    .min_thumb(24.0);
    WidgetTheme {
        background,
        card,
        foreground,
        muted,
        muted_foreground,
        primary,
        primary_foreground,
        border,
        button,
        outline_button,
        text_input,
        scrollbar,
    }
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
