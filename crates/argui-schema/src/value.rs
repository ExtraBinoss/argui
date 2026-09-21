use argui_assets::AssetHandle;
use argui_core::{Color, Insets, Name, Transform2D};
use argui_paint::{Border, CornerRadii, Fill, Shadow};
use argui_ui::Dimension;

/// Closed set of values accepted by native schema properties.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ValueType {
    Bool,
    Int,
    Float,
    String,
    Name,
    Color,
    Brush,
    Dimension,
    Insets,
    Radii,
    Border,
    Shadow,
    Transform,
    Asset,
}

/// Typed runtime value passed from generated code or a live interpreter to a native adapter.
#[derive(Clone, Debug, PartialEq)]
pub enum SchemaValue {
    Bool(bool),
    Int(i64),
    Float(f32),
    String(String),
    Name(Name),
    Color(Color),
    Brush(Fill),
    Dimension(Dimension),
    Insets(Insets),
    Radii(CornerRadii),
    Border(Border),
    Shadow(Shadow),
    Transform(Transform2D),
    Asset(AssetHandle),
}

impl SchemaValue {
    /// Returns the schema type represented by this value.
    #[must_use]
    pub const fn value_type(&self) -> ValueType {
        match self {
            Self::Bool(_) => ValueType::Bool,
            Self::Int(_) => ValueType::Int,
            Self::Float(_) => ValueType::Float,
            Self::String(_) => ValueType::String,
            Self::Name(_) => ValueType::Name,
            Self::Color(_) => ValueType::Color,
            Self::Brush(_) => ValueType::Brush,
            Self::Dimension(_) => ValueType::Dimension,
            Self::Insets(_) => ValueType::Insets,
            Self::Radii(_) => ValueType::Radii,
            Self::Border(_) => ValueType::Border,
            Self::Shadow(_) => ValueType::Shadow,
            Self::Transform(_) => ValueType::Transform,
            Self::Asset(_) => ValueType::Asset,
        }
    }
}
