//! Typed expression operations shared by AOT and live backends.

use super::*;

/// Typed expression and its normalized operation.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrExpression {
    pub id: ExpressionId,
    pub value_type: IrType,
    pub kind: IrExpressionKind,
    pub source: SourceInfo,
}

/// Expression operation with no unresolved semantic names.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrExpressionKind {
    Constant(IrValue),
    PropertyRead(PropertyId),
    /// Read-only engine state for one explicitly identified native element.
    ObservedRead {
        site: SiteId,
        property: argui_schema::PropertyId,
        observation: IrObservation,
    },
    /// Reads a public property of one stable child component instance.
    ChildPropertyRead {
        site: SiteId,
        property: PropertyId,
        #[cfg_attr(feature = "serde", serde(default))]
        optional: bool,
    },
    LocalRead(LocalId),
    FieldRead {
        base: Box<IrExpression>,
        field: FieldId,
    },
    TokenRead(TokenId),
    Asset(AssetId),
    BuiltinCall {
        function: BuiltinFunction,
        arguments: Vec<IrExpression>,
    },
    CallbackCall {
        callback: CallbackId,
        arguments: Vec<IrExpression>,
    },
    Unary {
        operator: UnaryOperator,
        operand: Box<IrExpression>,
    },
    Binary {
        operator: BinaryOperator,
        left: Box<IrExpression>,
        right: Box<IrExpression>,
    },
    Conditional {
        condition: Box<IrExpression>,
        then_value: Box<IrExpression>,
        else_value: Box<IrExpression>,
    },
    /// Constructs a checked user struct from stable field IDs.
    Struct {
        symbol: argui_dsl_semantic::SymbolId,
        fields: Vec<(FieldId, IrExpression)>,
    },
    /// One checked user enum variant with stable identity.
    EnumVariant {
        symbol: argui_dsl_semantic::SymbolId,
        variant: crate::VariantId,
    },
    Array(Vec<IrExpression>),
    /// Safe array or model access; missing and negative indices return null.
    Index {
        base: Box<IrExpression>,
        index: Box<IrExpression>,
    },
}

/// Engine-observed state independent of the native element that exposes it.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrObservation {
    Hover,
    Pressed,
    Focused,
    FocusVisible,
    PointerX,
    PointerY,
    PointerGlobalX,
    PointerGlobalY,
    PressedX,
    PressedY,
    ScrollX,
    ScrollY,
    ViewportWidth,
    ViewportHeight,
    ContentWidth,
    ContentHeight,
    MeasuredWidth,
    MeasuredHeight,
}

impl From<argui_schema::ObservationKind> for IrObservation {
    /// Converts a checked native observation into the transport-safe IR kind.
    ///
    /// * `value` — observation declared by native schema metadata.
    ///
    /// Returns the equivalent name-free IR observation.
    fn from(value: argui_schema::ObservationKind) -> Self {
        match value {
            argui_schema::ObservationKind::Hover => Self::Hover,
            argui_schema::ObservationKind::Pressed => Self::Pressed,
            argui_schema::ObservationKind::Focused => Self::Focused,
            argui_schema::ObservationKind::FocusVisible => Self::FocusVisible,
            argui_schema::ObservationKind::PointerX => Self::PointerX,
            argui_schema::ObservationKind::PointerY => Self::PointerY,
            argui_schema::ObservationKind::PointerGlobalX => Self::PointerGlobalX,
            argui_schema::ObservationKind::PointerGlobalY => Self::PointerGlobalY,
            argui_schema::ObservationKind::PressedX => Self::PressedX,
            argui_schema::ObservationKind::PressedY => Self::PressedY,
            argui_schema::ObservationKind::ScrollX => Self::ScrollX,
            argui_schema::ObservationKind::ScrollY => Self::ScrollY,
            argui_schema::ObservationKind::ViewportWidth => Self::ViewportWidth,
            argui_schema::ObservationKind::ViewportHeight => Self::ViewportHeight,
            argui_schema::ObservationKind::ContentWidth => Self::ContentWidth,
            argui_schema::ObservationKind::ContentHeight => Self::ContentHeight,
            argui_schema::ObservationKind::MeasuredWidth => Self::MeasuredWidth,
            argui_schema::ObservationKind::MeasuredHeight => Self::MeasuredHeight,
        }
    }
}

/// Constant runtime value with units represented by its enclosing expression type.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(String),
    Color(u32),
}

/// Side-effect-free built-in expression functions.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BuiltinFunction {
    Translate,
    Stringify,
    IsSome,
    UnwrapOr,
    Contains,
    Lower,
    Range,
    Slice,
    Hsv,
    ColorHex,
    ColorRed,
    ColorGreen,
    ColorBlue,
    LinearGradient,
    RadialGradient,
    ConicGradient,
    Solid,
}

/// Unary expression operator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum UnaryOperator {
    Not,
    Negate,
    Positive,
}

/// Binary expression operator.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Remainder,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
}
