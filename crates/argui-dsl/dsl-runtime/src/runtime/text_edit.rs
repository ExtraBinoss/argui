//! Incremental controlled text edits across live property aliases.

use std::collections::HashSet;

use argui_dsl_ir::PropertyId;

use crate::{InstanceId, LiveRuntime, RuntimeError};

impl LiveRuntime {
    /// Applies one native text edit through a controlled property and its aliases.
    ///
    /// * `instance` — component owning the edited property.
    /// * `property` — string property connected to the native input.
    /// * `edit` — accepted UTF-8 replacement from the text engine.
    ///
    /// Returns whether any linked value changed.
    ///
    /// # Errors
    ///
    /// Returns for a missing property, non-string value, or invalid edit range.
    pub(crate) fn apply_text_edit(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        edit: &argui_ui::TextEdit,
    ) -> Result<bool, RuntimeError> {
        self.apply_text_edit_inner(instance, property, edit, &mut HashSet::new())
    }

    /// Edits one linked property once, then forwards the same replacement.
    ///
    /// * `instance` — component containing the current property.
    /// * `property` — current linked string property.
    /// * `edit` — accepted replacement to apply.
    /// * `visited` — properties already edited during this propagation.
    ///
    /// Returns whether any visited value changed.
    ///
    /// # Errors
    ///
    /// Returns for a missing property, non-string value, or invalid edit range.
    fn apply_text_edit_inner(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        edit: &argui_ui::TextEdit,
        visited: &mut HashSet<(InstanceId, PropertyId)>,
    ) -> Result<bool, RuntimeError> {
        if !visited.insert((instance, property)) {
            return Ok(false);
        }
        let mounted = self
            .instances
            .get_mut(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let changed = mounted
            .properties
            .get_mut(&property)
            .ok_or(RuntimeError::MissingProperty(property.raw()))?
            .apply_text_edit(edit)?;
        let link = mounted.link(property);
        if let Some(link) = link {
            return self
                .apply_text_edit_inner(link.instance, link.property, edit, visited)
                .map(|linked| changed || linked);
        }
        Ok(changed)
    }
}
