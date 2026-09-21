//! Evaluation of declarative visual states before property animation.

use std::collections::HashMap;

use argui_dsl_ir::{IrComponent, IrType, LocalId, PropertyTargetId, SiteId};

use crate::{ComponentInstance, DslValue, LiveRuntime, RuntimeError};

/// Canonical base and current effective target of one stateful property.
pub(super) struct StateSelection {
    pub base: DslValue,
    pub effective: DslValue,
    pub value_type: IrType,
    pub base_type: IrType,
    pub active: bool,
    pub has_states: bool,
}

impl LiveRuntime {
    /// Converts a canonical native schema default to its live DSL value.
    ///
    /// * `property` — metadata for an unbound stateful native property.
    ///
    /// # Errors
    ///
    /// Returns if the native has no representable declarative default.
    pub(super) fn native_default_value(
        property: &argui_schema::PropertySchema,
    ) -> Result<(DslValue, IrType), RuntimeError> {
        use argui_schema::SchemaValue;
        match &property.default {
            Some(SchemaValue::Bool(value)) => Ok((DslValue::Bool(*value), IrType::Bool)),
            Some(SchemaValue::Int(value)) => Ok((DslValue::Int(*value), IrType::Int)),
            Some(SchemaValue::Float(value)) => {
                Ok((DslValue::Float(f64::from(*value)), IrType::Float))
            }
            Some(SchemaValue::String(value)) => {
                Ok((DslValue::String(value.clone()), IrType::String))
            }
            _ => Err(RuntimeError::Schema(format!(
                "unbound stateful property `{}` requires an explicit base or representable schema default",
                property.name
            ))),
        }
    }

    /// Evaluates ordered state assignments for one component or element property.
    ///
    /// * `instance` — owner instance for condition and replacement expressions.
    /// * `component` — definition containing states.
    /// * `owner` — element site, or `None` for a component's own properties.
    /// * `property` — resolved assignment target.
    /// * `base` — canonical declarative value before states.
    /// * `base_type` — resolved type of that declarative value.
    /// * `locals` — repeater variables visible to conditions.
    ///
    /// # Errors
    ///
    /// Returns for invalid condition type or expression evaluation failure.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn select_state_value(
        &mut self,
        instance: &mut ComponentInstance,
        component: &IrComponent,
        owner: Option<SiteId>,
        property: PropertyTargetId,
        base: DslValue,
        base_type: &IrType,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<StateSelection, RuntimeError> {
        let mut selection = StateSelection {
            base: base.clone(),
            effective: base,
            value_type: base_type.clone(),
            base_type: base_type.clone(),
            active: false,
            has_states: false,
        };
        for state in component.states.iter().filter(|state| state.owner == owner) {
            let Some((_, replacement)) = state
                .assignments
                .iter()
                .find(|(target, _)| *target == property)
            else {
                continue;
            };
            selection.has_states = true;
            match self.evaluate(instance, &state.condition, locals)? {
                DslValue::Bool(true) => {
                    selection.effective = self.evaluate(instance, replacement, locals)?;
                    selection.value_type = replacement.value_type.clone();
                    selection.active = true;
                }
                DslValue::Bool(false) => {}
                other => {
                    return Err(RuntimeError::TypeMismatch {
                        expected: "bool".into(),
                        actual: other.type_name().into(),
                    });
                }
            }
        }
        Ok(selection)
    }
}
