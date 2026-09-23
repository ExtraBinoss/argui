//! Child output preparation before visual siblings evaluate their bindings.

use std::collections::HashMap;

use argui_dsl_ir::{
    ComponentId, IrComponent, IrElementTarget, IrNode, IrPropertyBinding, PropertyTargetId, SiteId,
};

use crate::{ComponentInstance, LiveRuntime, RuntimeError, runtime::initialize_instance};

use super::identity::child_instance_id;

impl LiveRuntime {
    /// Prepares child property values before expressions in any sibling visual.
    ///
    /// `parent` is the retained owner, `definition` supplies identified child
    /// calls, and `context` samples current host observations. Returns a typed
    /// runtime error if a child input or derived default cannot be evaluated.
    pub(super) fn prepare_child_references(
        &mut self,
        parent: &mut ComponentInstance,
        definition: &IrComponent,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<(), RuntimeError> {
        parent.child_outputs.clear();
        self.prepare_scoped_child_references(
            parent,
            definition,
            &definition.body,
            &HashMap::new(),
            parent.id,
            None,
            context,
        )
    }

    /// Prepares child outputs visible in one lexical visual scope.
    ///
    /// `parent` owns output values, `definition` selects referenced sites,
    /// `nodes` is the current visual body, `locals` supplies repeater variables,
    /// `identity_owner` and `repeater_key` identify mounted row children, and
    /// `context` samples host observations. Returns an evaluation or type error.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn prepare_scoped_child_references(
        &mut self,
        parent: &mut ComponentInstance,
        definition: &IrComponent,
        nodes: &[IrNode],
        locals: &HashMap<argui_dsl_ir::LocalId, crate::DslValue>,
        identity_owner: crate::InstanceId,
        repeater_key: Option<&crate::DslValue>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<(), RuntimeError> {
        for node in nodes {
            match node {
                IrNode::Element {
                    site,
                    target,
                    source_id,
                    properties,
                    children,
                    ..
                } => match target {
                    IrElementTarget::Component(component)
                        if source_id.is_some()
                            && self
                                .package
                                .child_reference_sites
                                .get(&definition.id)
                                .is_some_and(|sites| sites.contains(site)) =>
                    {
                        self.prepare_identified_child(
                            parent,
                            *site,
                            *component,
                            properties,
                            locals,
                            identity_owner,
                            repeater_key,
                            context,
                        )?;
                    }
                    IrElementTarget::Native(_) => self.prepare_scoped_child_references(
                        parent,
                        definition,
                        children,
                        locals,
                        identity_owner,
                        repeater_key,
                        context,
                    )?,
                    _ => {}
                },
                IrNode::Conditional {
                    condition,
                    then_body,
                    else_body,
                    ..
                } => {
                    let selected = match self.evaluate(parent, condition, locals)? {
                        crate::DslValue::Bool(true) => then_body,
                        crate::DslValue::Bool(false) => else_body,
                        value => {
                            return Err(RuntimeError::TypeMismatch {
                                expected: "bool".into(),
                                actual: value.type_name().into(),
                            });
                        }
                    };
                    self.prepare_scoped_child_references(
                        parent,
                        definition,
                        selected,
                        locals,
                        identity_owner,
                        repeater_key,
                        context,
                    )?;
                }
                IrNode::Slot { fallback, .. } | IrNode::SlotContent { body: fallback, .. } => {
                    self.prepare_scoped_child_references(
                        parent,
                        definition,
                        fallback,
                        locals,
                        identity_owner,
                        repeater_key,
                        context,
                    )?;
                }
                IrNode::Repeater { .. } => {}
            }
        }
        Ok(())
    }

    /// Prepares one identified child's canonical outputs for its lexical owner.
    ///
    /// `parent` receives outputs, `site` and `component` identify the child,
    /// `bindings` provide inputs, `locals` supplies row variables,
    /// `identity_owner` and `repeater_key` identify its retained instance, and
    /// `context` supplies native observations. Returns a schema or evaluation error.
    #[allow(clippy::too_many_arguments)]
    fn prepare_identified_child(
        &mut self,
        parent: &mut ComponentInstance,
        site: SiteId,
        component: ComponentId,
        bindings: &[IrPropertyBinding],
        locals: &HashMap<argui_dsl_ir::LocalId, crate::DslValue>,
        identity_owner: crate::InstanceId,
        repeater_key: Option<&crate::DslValue>,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<(), RuntimeError> {
        let child_id = child_instance_id(identity_owner, site, repeater_key);
        if self
            .instances
            .get(&child_id)
            .is_none_or(|child| child.component != component)
        {
            self.instances.insert(
                child_id,
                initialize_instance(&self.package, component, child_id, &self.tokens)?,
            );
        }
        self.rendered_instances.insert(child_id);
        for binding in bindings {
            let PropertyTargetId::Component(property) = binding.target else {
                return Err(RuntimeError::Schema(
                    "native binding on referenced child component".into(),
                ));
            };
            let value = self.evaluate_canonical(parent, &binding.value, locals)?;
            self.instances
                .get_mut(&child_id)
                .ok_or(RuntimeError::MissingComponent(child_id.raw()))?
                .properties
                .get_mut(&property)
                .ok_or(RuntimeError::MissingProperty(property.raw()))?
                .set(value)?;
        }
        let project = std::sync::Arc::clone(&self.package.ir);
        let child_definition = project
            .components
            .iter()
            .find(|candidate| candidate.id == component)
            .ok_or(RuntimeError::MissingComponent(component.raw()))?;
        let mut child = self
            .instances
            .remove(&child_id)
            .ok_or(RuntimeError::MissingComponent(child_id.raw()))?;
        let prepared = (|| {
            if let Some(sites) = self.package.observed_sites.get(&component) {
                for native_site in sites {
                    let identity =
                        argui_ui::RetainedIdentity::new(child_id.raw(), native_site.raw());
                    let observed = context.as_deref().map_or_else(Default::default, |host| {
                        host.observed_interaction(&identity)
                    });
                    child.observations.insert(*native_site, observed);
                }
            }
            self.prepare_child_references(&mut child, child_definition, context)?;
            self.refresh_derived_defaults(&mut child, child_definition)
        })();
        if let Err(error) = prepared {
            self.instances.insert(child_id, child);
            return Err(error);
        }
        for property in &child_definition.properties {
            let value = child
                .properties
                .get(&property.id)
                .ok_or(RuntimeError::MissingProperty(property.id.raw()))?
                .get()
                .clone();
            parent.child_outputs.insert((site, property.id), value);
        }
        self.instances.insert(child_id, child);
        Ok(())
    }

    /// Refreshes unwritten defaults after input bindings and observations arrive.
    ///
    /// `instance` owns retained properties and `definition` supplies their
    /// checked defaults. Returns an evaluation or type error on malformed IR.
    pub(super) fn refresh_derived_defaults(
        &self,
        instance: &mut ComponentInstance,
        definition: &IrComponent,
    ) -> Result<(), RuntimeError> {
        for property in &definition.properties {
            if instance
                .properties
                .get(&property.id)
                .is_some_and(crate::DynamicProperty::has_override)
            {
                continue;
            }
            let Some(default) = &property.default else {
                continue;
            };
            if matches!(default.kind, argui_dsl_ir::IrExpressionKind::Constant(_)) {
                continue;
            }
            let value = self.evaluate(instance, default, &HashMap::new())?;
            instance
                .properties
                .get_mut(&property.id)
                .ok_or(RuntimeError::MissingProperty(property.id.raw()))?
                .refresh_default(value)?;
        }
        Ok(())
    }
}
