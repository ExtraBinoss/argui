//! Stable shader application and paint-region selection in the typed IR.

use crate::{EffectId, IrExpression, PropertyId, SourceInfo};

/// One resolved effect application on a visual element.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEffectBinding {
    pub effect: EffectId,
    /// Region of the visual receiving this effect.
    #[cfg_attr(feature = "serde", serde(default))]
    pub scope: IrEffectScope,
    /// Parameters in declaration order, including omitted defaults.
    pub parameters: Vec<IrEffectArgument>,
    pub source: SourceInfo,
}

/// One typed effect argument with its stable declared parameter identity.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IrEffectArgument {
    pub parameter: PropertyId,
    pub value: IrExpression,
    pub source: SourceInfo,
}

/// Paint region selected by a DSL effect application.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum IrEffectScope {
    /// The element and all of its visual children.
    #[default]
    Whole,
    /// Only the element's background brush.
    Background,
    /// Only the element's painted border.
    Border,
    /// The element's child content.
    Content,
    /// Text painted directly by the element.
    Text,
    /// Pixels already painted behind the element.
    Backdrop,
}
