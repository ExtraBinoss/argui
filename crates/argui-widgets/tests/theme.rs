use argui_core::{Color, ColorScheme};
use argui_theme::ThemeMode;
use argui_ui::{Dimension, sides};
use argui_widgets::shadcn;

#[test]
fn shadcn_palette_resolves_system_mode_and_contrasting_primary_text() {
    let mut themes = shadcn(Color::rgb(0.95, 0.95, 0.95));
    let light = themes.resolve(ColorScheme::Light).clone();
    let dark = themes.resolve(ColorScheme::Dark).clone();
    assert_ne!(light.background, dark.background);
    assert_eq!(light.primary_foreground, Color::rgb(0.04, 0.04, 0.05));
    assert_eq!(light.button.layout.size.height, Dimension::length(36.0));
    assert_eq!(light.button.layout.padding, sides(14.0, 0.0));
    themes.set_mode(ThemeMode::Dark);
    assert_eq!(themes.resolve(ColorScheme::Light), &dark);

    let dark_primary = shadcn(Color::rgb(0.05, 0.05, 0.05));
    assert_eq!(
        dark_primary.resolve(ColorScheme::Light).primary_foreground,
        Color::WHITE
    );
}
