use argui_core::{Color, Name, Transform2D};
use argui_media::AssetHandle;
use argui_paint::{Border, CornerRadii, Fill, Shadow};
use argui_ui::{
    AlignItems, ContainerQuery, Dimension, GridTemplateComponent, JustifyContent, LayoutInsets,
    LengthPercentageAuto, PositionInsets,
};

/// One typed native style override selected by an ancestor container's size.
#[derive(Clone, Debug, PartialEq)]
pub struct ContainerRule {
    /// Conditions combined with logical AND.
    pub conditions: Vec<ContainerQuery>,
    /// Layout fields changed while all conditions match.
    pub style: ContainerRuleStyle,
}

/// Layout properties accepted in a container-dependent style override.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ContainerRuleStyle {
    pub grid_rows: Option<Vec<GridTemplateComponent<String>>>,
    pub grid_columns: Option<Vec<GridTemplateComponent<String>>>,
    pub width: Option<Dimension>,
    pub height: Option<Dimension>,
    pub gap: Option<f32>,
    pub grow: Option<f32>,
    pub shrink: Option<f32>,
    pub align_items: Option<AlignItems>,
    pub justify_content: Option<JustifyContent>,
}

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
    Constraint,
    Insets,
    PositionInsets,
    Radii,
    Border,
    Shadow,
    Transform,
    Asset,
    GridTracks,
    ContainerRules,
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
    Constraint(LengthPercentageAuto),
    Insets(LayoutInsets),
    PositionInsets(PositionInsets),
    Radii(CornerRadii),
    Border(Border),
    Shadow(Shadow),
    Transform(Transform2D),
    Asset(AssetHandle),
    GridTracks(Vec<GridTemplateComponent<String>>),
    ContainerRules(Vec<ContainerRule>),
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
            Self::Constraint(_) => ValueType::Constraint,
            Self::Insets(_) => ValueType::Insets,
            Self::PositionInsets(_) => ValueType::PositionInsets,
            Self::Radii(_) => ValueType::Radii,
            Self::Border(_) => ValueType::Border,
            Self::Shadow(_) => ValueType::Shadow,
            Self::Transform(_) => ValueType::Transform,
            Self::Asset(_) => ValueType::Asset,
            Self::GridTracks(_) => ValueType::GridTracks,
            Self::ContainerRules(_) => ValueType::ContainerRules,
        }
    }
}
