use argui_core::{Color, ColorScheme};
use argui_theme::{ThemeOverrides, ThemeSource, ThemeValue};
use argui_widgets::{default_theme, shadcn};

struct Source(ThemeOverrides);
impl ThemeSource for Source {
    fn primary_color(&self) -> Color {
        Color::BLACK
    }
    fn theme_overrides(&self) -> Option<&ThemeOverrides> {
        Some(&self.0)
    }
}

#[test]
fn every_exposed_token_updates_its_widget_theme_and_primary_regenerates_contrast() {
    let mut source = Source(ThemeOverrides::default());
    let original = shadcn(Color::BLACK);
    for (name, value) in original.resolve(ColorScheme::Light).tokens() {
        let replacement = match value {
            ThemeValue::Color(_) => ThemeValue::Color(Color::srgb(0.2, 0.5, 0.8)),
            ThemeValue::Float(_) => ThemeValue::Float(9.0),
            value => value,
        };
        source.0.set(name, replacement);
    }
    source.0.set("unknown", ThemeValue::Color(Color::WHITE));
    source.0.set("unknown-number", ThemeValue::Float(2.0));
    for scheme in [ColorScheme::Light, ColorScheme::Dark] {
        let themes = shadcn(&source);
        for (name, value) in themes.resolve(scheme).tokens() {
            assert_eq!(source.0.get(name), Some(value));
        }
    }
    source.0 = ThemeOverrides::default();
    source.0.set("primary", ThemeValue::Color(Color::WHITE));
    let themes = shadcn(&source);
    let palette = themes.resolve(ColorScheme::Dark);
    assert_eq!(palette.primary, Color::WHITE);
    assert!(palette.primary_foreground.contrast_ratio(palette.primary) >= 4.5);
    source.0.set("primary", ThemeValue::Float(0.0));
    assert_eq!(
        shadcn(&source).resolve(ColorScheme::Dark).primary,
        Color::BLACK
    );
}

#[test]
fn default_theme_is_the_clear_name_for_the_shadcn_compatible_palette() {
    let primary = Color::srgb(0.2, 0.5, 0.8);
    assert_eq!(default_theme(primary), shadcn(primary));
}
