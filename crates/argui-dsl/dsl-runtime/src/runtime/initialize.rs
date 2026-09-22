//! Initial component values before presentation observations are available.

use std::collections::{BTreeMap, HashMap};

use argui_dsl_ir::{ComponentId, IrType, TokenId};

use super::ValueContext;
use crate::{ComponentInstance, DslValue, DynamicProperty, InstanceId, LivePackage, RuntimeError};

/// Initializes one component's properties in dependency/source order.
///
/// `package` supplies typed defaults, `component` selects the definition,
/// `id` is the retained instance identity, and `tokens` supplies active theme
/// values. Returns an instance with total initial values, or an error when
/// the component or a non-contextual default cannot be evaluated.
pub(crate) fn initialize_instance(
    package: &LivePackage,
    component: ComponentId,
    id: InstanceId,
    tokens: &HashMap<TokenId, DslValue>,
) -> Result<ComponentInstance, RuntimeError> {
    let definition = package
        .ir
        .components
        .iter()
        .find(|definition| definition.id == component)
        .ok_or(RuntimeError::MissingComponent(component.raw()))?;
    let mut properties = HashMap::new();
    for property in &definition.properties {
        let value = if let Some(default) = &property.default
            && !package
                .program(default.id)
                .ok_or(RuntimeError::MissingExpression(default.id.raw()))?
                .is_contextual()
        {
            let program = package
                .program(default.id)
                .ok_or(RuntimeError::MissingExpression(default.id.raw()))?;
            let mut context = ValueContext::new(&properties, tokens);
            program.evaluate(&mut context)?
        } else {
            default_value(&property.value_type)
        };
        properties.insert(
            property.id,
            DynamicProperty::new(property.id, property.value_type.clone(), value),
        );
    }
    Ok(ComponentInstance::new(
        id,
        component,
        properties.into_values(),
    ))
}

/// Provides a total initial value for an unset property of `value_type`.
///
/// Returns the type's neutral value until any contextual authored default is
/// evaluated during rendering.
fn default_value(value_type: &IrType) -> DslValue {
    match value_type {
        IrType::Bool => DslValue::Bool(false),
        IrType::Int => DslValue::Int(0),
        IrType::Float
        | IrType::Length
        | IrType::Dimension
        | IrType::Percentage
        | IrType::Duration
        | IrType::Angle
        | IrType::FontSize
        | IrType::LineHeight => DslValue::Float(0.0),
        IrType::String | IrType::FontFamily | IrType::FontWeight => DslValue::String(String::new()),
        IrType::Color => DslValue::Color(argui_core::Color::TRANSPARENT),
        IrType::Struct { .. } => DslValue::Struct(BTreeMap::new()),
        IrType::Enum(symbol) => DslValue::Enum {
            symbol: symbol.raw(),
            variant: 0,
        },
        IrType::Array(_) | IrType::Model(_) => DslValue::Array(Vec::new()),
        IrType::Optional(_) | IrType::Void | IrType::Unknown => DslValue::Null,
        IrType::Asset => DslValue::Asset(argui_dsl_ir::AssetId::from_raw(0)),
        _ => DslValue::Null,
    }
}
