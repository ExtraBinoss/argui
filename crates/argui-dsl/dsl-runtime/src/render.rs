use std::collections::{HashMap, HashSet};

use argui_dsl_ir::{
    EventTargetId, IrElementTarget, IrExpressionKind, IrNode, LocalId, PropertyTargetId, SlotId,
};

use crate::{
    ComponentInstance, DslValue, InstanceId, LiveRuntime, RuntimeError,
    instance::{EventRoute, PropertyLink},
    runtime::initialize_instance,
};

mod child_reference;
mod effect;
mod evaluate;
mod identity;
mod instance;
mod motion;
mod state;
mod virtual_list;

use identity::{child_instance_id, retained};

/// Caller-owned repeater retained as a lazy template for a nested component.
pub(super) struct TemplateSlot<'a> {
    slot: SlotId,
    owner: &'a mut ComponentInstance,
    component: &'a argui_dsl_ir::IrComponent,
    repeater: &'a IrNode,
    locals: &'a HashMap<LocalId, DslValue>,
    slots: &'a HashMap<SlotId, Vec<argui_ui::Element>>,
}

impl LiveRuntime {
    /// Renders the mounted live root through precompiled expression bytecode.
    ///
    /// # Errors
    ///
    /// Returns for missing state, invalid bytecode, incompatible native input, or slots.
    pub fn render(&mut self) -> Result<argui_ui::Element, RuntimeError> {
        let mut context = None;
        self.render_root(&mut context)
    }

    /// Renders through a retained application context so native events can mutate live state.
    ///
    /// * `context` — presentation context which owns the opaque event-handler identities.
    ///
    /// # Errors
    ///
    /// Returns for missing state, invalid bytecode, incompatible native input, or slots.
    pub fn render_with_context(
        &mut self,
        context: &mut argui_runtime::Context<Self>,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let mut context = Some(context);
        self.render_root(&mut context)
    }

