use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use argui_core::{Point, Rect};

mod frames;
mod memory;
pub use memory::MemorySnapshot;
mod trace;
pub use frames::FrameCursor;

use trace::TraceDocument;
pub use trace::{TRACE_VERSION, TraceError};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InspectNodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StyleProperty {
    Background,
    Border,
    Opacity,
    Overflow,
    Transform,
    Layer,
    Effects,
    Width,
    Height,
}

impl StyleProperty {
    pub const ALL: [Self; 9] = [
        Self::Background,
        Self::Border,
        Self::Opacity,
        Self::Overflow,
        Self::Transform,
        Self::Layer,
        Self::Effects,
        Self::Width,
        Self::Height,
    ];

    pub const fn label(self) -> &'static str {
        match self {
            Self::Background => "background",
            Self::Border => "border",
            Self::Opacity => "opacity",
            Self::Overflow => "overflow",
            Self::Transform => "transform",
            Self::Layer => "layer",
            Self::Effects => "effects",
            Self::Width => "width",
            Self::Height => "height",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum StyleUnit {
    #[default]
    Auto,
    Px,
    Percent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct StyleLength {
    pub value: f32,
    pub unit: StyleUnit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct StyleField {
    pub label: String,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum StyleValue {
    Length(StyleLength),
    Number(f32),
    Srgba([f32; 4]),
    Parameters(Vec<StyleField>),
    Choice(String),
    Summary(String),
}

impl StyleValue {
    #[must_use]
    pub fn fields(&self) -> Vec<StyleField> {
        match self {
            Self::Length(length) if length.unit != StyleUnit::Auto => vec![StyleField {
                label: match length.unit {
                    StyleUnit::Px => "px",
                    StyleUnit::Percent => "%",
                    StyleUnit::Auto => unreachable!(),
                }
                .into(),
                value: if length.unit == StyleUnit::Percent {
                    length.value * 100.0
                } else {
                    length.value
                },
            }],
            Self::Number(value) => vec![StyleField {
                label: "value".into(),
                value: *value,
            }],
            Self::Srgba(values) => ["r", "g", "b", "a"]
                .into_iter()
                .zip(values)
                .map(|(label, value)| StyleField {
                    label: label.into(),
                    value: *value,
                })
                .collect(),
            Self::Parameters(fields) => fields.clone(),
            Self::Length(_) | Self::Choice(_) | Self::Summary(_) => Vec::new(),
        }
    }

    pub fn set_field(&mut self, index: usize, value: f32) -> bool {
        match self {
            Self::Length(length) if index == 0 && length.unit != StyleUnit::Auto => {
                length.value = if length.unit == StyleUnit::Percent {
                    value / 100.0
                } else {
                    value
                };
                true
            }
            Self::Number(current) if index == 0 => {
                *current = value;
                true
            }
            Self::Srgba(values) => values.get_mut(index).is_some_and(|current| {
                *current = value.clamp(0.0, 1.0);
                true
            }),
            Self::Parameters(fields) => fields.get_mut(index).is_some_and(|field| {
                field.value = value;
                true
            }),
            _ => false,
        }
    }

    #[must_use]
    pub fn summary(&self) -> String {
        match self {
            Self::Length(length) => match length.unit {
                StyleUnit::Auto => "auto".into(),
                StyleUnit::Px => format!("{:.2}px", length.value),
                StyleUnit::Percent => format!("{:.2}%", length.value * 100.0),
            },
            Self::Number(value) => format!("{value:.3}"),
            Self::Srgba([red, green, blue, alpha]) => {
                format!("srgba({red:.3}, {green:.3}, {blue:.3}, {alpha:.3})")
            }
            Self::Parameters(fields) => format!("{} parameters", fields.len()),
            Self::Choice(value) | Self::Summary(value) => value.clone(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct PropertySnapshot {
    pub property: StyleProperty,
    pub authored: bool,
    pub value: StyleValue,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortalSnapshot {
    pub layer: String,
    pub anchor: Option<String>,
    pub requested: Option<String>,
    pub resolved: Option<String>,
    pub available_size: argui_core::Size,
    pub constrained_width: bool,
    pub constrained_height: bool,
}

#[derive(Clone, Debug, PartialEq)]
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
pub struct TreeSnapshot {
    pub revision: u64,
    pub nodes: Vec<NodeSnapshot>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
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
pub struct GpuPassRecord {
    pub label: String,
    pub start: Duration,
    pub duration: Duration,
    pub pixels: u64,
    pub object_domain: Option<String>,
    pub object_id: Option<u64>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct GpuFrameRecord {
    pub sequence: u64,
    pub total: Duration,
    pub passes: Vec<GpuPassRecord>,
}

#[derive(Clone, Debug, Default, PartialEq)]
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
    #[must_use]
    pub fn total_cpu(&self) -> Duration {
        self.model + self.surface + self.tree + self.layout + self.paint + self.render_cpu
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Invalidation {
    #[default]
    None,
    Paint,
    Layout,
}

#[derive(Clone, Debug)]
struct InspectorState {
    memory: Option<MemorySnapshot>,
    memory_requested: bool,
    tree: TreeSnapshot,
    frames: VecDeque<FrameRecord>,
    capacity: usize,
    selected: Option<InspectNodeId>,
    hovered: Option<InspectNodeId>,
    overrides: Vec<StyleOverride>,
    paused: bool,
    recording: bool,
    gpu_profiling: bool,
    frame_sequence: u64,
    frame_epoch: u64,
}

#[derive(Clone, Debug)]
struct StyleOverride {
    node: InspectNodeId,
    property: StyleProperty,
    enabled: bool,
    value: Option<StyleValue>,
}

#[derive(Clone, Debug)]
pub struct InspectorHandle(Rc<RefCell<InspectorState>>);

impl InspectorHandle {
    #[must_use]
    pub fn new(capacity: usize) -> Self {
        Self(Rc::new(RefCell::new(InspectorState {
            memory: None,
            memory_requested: false,
            tree: TreeSnapshot::default(),
            frames: VecDeque::with_capacity(capacity),
            capacity,
            selected: None,
            hovered: None,
            overrides: Vec::new(),
            paused: false,
            recording: true,
            gpu_profiling: true,
            frame_sequence: 0,
            frame_epoch: 0,
        })))
    }

    pub fn publish_tree(&self, tree: TreeSnapshot) {
        self.0.borrow_mut().tree = tree;
    }

    #[must_use]
    pub fn tree(&self) -> TreeSnapshot {
        self.0.borrow().tree.clone()
    }

    pub fn with_tree<T>(&self, read: impl FnOnce(&TreeSnapshot) -> T) -> T {
        read(&self.0.borrow().tree)
    }

    #[must_use]
    pub fn node(&self, id: InspectNodeId) -> Option<NodeSnapshot> {
        self.with_tree(|tree| tree.nodes.iter().find(|node| node.id == id).cloned())
    }

    pub fn record_ui(&self, record: FrameRecord) {
        let mut state = self.0.borrow_mut();
        if state.paused || !state.recording || state.capacity == 0 {
            return;
        }
        if state.frames.len() == state.capacity {
            state.frames.pop_front();
        }
        state.frames.push_back(record);
        state.frame_sequence = state.frame_sequence.wrapping_add(1);
    }

    pub fn record_render(&self, record: FrameRecord) {
        let mut state = self.0.borrow_mut();
        if state.paused || !state.recording || state.capacity == 0 {
            return;
        }
        if let Some(frame) = state.frames.back_mut() {
            frame.render_cpu = record.render_cpu;
            frame.layers = record.layers;
            frame.passes = record.passes;
            frame.offscreen_pixels = record.offscreen_pixels;
            frame.cached_layers = record.cached_layers;
            frame.damaged_pixels = record.damaged_pixels;
            frame.textures = record.textures;
            frame.reused_textures = record.reused_textures;
            frame.texture_bytes = record.texture_bytes;
            frame.vector_atlas_bytes = record.vector_atlas_bytes;
            frame.vector_atlas_entries = record.vector_atlas_entries;
            frame.vector_atlas_hits = record.vector_atlas_hits;
            frame.vector_rasterizations = record.vector_rasterizations;
            frame.adapter = record.adapter;
            frame.gpu = record.gpu;
        } else {
            state.frames.push_back(record);
            state.frame_sequence = state.frame_sequence.wrapping_add(1);
        }
    }

    #[must_use]
    pub fn frames(&self) -> Vec<FrameRecord> {
        self.0.borrow().frames.iter().cloned().collect()
    }

    pub fn clear_frames(&self) {
        let mut state = self.0.borrow_mut();
        state.frames.clear();
        state.frame_epoch = state.frame_epoch.wrapping_add(1);
    }

    pub fn set_paused(&self, paused: bool) {
        self.0.borrow_mut().paused = paused;
    }

    #[must_use]
    pub fn paused(&self) -> bool {
        self.0.borrow().paused
    }

    pub fn set_recording(&self, recording: bool) {
        self.0.borrow_mut().recording = recording;
    }

    #[must_use]
    pub fn recording(&self) -> bool {
        let state = self.0.borrow();
        state.recording && !state.paused
    }

    pub fn select(&self, node: Option<InspectNodeId>) {
        self.0.borrow_mut().selected = node;
    }

    #[must_use]
    pub fn selected(&self) -> Option<InspectNodeId> {
        self.0.borrow().selected
    }

    pub fn set_hovered(&self, node: Option<InspectNodeId>) {
        self.0.borrow_mut().hovered = node;
    }

    #[must_use]
    pub fn highlighted(&self) -> Option<InspectNodeId> {
        let state = self.0.borrow();
        state.hovered.or(state.selected)
    }

    /// Returns the visually foremost inspected node containing `point`.
    /// Later nodes win equal stacking levels, matching retained paint order;
    /// deeper nodes win inside the same branch.
    #[must_use]
    pub fn hit_test(&self, point: Point, viewport: Rect) -> Option<InspectNodeId> {
        self.hit_stack(point, viewport).into_iter().next()
    }

    /// Returns every inspected node under `point`, ordered from the most useful
    /// visual target to structural fallbacks.
    #[must_use]
    pub fn hit_stack(&self, point: Point, viewport: Rect) -> Vec<InspectNodeId> {
        if !viewport.contains(point) {
            return Vec::new();
        }
        let state = self.0.borrow();
        let mut nodes = state
            .tree
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.visible
                    && node.bounds.contains(point)
                    && node.clip.is_none_or(|clip| clip.contains(point))
            })
            .collect::<Vec<_>>();
        nodes.sort_by_key(|(order, node)| {
            (
                node.painted || node.interactive,
                node.z_index,
                node.child_count != 0,
                node.depth,
                *order,
            )
        });
        nodes.into_iter().rev().map(|(_, node)| node.id).collect()
    }

    pub fn toggle(&self, node: InspectNodeId, property: StyleProperty) -> bool {
        let mut state = self.0.borrow_mut();
        if let Some(entry) = state
            .overrides
            .iter_mut()
            .find(|entry| entry.node == node && entry.property == property)
        {
            entry.enabled = !entry.enabled;
            entry.enabled
        } else {
            state.overrides.push(StyleOverride {
                node,
                property,
                enabled: false,
                value: None,
            });
            false
        }
    }

    #[must_use]
    pub fn property_enabled(&self, node: InspectNodeId, property: StyleProperty) -> Option<bool> {
        self.0.borrow().overrides.iter().find_map(|entry| {
            (entry.node == node && entry.property == property).then_some(entry.enabled)
        })
    }

    pub fn set_property_value(
        &self,
        node: InspectNodeId,
        property: StyleProperty,
        value: StyleValue,
    ) {
        let mut state = self.0.borrow_mut();
        if let Some(entry) = state
            .overrides
            .iter_mut()
            .find(|entry| entry.node == node && entry.property == property)
        {
            entry.enabled = true;
            entry.value = Some(value);
        } else {
            state.overrides.push(StyleOverride {
                node,
                property,
                enabled: true,
                value: Some(value),
            });
        }
    }

    #[must_use]
    pub fn property_value(
        &self,
        node: InspectNodeId,
        property: StyleProperty,
    ) -> Option<StyleValue> {
        self.0
            .borrow()
            .overrides
            .iter()
            .find(|entry| entry.node == node && entry.property == property)
            .and_then(|entry| entry.value.clone())
    }

    pub fn clear_overrides(&self) {
        self.0.borrow_mut().overrides.clear();
    }

    /// Restore the authored value and enabled state of a single property.
    pub fn clear_property_override(&self, node: InspectNodeId, property: StyleProperty) {
        self.0
            .borrow_mut()
            .overrides
            .retain(|entry| entry.node != node || entry.property != property);
    }

    pub fn trace_json(&self) -> Result<String, TraceError> {
        let state = self.0.borrow();
        serde_json::to_string_pretty(&TraceDocument::capture(
            state.tree.revision,
            state.tree.nodes.len(),
            state.selected,
            &state.frames,
        ))
        .map_err(|error| TraceError::InvalidJson(error.to_string()))
    }

    pub fn import_trace_json(&self, json: &str) -> Result<(), TraceError> {
        let document: TraceDocument = serde_json::from_str(json)
            .map_err(|error| TraceError::InvalidJson(error.to_string()))?;
        document.validate_version()?;
        let mut state = self.0.borrow_mut();
        let (revision, selected, frames) = document.into_records();
        state.frames = frames.into();
        state.frame_epoch = state.frame_epoch.wrapping_add(1);
        state.tree.revision = revision;
        state.selected = selected;
        Ok(())
    }
}

impl Default for InspectorHandle {
    fn default() -> Self {
        Self::new(300)
    }
}
