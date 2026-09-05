use argui_core::{Color, ColorInterpolation, ColorScheme};
use argui_paint::{CornerRadii, QuadStyle};
#[cfg(any(feature = "button", feature = "input"))]
use argui_paint::{Border, PaintStyle};
#[cfg(any(feature = "button", feature = "input"))]
use argui_text::TextStyle;
use argui_theme::Theme;
use argui_ui::{
    ScrollbarPartStyle, ScrollbarStyle, Sides,
};

#[cfg(feature = "button")]
use crate::ButtonStyle;
#[cfg(feature = "button")]
use argui_ui::Dimension;
#[cfg(feature = "input")]
use crate::InputStyle;
#[cfg(feature = "input")]
use argui_ui::{CaretHeight, CaretPrimitive, CaretStyle, CaretVisual};

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
    pub input_border: Color,
    pub ring: Color,
    pub overlay_blur: f32,
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
        input_border: border,
        ring: primary,
        overlay_blur: 12.0,
        dialog_backdrop: Color::srgba(0.0, 0.0, 0.0, 0.62),
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

#[cfg(any(feature = "button", feature = "input"))]
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

impl WidgetTheme {
    #[cfg(any(feature = "button", feature = "input"))]
    fn text(&self) -> TextStyle {
        TextStyle { font_size: 15.0, line_height: 20.0, color: self.foreground, ..TextStyle::default() }
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn button(&self) -> ButtonStyle {
        button_style(self.primary, self.primary, self.primary_foreground, self.foreground, self.text())
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn secondary_button(&self) -> ButtonStyle {
        button_style(self.muted, self.border, self.foreground, self.foreground, self.text())
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn outline_button(&self) -> ButtonStyle {
        button_style(self.card, self.border, self.foreground, self.primary, self.text())
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn ghost_button(&self) -> ButtonStyle {
        button_style(Color::TRANSPARENT, Color::TRANSPARENT, self.foreground, self.primary, self.text())
    }
    #[cfg(feature = "button")]
    #[must_use]
    pub fn destructive_button(&self) -> ButtonStyle {
        button_style(self.destructive, self.destructive, self.destructive_foreground, self.foreground, self.text())
    }
    #[cfg(feature = "input")]
    #[must_use]
    pub fn input(&self) -> InputStyle {
        let mut input = InputStyle::new(PaintStyle::new(quad(self.card, self.border)), self.text());
        input.hovered = quad(self.card, mix(self.border, self.foreground, 0.28)).into();
        input.focused = quad(self.card, self.primary).border(Border::all(1.5, self.primary)).into();
        input.placeholder.color = self.muted_foreground;
        input.selection = self.primary.with_alpha(0.28);
        input.caret = CaretStyle::default();
        input.caret.visual = CaretVisual::new([CaretPrimitive::new(1.5, CaretHeight::Line, QuadStyle::solid(self.primary))]);
        input
    }
}
