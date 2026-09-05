use argui_core::{Color, ColorScheme};
use argui_theme::ThemeMode;
use argui_ui::{Dimension, sides};
use argui_widgets::shadcn;

#[test]
fn shadcn_palette_resolves_system_mode_and_contrasting_primary_text() {
    let mut themes = shadcn(Color::srgb(0.95, 0.95, 0.95));
    let light = themes.resolve(ColorScheme::Light).clone();
    let dark = themes.resolve(ColorScheme::Dark).clone();
    assert_ne!(light.background, dark.background);
    assert_eq!(light.primary_foreground, Color::from_srgb8(9, 9, 11));
    assert_eq!(light.background.to_srgba8(), [255, 255, 255, 255]);
    assert_eq!(light.foreground.to_srgba8(), [9, 9, 11, 255]);
    assert_eq!(dark.background.to_srgba8(), [9, 9, 11, 255]);
    assert_eq!(dark.foreground.to_srgba8(), [250, 250, 250, 255]);
    assert_eq!(dark.muted.to_srgba8(), [39, 39, 42, 255]);
    assert_eq!(dark.input_border, dark.border);
    assert!(light.foreground.contrast_ratio(light.background) >= 4.5);
    assert!(dark.foreground.contrast_ratio(dark.background) >= 4.5);
    assert_eq!(light.button().layout.size.height, Dimension::length(36.0));
    assert_eq!(light.button().layout.padding, sides(14.0, 0.0));
    themes.set_mode(ThemeMode::Dark);
    assert_eq!(themes.resolve(ColorScheme::Light), &dark);

    let dark_primary = shadcn(Color::srgb(0.05, 0.05, 0.05));
    assert_eq!(
        dark_primary.resolve(ColorScheme::Light).primary_foreground,
        Color::WHITE
    );
}
