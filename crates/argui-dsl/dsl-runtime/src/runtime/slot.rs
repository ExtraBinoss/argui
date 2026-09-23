//! Host-supplied content for public root component slots.

use argui_dsl_ir::SlotId;
use argui_ui::Element;

use super::LiveRuntime;
use crate::RuntimeError;

impl LiveRuntime {
    /// Supplies elements to one declared root component slot.
    ///
    /// `slot` is the stable public slot ID and `elements` are the replacement
    /// visual children. A later call replaces this slot without remounting the
    /// root. Returns after accepting the content.
    ///
    /// # Errors
    ///
    /// Returns for an unmounted root or undeclared slot.
    pub fn set_root_slot(
        &mut self,
        slot: SlotId,
        elements: impl IntoIterator<Item = Element>,
    ) -> Result<(), RuntimeError> {
        let root = self
            .root
            .ok_or_else(|| RuntimeError::InvalidBytecode("no live root is mounted".into()))?;
        let component = self
            .instances
            .get(&root)
            .ok_or(RuntimeError::MissingComponent(root.raw()))?
            .component;
        let definition = self
            .package
            .ir
            .components
            .iter()
            .find(|definition| definition.id == component)
            .ok_or(RuntimeError::MissingComponent(component.raw()))?;
        if !definition.slots.contains(&slot) {
            return Err(RuntimeError::Schema(format!(
                "root component {} has no slot {}",
                component.raw(),
                slot.raw(),
            )));
        }
        self.root_slots.insert(slot, elements.into_iter().collect());
        Ok(())
    }
}
