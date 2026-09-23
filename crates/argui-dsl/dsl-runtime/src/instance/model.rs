//! Direct row edits for live model properties.

use argui_dsl_ir::IrType;

use super::DynamicProperty;
use crate::{DslValue, RuntimeError};

/// Creates sequential row IDs only for a checked `model<T>` value.
///
/// `value_type` is the declared property type and `value` its current value.
/// Returns IDs and the next allocatable ID; ordinary arrays receive no IDs.
pub(super) fn initial_ids(value_type: &IrType, value: &DslValue) -> (Vec<u64>, u64) {
    if matches!(value_type, IrType::Model(_))
        && let DslValue::Array(rows) = value
    {
        let count = rows.len() as u64;
        return ((1..=count).collect(), count.saturating_add(1).max(1));
    }
    (Vec::new(), 1)
}

impl DynamicProperty {
    /// Returns the stable identity of a model row at `index`.
    #[must_use]
    pub fn model_row_id(&self, index: usize) -> Option<u64> {
        self.model_ids.get(index).copied()
    }

    /// Returns the stable model row identities without cloning the collection.
    #[must_use]
    pub fn model_row_ids(&self) -> Option<&[u64]> {
        matches!(self.value_type, IrType::Model(_)).then_some(self.model_ids.as_slice())
    }

    /// Inserts one checked row into a model property.
    ///
    /// `index` is a position from zero through length; `row` must match `T`.
    /// Returns the new row ID or `None` if the index is out of bounds.
    ///
    /// # Errors
    ///
    /// Returns a type error for an array or incompatible row value, or an ID exhaustion error.
    pub fn model_insert(
        &mut self,
        index: usize,
        row: DslValue,
    ) -> Result<Option<u64>, RuntimeError> {
        let row_type = self.model_item_type()?.clone();
        if !row.compatible_with(&row_type) {
            return Err(RuntimeError::TypeMismatch {
                expected: format!("{row_type:?}"),
                actual: row.type_name().into(),
            });
        }
        let id = self.next_model_id;
        let next = id.checked_add(1).ok_or_else(|| {
            RuntimeError::IncompatiblePackage("model row identity exhausted".into())
        })?;
        let rows = self.model_rows_mut()?;
        if index > rows.len() {
            return Ok(None);
        }
        rows.insert(index, row.coerce(&row_type));
        self.model_ids.insert(index, id);
        self.next_model_id = next;
        self.changed_model();
        Ok(Some(id))
    }

    /// Removes the model row at `index`, returning its ID and value if present.
    ///
    /// # Errors
    ///
    /// Returns a type error when this property is an ordinary array or scalar.
    pub fn model_remove(&mut self, index: usize) -> Result<Option<(u64, DslValue)>, RuntimeError> {
        self.model_item_type()?;
        let rows = self.model_rows_mut()?;
        if index >= rows.len() {
            return Ok(None);
        }
        let row = rows.remove(index);
        let id = self.model_ids.remove(index);
        self.changed_model();
        Ok(Some((id, row)))
    }

    /// Moves a model row from `from` to final index `to`, preserving its ID.
    /// Returns false for invalid or equal positions.
    ///
    /// # Errors
    ///
    /// Returns a type error when this property is not `model<T>`.
    pub fn model_move(&mut self, from: usize, to: usize) -> Result<bool, RuntimeError> {
        self.model_item_type()?;
        let rows = self.model_rows_mut()?;
        if from >= rows.len() || to >= rows.len() || from == to {
            return Ok(false);
        }
        let row = rows.remove(from);
        rows.insert(to, row);
        let id = self.model_ids.remove(from);
        self.model_ids.insert(to, id);
        self.changed_model();
        Ok(true)
    }

    /// Replaces a model row at `index` while retaining its stable identity.
    /// Returns false for an invalid index or equal value.
    ///
    /// # Errors
    ///
    /// Returns a type error for a non-model property or incompatible row.
    pub fn model_update(&mut self, index: usize, row: DslValue) -> Result<bool, RuntimeError> {
        let row_type = self.model_item_type()?.clone();
        if !row.compatible_with(&row_type) {
            return Err(RuntimeError::TypeMismatch {
                expected: format!("{row_type:?}"),
                actual: row.type_name().into(),
            });
        }
        let rows = self.model_rows_mut()?;
        let Some(current) = rows.get_mut(index) else {
            return Ok(false);
        };
        let row = row.coerce(&row_type);
        if *current == row {
            return Ok(false);
        }
        *current = row;
        self.changed_model();
        Ok(true)
    }

    /// Returns the declared item type for this model property.
    ///
    /// # Errors
    ///
    /// Returns a type mismatch when the property is not `model<T>`.
    fn model_item_type(&self) -> Result<&IrType, RuntimeError> {
        match &self.value_type {
            IrType::Model(item) => Ok(item),
            other => Err(RuntimeError::TypeMismatch {
                expected: "model<T>".into(),
                actual: format!("{other:?}"),
            }),
        }
    }

    /// Returns the mutable row array backing a checked model property.
    ///
    /// # Errors
    ///
    /// Returns a mismatch for a malformed live value.
    fn model_rows_mut(&mut self) -> Result<&mut Vec<DslValue>, RuntimeError> {
        let actual = self.value.type_name();
        match &mut self.value {
            DslValue::Array(rows) => Ok(rows),
            _ => Err(RuntimeError::TypeMismatch {
                expected: "model row array".into(),
                actual: actual.into(),
            }),
        }
    }

    /// Marks an accepted row edit as an explicit property update.
    fn changed_model(&mut self) {
        self.modified = true;
        self.revision = self.revision.wrapping_add(1);
    }
}
