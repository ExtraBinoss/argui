use std::sync::Arc;

use argui_core::{Color, ColorScheme};
use argui_runtime::{
    ThemeRuntime, ThemeSchema, ThemeTokenDefinition, ThemeValue, WindowEnvironment,
};
use argui_theme::{ThemeOverrides, ThemeSource};

/// A window exposes its sparse overrides and coherent typed snapshot independently.
#[test]
fn window_environment_exposes_theme_source_and_typed_tokens() {
    let mut overrides = ThemeOverrides::default();
    assert!(overrides.set("loose", ThemeValue::Bool(true)));
    let schema = Arc::new(
        ThemeSchema::new([ThemeTokenDefinition::new(
            "foreground",
            ThemeValue::Color(Color::WHITE),
        )])
        .unwrap(),
    );
    let runtime = ThemeRuntime::new(schema);
    let token = runtime.schema().token("foreground").unwrap();
    let environment = WindowEnvironment {
        color_scheme: ColorScheme::Dark,
        primary: Color::BLACK,
        theme_overrides: Some(Arc::new(overrides)),
        theme: Some(runtime.snapshot()),
        ..WindowEnvironment::default()
    };

    assert_eq!(environment.primary_color(), Color::BLACK);
    assert_eq!(
        environment.theme_overrides().unwrap().get("loose"),
        Some(ThemeValue::Bool(true))
    );
    assert_eq!(
        environment.theme_value(token),
        Some(&ThemeValue::Color(Color::WHITE))
    );
    assert!(WindowEnvironment::default().theme_value(token).is_none());
}
