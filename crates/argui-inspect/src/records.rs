use std::time::Duration;

use argui_core::{Rect, Size};

use crate::{InspectNodeId, StyleProperty, StyleValue};

#[derive(Clone, Debug, PartialEq)]
/// Captured authored/overridden value for one style property.
pub struct PropertySnapshot {
    pub property: StyleProperty,
    pub authored: bool,
    pub value: StyleValue,
}

#[derive(Clone, Debug, PartialEq)]
/// Captured portal placement and available-space information.
pub struct PortalSnapshot {
    pub layer: String,
    pub anchor: Option<String>,
    pub requested: Option<String>,
    pub resolved: Option<String>,
    pub available_size: Size,
    pub constrained_width: bool,
    pub constrained_height: bool,
}

#[derive(Clone, Debug, PartialEq)]
/// Captured geometry, semantics, and style summary for one UI node.
pub struct NodeSnapshot {
    pub id: InspectNodeId,
    pub parent: Option<InspectNodeId>,
    pub depth: usize,
    pub key: Option<String>,
    pub kind: String,
    pub summary: Option<String>,
    pub bounds: Rect,
    pub clip: Option<Rect>,
    pub z_index: i32,
    pub portal: Option<PortalSnapshot>,
    pub visible: bool,
    pub painted: bool,
    pub interactive: bool,
    pub child_count: usize,
    pub properties: Vec<PropertySnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Revisioned retained snapshot of the inspected UI tree.
pub struct TreeSnapshot {
    pub revision: u64,
    pub nodes: Vec<NodeSnapshot>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
/// Graphics adapter identity and capability details associated with a frame.
pub struct AdapterRecord {
    pub name: String,
    pub vendor: u32,
    pub device: u32,
    pub device_type: String,
    pub driver: String,
    pub driver_info: String,
    pub backend: String,
    pub features: String,
    pub timestamp_queries: bool,
    pub max_texture_dimension_2d: u32,
    pub max_buffer_size: u64,
    pub max_storage_buffer_binding_size: u64,
    pub max_bind_groups: u32,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// Timing and workload details for one GPU render pass.
pub struct GpuPassRecord {
    pub label: String,
    pub start: Duration,
    pub duration: Duration,
    pub pixels: u64,
    pub object_domain: Option<String>,
    pub object_id: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// GPU timings and pass records associated with one frame.
pub struct GpuFrameRecord {
    pub sequence: u64,
    pub total: Duration,
    pub passes: Vec<GpuPassRecord>,
}

#[derive(Clone, Debug, Default, PartialEq)]
/// CPU, renderer, resource, and optional GPU measurements for one frame.
pub struct FrameRecord {
    pub interval: Duration,
    pub model: Duration,
    pub surface: Duration,
    pub tree: Duration,
    pub layout: Duration,
    pub paint: Duration,
    pub render_cpu: Duration,
    pub resize_events: u32,
    pub update: Invalidation,
    pub layers: usize,
    pub passes: usize,
    pub offscreen_pixels: u64,
    pub cached_layers: usize,
    pub damaged_pixels: u64,
    pub textures: usize,
    pub reused_textures: usize,
    pub texture_bytes: u64,
    pub vector_atlas_bytes: u64,
    pub vector_atlas_entries: usize,
    pub vector_atlas_hits: usize,
    pub vector_rasterizations: usize,
    pub adapter: AdapterRecord,
    pub gpu: Option<GpuFrameRecord>,
}

impl FrameRecord {
    /// Sums the recorded CPU-side model, surface, tree, layout, paint, and render times.
    #[must_use]
    pub fn total_cpu(&self) -> Duration {
        self.model + self.surface + self.tree + self.layout + self.paint + self.render_cpu
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
/// Strongest requested UI invalidation recorded for a frame.
pub enum Invalidation {
    #[default]
    None,
    Paint,
    Layout,
}
