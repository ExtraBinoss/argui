use crate::FieldId;

/// Backend-stable semantic value type used by every IR expression and property.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrType {
    Unknown,
    Void,
    Bool,
    Int,
    Float,
    String,
    Color,
    Brush,
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
    Asset,
    Struct {
        symbol: argui_dsl_semantic::SymbolId,
        fields: Vec<FieldId>,
    },
    Enum(argui_dsl_semantic::SymbolId),
    Optional(Box<Self>),
    Array(Box<Self>),
    Model(Box<Self>),
    Callback {
        parameters: Vec<Self>,
        result: Box<Self>,
    },
}

impl IrType {
    pub(crate) fn from_semantic(value: &argui_dsl_semantic::Type) -> Self {
        use argui_dsl_semantic::Type;
        match value {
            Type::Unknown => Self::Unknown,
            Type::Void => Self::Void,
            Type::Bool => Self::Bool,
            Type::Int => Self::Int,
            Type::Float => Self::Float,
            Type::String => Self::String,
            Type::Color => Self::Color,
            Type::Brush => Self::Brush,
            Type::Length => Self::Length,
            Type::Dimension => Self::Dimension,
            Type::Percentage => Self::Percentage,
            Type::Duration => Self::Duration,
            Type::Angle => Self::Angle,
            Type::Radii => Self::Radii,
            Type::Insets => Self::Insets,
            Type::Border => Self::Border,
            Type::Shadow => Self::Shadow,
            Type::FontFamily => Self::FontFamily,
            Type::FontWeight => Self::FontWeight,
            Type::FontSize => Self::FontSize,
            Type::LineHeight => Self::LineHeight,
            Type::Transform => Self::Transform,
            Type::Asset => Self::Asset,
            Type::Struct(symbol) => Self::Struct {
                symbol: *symbol,
                fields: Vec::new(),
            },
            Type::Enum(symbol) => Self::Enum(*symbol),
            Type::Optional(inner) => Self::Optional(Box::new(Self::from_semantic(inner))),
            Type::Array(inner) => Self::Array(Box::new(Self::from_semantic(inner))),
            Type::Model(inner) => Self::Model(Box::new(Self::from_semantic(inner))),
            Type::Callback { parameters, result } => Self::Callback {
                parameters: parameters.iter().map(Self::from_semantic).collect(),
                result: Box::new(Self::from_semantic(result)),
            },
        }
    }

    /// Resolves a semantic type and embeds stable fields for user structs.
    pub(crate) fn resolved(
        value: &argui_dsl_semantic::Type,
        fields: &std::collections::HashMap<argui_dsl_semantic::SymbolId, Vec<FieldId>>,
    ) -> Self {
        use argui_dsl_semantic::Type;
        match value {
            Type::Struct(symbol) => Self::Struct {
                symbol: *symbol,
                fields: fields.get(symbol).cloned().unwrap_or_default(),
            },
            Type::Optional(inner) => Self::Optional(Box::new(Self::resolved(inner, fields))),
            Type::Array(inner) => Self::Array(Box::new(Self::resolved(inner, fields))),
            Type::Model(inner) => Self::Model(Box::new(Self::resolved(inner, fields))),
            Type::Callback { parameters, result } => Self::Callback {
                parameters: parameters
                    .iter()
                    .map(|parameter| Self::resolved(parameter, fields))
                    .collect(),
                result: Box::new(Self::resolved(result, fields)),
            },
            _ => Self::from_semantic(value),
        }
    }

    /// Converts a canonical native-schema type to its DSL IR counterpart.
    pub(crate) const fn from_schema(value: argui_schema::ValueType) -> Self {
        match value {
            argui_schema::ValueType::Bool => Self::Bool,
            argui_schema::ValueType::Int => Self::Int,
            argui_schema::ValueType::Float => Self::Float,
            argui_schema::ValueType::String | argui_schema::ValueType::Name => Self::String,
            argui_schema::ValueType::Color => Self::Color,
            argui_schema::ValueType::Brush => Self::Brush,
            argui_schema::ValueType::Dimension => Self::Dimension,
            argui_schema::ValueType::Insets => Self::Insets,
            argui_schema::ValueType::Radii => Self::Radii,
            argui_schema::ValueType::Border => Self::Border,
            argui_schema::ValueType::Shadow => Self::Shadow,
            argui_schema::ValueType::Transform => Self::Transform,
        }
    }
}
