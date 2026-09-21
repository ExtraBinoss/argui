use argui_core::{Color, Insets, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, Shadow};
use argui_theme::{ThemeDimension, ThemeValue, ThemeValueType};

#[test]
fn every_theme_value_preserves_its_exact_schema_type() {
    let values = [
        (ThemeValue::Color(Color::WHITE), ThemeValueType::Color),
        (
            ThemeValue::Brush(Fill::Solid(Color::WHITE)),
            ThemeValueType::Brush,
        ),
        (ThemeValue::Float(1.0), ThemeValueType::Float),
        (ThemeValue::Int(1), ThemeValueType::Int),
        (ThemeValue::Bool(true), ThemeValueType::Bool),
        (ThemeValue::Length(1.0), ThemeValueType::Length),
        (
            ThemeValue::Dimension(ThemeDimension::Auto),
            ThemeValueType::Dimension,
        ),
        (ThemeValue::Percentage(50.0), ThemeValueType::Percentage),
        (ThemeValue::DurationMillis(1.0), ThemeValueType::Duration),
        (ThemeValue::AngleRadians(1.0), ThemeValueType::Angle),
        (
            ThemeValue::Radii(CornerRadii::all(2.0)),
            ThemeValueType::Radii,
        ),
        (ThemeValue::Insets(Insets::ZERO), ThemeValueType::Insets),
        (
            ThemeValue::Border(Border::all(1.0, Color::WHITE)),
            ThemeValueType::Border,
        ),
        (
            ThemeValue::Shadow(Shadow::glow(1.0, Color::WHITE)),
            ThemeValueType::Shadow,
        ),
        (
            ThemeValue::FontFamily("Inter".into()),
            ThemeValueType::FontFamily,
        ),
        (ThemeValue::FontWeight(400), ThemeValueType::FontWeight),
        (ThemeValue::FontSize(12.0), ThemeValueType::FontSize),
        (ThemeValue::LineHeight(1.2), ThemeValueType::LineHeight),
        (
            ThemeValue::Transform(Transform2D::IDENTITY),
            ThemeValueType::Transform,
        ),
    ];
    for (value, expected) in values {
        assert_eq!(value.value_type(), expected);
        assert!(value.is_valid(), "{value:?}");
    }
    assert!(ThemeValue::Dimension(ThemeDimension::Fill).is_valid());
    assert!(ThemeValue::Dimension(ThemeDimension::Length(10.0)).is_valid());
    assert!(ThemeValue::Dimension(ThemeDimension::Percentage(10.0)).is_valid());
}

#[test]
fn numeric_theme_domains_reject_nonfinite_and_out_of_range_fields() {
    let invalid = [
        ThemeValue::Float(f32::NAN),
        ThemeValue::Length(f32::INFINITY),
        ThemeValue::Percentage(f32::NEG_INFINITY),
        ThemeValue::AngleRadians(f32::NAN),
        ThemeValue::DurationMillis(-1.0),
        ThemeValue::DurationMillis(f32::NAN),
        ThemeValue::FontSize(-1.0),
        ThemeValue::FontSize(f32::NAN),
        ThemeValue::LineHeight(0.0),
        ThemeValue::LineHeight(f32::INFINITY),
        ThemeValue::Dimension(ThemeDimension::Length(f32::NAN)),
        ThemeValue::Dimension(ThemeDimension::Percentage(f32::INFINITY)),
        ThemeValue::Radii(CornerRadii::all(-1.0)),
        ThemeValue::Radii(CornerRadii::all(f32::NAN)),
        ThemeValue::Insets(Insets::new(0.0, 0.0, f32::NAN, 0.0)),
        ThemeValue::Border(Border::all(-1.0, Color::WHITE)),
        ThemeValue::Border(Border::all(f32::NAN, Color::WHITE)),
        ThemeValue::Shadow(Shadow::drop([f32::NAN, 0.0], 1.0, Color::WHITE)),
        ThemeValue::Shadow(Shadow::glow(-1.0, Color::WHITE)),
        ThemeValue::Shadow(Shadow::glow(f32::NAN, Color::WHITE)),
        ThemeValue::Shadow(Shadow::glow(1.0, Color::WHITE).spread(f32::NAN)),
        ThemeValue::FontFamily("  ".into()),
        ThemeValue::FontWeight(0),
        ThemeValue::FontWeight(1001),
        ThemeValue::Transform(Transform2D::IDENTITY.translate(f32::NAN, 0.0)),
    ];
    for value in invalid {
        assert!(!value.is_valid(), "{value:?}");
    }
    assert!(ThemeValue::DurationMillis(0.0).is_valid());
    assert!(ThemeValue::FontWeight(1000).is_valid());
    assert!(ThemeValue::LineHeight(f32::MIN_POSITIVE).is_valid());
}
