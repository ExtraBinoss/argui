use std::collections::HashMap;

use argui_dsl_ir::{
    EventTargetId, IrEventBinding, IrExpressionKind, IrPropertyBinding, LocalId, PropertyTargetId,
};

use crate::{ComponentInstance, DslValue, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Registers explicit and two-way native event routes through the host context.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn bind_native_events(
        &mut self,
        instance: &ComponentInstance,
        schema: &argui_schema::NativeSchema,
        properties: &[IrPropertyBinding],
        events: &[IrEventBinding],
        locals: &HashMap<LocalId, DslValue>,
        mut input: argui_schema::NativeElementInput,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_schema::NativeElementInput, RuntimeError> {
        for event_schema in &schema.events {
            let explicit = events
                .iter()
                .find(|binding| binding.target == EventTargetId::Native(event_schema.id));
            let updates = properties
                .iter()
                .filter(|binding| binding.two_way)
                .filter_map(|binding| two_way_update(schema, event_schema.id, binding))
                .collect::<Vec<_>>();
            if explicit.is_none() && updates.is_empty() {
                continue;
            }
            let Some(host) = context.as_deref_mut() else {
                continue;
            };
            let statements = explicit
                .map(|binding| binding.statements.clone())
                .unwrap_or_default();
            let locals = locals.clone();
            let instance = instance.id;
            let handler = host.event_handler(move |runtime, event, host| {
                match runtime.deliver_native_event(
                    instance,
                    &statements,
                    locals.clone(),
                    &updates,
                    event,
                ) {
                    Ok(()) => runtime.event_error = None,
                    Err(error) => runtime.event_error = Some(error),
                }
                host.notify();
            });
            input = input.event(argui_schema::NativeEventValue::new(
                event_schema.id,
                handler,
            ));
        }
        Ok(input)
    }
}

impl argui_runtime::Render for LiveRuntime {
    /// Renders the live package and preserves the last valid tree if evaluation fails.
    fn render(&mut self, context: &mut argui_runtime::Context<Self>) -> argui_ui::Element {
        #[cfg(not(target_arch = "wasm32"))]
        self.ensure_client_listener(context);
        #[cfg(target_arch = "wasm32")]
        self.ensure_web_listener(context);
        match self.render_with_context(context) {
            Ok(element) => {
                self.render_error = None;
                self.last_valid_element = Some(element.clone());
                element
            }
            Err(error) => {
                self.render_error = Some(error);
                self.last_valid_element
                    .clone()
                    .unwrap_or_else(|| argui_ui::Element::container([]))
            }
        }
    }
}

/// Resolves one native two-way binding into its owning DSL property update.
fn two_way_update(
    schema: &argui_schema::NativeSchema,
    event: argui_schema::EventId,
    binding: &IrPropertyBinding,
) -> Option<(argui_dsl_ir::PropertyId, argui_schema::ValueType)> {
    let PropertyTargetId::Native(native_property) = binding.target else {
        return None;
    };
    let property_schema = schema
        .properties
        .iter()
        .find(|property| property.id == native_property)?;
    if property_schema.change_event != Some(event) {
        return None;
    }
    let IrExpressionKind::PropertyRead(property) = binding.value.kind else {
        return None;
    };
    Some((property, property_schema.value_type))
}
