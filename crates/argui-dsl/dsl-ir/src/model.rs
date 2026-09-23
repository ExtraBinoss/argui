mod expression;
pub use expression::*;

use argui_dsl_syntax::Span;

use crate::{
    AnimationId, AssetId, CallbackId, ComponentId, EffectId, EventTargetId, ExpressionId, FieldId,
    IrEffectBinding, IrType, LocalId, ModuleId, PropertyId, PropertyTargetId, SiteId, SlotId,
    StyleId, StyleStateId, ThemeId, ThemeModeId, TokenId, VariantId,
};

/// Optional development metadata retained at IR boundaries.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SourceInfo {
    pub span: Option<Span>,
    pub component: Option<ComponentId>,
    pub site: Option<SiteId>,
}

impl SourceInfo {
    /// Creates development metadata with an exact source span.
    ///
    /// * `span` — source range retained for diagnostics and inspection.
    /// * `component` — optional containing component identity.
    /// * `site` — optional retained visual-site identity.
    #[must_use]
    pub const fn new(span: Span, component: Option<ComponentId>, site: Option<SiteId>) -> Self {
        Self {
            span: Some(span),
            component,
            site,
        }
    }

    /// Drops byte-span metadata while preserving runtime ownership identities.
    #[must_use]
    pub const fn without_span(self) -> Self {
        Self { span: None, ..self }
    }
}

/// Fully lowered project shared by AOT codegen and the live runtime.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrProject {
    /// ABI of the exact native registry used for checking and lowering.
    #[cfg_attr(feature = "serde", serde(default))]
    pub native_schema_hash: u64,
    pub modules: Vec<IrModule>,
    pub structs: Vec<IrStruct>,
    pub enums: Vec<IrEnum>,
    pub components: Vec<IrComponent>,
    pub themes: Vec<IrTheme>,
    pub styles: Vec<IrStyle>,
    pub effects: Vec<IrEffect>,
    pub assets: Vec<IrAsset>,
}

/// Canonical module and the definition IDs it exports.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrModule {
    pub id: ModuleId,
    pub path: String,
    pub imports: Vec<IrImport>,
    pub definitions: Vec<argui_dsl_semantic::SymbolId>,
}

/// Resolved import edge with no source module or symbol names in hot paths.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrImport {
    pub module: Option<ModuleId>,
    pub definitions: Vec<argui_dsl_semantic::SymbolId>,
    pub natives: Vec<argui_schema::NativeTypeId>,
    pub source: Span,
}

/// User struct with stable field IDs.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrStruct {
    pub symbol: argui_dsl_semantic::SymbolId,
    pub fields: Vec<IrStructField>,
    pub source: SourceInfo,
}

/// Typed user-struct field.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrStructField {
    pub id: FieldId,
    pub value_type: IrType,
    pub source: SourceInfo,
}

/// User enum with stable variant identities.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEnum {
    pub symbol: argui_dsl_semantic::SymbolId,
    pub variants: Vec<IrEnumVariant>,
    pub source: SourceInfo,
}

/// One stable user-enum variant.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEnumVariant {
    pub id: VariantId,
    pub source: SourceInfo,
}

/// Typed component property.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrProperty {
    pub id: PropertyId,
    pub value_type: IrType,
    pub direction: argui_dsl_semantic::PropertyDirection,
    pub required: bool,
    pub default: Option<IrExpression>,
    pub dependencies: Vec<PropertyId>,
    pub source: SourceInfo,
}

/// Typed component callback.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrCallback {
    pub id: CallbackId,
    pub parameters: Vec<IrType>,
    pub result: IrType,
    pub source: SourceInfo,
}

/// Fully normalized component definition.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrComponent {
    pub id: ComponentId,
    pub properties: Vec<IrProperty>,
    pub callbacks: Vec<IrCallback>,
    pub slots: Vec<SlotId>,
    #[cfg_attr(feature = "serde", serde(default))]
    pub template_slots: Vec<SlotId>,
    pub body: Vec<IrNode>,
    pub states: Vec<IrState>,
    pub animations: Vec<IrAnimation>,
    pub source: SourceInfo,
}

/// Native primitive or DSL component target.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrElementTarget {
    Native(argui_schema::NativeTypeId),
    Component(ComponentId),
}

/// Typed property expression attached to a resolved target ID.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrPropertyBinding {
    pub target: PropertyTargetId,
    pub value: IrExpression,
    pub two_way: bool,
    pub source: SourceInfo,
}

/// Event handler attached to a resolved event/callback ID.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEventBinding {
    pub target: EventTargetId,
    /// Handler locals bound to event payload values in declaration order.
    pub parameters: Vec<LocalId>,
    pub statements: Vec<IrStatement>,
    pub source: SourceInfo,
}

/// Restricted event-handler statement with resolved mutation destinations.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrStatement {
    Expression(IrExpression),
    /// Requests focus on the next enabled stop in the active scope.
    FocusNext,
    /// Requests focus on the previous enabled stop in the active scope.
    FocusPrevious,
    /// Cancels the current event's host default action when it is cancelable.
    PreventDefault,
    /// Stops the current event from reaching later ancestors.
    StopPropagation,
    /// Scrolls the uniquely retained native element at `site` to a typed offset.
    ScrollTo {
        site: SiteId,
        x: IrExpression,
        y: IrExpression,
    },
    /// Activates every theme override matching a mode-name expression.
    SetThemeMode(IrExpression),
    /// Declares a mutable lexical value initialized once at this handler site.
    Let {
        local: LocalId,
        value: IrExpression,
    },
    /// Executes exactly one lexical branch without leaking branch locals.
    If {
        condition: IrExpression,
        then_body: Vec<Self>,
        else_body: Vec<Self>,
    },
    Assignment {
        target: IrAssignmentTarget,
        operator: AssignmentOperator,
        value: IrExpression,
    },
    Return(Option<IrExpression>),
}

