//! Read-only validation for native host transactions.

use crate::Operation;

use super::{Host, HostError, Stage};

impl Host {
    /// Checks `operations` against the current graph without publishing their changes.
    ///
    /// `operations` may build detached nodes and replace the root. Returns only
    /// after every changed node and the resulting root can be constructed.
    ///
    /// # Errors
    /// Returns a host or schema error for an invalid identity, structure, or
    /// native property; the host remains unchanged.
    pub fn validate(&self, operations: &[Operation]) -> Result<(), HostError> {
        let mut stage = Stage::new(self);
        for operation in operations {
            stage.apply(operation)?;
        }
        let changed_ids = stage
            .changes
            .iter()
            .filter_map(|(id, node)| node.as_ref().map(|_| *id))
            .collect::<Vec<_>>();
        for id in changed_ids {
            stage.materialize(id)?;
        }
        if let Some(root) = stage.root {
            stage.materialize(root)?;
        }
        Ok(())
    }
}
