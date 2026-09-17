use argui::widgets::WidgetTheme;
use argui::{
    core::Color,
    text::{EllipsisPosition, TextColor, TextOverflow, TextStyle, TextWrap},
    ui::{Element, ScrollEffect},
};
use argui_effects::EdgeShadow;

/// Builds a single-line text element for compact application chrome.
#[must_use]
pub(crate) fn label(
    value: impl Into<String>,
    size: f32,
    color: impl Into<TextColor>,
    weight: u16,
) -> Element {
    Element::text(value.into()).text_style(TextStyle {
        font_size: size,
        line_height: size * 1.35,
        color: color.into(),
        weight,
        wrap: TextWrap::None,
        overflow: TextOverflow::Ellipsis(EllipsisPosition::End),
        ..TextStyle::default()
    })
}

/// Returns the editor accent blue with the requested alpha.
#[must_use]
pub(crate) fn accent(alpha: f32) -> Color {
    Color::from_srgb8(43, 110, 242).with_alpha(alpha)
}

/// Returns a subtle GPU edge shadow bound to the current scroll offset.
#[must_use]
pub(crate) fn scroll_shadow(theme: &WidgetTheme) -> ScrollEffect {
    let alpha = if theme.foreground.relative_luminance() > 0.5 {
        0.07
    } else {
        0.18
    };
    EdgeShadow::new(14.0, theme.foreground.with_alpha(alpha))
        .intensity(0.9)
        .scroll()
}
