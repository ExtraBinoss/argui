use std::collections::BTreeMap;

use argui_dsl_ir::{AssetId, FieldId, IrType};

use crate::RuntimeError;

/// Bounded, type-erased development value; never crosses into engine crates.
#[derive(Clone, Debug, PartialEq)]
pub enum DslValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(argui_core::Color),
    Brush(argui_paint::Fill),
    Dimension(argui_ui::Dimension),
    Insets(argui_core::Insets),
    Radii(argui_paint::CornerRadii),
    Border(argui_paint::Border),
    Shadow(argui_paint::Shadow),
    Transform(argui_core::Transform2D),
    AssetHandle(argui_assets::AssetHandle),
    Struct(BTreeMap<FieldId, Self>),
    Enum { symbol: u64, variant: u64 },
    Array(Vec<Self>),
    Asset(AssetId),
}

impl DslValue {
    /// Normalizes this value to the checked destination `expected`.
    /// Returns the value with integers widened inside floats and containers.
    pub(crate) fn coerce(self, expected: &IrType) -> Self {
        match (self, expected) {
            (Self::Int(value), IrType::Float) => Self::Float(f64::from(value as f32)),
            (Self::Float(value), _) => Self::Float(f64::from(value as f32)),
            (Self::Null, IrType::Optional(_)) => Self::Null,
            (value, IrType::Optional(inner)) => value.coerce(inner),
            (Self::Array(values), IrType::Array(inner) | IrType::Model(inner)) => Self::Array(
                values
                    .into_iter()
                    .map(|value| value.coerce(inner))
                    .collect(),
            ),
            (value, _) => value,
        }
    }

    /// Returns a concise runtime type name for diagnostics.
    #[must_use]
    pub const fn type_name(&self) -> &'static str {
        match self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Int(_) => "int",
            Self::Float(_) => "float",
            Self::String(_) => "string",
            Self::Color(_) => "color",
            Self::Brush(_) => "brush",
            Self::Dimension(_) => "dimension",
            Self::Insets(_) => "insets",
            Self::Radii(_) => "radii",
            Self::Border(_) => "border",
            Self::Shadow(_) => "shadow",
            Self::Transform(_) => "transform",
            Self::AssetHandle(_) => "asset",
            Self::Struct(_) => "struct",
            Self::Enum { .. } => "enum",
            Self::Array(_) => "array",
            Self::Asset(_) => "asset",
        }
    }

    /// Converts a constant IR value without parsing source text.
    pub(crate) fn constant(value: &argui_dsl_ir::IrValue) -> Self {
        match value {
            argui_dsl_ir::IrValue::Null => Self::Null,
            argui_dsl_ir::IrValue::Bool(value) => Self::Bool(*value),
            argui_dsl_ir::IrValue::Int(value) => Self::Int(*value),
            argui_dsl_ir::IrValue::Float(value) => Self::Float(f64::from(*value as f32)),
            argui_dsl_ir::IrValue::String(value) => Self::String(value.clone()),
            argui_dsl_ir::IrValue::Color(value) => {
                let [red, green, blue, alpha] = value.to_be_bytes();
                Self::Color(argui_core::Color::from_srgba8(red, green, blue, alpha))
            }
        }
    }

    /// Returns whether this value can be assigned to `expected`, including widening.
    #[must_use]
    pub fn compatible_with(&self, expected: &IrType) -> bool {
        matches!(
            (self, expected),
            (Self::Null, IrType::Optional(_))
                | (Self::Bool(_), IrType::Bool)
                | (Self::Int(_), IrType::Int | IrType::Float)
                | (
                    Self::Float(_),
                    IrType::Float
                        | IrType::Length
                        | IrType::Dimension
                        | IrType::Percentage
                        | IrType::Duration
                        | IrType::Angle
                        | IrType::FontSize
                        | IrType::LineHeight
                )
                | (
                    Self::String(_),
                    IrType::String | IrType::FontFamily | IrType::FontWeight
                )
                | (Self::Color(_), IrType::Color)
                | (Self::Brush(_), IrType::Brush)
                | (Self::Dimension(_), IrType::Dimension)
                | (Self::Insets(_), IrType::Insets)
                | (Self::Radii(_), IrType::Radii)
                | (Self::Border(_), IrType::Border)
                | (Self::Shadow(_), IrType::Shadow)
                | (Self::Transform(_), IrType::Transform)
                | (Self::AssetHandle(_), IrType::Asset)
                | (Self::Struct(_), IrType::Struct { .. })
                | (Self::Enum { .. }, IrType::Enum(_))
                | (Self::Asset(_), IrType::Asset)
        ) || match (self, expected) {
            (Self::Array(values), IrType::Array(inner) | IrType::Model(inner)) => {
                values.iter().all(|value| value.compatible_with(inner))
            }
            (_, IrType::Optional(inner)) => self.compatible_with(inner),
            _ => false,
        }
    }

    /// Converts to a native schema value using the statically known IR type.
    pub(crate) fn to_schema(
        &self,
        value_type: &IrType,
    ) -> Result<argui_schema::SchemaValue, RuntimeError> {
        use argui_schema::SchemaValue;
        match (self, value_type) {
            (Self::Bool(value), IrType::Bool) => Ok(SchemaValue::Bool(*value)),
            (Self::Int(value), IrType::Int) => Ok(SchemaValue::Int(*value)),
            (Self::Float(value), IrType::Float) => Ok(SchemaValue::Float(*value as f32)),
            (Self::String(value), IrType::String | IrType::FontFamily | IrType::FontWeight) => {
                Ok(SchemaValue::String(value.clone()))
            }
            (Self::Color(value), IrType::Color) => Ok(SchemaValue::Color(*value)),
            (Self::Brush(value), IrType::Brush) => Ok(SchemaValue::Brush(value.clone())),
            (Self::Dimension(value), IrType::Dimension) => Ok(SchemaValue::Dimension(*value)),
            (Self::Insets(value), IrType::Insets) => Ok(SchemaValue::Insets(*value)),
            (Self::Radii(value), IrType::Radii) => Ok(SchemaValue::Radii(*value)),
            (Self::Border(value), IrType::Border) => Ok(SchemaValue::Border(*value)),
            (Self::Shadow(value), IrType::Shadow) => Ok(SchemaValue::Shadow(*value)),
            (Self::Transform(value), IrType::Transform) => Ok(SchemaValue::Transform(*value)),
            (Self::AssetHandle(value), IrType::Asset) => Ok(SchemaValue::Asset(*value)),
            (Self::Float(value), IrType::Length | IrType::FontSize | IrType::LineHeight) => {
                Ok(SchemaValue::Dimension(argui_ui::length(*value as f32)))
            }
            (Self::Float(value), IrType::Percentage) => Ok(SchemaValue::Dimension(
                argui_ui::percent(*value as f32 / 100.0),
            )),
            _ => Err(RuntimeError::TypeMismatch {
                expected: format!("{value_type:?}"),
                actual: self.type_name().into(),
            }),
        }
    }
}
