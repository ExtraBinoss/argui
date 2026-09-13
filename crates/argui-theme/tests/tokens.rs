use argui_core::Color;
use argui_theme::{ThemeOverrides, ThemeSource, ThemeValue};

#[test]
fn tokens_are_sparse_typed_and_reject_nonfinite_values_without_changing_snapshots() {
    let mut tokens = ThemeOverrides::default();
    assert!(tokens.is_empty());
    assert!(tokens.set("accent", ThemeValue::Color(Color::BLACK)));
    assert!(!tokens.set("accent", ThemeValue::Color(Color::BLACK)));
    let snapshot = tokens.clone();
    assert!(!tokens.set("radius", ThemeValue::Number(f32::NAN)));
    assert!(!tokens.set("radius", ThemeValue::Number(f32::INFINITY)));
    assert_eq!(tokens, snapshot);
    assert!(tokens.set("radius", ThemeValue::Number(12.0)));
    assert_eq!(tokens.iter().count(), 2);
    assert_eq!(snapshot.get("radius"), None);
    assert!(tokens.remove("accent"));
    assert!(!tokens.remove("accent"));
    assert_eq!(Color::WHITE.primary_color(), Color::WHITE);
    assert_eq!(Color::WHITE.theme_overrides(), None);
    assert_eq!(
        <&Color as ThemeSource>::primary_color(&&Color::WHITE),
        Color::WHITE
    );
}
