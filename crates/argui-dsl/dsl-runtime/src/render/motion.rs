//! Evaluation of retained animation clauses during live rendering.

use std::collections::HashMap;

use argui_dsl_ir::{
    IrAnimation, IrAnimationDriver, IrTransitionPolicy, IrType, LocalId, PropertyTargetId, SiteId,
};

use super::state::StateSelection;
use crate::{ComponentInstance, DslValue, LiveRuntime, RuntimeError};

struct DriverValues {
    from: Option<(DslValue, IrType)>,
    to: Option<(DslValue, IrType)>,
    duration_ms: Option<f64>,
    iterations: String,
    stiffness: f64,
    damping: f64,
    easing: Option<String>,
    keyframes: Vec<(f32, DslValue, IrType)>,
}

impl DriverValues {
    /// Converts typed endpoints and constructs the selected engine driver.
    ///
    /// * `driver` — timeline or spring selected by the IR.
    /// * `convert` — type-specific endpoint conversion for the target property.
    ///
    /// # Errors
    ///
    /// Returns for incompatible endpoints or invalid timing/physics values.
    fn specification<T>(
        self,
        driver: IrAnimationDriver,
        mut convert: impl FnMut(&str, DslValue, &IrType) -> Result<T, RuntimeError>,
    ) -> Result<argui_schema::PropertyAnimation<T>, RuntimeError> {
        let from = self
            .from
            .map(|(value, value_type)| convert("from", value, &value_type))
            .transpose()?;
        let to = self
            .to
            .map(|(value, value_type)| convert("to", value, &value_type))
            .transpose()?;
        let frames = self
            .keyframes
            .into_iter()
            .map(|(offset, value, value_type)| {
                convert("keyframe", value, &value_type).map(|value| (offset, value))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let mut specification = match driver {
            IrAnimationDriver::Timeline if !frames.is_empty() => {
                if from.is_some() || to.is_some() {
                    return Err(RuntimeError::Schema(
                        "keyframes cannot be mixed with `from` or `to`".into(),
                    ));
                }
                argui_schema::PropertyAnimation::keyframes(
                    frames,
                    self.duration_ms.ok_or_else(|| {
                        RuntimeError::Schema("timeline animation requires `duration`".into())
                    })?,
                    &self.iterations,
                )
                .map_err(RuntimeError::Schema)?
            }
            IrAnimationDriver::Timeline => argui_schema::PropertyAnimation::new(
                from,
                to,
                self.duration_ms.ok_or_else(|| {
                    RuntimeError::Schema("timeline animation requires `duration`".into())
                })?,
                &self.iterations,
            )
            .map_err(RuntimeError::Schema)?,
            IrAnimationDriver::Spring => {
                if !frames.is_empty() {
                    return Err(RuntimeError::Schema(
                        "spring animations do not accept keyframes".into(),
                    ));
                }
                argui_schema::PropertyAnimation::spring(from, to, self.stiffness, self.damping)
                    .map_err(RuntimeError::Schema)?
            }
        };
        if let Some(easing) = self.easing {
            specification = specification
                .with_easing_name(&easing)
                .map_err(RuntimeError::Schema)?;
        }
        Ok(specification)
    }
}

impl LiveRuntime {
    /// Samples an animated native property before its adapter constructs the element.
    ///
    /// * `instance` — owner of the property expression.
    /// * `component` — owner definition containing the animation clause.
    /// * `site` — element source site.
    /// * `property` — resolved native property ID.
    /// * `value` — declarative target after schema conversion.
    /// * `selection` — canonical base and current state activity.
    /// * `locals` — active repeater variables.
    /// * `identity` — retained instance/site/repeater identity.
    /// * `reduced_motion` — environment accessibility preference.
    ///
    /// # Errors
    ///
    /// Returns for malformed timing, types, or unsupported dimension units.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn animate_native_value(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        site: SiteId,
        property: PropertyTargetId,
        value: argui_schema::SchemaValue,
        selection: &StateSelection,
        locals: &HashMap<LocalId, DslValue>,
        identity: argui_ui::RetainedIdentity,
        reduced_motion: bool,
    ) -> Result<argui_schema::SchemaValue, RuntimeError> {
        let Some(animation) = component
            .animations
            .iter()
            .find(|animation| animation.owner == Some(site) && animation.property == property)
        else {
            return Ok(value);
        };
        let driver = self.evaluate_driver(instance, animation, locals)?;
        let key = motion_key(identity, property, animation);
        match value {
            argui_schema::SchemaValue::Float(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_number(name, value)
                    })?,
                    |value, _| animation_number("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_number(key, f64::from(target), specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(argui_schema::SchemaValue::Float(sampled as f32))
            }
            argui_schema::SchemaValue::Int(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_number(name, value)
                    })?,
                    |value, _| animation_number("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_number(key, target as f64, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(argui_schema::SchemaValue::Int(sampled.round() as i64))
            }
            argui_schema::SchemaValue::Color(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_color(name, value)
                    })?,
                    |value, _| animation_color("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_color(key, target, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(argui_schema::SchemaValue::Color(sampled))
            }
            argui_schema::SchemaValue::Dimension(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, animation_dimension)?,
                    |value, value_type| animation_dimension("base", value, value_type),
                )?;
                let sampled = self
                    .property_motions
                    .sample_dimension(key, target, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(argui_schema::SchemaValue::Dimension(sampled))
            }
            other => Err(RuntimeError::Schema(format!(
                "property `{property:?}` with type {:?} is not interpolable",
                other.value_type()
            ))),
        }
    }

    /// Samples a user-component property before the child instance receives it.
    ///
    /// * `instance` — parent component state.
    /// * `component` — parent definition containing the clause.
    /// * `site` — child component source site.
    /// * `property` — resolved child property ID.
    /// * `selection` — declarative child input and state activity.
    /// * `locals` — repeater variables.
    /// * `identity` — retained child component identity.
    /// * `reduced_motion` — accessibility preference.
    ///
    /// # Errors
    ///
    /// Returns for malformed driver values or non-interpolable inputs.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn animate_component_value(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        site: SiteId,
        property: PropertyTargetId,
        selection: &StateSelection,
        locals: &HashMap<LocalId, DslValue>,
        identity: argui_ui::RetainedIdentity,
        reduced_motion: bool,
    ) -> Result<DslValue, RuntimeError> {
        let Some(animation) = component
            .animations
            .iter()
            .find(|animation| animation.owner == Some(site) && animation.property == property)
        else {
            return Ok(selection.effective.clone());
        };
        self.sample_component_animation(
            instance,
            animation,
            property,
            selection,
            locals,
            identity,
            reduced_motion,
        )
    }

    /// Samples the component's own animated properties into render-only values.
    ///
    /// * `instance` — mounted component with canonical inputs.
    /// * `component` — definition containing component-level animation clauses.
    /// * `context` — optional host context for reduced-motion preference.
    ///
    /// # Errors
    ///
    /// Returns if a target is missing or a driver value is incompatible.
    pub(super) fn apply_own_animations(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        context: &Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<(), RuntimeError> {
        let reduced_motion = context
            .as_deref()
            .is_some_and(|host| host.environment().reduced_motion);
        for property in &component.properties {
            let target = PropertyTargetId::Component(property.id);
            let animation = component
                .animations
                .iter()
                .find(|animation| animation.owner.is_none() && animation.property == target);
            let has_state = component.states.iter().any(|state| {
                state.owner.is_none()
                    && state
                        .assignments
                        .iter()
                        .any(|(assigned, _)| *assigned == target)
            });
            if animation.is_none() && !has_state {
                continue;
            }
            let base = instance
                .properties
                .get(&property.id)
                .ok_or(RuntimeError::MissingProperty(property.id.raw()))?
                .get()
                .clone();
            let selected = self.select_state_value(
                instance,
                component,
                None,
                target,
                base,
                &property.value_type,
                &HashMap::new(),
            )?;
            let sampled = if let Some(animation) = animation {
                let identity =
                    argui_ui::RetainedIdentity::new(instance.id.raw(), component.id.raw());
                self.sample_component_animation(
                    instance,
                    animation,
                    target,
                    &selected,
                    &HashMap::new(),
                    identity,
                    reduced_motion,
                )?
            } else {
                selected.effective
            };
            instance.presented_properties.insert(property.id, sampled);
        }
        Ok(())
    }

    /// Samples an already-resolved animation targeting a user component property.
    ///
    /// * `instance` — source instance for dynamic driver expressions.
    /// * `animation` — resolved clause.
    /// * `property` — stable property target.
    /// * `selection` — declarative target and state activity.
    /// * `locals` — active repeater values.
    /// * `identity` — retained owner identity.
    /// * `reduced_motion` — accessibility preference.
    ///
    /// # Errors
    ///
    /// Returns for invalid driver values or non-interpolable input.
    #[allow(clippy::too_many_arguments)]
    fn sample_component_animation(
        &mut self,
        instance: &mut ComponentInstance,
        animation: &IrAnimation,
        property: PropertyTargetId,
        selection: &StateSelection,
        locals: &HashMap<LocalId, DslValue>,
        identity: argui_ui::RetainedIdentity,
        reduced_motion: bool,
    ) -> Result<DslValue, RuntimeError> {
        let driver = self.evaluate_driver(instance, animation, locals)?;
        let key = motion_key(identity, property, animation);
        match selection.effective.clone() {
            DslValue::Float(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_number(name, value)
                    })?,
                    |value, _| animation_number("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_number(key, target, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(DslValue::Float(sampled))
            }
            DslValue::Int(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_number(name, value)
                    })?,
                    |value, _| animation_number("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_number(key, target as f64, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(DslValue::Int(sampled.round() as i64))
            }
            DslValue::Color(target) => {
                let specification = state_specification(
                    animation,
                    selection,
                    driver.specification(animation.driver, |name, value, _| {
                        animation_color(name, value)
                    })?,
                    |value, _| animation_color("base", value),
                )?;
                let sampled = self
                    .property_motions
                    .sample_color(key, target, specification, reduced_motion)
                    .map_err(RuntimeError::Schema)?;
                Ok(DslValue::Color(sampled))
            }
            other => Err(RuntimeError::Schema(format!(
                "component property `{property:?}` with type {} is not interpolable",
                other.type_name()
            ))),
        }
    }

    /// Evaluates one typed animation driver without touching motion state.
    ///
    /// * `instance` — component state for dynamic parameter expressions.
    /// * `animation` — resolved target and named driver parameters.
    /// * `locals` — active repeater variables.
    ///
    /// # Errors
    ///
    /// Returns for unknown parameters or invalid/missing duration.
    fn evaluate_driver(
        &mut self,
        instance: &mut ComponentInstance,
        animation: &IrAnimation,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<DriverValues, RuntimeError> {
        let (mut from, mut to, mut duration_ms) = (None, None, None);
        let mut iterations = String::from("once");
        let (mut stiffness, mut damping, mut easing) = (170.0, 26.0, None);
        for parameter in &animation.parameters {
            let value = self.evaluate(instance, &parameter.value, locals)?;
            match parameter.name.as_str() {
                "from" => from = Some((value, parameter.value.value_type.clone())),
                "to" => to = Some((value, parameter.value.value_type.clone())),
                "duration" => duration_ms = Some(animation_number("duration", value)?),
                "iterations" => match value {
                    DslValue::String(value) => iterations = value,
                    other => {
                        return Err(RuntimeError::Schema(format!(
                            "animation `iterations` must be a string, found {}",
                            other.type_name()
                        )));
                    }
                },
                "stiffness" => stiffness = animation_number("stiffness", value)?,
                "damping" => damping = animation_number("damping", value)?,
                "easing" => match value {
                    DslValue::String(value) => easing = Some(value),
                    other => {
                        return Err(RuntimeError::Schema(format!(
                            "animation `easing` must be a string, found {}",
                            other.type_name()
                        )));
                    }
                },
                name => {
                    return Err(RuntimeError::Schema(format!(
                        "unsupported animation parameter `{name}`"
                    )));
                }
            }
        }
        let keyframes = animation
            .keyframes
            .iter()
            .map(|frame| {
                self.evaluate(instance, &frame.value, locals)
                    .map(|value| (frame.offset, value, frame.value.value_type.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        Ok(DriverValues {
            from,
            to,
            duration_ms,
            iterations,
            stiffness,
            damping,
            easing,
            keyframes,
        })
    }
}

/// Converts a typed animation parameter to the numeric value expected by the driver.
///
/// * `name` — parameter name used in diagnostics.
/// * `value` — evaluated DSL parameter.
///
/// # Errors
///
/// Returns when the parameter is not numeric.
fn animation_number(name: &str, value: DslValue) -> Result<f64, RuntimeError> {
    match value {
        DslValue::Float(value) => Ok(value),
        DslValue::Int(value) => Ok(value as f64),
        other => Err(RuntimeError::Schema(format!(
            "animation `{name}` must be numeric, found {}",
            other.type_name()
        ))),
    }
}

/// Reads a color animation endpoint with a parameter-specific diagnostic.
///
/// * `name` — driver parameter name.
/// * `value` — evaluated endpoint.
///
/// # Errors
///
/// Returns when the endpoint is not a color.
fn animation_color(name: &str, value: DslValue) -> Result<argui_core::Color, RuntimeError> {
    match value {
        DslValue::Color(value) => Ok(value),
        other => Err(RuntimeError::Schema(format!(
            "animation `{name}` must be a color, found {}",
            other.type_name()
        ))),
    }
}

/// Converts a typed length or percentage endpoint to a native dimension.
///
/// * `name` — driver parameter name.
/// * `value` — evaluated numeric endpoint.
/// * `value_type` — unit-bearing IR type of the endpoint.
///
/// # Errors
///
/// Returns for non-numeric or non-dimensional values.
fn animation_dimension(
    name: &str,
    value: DslValue,
    value_type: &IrType,
) -> Result<argui_ui::Dimension, RuntimeError> {
    let value = animation_number(name, value)? as f32;
    match value_type {
        IrType::Length => Ok(argui_ui::length(value)),
        IrType::Percentage => Ok(argui_ui::percent(value / 100.0)),
        other => Err(RuntimeError::Schema(format!(
            "animation `{name}` cannot convert `{other:?}` to a dimension"
        ))),
    }
}

/// Attaches a named state's edge and canonical base to any typed animation.
///
/// * `animation` — declaration carrying the optional transition policy.
/// * `selection` — selected value, canonical base, and state activity.
/// * `specification` — validated tween or spring driver.
/// * `convert` — conversion of the canonical DSL value to the driver's type.
///
/// # Errors
///
/// Returns if the state is absent or its base cannot be interpolated.
fn state_specification<T>(
    animation: &IrAnimation,
    selection: &StateSelection,
    specification: argui_schema::PropertyAnimation<T>,
    convert: impl FnOnce(DslValue, &IrType) -> Result<T, RuntimeError>,
) -> Result<argui_schema::PropertyAnimation<T>, RuntimeError> {
    let Some(policy) = animation.transition else {
        return Ok(specification);
    };
    if !selection.has_states {
        return Err(RuntimeError::Schema(
            "state transition has no state assignment".into(),
        ));
    }
    let policy = match policy {
        IrTransitionPolicy::Enter => argui_schema::StateTransitionPolicy::Enter,
        IrTransitionPolicy::Leave => argui_schema::StateTransitionPolicy::Leave,
        IrTransitionPolicy::InOut => argui_schema::StateTransitionPolicy::InOut,
    };
    let base = convert(selection.base.clone(), &selection.base_type)?;
    specification
        .with_state_transition(policy, selection.active, base)
        .map_err(RuntimeError::Schema)
}

/// Constructs a retained motion key from DSL property and animation identities.
///
/// * `identity` — retained instance/site/repeater key.
/// * `property` — resolved native or component property ID.
/// * `animation` — stable animation declaration.
fn motion_key(
    identity: argui_ui::RetainedIdentity,
    property: PropertyTargetId,
    animation: &IrAnimation,
) -> argui_schema::PropertyMotionKey {
    let property = match property {
        PropertyTargetId::Native(id) => u64::from(id.raw()),
        PropertyTargetId::Component(id) => id.raw(),
    };
    argui_schema::PropertyMotionKey::new(identity, property, animation.id.raw())
}
