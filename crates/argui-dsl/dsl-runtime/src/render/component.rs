//! Stateful DSL child components and caller-scoped slot projection.
use super::*;
use argui_dsl_ir::{IrComponent, IrEffectBinding, IrEventBinding, IrPropertyBinding};

impl LiveRuntime {
    /// Renders `target` at `site` from the parent `instance` and `component`.
    /// `properties`, `events`, `children`, and `effect` are checked call-site inputs;
    /// `locals`, `slots`, and `template` preserve caller scope. `identity_owner`
    /// and `repeater_key` isolate retained state; `context` carries host services.
    /// Returns the child element, or a schema, binding, evaluation, or state error.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn render_component_call(
        &mut self,
        instance: &mut ComponentInstance,
        component: &IrComponent,
        target: argui_dsl_ir::ComponentId,
        site: argui_dsl_ir::SiteId,
        properties: &[IrPropertyBinding],
        events: &[IrEventBinding],
        children: &[IrNode],
        effect: Option<&IrEffectBinding>,
        locals: &HashMap<LocalId, DslValue>,
        slots: &HashMap<SlotId, Vec<argui_ui::Element>>,
        mut template: Option<&mut TemplateSlot<'_>>,
        identity_owner: InstanceId,
        repeater_key: Option<&DslValue>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let child_id = child_instance_id(identity_owner, site, repeater_key);
        let identity = retained(identity_owner, site, repeater_key)?;
        let reduced_motion = context
            .as_deref()
            .is_some_and(|host| host.environment().reduced_motion);
        let requires_new = self
            .instances
            .get(&child_id)
            .is_none_or(|child| child.component != target);
        if requires_new {
            let child = initialize_instance(&self.package, target, child_id, &self.tokens)?;
            self.instances.insert(child_id, child);
        }
        self.rendered_instances.insert(child_id);
        for binding in properties {
            let PropertyTargetId::Component(property) = binding.target else {
                return Err(RuntimeError::Schema(
                    "native property was assigned to a component".into(),
                ));
            };
            let canonical = self.evaluate_canonical(instance, &binding.value, locals)?;
            let base = self.evaluate(instance, &binding.value, locals)?;
            let inherited_presentation = base != canonical;
            let selected = self.select_state_value(
                instance,
                component,
                Some(site),
                binding.target,
                base,
                &binding.value.value_type,
                locals,
            )?;
            let animated = component.animations.iter().any(|animation| {
                animation.owner == Some(site) && animation.property == binding.target
            });
            let presented = if animated {
                Some(self.animate_component_value(
                    instance,
                    component,
                    site,
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
                    IrExpressionKind::PropertyRead(parent_property) => Some(PropertyLink {
                        instance: instance.id,
                        property: parent_property,
                    }),
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
            .filter(|state| state.owner == Some(site))
        {
            unbound.extend(state.assignments.iter().map(|(target, _)| *target));
        }
        unbound.extend(
            component
                .animations
                .iter()
                .filter(|animation| animation.owner == Some(site))
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
                .find(|definition| definition.id == target)
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
                Some(site),
                target_property,
                base,
                &value_type,
                locals,
            )?;
            let presented = self.animate_component_value(
                instance,
                component,
                site,
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
            .find(|component| component.id == target)
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
        let slot_ids = child_definition.slots.clone();
        let callbacks = child_definition
            .callbacks
            .iter()
            .map(|callback| callback.id)
            .collect::<Vec<_>>();
        let named = children
            .iter()
            .any(|child| matches!(child, IrNode::SlotContent { .. }));
        let content = |slot, index| {
            children
                .iter()
                .find_map(|child| match child {
                    IrNode::SlotContent {
                        slot: target, body, ..
                    } if *target == slot => Some(body.as_slice()),
                    _ => None,
                })
                .unwrap_or(if index == 0 && !named { children } else { &[] })
        };
        let mut child_slots = HashMap::new();
        for (index, slot) in slot_ids.iter().enumerate() {
            if template_slot == Some(*slot) {
                continue;
            }
            let supplied = self.render_nodes(
                instance,
                component,
                content(*slot, index),
                locals,
                slots,
                template.as_deref_mut(),
                identity_owner,
                repeater_key,
                context,
            )?;
            child_slots.insert(*slot, supplied);
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
            repeater: content(
                slot,
                slot_ids.iter().position(|id| *id == slot).unwrap_or(0),
            )
            .first()
            .expect("validated template slot repeater"),
            locals,
            slots,
        });
        let element = self.render_instance(child_id, child_slots, row_template, context)?;
        self.apply_effect(instance, effect, locals, element)
    }
}
