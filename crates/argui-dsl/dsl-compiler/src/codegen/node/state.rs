//! Typed visual-state selection shared by native and component code generation.

use argui_dsl_ir::{IrState, IrType, PropertyTargetId, SiteId};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope},
};

/// Declarative base and effective value selected by applicable visual states.
pub(in crate::codegen) struct StateSelection {
    pub base: String,
    pub effective: String,
    pub value_type: IrType,
    pub active: Option<String>,
}

impl Context<'_> {
    /// Converts a canonical native schema default into a generated typed value.
    ///
    /// * `property` — native metadata carrying the default.
    ///
    /// # Errors
    ///
    /// Returns when a stateful unbound property lacks a representable default.
    pub(super) fn native_default_expression(
        &self,
        property: &argui_schema::PropertySchema,
    ) -> Result<(String, IrType), CompilerError> {
        use argui_schema::SchemaValue;
        match &property.default {
            Some(SchemaValue::Bool(value)) => Ok((value.to_string(), IrType::Bool)),
            Some(SchemaValue::Int(value)) => Ok((format!("{value}_i64"), IrType::Int)),
            Some(SchemaValue::Float(value)) => Ok((format!("{value}_f32"), IrType::Float)),
            Some(SchemaValue::String(value)) => {
                Ok((format!("String::from({value:?})"), IrType::String))
            }
            _ => Err(CompilerError::Codegen(format!(
                "unbound stateful property `{}` requires an explicit base or representable schema default",
                property.name
            ))),
        }
    }

    /// Applies same-owner state assignments in declaration order to one property.
    ///
    /// * `site` — native or child component source site.
    /// * `property` — resolved destination property.
    /// * `base` — generated declarative expression before state overrides.
    /// * `base_type` — type of the base expression.
    /// * `expected_type` — destination property's resolved type.
    /// * `scope` — available component and state expression bindings.
    ///
    /// # Errors
    ///
    /// Returns when a state expression cannot be generated or dimension units are invalid.
    pub(super) fn select_state_value(
        &self,
        site: SiteId,
        property: PropertyTargetId,
        base: String,
        base_type: &IrType,
        expected_type: &IrType,
        scope: &Scope,
    ) -> Result<StateSelection, CompilerError> {
        self.select_state_assignments(
            scope.states.get(&site).map(Vec::as_slice).unwrap_or(&[]),
            property,
            base,
            base_type,
            expected_type,
            scope,
        )
    }

    /// Applies component-level states to an exposed or private property.
    ///
    /// * `property` — property declared by the current component.
    /// * `base` — canonical property expression.
    /// * `value_type` — resolved property type.
    /// * `scope` — current component state and expression bindings.
    ///
    /// # Errors
    ///
    /// Returns when a state condition or replacement expression cannot be generated.
    pub(in crate::codegen) fn select_own_state_value(
        &self,
        property: PropertyTargetId,
        base: String,
        value_type: &IrType,
        scope: &Scope,
    ) -> Result<StateSelection, CompilerError> {
        self.select_state_assignments(
            &scope.component_states,
            property,
            base,
            value_type,
            value_type,
            scope,
        )
    }

    /// Evaluates one ordered list of states for a property target.
    ///
    /// * `states` — states sharing this owner.
    /// * `property` — resolved target property.
    /// * `base` — canonical property expression.
    /// * `base_type` — type of the canonical expression.
    /// * `expected_type` — target property type.
    /// * `scope` — expression bindings.
    ///
    /// # Errors
    ///
    /// Returns for invalid expression or dimension conversion.
    #[allow(clippy::too_many_arguments)]
    fn select_state_assignments(
        &self,
        states: &[IrState],
        property: PropertyTargetId,
        base: String,
        base_type: &IrType,
        expected_type: &IrType,
        scope: &Scope,
    ) -> Result<StateSelection, CompilerError> {
        let mut base = base;
        let mut value_type = base_type.clone();
        if *expected_type == IrType::Dimension {
            base = self.dimension_expression(&base, base_type)?;
            value_type = IrType::Dimension;
        }
        let mut effective = base.clone();
        let mut active = Vec::new();
        for state in states {
            let Some((_, replacement)) = state
                .assignments
                .iter()
                .find(|(target, _)| *target == property)
            else {
                continue;
            };
            let condition = self.expression(&state.condition, scope)?;
            let mut value = self.expression(replacement, scope)?;
            if *expected_type == IrType::Dimension {
                value = self.dimension_expression(&value, &replacement.value_type)?;
            }
            effective = format!("if {condition} {{ {value} }} else {{ {effective} }}");
            active.push(format!("({condition})"));
        }
        Ok(StateSelection {
            base,
            effective,
            value_type,
            active: (!active.is_empty()).then(|| active.join(" || ")),
        })
    }
}
