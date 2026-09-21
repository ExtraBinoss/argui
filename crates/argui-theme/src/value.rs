use argui_core::{Color, Insets, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, Shadow};

/// A layout dimension that preserves automatic, fill, length, and percentage semantics.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ThemeDimension {
    Auto,
    Fill,
    Length(f32),
    Percentage(f32),
}

/// The closed set of token types understood by theme schemas.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ThemeValueType {
    Color,
    Brush,
    Float,
    Int,
    Bool,
    Length,
    Dimension,
    Percentage,
    Duration,
    Angle,
    Radii,
    Insets,
    Border,
    Shadow,
    FontFamily,
    FontWeight,
    FontSize,
    LineHeight,
    Transform,
}

/// Renderer-independent, strongly typed theme-token value.
#[derive(Clone, Debug, PartialEq)]
pub enum ThemeValue {
    Color(Color),
    Brush(Fill),
    Float(f32),
    Int(i64),
    Bool(bool),
    Length(f32),
    Dimension(ThemeDimension),
    Percentage(f32),
    DurationMillis(f32),
    AngleRadians(f32),
    Radii(CornerRadii),
    Insets(Insets),
    Border(Border),
    Shadow(Shadow),
    FontFamily(String),
    FontWeight(u16),
    FontSize(f32),
    LineHeight(f32),
    Transform(Transform2D),
}

impl ThemeValue {
    /// Returns this value's exact schema type.
    #[must_use]
    pub const fn value_type(&self) -> ThemeValueType {
        match self {
            Self::Color(_) => ThemeValueType::Color,
            Self::Brush(_) => ThemeValueType::Brush,
            Self::Float(_) => ThemeValueType::Float,
            Self::Int(_) => ThemeValueType::Int,
            Self::Bool(_) => ThemeValueType::Bool,
            Self::Length(_) => ThemeValueType::Length,
            Self::Dimension(_) => ThemeValueType::Dimension,
            Self::Percentage(_) => ThemeValueType::Percentage,
            Self::DurationMillis(_) => ThemeValueType::Duration,
            Self::AngleRadians(_) => ThemeValueType::Angle,
            Self::Radii(_) => ThemeValueType::Radii,
            Self::Insets(_) => ThemeValueType::Insets,
            Self::Border(_) => ThemeValueType::Border,
            Self::Shadow(_) => ThemeValueType::Shadow,
            Self::FontFamily(_) => ThemeValueType::FontFamily,
            Self::FontWeight(_) => ThemeValueType::FontWeight,
            Self::FontSize(_) => ThemeValueType::FontSize,
            Self::LineHeight(_) => ThemeValueType::LineHeight,
            Self::Transform(_) => ThemeValueType::Transform,
        }
    }

    /// Returns whether all numeric fields satisfy their domain constraints.
    #[must_use]
    pub fn is_valid(&self) -> bool {
        match self {
            Self::Float(value)
            | Self::Length(value)
            | Self::Percentage(value)
            | Self::AngleRadians(value) => value.is_finite(),
            Self::DurationMillis(value) | Self::FontSize(value) => {
                value.is_finite() && *value >= 0.0
            }
            Self::LineHeight(value) => value.is_finite() && *value > 0.0,
            Self::Dimension(ThemeDimension::Length(value))
            | Self::Dimension(ThemeDimension::Percentage(value)) => value.is_finite(),
            Self::Radii(value) => finite_nonnegative(&value.as_array()),
            Self::Insets(value) => [value.top, value.right, value.bottom, value.left]
                .iter()
                .all(|value| value.is_finite()),
            Self::Border(value) => finite_nonnegative(&value.widths.as_array()),
            Self::Shadow(value) => {
                value.offset.iter().all(|value| value.is_finite())
                    && value.blur.is_finite()
                    && value.blur >= 0.0
                    && value.spread.is_finite()
            }
            Self::FontFamily(value) => !value.trim().is_empty(),
            Self::FontWeight(value) => (1..=1000).contains(value),
            Self::Transform(value) => [
                value.translation.x,
                value.translation.y,
                value.scale.x,
                value.scale.y,
                value.rotation,
                value.skew.x,
                value.skew.y,
            ]
            .iter()
            .all(|value| value.is_finite()),
            Self::Color(_)
            | Self::Brush(_)
            | Self::Int(_)
            | Self::Bool(_)
            | Self::Dimension(ThemeDimension::Auto | ThemeDimension::Fill) => true,
        }
    }
}

fn finite_nonnegative(values: &[f32]) -> bool {
    values
        .iter()
        .all(|value| value.is_finite() && *value >= 0.0)
}
