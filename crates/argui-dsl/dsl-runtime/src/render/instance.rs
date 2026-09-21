//! Stateful component instance traversal for live DSL rendering.

use std::collections::HashMap;

use argui_dsl_ir::SlotId;

use crate::{InstanceId, LiveRuntime, RuntimeError};

use super::TemplateSlot;

impl LiveRuntime {
    /// Renders the current root with an optional retained handler-registration context.
    pub(super) fn render_root(
        &mut self,
        context: &mut Option<&mut argui_runtime::Context<Self>>,
    ) -> Result<argui_ui::Element, RuntimeError> {
        let root = self
            .root()
            .ok_or_else(|| RuntimeError::InvalidBytecode("no live root is mounted".into()))?;
        self.rendered_instances.clear();
        self.rendered_instances.insert(root);
        self.property_motions.begin_render();
        let element = self.render_instance(root, HashMap::new(), None, context);
        if element.is_ok() {
            self.property_motions.end_render();
        }
        self.instances
            .retain(|id, _| self.rendered_instances.contains(id));
        element
    }

    /// Renders one stateful component instance and returns it to the instance store.
    pub(super) fn render_instance(
        &mut self,
        instance_id: InstanceId,
        slots: HashMap<SlotId, Vec<argui_ui::Element>>,
        mut template: Option<TemplateSlot<'_>>,
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
        let result = self
            .apply_own_animations(&mut instance, &definition, context)
            .and_then(|()| {
                self.render_nodes(
                    &mut instance,
                    &definition,
                    &definition.body,
                    &HashMap::new(),
                    &slots,
                    template.as_mut(),
                    instance_id,
                    None,
                    context,
                )
            });
        instance.presented_properties.clear();
        self.instances.insert(instance_id, instance);
        let mut roots = result?;
        Ok(if roots.len() == 1 {
            roots.remove(0)
        } else {
            argui_ui::Element::container(roots)
        })
    }
}
