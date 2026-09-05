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

#[test]
fn explicit_light_mode_and_all_button_variants_keep_shared_dimensions() {
    let themes = shadcn(Color::WHITE).with_mode(ThemeMode::Light);
    assert_eq!(themes.mode(), ThemeMode::Light);
    let theme = themes.resolve(ColorScheme::Dark);
    assert_eq!(theme.background, Color::WHITE);
    for style in [
        theme.button(),
        theme.secondary_button(),
        theme.outline_button(),
        theme.ghost_button(),
        theme.destructive_button(),
    ] {
        assert_eq!(style.layout.size.height, Dimension::length(36.0));
        assert_eq!(style.layout.padding, sides(14.0, 0.0));
    }

    let black_palette = shadcn(Color::BLACK);
    let black_primary = black_palette.resolve(ColorScheme::Light);
    assert_eq!(black_primary.primary_foreground, Color::WHITE);
    let white_palette = shadcn(Color::WHITE);
    let white_primary = white_palette.resolve(ColorScheme::Light);
    assert_eq!(
        white_primary.primary_foreground,
        Color::from_srgb8(9, 9, 11)
    );
}