/// Writable destination allowed by the event-handler subset.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrAssignmentTarget {
    Property(PropertyId),
    Local(LocalId),
}

/// Supported assignment and compound-assignment operations.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssignmentOperator {
    Set,
    Add,
    Subtract,
    Multiply,
    Divide,
}

/// Declarative component tree with all names normalized to IDs.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrNode {
    Element {
        site: SiteId,
        target: IrElementTarget,
        source_id: Option<String>,
        properties: Vec<IrPropertyBinding>,
        /// Optional generic shader effect applied to an authored paint region.
        #[cfg_attr(feature = "serde", serde(default))]
        effect: Option<IrEffectBinding>,
        events: Vec<IrEventBinding>,
        children: Vec<Self>,
        source: SourceInfo,
    },
    Repeater {
        site: SiteId,
        local: LocalId,
        model: IrExpression,
        key: IrExpression,
        body: Vec<Self>,
        source: SourceInfo,
    },
    Conditional {
        site: SiteId,
        condition: IrExpression,
        then_body: Vec<Self>,
        else_body: Vec<Self>,
        source: SourceInfo,
    },
    Slot {
        site: SiteId,
        slot: SlotId,
        fallback: Vec<IrNode>,
        source: SourceInfo,
    },
    /// Caller-scoped visual content assigned to a resolved component slot.
    SlotContent {
        slot: SlotId,
        body: Vec<IrNode>,
        source: SourceInfo,
    },
}

/// Named visual state with resolved assignments.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrState {
    pub site: SiteId,
    pub owner: Option<SiteId>,
    pub condition: IrExpression,
    pub assignments: Vec<(PropertyTargetId, IrExpression)>,
    pub source: SourceInfo,
}

/// Property animation retaining a stable animation slot identity.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrAnimation {
    pub id: AnimationId,
    pub owner: Option<SiteId>,
    pub property: PropertyTargetId,
    pub value_type: IrType,
    pub driver: IrAnimationDriver,
    pub transition: Option<IrTransitionPolicy>,
    pub parameters: Vec<IrAnimationParameter>,
    pub keyframes: Vec<IrAnimationKeyframe>,
    pub source: SourceInfo,
}

/// Animation timing model selected by the DSL clause.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrAnimationDriver {
    Timeline,
    Spring,
}

/// Edge of a named visual state that may start a property transition.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrTransitionPolicy {
    Enter,
    Leave,
    InOut,
}

/// One typed animation-driver parameter.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrAnimationParameter {
    pub id: PropertyId,
    pub name: String,
    pub value: IrExpression,
    pub source: SourceInfo,
}

/// One typed stop in a property animation timeline.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrAnimationKeyframe {
    /// Normalized stop position, with `0.0` = 0% and `1.0` = 100%.
    pub offset: f32,
    pub value: IrExpression,
    pub source: SourceInfo,
}

/// Typed theme and dependency-ordered token definitions.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrTheme {
    pub id: ThemeId,
    pub tokens: Vec<IrThemeToken>,
    pub modes: Vec<IrThemeMode>,
    pub source: SourceInfo,
}

/// Typed theme token with direct token-ID dependencies.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrThemeToken {
    pub id: TokenId,
    pub value_type: IrType,
    pub dependencies: Vec<TokenId>,
    pub default: IrExpression,
    pub source: SourceInfo,
}

/// Named runtime theme mode containing token overrides.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrThemeMode {
    pub id: ThemeModeId,
    pub overrides: Vec<(TokenId, IrExpression)>,
    pub source: SourceInfo,
}

/// Resolved named style with typed target-property assignments.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrStyle {
    pub id: StyleId,
    pub target: IrElementTarget,
    pub properties: Vec<IrPropertyBinding>,
    pub states: Vec<IrStyleState>,
    pub source: SourceInfo,
}

/// Named style-state overrides such as hover or focus.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrStyleState {
    pub id: StyleStateId,
    pub observation: crate::IrObservation,
    pub properties: Vec<IrPropertyBinding>,
    pub source: SourceInfo,
}

/// External WGSL effect and its typed parameter schema.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEffect {
    pub id: EffectId,
    pub shader: AssetId,
    /// Explicit author promise; omitted metadata propagates conservative damage.
    #[cfg_attr(feature = "serde", serde(default))]
    pub bounded_damage: bool,
    pub parameters: Vec<IrEffectParameter>,
    pub source: SourceInfo,
}

/// Typed effect parameter with a stable positional ID.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEffectParameter {
    pub id: PropertyId,
    /// Shader-visible argument name shared by compiled and live renderers.
    #[cfg_attr(feature = "serde", serde(default))]
    pub name: String,
    pub value_type: IrType,
    pub default: Option<IrExpression>,
    pub source: SourceInfo,
}

/// Reachable imported asset category.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum AssetKind {
    Image,
    Vector,
    Shader,
    Other,
}

/// Canonical asset table entry referenced by ID from expressions/effects.
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrAsset {
    pub id: AssetId,
    pub path: String,
    pub kind: AssetKind,
    /// Validated generated bytes embedded by a static DSL asset expression.
    #[cfg_attr(feature = "serde", serde(default))]
    pub inline_bytes: Option<Vec<u8>>,
}
