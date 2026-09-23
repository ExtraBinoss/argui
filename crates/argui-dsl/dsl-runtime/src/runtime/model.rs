//! Public live model operations without full collection replacement.

use argui_dsl_ir::PropertyId;

use super::LiveRuntime;
use crate::{DslValue, InstanceId, RuntimeError};

impl LiveRuntime {
    /// Returns a typed model snapshot retaining row identities and revision.
    ///
    /// `instance` and `property` select a declared `model<T>` property. The
    /// snapshot owns decoded rows and can be edited independently; call the
    /// generated row methods to commit edits to the mounted live component.
    ///
    /// # Errors
    ///
    /// Returns for a missing component/property, invalid model data or a row
    /// value incompatible with `T`.
    pub fn model_snapshot<T: crate::LiveValue + Clone>(
        &self,
        instance: InstanceId,
        property: PropertyId,
    ) -> Result<argui_reactive::Model<T>, RuntimeError> {
        let mounted = self
            .instances
            .get(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let property = mounted
            .properties
            .get(&property)
            .ok_or(RuntimeError::MissingProperty(property.raw()))?;
        let ids = property
            .model_row_ids()
            .ok_or_else(|| RuntimeError::TypeMismatch {
                expected: "model<T>".into(),
                actual: format!("{:?}", property.value_type),
            })?;
        let DslValue::Array(rows) = property.get() else {
            return Err(RuntimeError::TypeMismatch {
                expected: "model row array".into(),
                actual: property.get().type_name().into(),
            });
        };
        if ids.len() != rows.len() {
            return Err(RuntimeError::IncompatiblePackage(
                "model row identities and values differ".into(),
            ));
        }
        let rows = ids
            .iter()
            .copied()
            .zip(rows.iter().cloned())
            .map(|(id, value)| T::from_dsl(value).map(|row| (id, row)))
            .collect::<Result<Vec<_>, _>>()?;
        argui_reactive::Model::from_identified_rows(rows, property.revision())
            .ok_or_else(|| RuntimeError::IncompatiblePackage("invalid model row identities".into()))
    }

    /// Returns the current model row and its stable identity at `index`.
    ///
    /// # Errors
    ///
    /// Returns an error for a missing instance or property.
    pub fn model_row(
        &self,
        instance: InstanceId,
        property: PropertyId,
        index: usize,
    ) -> Result<Option<(u64, DslValue)>, RuntimeError> {
        let mounted = self
            .instances
            .get(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?;
        let value = mounted
            .properties
            .get(&property)
            .ok_or(RuntimeError::MissingProperty(property.raw()))?;
        if !matches!(value.value_type, argui_dsl_ir::IrType::Model(_)) {
            return Err(RuntimeError::TypeMismatch {
                expected: "model<T>".into(),
                actual: format!("{:?}", value.value_type),
            });
        }
        let Some(id) = value.model_row_id(index) else {
            return Ok(None);
        };
        let DslValue::Array(rows) = value.get() else {
            return Err(RuntimeError::TypeMismatch {
                expected: "model row array".into(),
                actual: value.get().type_name().into(),
            });
        };
        Ok(rows.get(index).cloned().map(|row| (id, row)))
    }

    /// Inserts `row` into a model property at `index` without replacing the array.
    /// Returns its stable ID or `None` outside the valid insertion range.
    ///
    /// # Errors
    ///
    /// Returns for a missing instance/property, wrong type, or exhausted identity.
    pub fn model_insert(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        index: usize,
        row: DslValue,
    ) -> Result<Option<u64>, RuntimeError> {
        self.model_property(instance, property)?
            .model_insert(index, row)
    }

    /// Removes a row at `index`, returning its stable ID and value if present.
    ///
    /// # Errors
    ///
    /// Returns for a missing instance/property or a non-model type.
    pub fn model_remove(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        index: usize,
    ) -> Result<Option<(u64, DslValue)>, RuntimeError> {
        self.model_property(instance, property)?.model_remove(index)
    }

    /// Moves a row from `from` to final index `to`, retaining its stable ID.
    /// Returns whether it moved.
    ///
    /// # Errors
    ///
    /// Returns for a missing instance/property or a non-model type.
    pub fn model_move(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        from: usize,
        to: usize,
    ) -> Result<bool, RuntimeError> {
        self.model_property(instance, property)?
            .model_move(from, to)
    }

    /// Replaces the row at `index` with `row` while retaining its stable ID.
    /// Returns whether its value changed.
    ///
    /// # Errors
    ///
    /// Returns for a missing instance/property or an incompatible value.
    pub fn model_update(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
        index: usize,
        row: DslValue,
    ) -> Result<bool, RuntimeError> {
        self.model_property(instance, property)?
            .model_update(index, row)
    }

    /// Resolves one writable dynamic property for a model operation.
    ///
    /// # Errors
    ///
    /// Returns a missing instance or property error.
    fn model_property(
        &mut self,
        instance: InstanceId,
        property: PropertyId,
    ) -> Result<&mut crate::DynamicProperty, RuntimeError> {
        self.instances
            .get_mut(&instance)
            .ok_or(RuntimeError::MissingComponent(instance.raw()))?
            .properties
            .get_mut(&property)
            .ok_or(RuntimeError::MissingProperty(property.raw()))
    }
}
