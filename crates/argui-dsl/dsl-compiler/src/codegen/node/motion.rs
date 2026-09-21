//! Code generation for typed native-element animation specifications.

use std::fmt::Write;

use argui_dsl_ir::{
    IrAnimation, IrAnimationDriver, IrPropertyBinding, IrTransitionPolicy, IrType,
    PropertyTargetId, SiteId,
};

use crate::{
    CompilerError,
    codegen::{Context, expression::Scope, node::state::StateSelection},
};

/// Generated endpoint and driver expressions for one animation declaration.
#[derive(Default)]
struct AnimationParameters {
    from: Option<(String, IrType)>,
    to: Option<(String, IrType)>,
    duration: Option<String>,
    iterations: Option<String>,
    stiffness: Option<String>,
    damping: Option<String>,
    easing: Option<String>,
}

impl Context<'_> {
    /// Emits animated native properties that have no ordinary property assignment.
    ///
    /// * `output` — generated renderer source.
    /// * `native_schema` — target primitive's canonical properties.
    /// * `site` — stable element source site.
    /// * `bindings` — explicit assignments already emitted.
    /// * `input` — generated native input variable.
    /// * `pad` — indentation for the generated block.
    /// * `scope` — animation and expression bindings.
    /// * `identity` — generated retained identity variable.
    ///
    /// # Errors
    ///
    /// Returns when an unbound animation has no target endpoint or conversion fails.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_unbound_native_animations(
        &self,
        output: &mut String,
        native_schema: &argui_schema::NativeSchema,
        site: SiteId,
        bindings: &[IrPropertyBinding],
        input: &str,
        pad: &str,
        scope: &Scope,
        identity: &str,
    ) -> Result<(), CompilerError> {
        for native_property in &native_schema.properties {
            if native_schema.id == argui_schema::builtin::VIRTUAL_LIST
                && native_property.id == argui_schema::builtin::VIEWPORT_HEIGHT
            {
                continue;
            }
            let target = PropertyTargetId::Native(native_property.id);
            if bindings.iter().any(|binding| binding.target == target) {
                continue;
            }
            let animation = scope.animations.get(&(site, target));
            let has_state = scope.states.get(&site).is_some_and(|states| {
                states.iter().any(|state| {
                    state
                        .assignments
                        .iter()
                        .any(|(assigned, _)| *assigned == target)
                })
            });
            if animation.is_none() && !has_state {
                continue;
            }
            let endpoint = animation.and_then(|animation| {
                animation
                    .parameters
                    .iter()
                    .find(|parameter| parameter.name == "to")
                    .map(|parameter| &parameter.value)
                    .or_else(|| animation.keyframes.last().map(|frame| &frame.value))
            });
            let (base, base_type) = if let Some(endpoint) = endpoint {
                (
                    self.expression(endpoint, scope)?,
                    endpoint.value_type.clone(),
                )
            } else {
                self.native_default_expression(native_property)?
            };
            let selected = self.select_state_value(
                site,
                target,
                base,
                &base_type,
                &IrType::from_schema(native_property.value_type),
                scope,
            )?;
            let sampled = self.animated_expression(site, target, &selected, scope, identity)?;
            let value_type =
                animation.map_or(&selected.value_type, |animation| &animation.value_type);
            let value = Self::schema_value_expression(value_type, &sampled)?;
            writeln!(output, "{pad}{input} = {input}.property(::argui::schema::PropertyId::from_raw({}), {value});", native_property.id.raw()).unwrap();
        }
        Ok(())
    }

    /// Samples an animable property expression through the retained motion store.
    ///
    /// * `site` — source site of the native or user component element.
    /// * `property` — resolved property destination on that element.
    /// * `selection` — canonical and state-selected target expressions.
    /// * `scope` — available bindings and animation metadata.
    /// * `identity` — generated retained identity expression.
    ///
    /// # Errors
    ///
    /// Returns when the property type has no interpolation backend or parameters are malformed.
    pub(super) fn animated_expression(
        &self,
        site: SiteId,
        property: PropertyTargetId,
        selection: &StateSelection,
        scope: &Scope,
        identity: &str,
    ) -> Result<String, CompilerError> {
        let Some(animation) = scope.animations.get(&(site, property)) else {
            return Ok(selection.effective.clone());
        };
        self.animated_decl_expression(animation, property, selection, scope, identity)
    }

    /// Samples one already-resolved animation, including a component's own property.
    ///
    /// * `animation` — typed animation declaration.
    /// * `property` — target property identity.
    /// * `selection` — canonical and state-selected target expressions.
    /// * `scope` — pre-animation expression scope.
    /// * `identity` — retained element or component identity.
    ///
    /// # Errors
    ///
    /// Returns for unsupported types or malformed driver parameters.
    pub(in crate::codegen) fn animated_decl_expression(
        &self,
        animation: &IrAnimation,
        property: PropertyTargetId,
        selection: &StateSelection,
        scope: &Scope,
        identity: &str,
    ) -> Result<String, CompilerError> {
        let property_id = match property {
            PropertyTargetId::Native(id) => u64::from(id.raw()),
            PropertyTargetId::Component(id) => id.raw(),
        };
        let key = format!(
            "::argui::schema::PropertyMotionKey::new({identity}.clone(), {property_id}, {})",
            animation.id.raw()
        );
        let AnimationParameters {
            from,
            to,
            duration,
            iterations,
            stiffness,
            damping,
            easing,
        } = self.animation_parameters(animation, scope)?;
        let base = &selection.effective;
        let (method, target, from, to, suffix, canonical) = match &animation.value_type {
            IrType::Int => (
                "sample_number",
                format!("({base}) as f64"),
                from.map(|(value, _)| format!("({value}) as f64")),
                to.map(|(value, _)| format!("({value}) as f64")),
                " as i64",
                format!("({}) as f64", selection.base),
            ),
            IrType::Float | IrType::Length | IrType::Percentage | IrType::Angle => (
                "sample_number",
                format!("({base}) as f64"),
                from.map(|(value, _)| format!("({value}) as f64")),
                to.map(|(value, _)| format!("({value}) as f64")),
                " as f32",
                format!("({}) as f64", selection.base),
            ),
            IrType::Color => (
                "sample_color",
                base.clone(),
                from.map(|(value, _)| value),
                to.map(|(value, _)| value),
                "",
                selection.base.clone(),
            ),
            IrType::Dimension => (
                "sample_dimension",
                self.dimension_expression(base, &selection.value_type)?,
                from.map(|(value, value_type)| self.dimension_expression(&value, &value_type))
                    .transpose()?,
                to.map(|(value, value_type)| self.dimension_expression(&value, &value_type))
                    .transpose()?,
                "",
                self.dimension_expression(&selection.base, &selection.value_type)?,
            ),
            unsupported => {
                return Err(CompilerError::Codegen(format!(
                    "animation property type `{unsupported:?}` is not interpolable"
                )));
            }
        };
        let from = from.map_or_else(|| "None".into(), |value| format!("Some({value})"));
        let to = to.map_or_else(|| "None".into(), |value| format!("Some({value})"));
        if !animation.keyframes.is_empty() && (from != "None" || to != "None") {
            return Err(CompilerError::Codegen(
                "keyframes cannot be mixed with `from` or `to`".into(),
            ));
        }
        let mut specification = match animation.driver {
            IrAnimationDriver::Timeline => {
                let duration = duration.ok_or_else(|| {
                    CompilerError::Codegen("timeline animation requires `duration`".into())
                })?;
                let iterations = iterations.unwrap_or_else(|| "String::from(\"once\")".into());
                if animation.keyframes.is_empty() {
                    format!(
                        "::argui::schema::PropertyAnimation::new({from}, {to}, ({duration}) as f64, &({iterations}))"
                    )
                } else {
                    let frames = self.animation_frames(animation, scope)?;
                    format!(
                        "::argui::schema::PropertyAnimation::keyframes(vec![{frames}], ({duration}) as f64, &({iterations}))"
                    )
                }
            }
            IrAnimationDriver::Spring => {
                let stiffness = stiffness.unwrap_or_else(|| "170.0_f32".into());
                let damping = damping.unwrap_or_else(|| "26.0_f32".into());
                format!(
                    "::argui::schema::PropertyAnimation::spring({from}, {to}, ({stiffness}) as f64, ({damping}) as f64)"
                )
            }
        };
        if let Some(easing) = easing {
            specification = format!(
                "({specification}).and_then(|specification| specification.with_easing_name(&({easing})))"
            );
        }
        if let Some(policy) = animation.transition {
            let policy = match policy {
                IrTransitionPolicy::Enter => "Enter",
                IrTransitionPolicy::Leave => "Leave",
                IrTransitionPolicy::InOut => "InOut",
            };
            let active = selection.active.as_deref().ok_or_else(|| {
                CompilerError::Codegen("state transition has no state assignment".into())
            })?;
            specification = format!(
                "({specification}).and_then(|specification| specification.with_state_transition(::argui::schema::StateTransitionPolicy::{policy}, {active}, {canonical}))"
            );
        }
        Ok(format!(
            "{{ let target = {target}; let sampled = ({specification}).and_then(|specification| property_motions.{method}({key}, target, specification, reduced_motion)); match sampled {{ Ok(value) => value, Err(error) => {{ property_motions.report_error(error); target }} }} }}{suffix}"
        ))
    }

    /// Emits typed timeline stops in source order for one animated property.
    ///
    /// * `animation` — resolved animation and its typed stop expressions.
    /// * `scope` — values available to the stop expressions.
    ///
    /// # Errors
    ///
    /// Returns for unsupported property types or expression generation failures.
    fn animation_frames(
        &self,
        animation: &IrAnimation,
        scope: &Scope,
    ) -> Result<String, CompilerError> {
        let mut frames = Vec::new();
        for frame in &animation.keyframes {
            let value = self.expression(&frame.value, scope)?;
            let value = match &animation.value_type {
                IrType::Int
                | IrType::Float
                | IrType::Length
                | IrType::Percentage
                | IrType::Angle => format!("({value}) as f64"),
                IrType::Color => value,
                IrType::Dimension => self.dimension_expression(&value, &frame.value.value_type)?,
                unsupported => {
                    return Err(CompilerError::Codegen(format!(
                        "keyframe type `{unsupported:?}` is not interpolable"
                    )));
                }
            };
            frames.push(format!("({}_f32, {value})", frame.offset));
        }
        Ok(frames.join(", "))
    }

    /// Emits raw driver expressions with their resolved types for later conversion.
    ///
    /// * `animation` — lowered declaration with named parameters.
    /// * `scope` — generated values available at this site.
    ///
    /// # Errors
    ///
    /// Returns for unknown parameter names or expression generation failures.
    fn animation_parameters(
        &self,
        animation: &IrAnimation,
        scope: &Scope,
    ) -> Result<AnimationParameters, CompilerError> {
        let mut values = AnimationParameters::default();
        for parameter in &animation.parameters {
            let expression = self.expression(&parameter.value, scope)?;
            match parameter.name.as_str() {
                "from" => values.from = Some((expression, parameter.value.value_type.clone())),
                "to" => values.to = Some((expression, parameter.value.value_type.clone())),
                "duration" => values.duration = Some(expression),
                "iterations" => values.iterations = Some(expression),
                "stiffness" => values.stiffness = Some(expression),
                "damping" => values.damping = Some(expression),
                "easing" => values.easing = Some(expression),
                name => {
                    return Err(CompilerError::Codegen(format!(
                        "unsupported animation parameter `{name}`"
                    )));
                }
            }
        }
        Ok(values)
    }

    /// Converts a generated length or percentage into the native dimension type.
    ///
    /// * `expression` — generated expression.
    /// * `value_type` — resolved DSL expression type.
    ///
    /// # Errors
    ///
    /// Returns when a non-dimensional value is supplied.
    pub(super) fn dimension_expression(
        &self,
        expression: &str,
        value_type: &IrType,
    ) -> Result<String, CompilerError> {
        match value_type {
            IrType::Length => Ok(format!("::argui::ui::length({expression})")),
            IrType::Percentage => Ok(format!("::argui::ui::percent(({expression}) / 100.0_f32)")),
            IrType::Dimension => Ok(expression.into()),
            other => Err(CompilerError::Codegen(format!(
                "`{other:?}` cannot animate a dimension"
            ))),
        }
    }
}
