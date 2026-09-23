//! Validation of keyed row collections before mounting any row.

use crate::{ComponentInstance, DslValue, InstanceId, LiveRuntime, RuntimeError};
use argui_dsl_ir::{IrExpression, LocalId, SiteId};
use std::collections::{HashMap, HashSet};

impl LiveRuntime {
    /// Evaluates `model` and each row's `key` in `locals`, binding `local` to its item.
    /// Returns ordered item/key pairs scoped to `owner` and `site`, or a type,
    /// expression, or duplicate-key error before row rendering starts.
    #[allow(clippy::too_many_arguments)]
    pub(super) fn repeater_rows(
        &mut self,
        instance: &mut ComponentInstance,
        model: &IrExpression,
        key: &IrExpression,
        local: LocalId,
        locals: &HashMap<LocalId, DslValue>,
        owner: InstanceId,
        site: SiteId,
    ) -> Result<Vec<(DslValue, DslValue)>, RuntimeError> {
        let value = self.evaluate(instance, model, locals)?;
        let DslValue::Array(items) = value else {
            return Err(RuntimeError::TypeMismatch {
                expected: "model/array".into(),
                actual: value.type_name().into(),
            });
        };
        let mut seen = HashSet::new();
        let mut rows = Vec::with_capacity(items.len());
        let mut nested = locals.clone();
        for item in items {
            nested.insert(local, item.clone());
            let value = self.evaluate(instance, key, &nested)?;
            if !seen.insert(super::identity::retained(owner, site, Some(&value))?) {
                return Err(RuntimeError::InvalidBytecode(format!(
                    "duplicate repeater key {value:?} at site {}",
                    site.raw()
                )));
            }
            rows.push((item, value));
        }
        Ok(rows)
    }
}
