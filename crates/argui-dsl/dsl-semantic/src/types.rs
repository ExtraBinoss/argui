use crate::SymbolId;

/// Semantic value type with units preserved until backend lowering.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
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
    Struct(SymbolId),
    Enum(SymbolId),
    Optional(Box<Self>),
    Array(Box<Self>),
    Model(Box<Self>),
    Callback {
        parameters: Vec<Self>,
        result: Box<Self>,
    },
}

impl Type {
    /// Parses a built-in scalar or generic type name.
    ///
    /// User-defined names are resolved separately by the module resolver.
    ///
    /// * `name` — source type spelling without trivia.
    #[must_use]
    pub fn builtin(name: &str) -> Option<Self> {
        Some(match name {
            "void" => Self::Void,
            "bool" => Self::Bool,
            "int" => Self::Int,
            "float" => Self::Float,
            "string" => Self::String,
            "color" => Self::Color,
            "brush" => Self::Brush,
            "length" => Self::Length,
            "dimension" => Self::Dimension,
            "percentage" => Self::Percentage,
            "duration" => Self::Duration,
            "angle" => Self::Angle,
            "radii" => Self::Radii,
            "insets" => Self::Insets,
            "border" => Self::Border,
            "shadow" => Self::Shadow,
            "font-family" => Self::FontFamily,
            "font-weight" => Self::FontWeight,
            "font-size" => Self::FontSize,
            "line-height" => Self::LineHeight,
            "transform" => Self::Transform,
            "asset" => Self::Asset,
            _ => return None,
        })
    }

    /// Returns whether a value of `actual` can be assigned to this expected type.
    ///
    /// Unknown values are compatible so one root diagnostic does not cascade.
    /// Integer values widen to float, while unit-bearing numerics never erase units.
    ///
    /// * `actual` — inferred source value type.
    #[must_use]
    pub fn accepts(&self, actual: &Self) -> bool {
        self == actual
            || matches!((self, actual), (Self::Unknown, _) | (_, Self::Unknown))
            || matches!((self, actual), (Self::Float, Self::Int))
            || matches!(
                (self, actual),
                (Self::Dimension, Self::Length | Self::Percentage)
            )
            || match (self, actual) {
                (Self::Optional(expected), Self::Optional(actual))
                | (Self::Array(expected), Self::Array(actual))
                | (Self::Model(expected), Self::Model(actual)) => expected.accepts(actual),
                (Self::Optional(expected), actual) => expected.accepts(actual),
                _ => false,
            }
    }

    /// Returns whether arithmetic is defined for this type.
    #[must_use]
    pub const fn is_numeric(&self) -> bool {
        matches!(
            self,
            Self::Int
                | Self::Float
                | Self::Length
                | Self::Percentage
                | Self::Duration
                | Self::Angle
                | Self::FontSize
                | Self::LineHeight
        )
    }
}

impl std::fmt::Display for Type {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Struct(id) => write!(formatter, "struct#{id}"),
            Self::Enum(id) => write!(formatter, "enum#{id}"),
            Self::Optional(inner) => write!(formatter, "optional<{inner}>"),
            Self::Array(inner) => write!(formatter, "array<{inner}>"),
            Self::Model(inner) => write!(formatter, "model<{inner}>"),
            Self::Callback { parameters, result } => {
                write!(formatter, "callback(")?;
                for (index, parameter) in parameters.iter().enumerate() {
                    if index > 0 {
                        formatter.write_str(", ")?;
                    }
                    write!(formatter, "{parameter}")?;
                }
                write!(formatter, ") -> {result}")
            }
            value => write!(formatter, "{:?}", value).map(|_| ()),
        }
    }
}

/// Maps canonical native schema value types to semantic DSL types.
pub(crate) fn from_schema(value: argui_schema::ValueType) -> Type {
    match value {
        argui_schema::ValueType::Bool => Type::Bool,
        argui_schema::ValueType::Int => Type::Int,
        argui_schema::ValueType::Float => Type::Float,
        argui_schema::ValueType::String | argui_schema::ValueType::Name => Type::String,
        argui_schema::ValueType::Color => Type::Color,
        argui_schema::ValueType::Brush => Type::Brush,
        argui_schema::ValueType::Dimension => Type::Dimension,
        argui_schema::ValueType::Insets => Type::Insets,
        argui_schema::ValueType::Radii => Type::Radii,
        argui_schema::ValueType::Border => Type::Border,
        argui_schema::ValueType::Shadow => Type::Shadow,
        argui_schema::ValueType::Transform => Type::Transform,
    }
}
