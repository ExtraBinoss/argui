use std::{collections::VecDeque, error::Error, fmt, time::Duration};

use serde::{Deserialize, Serialize};

use crate::{
    AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord, InspectNodeId, Invalidation,
};

pub const TRACE_VERSION: &str = "argui-gpu-trace-v1";

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TraceError {
    InvalidJson(String),
    UnsupportedVersion(String),
}

impl fmt::Display for TraceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => {
                write!(formatter, "invalid Argui GPU trace JSON: {message}")
            }
            Self::UnsupportedVersion(version) => {
                write!(formatter, "unsupported Argui GPU trace version '{version}'")
            }
        }
    }
}

impl Error for TraceError {}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct TraceDocument {
    version: String,
    tree_revision: u64,
    tree_nodes: usize,
    selected: Option<u64>,
    adapter: TraceAdapter,
    frames: Vec<TraceFrame>,
}

#[derive(Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceAdapter {
    name: String,
    vendor: u32,
    device: u32,
    device_type: String,
    driver: String,
    driver_info: String,
    backend: String,
    features: String,
    timestamp_queries: bool,
    max_texture_dimension_2d: u32,
    max_buffer_size: u64,
    max_storage_buffer_binding_size: u64,
    max_bind_groups: u32,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceFrame {
    interval_ns: u64,
    model_ns: u64,
    surface_ns: u64,
    tree_ns: u64,
    layout_ns: u64,
    paint_ns: u64,
    render_cpu_ns: u64,
    resize_events: u32,
    invalidation: TraceInvalidation,
    layers: usize,
    passes: usize,
    offscreen_pixels: u64,
    cached_layers: usize,
    damaged_pixels: u64,
    textures: usize,
    reused_textures: usize,
    texture_bytes: u64,
    gpu: Option<TraceGpuFrame>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceGpuFrame {
    sequence: u64,
    total_ns: u64,
    passes: Vec<TraceGpuPass>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceGpuPass {
    label: String,
    start_ns: u64,
    duration_ns: u64,
    pixels: u64,
    object_domain: Option<String>,
    object_id: Option<u64>,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
enum TraceInvalidation {
    None,
    Paint,
    Layout,
}

impl TraceDocument {
    pub(crate) fn validate_version(&self) -> Result<(), TraceError> {
        if self.version == TRACE_VERSION {
            Ok(())
        } else {
            Err(TraceError::UnsupportedVersion(self.version.clone()))
        }
    }

    pub(crate) fn capture(
        tree_revision: u64,
        tree_nodes: usize,
        selected: Option<InspectNodeId>,
        frames: &VecDeque<FrameRecord>,
    ) -> Self {
        let adapter = frames
            .back()
            .map(|frame| TraceAdapter::from(&frame.adapter))
            .unwrap_or_default();
        Self {
            version: TRACE_VERSION.into(),
            tree_revision,
            tree_nodes,
            selected: selected.map(|id| id.0),
            adapter,
            frames: frames.iter().map(TraceFrame::from).collect(),
        }
    }

    pub(crate) fn into_records(self) -> (u64, Option<InspectNodeId>, Vec<FrameRecord>) {
        let adapter = AdapterRecord::from(self.adapter);
        let frames = self
            .frames
            .into_iter()
            .map(|frame| frame.into_record(adapter.clone()))
            .collect();
        (self.tree_revision, self.selected.map(InspectNodeId), frames)
    }
}

impl From<&AdapterRecord> for TraceAdapter {
    fn from(value: &AdapterRecord) -> Self {
        Self {
            name: value.name.clone(),
            vendor: value.vendor,
            device: value.device,
            device_type: value.device_type.clone(),
            driver: value.driver.clone(),
            driver_info: value.driver_info.clone(),
            backend: value.backend.clone(),
            features: value.features.clone(),
            timestamp_queries: value.timestamp_queries,
            max_texture_dimension_2d: value.max_texture_dimension_2d,
            max_buffer_size: value.max_buffer_size,
            max_storage_buffer_binding_size: value.max_storage_buffer_binding_size,
            max_bind_groups: value.max_bind_groups,
        }
    }
}

impl From<TraceAdapter> for AdapterRecord {
    fn from(value: TraceAdapter) -> Self {
        Self {
            name: value.name,
            vendor: value.vendor,
            device: value.device,
            device_type: value.device_type,
            driver: value.driver,
            driver_info: value.driver_info,
            backend: value.backend,
            features: value.features,
            timestamp_queries: value.timestamp_queries,
            max_texture_dimension_2d: value.max_texture_dimension_2d,
            max_buffer_size: value.max_buffer_size,
            max_storage_buffer_binding_size: value.max_storage_buffer_binding_size,
            max_bind_groups: value.max_bind_groups,
        }
    }
}

impl From<&FrameRecord> for TraceFrame {
    fn from(value: &FrameRecord) -> Self {
        Self {
            interval_ns: nanos(value.interval),
            model_ns: nanos(value.model),
            surface_ns: nanos(value.surface),
            tree_ns: nanos(value.tree),
            layout_ns: nanos(value.layout),
            paint_ns: nanos(value.paint),
            render_cpu_ns: nanos(value.render_cpu),
            resize_events: value.resize_events,
            invalidation: TraceInvalidation::from(value.update),
            layers: value.layers,
            passes: value.passes,
            offscreen_pixels: value.offscreen_pixels,
            cached_layers: value.cached_layers,
            damaged_pixels: value.damaged_pixels,
            textures: value.textures,
            reused_textures: value.reused_textures,
            texture_bytes: value.texture_bytes,
            gpu: value.gpu.as_ref().map(TraceGpuFrame::from),
        }
    }
}

impl TraceFrame {
    fn into_record(self, adapter: AdapterRecord) -> FrameRecord {
        FrameRecord {
            interval: Duration::from_nanos(self.interval_ns),
            model: Duration::from_nanos(self.model_ns),
            surface: Duration::from_nanos(self.surface_ns),
            tree: Duration::from_nanos(self.tree_ns),
            layout: Duration::from_nanos(self.layout_ns),
            paint: Duration::from_nanos(self.paint_ns),
            render_cpu: Duration::from_nanos(self.render_cpu_ns),
            resize_events: self.resize_events,
            update: self.invalidation.into(),
            layers: self.layers,
            passes: self.passes,
            offscreen_pixels: self.offscreen_pixels,
            cached_layers: self.cached_layers,
            damaged_pixels: self.damaged_pixels,
            textures: self.textures,
            reused_textures: self.reused_textures,
            texture_bytes: self.texture_bytes,
            adapter,
            gpu: self.gpu.map(TraceGpuFrame::into_record),
        }
    }
}

impl From<&GpuFrameRecord> for TraceGpuFrame {
    fn from(value: &GpuFrameRecord) -> Self {
        Self {
            sequence: value.sequence,
            total_ns: nanos(value.total),
            passes: value.passes.iter().map(TraceGpuPass::from).collect(),
        }
    }
}

impl TraceGpuFrame {
    fn into_record(self) -> GpuFrameRecord {
        GpuFrameRecord {
            sequence: self.sequence,
            total: Duration::from_nanos(self.total_ns),
            passes: self
                .passes
                .into_iter()
                .map(TraceGpuPass::into_record)
                .collect(),
        }
    }
}

impl From<&GpuPassRecord> for TraceGpuPass {
    fn from(value: &GpuPassRecord) -> Self {
        Self {
            label: value.label.clone(),
            start_ns: nanos(value.start),
            duration_ns: nanos(value.duration),
            pixels: value.pixels,
            object_domain: value.object_domain.clone(),
            object_id: value.object_id,
        }
    }
}

impl TraceGpuPass {
    fn into_record(self) -> GpuPassRecord {
        GpuPassRecord {
            label: self.label,
            start: Duration::from_nanos(self.start_ns),
            duration: Duration::from_nanos(self.duration_ns),
            pixels: self.pixels,
            object_domain: self.object_domain,
            object_id: self.object_id,
        }
    }
}

fn nanos(duration: Duration) -> u64 {
    duration.as_nanos().min(u128::from(u64::MAX)) as u64
}

impl From<Invalidation> for TraceInvalidation {
    fn from(value: Invalidation) -> Self {
        match value {
            Invalidation::None => Self::None,
            Invalidation::Paint => Self::Paint,
            Invalidation::Layout => Self::Layout,
        }
    }
}

impl From<TraceInvalidation> for Invalidation {
    fn from(value: TraceInvalidation) -> Self {
        match value {
            TraceInvalidation::None => Self::None,
            TraceInvalidation::Paint => Self::Paint,
            TraceInvalidation::Layout => Self::Layout,
        }
    }
}
