//! Child output preparation before visual siblings evaluate their bindings.

use std::collections::{HashMap, HashSet};

use argui_dsl_ir::{
    ComponentId, IrComponent, IrElementTarget, IrNode, IrPropertyBinding, PropertyTargetId, SiteId,
};

use crate::{ComponentInstance, LiveRuntime, RuntimeError, runtime::initialize_instance};

use super::identity::child_instance_id;

/// Collects unconditional identified component calls in lexical visual order.
///
/// `nodes` are one component body, `referenced` selects only sites read by
/// expressions, and `output` receives stable site, target, and bindings.
/// Conditional/repeated calls are excluded because they do not identify a
/// single mounted child for a sibling expression.
fn collect_children(
    nodes: &[IrNode],
    referenced: &HashSet<SiteId>,
    output: &mut Vec<(SiteId, ComponentId, Vec<IrPropertyBinding>)>,
) {
    for node in nodes {
        if let IrNode::Element {
            site,
            target,
            source_id,
            properties,
            children,
            ..
        } = node
        {
            match target {
                IrElementTarget::Component(component)
                    if source_id.is_some() && referenced.contains(site) =>
                {
                    output.push((*site, *component, properties.clone()));
                }
                IrElementTarget::Native(_) => collect_children(children, referenced, output),
                _ => {}
            }
        }
    }
}

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
        let mut children = Vec::new();
        collect_children(
            &definition.body,
            &definition.referenced_child_sites(),
            &mut children,
        );
        for (site, component, bindings) in children {
            let child_id = child_instance_id(parent.id, site, None);
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
            for binding in &bindings {
                let PropertyTargetId::Component(property) = binding.target else {
                    return Err(RuntimeError::Schema(
                        "native binding on referenced child component".into(),
                    ));
                };
                let value = self.evaluate_canonical(parent, &binding.value, &HashMap::new())?;
                self.instances
                    .get_mut(&child_id)
                    .ok_or(RuntimeError::MissingComponent(child_id.raw()))?
                    .properties
                    .get_mut(&property)
                    .ok_or(RuntimeError::MissingProperty(property.raw()))?
                    .set(value)?;
            }
            let child_definition = self
                .package
                .ir
                .components
                .iter()
                .find(|candidate| candidate.id == component)
                .cloned()
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
                self.prepare_child_references(&mut child, &child_definition, context)?;
                self.refresh_derived_defaults(&mut child, &child_definition)
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
        }
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
            let Some(default) = &property.default else {
                continue;
            };
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