    /// Renders a sequence of nodes in one component/local environment.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_nodes(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        nodes: &[IrNode],
        locals: &HashMap<LocalId, DslValue>,
        slots: &HashMap<SlotId, Vec<argui_ui::Element>>,
        mut template: Option<&mut TemplateSlot<'_>>,
        identity_owner: InstanceId,
        repeater_key: Option<&DslValue>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<Vec<argui_ui::Element>, RuntimeError> {
        let mut output = Vec::new();
        for node in nodes {
            match node {
                IrNode::Element {
                    site,
                    target,
                    properties,
                    events,
                    children,
                    source_id,
                    effect,
                    ..
                } => match target {
                    IrElementTarget::Native(native) => {
                        let schema = self.schema.schema(*native).cloned().ok_or_else(|| {
                            RuntimeError::Schema(format!("unknown native {}", native.raw()))
                        })?;
                        let (children, virtual_window) = if schema.virtual_window {
                            let (rows, count, start, viewport) = self.render_virtual_children(
                                instance,
                                component,
                                *site,
                                children,
                                properties,
                                locals,
                                slots,
                                template.as_deref_mut(),
                                identity_owner,
                                repeater_key,
                                context,
                            )?;
                            (rows, Some((count, start, viewport)))
                        } else {
                            (
                                self.render_nodes(
                                    instance,
                                    component,
                                    children,
                                    locals,
                                    slots,
                                    template.as_deref_mut(),
                                    identity_owner,
                                    repeater_key,
                                    context,
                                )?,
                                None,
                            )
                        };
                        if schema.slots.is_empty() && !children.is_empty() {
                            return Err(RuntimeError::Schema(format!(
                                "native `{}` does not accept children",
                                schema.name
                            )));
                        }
                        let mut input = argui_schema::NativeElementInput::new();
                        let identity = retained(identity_owner, *site, repeater_key)?;
                        let reduced_motion = context
                            .as_deref()
                            .is_some_and(|host| host.environment().reduced_motion);
                        if let Some(key) = schema
                            .properties
                            .iter()
                            .find(|property| property.name.as_str() == "key")
                            .filter(|key| {
                                !properties.iter().any(|binding| {
                                    binding.target == PropertyTargetId::Native(key.id)
                                })
                            })
                        {
                            input = input.property(
                                key.id,
                                argui_schema::SchemaValue::String(
                                    source_id.clone().unwrap_or_else(|| {
                                        format!("dsl:{}:{}", instance.id.raw(), site.raw())
                                    }),
                                ),
                            );
                        }
                        for binding in properties {
                            if schema.virtual_window
                                && binding.target
                                    == PropertyTargetId::Native(
                                        argui_schema::builtin::VIEWPORT_HEIGHT,
                                    )
                            {
                                continue;
                            }
                            let PropertyTargetId::Native(property) = binding.target else {
                                return Err(RuntimeError::Schema(
                                    "component property was assigned to a native".into(),
                                ));
                            };
                            let base = self.evaluate(instance, &binding.value, locals)?;
                            let selected = self.select_state_value(
                                instance,
                                component,
                                Some(*site),
                                binding.target,
                                base,
                                &binding.value.value_type,
                                locals,
                            )?;
                            let schema_value = match selected.effective.clone() {
                                DslValue::Asset(id) => {
                                    argui_schema::SchemaValue::Asset(self.asset_handle(id)?)
                                }
                                value => value.to_schema(&selected.value_type)?,
                            };
                            let schema_value = self.animate_native_value(
                                instance,
                                component,
                                *site,
                                binding.target,
                                schema_value,
                                &selected,
                                locals,
                                identity.clone(),
                                reduced_motion,
                            )?;
                            input = input.property(property, schema_value);
                        }
                        for native_property in &schema.properties {
                            let property = native_property.id;
                            let target_property = PropertyTargetId::Native(property);
                            if schema.virtual_window
                                && property == argui_schema::builtin::VIEWPORT_HEIGHT
                            {
                                continue;
                            }
                            if properties
                                .iter()
                                .any(|binding| binding.target == target_property)
                            {
                                continue;
                            }
                            let animation = component.animations.iter().find(|animation| {
                                animation.owner == Some(*site)
                                    && animation.property == target_property
                            });
                            let has_state = component.states.iter().any(|state| {
                                state.owner == Some(*site)
                                    && state
                                        .assignments
                                        .iter()
                                        .any(|(assigned, _)| *assigned == target_property)
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
                                    .or_else(|| {
                                        animation.keyframes.last().map(|frame| &frame.value)
                                    })
                            });
                            let (base, base_type) = if let Some(endpoint) = endpoint {
                                (
                                    self.evaluate(instance, endpoint, locals)?,
                                    endpoint.value_type.clone(),
                                )
                            } else {
                                Self::native_default_value(native_property)?
                            };
                            let selected = self.select_state_value(
                                instance,
                                component,
                                Some(*site),
                                target_property,
                                base,
                                &base_type,
                                locals,
                            )?;
                            let target = selected.effective.to_schema(&selected.value_type)?;
                            let sampled = self.animate_native_value(
                                instance,
                                component,
                                *site,
                                target_property,
                                target,
                                &selected,
                                locals,
                                identity.clone(),
                                reduced_motion,
                            )?;
                            input = input.property(property, sampled);
                        }
                        if let Some((count, start, viewport)) = virtual_window {
                            input = input
                                .property(
                                    argui_schema::builtin::VIEWPORT_HEIGHT,
                                    argui_schema::SchemaValue::Float(viewport),
                                )
                                .property(
                                    argui_schema::builtin::ITEM_COUNT,
                                    argui_schema::SchemaValue::Int(count as i64),
                                )
                                .property(
                                    argui_schema::builtin::WINDOW_START,
                                    argui_schema::SchemaValue::Int(start as i64),
                                );
                        }
                        if let Some(slot) = schema.slots.first() {
                            input =
                                input.slot(argui_schema::NativeSlotValue::new(slot.id, children));
                        }
                        input = self.bind_native_events(
                            instance, &schema, properties, events, locals, input, context,
                        )?;
                        let element = self
                            .schema
                            .construct(*native, &input)
                            .map_err(|error| RuntimeError::Schema(error.to_string()))?;
                        let element =
                            self.apply_effect(instance, effect.as_ref(), locals, element)?;
                        output.push(element.retained_identity(identity));
                    }
                    IrElementTarget::Component(target) => {
                        let child_id = child_instance_id(identity_owner, *site, repeater_key);
                        let identity = retained(identity_owner, *site, repeater_key)?;
                        let reduced_motion = context
                            .as_deref()
                            .is_some_and(|host| host.environment().reduced_motion);
                        let requires_new = self
                            .instances
                            .get(&child_id)
                            .is_none_or(|child| child.component != *target);
                        if requires_new {
                            let child = initialize_instance(
                                &self.package,
                                *target,
                                child_id,
                                &self.tokens,
                            )?;
                            self.instances.insert(child_id, child);
                        }
                        self.rendered_instances.insert(child_id);
                        for binding in properties {
                            let PropertyTargetId::Component(property) = binding.target else {
                                return Err(RuntimeError::Schema(
                                    "native property was assigned to a component".into(),
                                ));
                            };
                            let canonical =
                                self.evaluate_canonical(instance, &binding.value, locals)?;
                            let base = self.evaluate(instance, &binding.value, locals)?;
                            let inherited_presentation = base != canonical;
                            let selected = self.select_state_value(
                                instance,
                                component,
                                Some(*site),
                                binding.target,
                                base,
                                &binding.value.value_type,
                                locals,
                            )?;
                            let animated = component.animations.iter().any(|animation| {
                                animation.owner == Some(*site)
                                    && animation.property == binding.target
                            });
                            let presented = if animated {
                                Some(self.animate_component_value(
                                    instance,
                                    component,
                                    *site,
                                    binding.target,
                                    &selected,
                                    locals,
                                    identity.clone(),
                                    reduced_motion,
                                )?)
                            } else if selected.has_states || inherited_presentation {
                                Some(selected.effective)
                            } else {
                                None
                            };
                            let child = self
                                .instances
                                .get_mut(&child_id)
                                .ok_or(RuntimeError::MissingComponent(child_id.raw()))?;
                            child
                                .properties
                                .get_mut(&property)
                                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                                .set(canonical)?;
                            if let Some(presented) = presented {
                                child.presented_properties.insert(property, presented);
                            }
                            let link = if binding.two_way {
                                match binding.value.kind {
                                    IrExpressionKind::PropertyRead(parent_property) => {
                                        Some(PropertyLink {
                                            instance: instance.id,
                                            property: parent_property,
                                        })
                                    }
                                    _ => {
                                        return Err(RuntimeError::InvalidBytecode(
                                            "two-way binding source is not a property".into(),
                                        ));
                                    }
                                }
                            } else {
                                None
                            };
                            child.set_link(property, link);
                        }
                        let mut unbound = HashSet::new();
                        for state in component
                            .states
                            .iter()
                            .filter(|state| state.owner == Some(*site))
                        {
                            unbound.extend(state.assignments.iter().map(|(target, _)| *target));
                        }
                        unbound.extend(
                            component
                                .animations
                                .iter()
                                .filter(|animation| animation.owner == Some(*site))
                                .map(|animation| animation.property),
                        );
                        for target_property in unbound {
                            let PropertyTargetId::Component(property) = target_property else {
                                continue;
                            };
                            if properties
                                .iter()
                                .any(|binding| binding.target == target_property)
                            {
                                continue;
                            }
                            let base = self
                                .instances
                                .get(&child_id)
                                .and_then(|child| child.properties.get(&property))
                                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                                .get()
                                .clone();
                            let value_type = self
                                .package
                                .ir
                                .components
                                .iter()
                                .find(|definition| definition.id == *target)
                                .and_then(|definition| {
                                    definition
                                        .properties
                                        .iter()
                                        .find(|candidate| candidate.id == property)
                                })
                                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                                .value_type
                                .clone();
                            let selected = self.select_state_value(
                                instance,
                                component,
                                Some(*site),
                                target_property,
                                base,
                                &value_type,
                                locals,
                            )?;
                            let presented = self.animate_component_value(
                                instance,
                                component,
                                *site,
                                target_property,
                                &selected,
                                locals,
                                identity.clone(),
                                reduced_motion,
                            )?;
                            self.instances
                                .get_mut(&child_id)
                                .ok_or(RuntimeError::MissingComponent(child_id.raw()))?
                                .presented_properties
                                .insert(property, presented);
                        }
                        let child_definition = self
                            .package
                            .ir
                            .components
                            .iter()
                            .find(|component| component.id == *target)
                            .ok_or(RuntimeError::MissingComponent(target.raw()))?;
                        let template_slot = match child_definition.template_slots.as_slice() {
                            [] => None,
                            [slot] => Some(*slot),
                            _ => {
                                return Err(RuntimeError::InvalidBytecode(
                                    "template components support exactly one template slot".into(),
                                ));
                            }
                        };
                        let first_slot = child_definition.slots.first().copied();
                        let callbacks = child_definition
                            .callbacks
                            .iter()
                            .map(|callback| callback.id)
                            .collect::<Vec<_>>();
                        let supplied = if template_slot.is_some() {
                            Vec::new()
                        } else {
                            self.render_nodes(
                                instance,
                                component,
                                children,
                                locals,
                                slots,
                                template.as_deref_mut(),
                                identity_owner,
                                repeater_key,
                                context,
                            )?
                        };
                        let mut child_slots = HashMap::new();
                        if let Some(slot) = first_slot.filter(|slot| template_slot != Some(*slot)) {
                            child_slots.insert(slot, supplied);
                        }
                        let child = self
                            .instances
                            .get_mut(&child_id)
                            .ok_or(RuntimeError::MissingComponent(child_id.raw()))?;
                        for callback in callbacks {
                            let route = events
                                .iter()
                                .find(|event| event.target == EventTargetId::Component(callback))
                                .map(|event| EventRoute {
                                    parent: instance.id,
                                    statements: event.statements.clone(),
                                    parameters: event.parameters.clone(),
                                    locals: locals.clone(),
                                });
                            child.set_route(callback, route);
                        }
                        let row_template = template_slot.map(|slot| TemplateSlot {
                            slot,
                            owner: instance,
                            component,
                            repeater: children.first().expect("validated template slot repeater"),
                            locals,
                            slots,
                        });
                        let element =
                            self.render_instance(child_id, child_slots, row_template, context)?;
                        output.push(self.apply_effect(
                            instance,
                            effect.as_ref(),
                            locals,
                            element,
                        )?);
                    }
                },
                IrNode::Repeater {
                    local,
                    model,
                    key,
                    body,
                    ..
                } => {
                    let value = self.evaluate(instance, model, locals)?;
                    let DslValue::Array(items) = value else {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "model/array".into(),
                            actual: value.type_name().into(),
                        });
                    };
                    for item in items {
                        let mut nested = locals.clone();
                        nested.insert(*local, item);
                        let key = self.evaluate(instance, key, &nested)?;
                        output.extend(self.render_nodes(
                            instance,
                            component,
                            body,
                            &nested,
                            slots,
                            template.as_deref_mut(),
                            identity_owner,
                            Some(&key),
                            context,
                        )?);
                    }
                }
                IrNode::Conditional {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    let value = self.evaluate(instance, condition, locals)?;
                    let DslValue::Bool(value) = value else {
                        return Err(RuntimeError::TypeMismatch {
                            expected: "bool".into(),
                            actual: value.type_name().into(),
                        });
                    };
                    output.extend(self.render_nodes(
                        instance,
                        component,
                        if value { then_body } else { else_body },
                        locals,
                        slots,
                        template.as_deref_mut(),
                        identity_owner,
                        repeater_key,
                        context,
                    )?);
                }
                IrNode::Slot { slot, .. } => {
                    output.extend(slots.get(slot).cloned().unwrap_or_default());
                }
            }
        }
        Ok(output)
    }
}
