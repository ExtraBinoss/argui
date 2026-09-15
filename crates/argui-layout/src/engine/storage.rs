use super::{LayoutEngine, NodeId};

/// Dense allocated layout storage. Excludes nested styles, child arrays, maps,
/// custom state and paint caches; querying capacities never traverses the tree.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct LayoutStorage {
    pub metadata_bytes: usize,
    pub cache_bytes: usize,
    pub geometry_bytes: usize,
}

impl LayoutEngine {
    /// Reports capacities of dense layout storage owned by this engine.
    #[must_use]
    pub fn storage(&self) -> LayoutStorage {
        let mut storage = self.tree.storage();
        storage.metadata_bytes += self.nodes_by_index.capacity() * size_of::<NodeId>();
        storage
    }
}
