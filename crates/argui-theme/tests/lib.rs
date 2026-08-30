use argui_core::ColorScheme;
use argui_theme::{Theme, ThemeMode};

#[test]
fn system_and_explicit_modes_resolve_without_mutating_values() {
    let mut theme = Theme::new("light", "dark");
    assert_eq!(theme.resolve(ColorScheme::Dark), &"dark");
    theme.set_mode(ThemeMode::Light);
    assert_eq!(theme.resolve(ColorScheme::Dark), &"light");
    theme.set_mode(ThemeMode::Dark);
    assert_eq!(theme.resolve(ColorScheme::Light), &"dark");
}
