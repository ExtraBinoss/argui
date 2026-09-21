//! Expression evaluation and source-site event lookup for live rendering.

use std::collections::HashMap;

use argui_dsl_ir::{EventTargetId, IrEventBinding, IrExpression, IrNode, LocalId, SiteId};

use crate::{
    ComponentInstance, DslValue, InstanceId, LiveRuntime, RuntimeError, instance::EvaluationStamp,
    runtime::InstanceValueContext,
};

impl LiveRuntime {
    /// Dispatches a resolved native event block without source-level name lookup.
    ///
    /// This is also the host integration boundary used by native schema event adapters.
    pub fn dispatch_native_event(
        &mut self,
        instance: InstanceId,
        site: SiteId,
        event: argui_schema::EventId,
    ) -> Result<DslValue, RuntimeError> {
        let component = self
            .instances
            .get(&instance)
            .map(|instance| instance.component)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let definition = self
            .package
            .ir
            .components
            .iter()
            .find(|definition| definition.id == component)
            .ok_or(RuntimeError::MissingComponent(component.raw()))?;
        let binding = find_native_event(&definition.body, site, event)
            .cloned()
            .ok_or_else(|| {
                RuntimeError::Schema(format!(
                    "event {} is not bound at site {}",
                    event.raw(),
                    site.raw()
                ))
            })?;
        self.execute_statements(instance, &binding.statements, HashMap::new())
    }

    /// Evaluates one already-compiled expression for a component instance.
    pub(super) fn evaluate(
        &self,
        instance: &mut ComponentInstance,
        expression: &IrExpression,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        self.evaluate_with_presentation(instance, expression, locals, true)
    }

    /// Evaluates a child's canonical input without parent presentation overlays.
    ///
    /// * `instance` — parent with retained canonical properties.
    /// * `expression` — compiled child-binding expression.
    /// * `locals` — active repeater values.
    ///
    /// # Errors
    ///
    /// Returns expression lookup or evaluation failures.
    pub(super) fn evaluate_canonical(
        &self,
        instance: &mut ComponentInstance,
        expression: &IrExpression,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        self.evaluate_with_presentation(instance, expression, locals, false)
    }

    /// Evaluates one expression in either canonical or render-only mode.
    ///
    /// * `instance` — mounted component carrying property values.
    /// * `expression` — compiled expression identity.
    /// * `locals` — active repeater values.
    /// * `presentation` — whether animated/state overlays are visible.
    ///
    /// # Errors
    ///
    /// Returns missing-expression/property or bytecode evaluation errors.
    fn evaluate_with_presentation(
        &self,
        instance: &mut ComponentInstance,
        expression: &IrExpression,
        locals: &HashMap<LocalId, DslValue>,
        presentation: bool,
    ) -> Result<DslValue, RuntimeError> {
        let program = self
            .package
            .program(expression.id)
            .ok_or(RuntimeError::MissingExpression(expression.id.raw()))?;
        let stamp = EvaluationStamp {
            properties: program
                .property_dependencies()
                .iter()
                .map(|id| {
                    instance
                        .properties
                        .get(id)
                        .map(|property| (*id, property.revision()))
                        .ok_or(RuntimeError::MissingProperty(id.raw()))
                })
                .collect::<Result<Vec<_>, _>>()?,
            token_revision: if program.token_dependencies().is_empty() {
                0
            } else {
                self.token_revision
            },
        };
        let animated_dependency = presentation
            && program
                .property_dependencies()
                .iter()
                .any(|id| instance.presented_properties.contains_key(id));
        if !program.is_contextual()
            && !animated_dependency
            && let Some(value) = instance.cached_expression(expression.id, &stamp)
        {
            return Ok(value);
        }
        let context =
            InstanceValueContext::new(instance, locals, &self.tokens, self.translator.as_deref());
        let mut context = if presentation {
            context
        } else {
            context.canonical()
        };
        let value = program.evaluate(&mut context)?;
        if !program.is_contextual() && !animated_dependency {
            instance.cache_expression(expression.id, stamp, value.clone());
        }
        Ok(value)
    }
}

/// Finds one native event binding recursively by stable source site and event ID.
fn find_native_event(
    nodes: &[IrNode],
    site: SiteId,
    event: argui_schema::EventId,
) -> Option<&IrEventBinding> {
    for node in nodes {
        match node {
            IrNode::Element {
                site: candidate,
                events,
                children,
                ..
            } => {
                if *candidate == site
                    && let Some(binding) = events
                        .iter()
                        .find(|binding| binding.target == EventTargetId::Native(event))
                {
                    return Some(binding);
                }
                if let Some(binding) = find_native_event(children, site, event) {
                    return Some(binding);
                }
            }
            IrNode::Repeater { body, .. } => {
                if let Some(binding) = find_native_event(body, site, event) {
                    return Some(binding);
                }
            }
            IrNode::Conditional {
                then_body,
                else_body,
                ..
            } => {
                if let Some(binding) = find_native_event(then_body, site, event)
                    .or_else(|| find_native_event(else_body, site, event))
                {
                    return Some(binding);
                }
            }
            IrNode::Slot { .. } => {}
        }
    }
    None
}
