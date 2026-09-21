use std::collections::HashMap;

use argui_dsl_ir::{
    EventTargetId, IrElementTarget, IrEventBinding, IrExpression, IrExpressionKind, IrNode,
    LocalId, PropertyTargetId, SiteId, SlotId,
};

use crate::{
    AnimationKey, ComponentInstance, DslValue, InstanceId, LiveRuntime, RuntimeError,
    instance::EvaluationStamp,
    instance::{EventRoute, PropertyLink},
    runtime::{InstanceValueContext, initialize_instance},
};

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

    /// Renders the current root with an optional retained handler-registration context.
    fn render_root(
        &mut self,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let root = self
            .root()
            .ok_or_else(|| RuntimeError::InvalidBytecode("no live root is mounted".into()))?;
        self.rendered_instances.clear();
        self.rendered_instances.insert(root);
        let element = self.render_instance(root, HashMap::new(), context)?;
        self.instances
            .retain(|id, _| self.rendered_instances.contains(id));
        Ok(element)
    }

    /// Renders one stateful component instance and returns it to the instance store.
    fn render_instance(
        &mut self,
        instance_id: InstanceId,
        slots: HashMap<SlotId, Vec<argui_ui::Element>>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let mut instance = self
            .instances
            .remove(&instance_id)
            .ok_or(RuntimeError::MissingComponent(instance_id.raw()))?;
        let definition = self
            .package
            .ir
            .components
            .iter()
            .find(|component| component.id == instance.component)
            .cloned()
            .ok_or(RuntimeError::MissingComponent(instance.component.raw()))?;
        let result = self.render_nodes(
            &mut instance,
            &definition,
            &definition.body,
            &HashMap::new(),
            &slots,
            None,
            context,
        );
        self.instances.insert(instance_id, instance);
        let mut roots = result?;
        Ok(if roots.len() == 1 {
            roots.remove(0)
        } else {
            argui_ui::Element::container(roots)
        })
    }

    /// Renders a sequence of nodes in one component/local environment.
    #[allow(clippy::too_many_arguments)]
    fn render_nodes(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        nodes: &[IrNode],
        locals: &HashMap<LocalId, DslValue>,
        slots: &HashMap<SlotId, Vec<argui_ui::Element>>,
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
                    ..
                } => match target {
                    IrElementTarget::Native(native) => {
                        let children = self.render_nodes(
                            instance,
                            component,
                            children,
                            locals,
                            slots,
                            repeater_key,
                            context,
                        )?;
                        let schema = self.schema.schema(*native).cloned().ok_or_else(|| {
                            RuntimeError::Schema(format!("unknown native {}", native.raw()))
                        })?;
                        if schema.slots.is_empty() && !children.is_empty() {
                            return Err(RuntimeError::Schema(format!(
                                "native `{}` does not accept children",
                                schema.name
                            )));
                        }
                        let mut input = argui_schema::NativeElementInput::new();
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
                            let PropertyTargetId::Native(property) = binding.target else {
                                return Err(RuntimeError::Schema(
                                    "component property was assigned to a native".into(),
                                ));
                            };
                            let mut value = self.evaluate(instance, &binding.value, locals)?;
                            for state in component
                                .states
                                .iter()
                                .filter(|state| state.owner == Some(*site))
                            {
                                if self.evaluate(instance, &state.condition, locals)?
                                    == DslValue::Bool(true)
                                    && let Some((_, replacement)) = state
                                        .assignments
                                        .iter()
                                        .find(|(target, _)| *target == binding.target)
                                {
                                    value = self.evaluate(instance, replacement, locals)?;
                                }
                            }
                            value = self.animated_value(
                                instance,
                                component,
                                *site,
                                binding.target,
                                value,
                                locals,
                            )?;
                            input = input
                                .property(property, value.to_schema(&binding.value.value_type)?);
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
                        output.push(element.retained_identity(retained(
                            instance.id,
                            *site,
                            repeater_key,
                        )?));
                    }
                    IrElementTarget::Component(target) => {
                        let child_id = child_instance_id(instance.id, *site, repeater_key);
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
                            let value = self.evaluate(instance, &binding.value, locals)?;
                            let child = self
                                .instances
                                .get_mut(&child_id)
                                .ok_or(RuntimeError::MissingComponent(child_id.raw()))?;
                            child
                                .properties
                                .get_mut(&property)
                                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                                .set(value)?;
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
                        let supplied = self.render_nodes(
                            instance,
                            component,
                            children,
                            locals,
                            slots,
                            repeater_key,
                            context,
                        )?;
                        let child_definition = self
                            .package
                            .ir
                            .components
                            .iter()
                            .find(|component| component.id == *target)
                            .ok_or(RuntimeError::MissingComponent(target.raw()))?;
                        let mut child_slots = HashMap::new();
                        if let Some(slot) = child_definition.slots.first() {
                            child_slots.insert(*slot, supplied);
                        }
                        let child = self
                            .instances
                            .get_mut(&child_id)
                            .ok_or(RuntimeError::MissingComponent(child_id.raw()))?;
                        for callback in &child_definition.callbacks {
                            let route = events
                                .iter()
                                .find(|event| event.target == EventTargetId::Component(callback.id))
                                .map(|event| EventRoute {
                                    parent: instance.id,
                                    statements: event.statements.clone(),
                                    locals: locals.clone(),
                                });
                            child.set_route(callback.id, route);
                        }
                        output.push(self.render_instance(child_id, child_slots, context)?);
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
    fn evaluate(
        &self,
        instance: &mut ComponentInstance,
        expression: &IrExpression,
        locals: &HashMap<LocalId, DslValue>,
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
        if !program.is_contextual()
            && let Some(value) = instance.cached_expression(expression.id, &stamp)
        {
            return Ok(value);
        }
        let mut context =
            InstanceValueContext::new(instance, locals, &self.tokens, self.translator.as_deref());
        let value = program.evaluate(&mut context)?;
        if !program.is_contextual() {
            instance.cache_expression(expression.id, stamp, value.clone());
        }
        Ok(value)
    }

    /// Retargets a declared scalar animation and returns its retained presentation value.
    fn animated_value(
        &mut self,
        instance: &mut ComponentInstance,
        component: &argui_dsl_ir::IrComponent,
        site: SiteId,
        property: PropertyTargetId,
        value: DslValue,
        locals: &HashMap<LocalId, DslValue>,
    ) -> Result<DslValue, RuntimeError> {
        let Some(animation) = component
            .animations
            .iter()
            .find(|animation| animation.owner == Some(site) && animation.property == property)
        else {
            return Ok(value);
        };
        let mut specification = 0xcbf2_9ce4_8422_2325_u64;
        for parameter in &animation.parameters {
            specification ^= parameter.id.raw();
            specification = specification.wrapping_mul(0x0000_0100_0000_01b3);
            let parameter = self.evaluate(instance, &parameter.value, locals)?;
            hash_value(&mut specification, &parameter);
        }
        let integer = matches!(value, DslValue::Int(_));
        let initial = match &value {
            DslValue::Float(value) => *value,
            DslValue::Int(value) => *value as f64,
            _ => return Ok(value),
        };
        let key = AnimationKey {
            instance: instance.id,
            component: component.id,
            site,
            property,
            animation: animation.id,
        };
        let current = self.animations.retarget(key, initial, specification).value;
        Ok(if integer {
            DslValue::Int(current.round() as i64)
        } else {
            DslValue::Float(current)
        })
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

/// Derives a deterministic child instance from owner, site, and repeater key.
fn child_instance_id(owner: InstanceId, site: SiteId, key: Option<&DslValue>) -> InstanceId {
    let mut value = owner.raw() ^ site.raw().rotate_left(17);
    if let Some(key) = key {
        value ^= key_hash(key).rotate_left(31);
    }
    InstanceId::from_raw(value.max(1))
}

/// Builds engine retained identity with the supported stable key classes.
fn retained(
    owner: InstanceId,
    site: SiteId,
    key: Option<&DslValue>,
) -> Result<argui_ui::RetainedIdentity, RuntimeError> {
    let identity = argui_ui::RetainedIdentity::new(owner.raw(), site.raw());
    match key {
        None => Ok(identity),
        Some(DslValue::Int(value)) => Ok(identity.with_signed_key(*value)),
        Some(DslValue::String(value)) => Ok(identity.with_name_key(value.clone())),
        Some(value) => Err(RuntimeError::TypeMismatch {
            expected: "string or int repeater key".into(),
            actual: value.type_name().into(),
        }),
    }
}

/// Hashes supported repeater keys without source/runtime name lookup.
fn key_hash(value: &DslValue) -> u64 {
    match value {
        DslValue::Int(value) => *value as u64,
        DslValue::String(value) => value.bytes().fold(0xcbf2_9ce4_8422_2325, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
        }),
        _ => 0,
    }
}

/// Extends a deterministic animation specification hash with one DSL value.
fn hash_value(hash: &mut u64, value: &DslValue) {
    fn bytes(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash = (*hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3);
        }
    }
    match value {
        DslValue::Null => bytes(hash, &[0]),
        DslValue::Bool(value) => bytes(hash, &[1, u8::from(*value)]),
        DslValue::Int(value) => bytes(hash, &value.to_le_bytes()),
        DslValue::Float(value) => bytes(hash, &value.to_bits().to_le_bytes()),
        DslValue::String(value) => bytes(hash, value.as_bytes()),
        DslValue::Color(value) => bytes(hash, &value.to_srgba8()),
        DslValue::Struct(fields) => {
            for (field, value) in fields {
                bytes(hash, &field.raw().to_le_bytes());
                hash_value(hash, value);
            }
        }
        DslValue::Enum { symbol, variant } => {
            bytes(hash, &symbol.to_le_bytes());
            bytes(hash, &variant.to_le_bytes());
        }
        DslValue::Array(values) => {
            for value in values {
                hash_value(hash, value);
            }
        }
        DslValue::Asset(asset) => bytes(hash, &asset.raw().to_le_bytes()),
    }
}
