//! Stable, typed, fully resolved intermediate representation for Argui DSL backends.

mod child_reference;
mod component;
mod declaration;
mod effect;
mod error;
mod expression;
mod id;
mod lower;
mod model;
mod site;
mod types;
mod visual;

pub use effect::{IrEffectArgument, IrEffectBinding, IrEffectScope};
pub use error::LowerError;
pub use id::{
    AnimationId, AssetId, CallbackId, ComponentId, EffectId, EventTargetId, ExpressionId, FieldId,
    LocalId, ModuleId, PropertyId, PropertyTargetId, SiteId, SlotId, StyleId, StyleStateId,
    ThemeId, ThemeModeId, TokenId, VariantId,
};
pub use lower::lower;
pub use model::{
    AssetKind, AssignmentOperator, BinaryOperator, BuiltinFunction, IrAnimation, IrAnimationDriver,
    IrAnimationKeyframe, IrAnimationParameter, IrAsset, IrAssignmentTarget, IrCallback,
    IrComponent, IrEffect, IrEffectParameter, IrElementTarget, IrEnum, IrEnumVariant,
    IrEventBinding, IrExpression, IrExpressionKind, IrImport, IrModule, IrNode, IrObservation,
    IrProject, IrProperty, IrPropertyBinding, IrState, IrStatement, IrStruct, IrStructField,
    IrStyle, IrStyleState, IrTheme, IrThemeMode, IrThemeToken, IrTransitionPolicy, IrValue,
    SourceInfo, UnaryOperator,
};
pub use types::IrType;
