use super::Inspection;
use argui_ui::{NodeId, UiTree};

impl Inspection {
    /// Reads retained memory capacities without walking nodes or shared data.
    ///
    /// `tree`, `engine`, and `output` provide the retained UI, layout storage, and
    /// most recent layout output whose capacities are measured.
    #[must_use]
    pub fn memory(
        tree: &UiTree,
        engine: &argui_layout::LayoutEngine,
        output: &argui_layout::LayoutOutput,
    ) -> argui_inspect::MemorySnapshot {
        let storage = engine.storage();
        argui_inspect::MemorySnapshot {
            ui_nodes: tree.node_ids().len(),
            layout_nodes: engine.retained_node_count(),
            ui_index_bytes: tree.index_storage_bytes(),
            layout_metadata_bytes: storage.metadata_bytes,
            layout_cache_bytes: storage.cache_bytes,
            layout_geometry_bytes: storage.geometry_bytes,
            layout_output_bytes: output.nodes.capacity() * size_of::<argui_layout::LayoutNode>()
                + output.semantic_bounds.capacity() * size_of::<(NodeId, argui_core::Rect)>()
                + output.hit_regions.capacity() * size_of::<argui_ui::HitRegion>()
                + output.scroll_regions.capacity() * size_of::<argui_ui::ScrollRegion>()
                + output.text_inputs.capacity() * size_of::<argui_layout::TextInputRegion>()
                + output.text_regions.capacity() * size_of::<argui_layout::TextRegion>(),
            paint_command_bytes: output.display_list.storage_bytes(),
        }
    }
}
