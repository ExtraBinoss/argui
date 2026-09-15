use std::{cell::RefCell, collections::VecDeque, rc::Rc};

use argui_core::{Point, Rect};

mod frames;
mod memory;
mod records;
pub use memory::MemorySnapshot;
mod trace;
pub use frames::FrameCursor;
pub use records::{
    AdapterRecord, FrameRecord, GpuFrameRecord, GpuPassRecord, Invalidation, NodeSnapshot,
    PortalSnapshot, PropertySnapshot, TreeSnapshot,
};

use trace::TraceDocument;
pub use trace::{TRACE_VERSION, TraceError};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Stable identifier for a node in an inspected tree.
pub struct InspectNodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
/// Style attributes that can be overridden in an inspection session.
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
    /// All inspectable style properties in display order.
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

    /// Returns the lowercase label used for this property in inspector controls.
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
/// Unit used by an editable style length.
pub enum StyleUnit {
    #[default]
    Auto,
    Px,
    Percent,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
/// Numeric length value paired with its unit.
pub struct StyleLength {
    pub value: f32,
    pub unit: StyleUnit,
}

#[derive(Clone, Debug, PartialEq)]
/// Editable scalar exposed for a structured style value.
pub struct StyleField {
    pub label: String,
    pub value: f32,
}

#[derive(Clone, Debug, PartialEq)]
/// Serializable or editable representation of an inspected style value.
pub enum StyleValue {
    Length(StyleLength),
    Number(f32),
    Srgba([f32; 4]),
    Parameters(Vec<StyleField>),
    Choice(String),
    Summary(String),
}

impl StyleValue {
    /// Returns the editable scalar fields represented by this value.
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

    /// Updates the editable field at `index` to `value`, returning `false` when unsupported.
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

    /// Formats the value for compact display in inspector tools.
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
    /// Creates a shared inspector session retaining at most `capacity` frame records.
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

    /// Replaces the current inspected tree snapshot with `tree`.
    pub fn publish_tree(&self, tree: TreeSnapshot) {
        self.0.borrow_mut().tree = tree;
    }

    /// Clones and returns the current inspected tree snapshot.
    #[must_use]
    pub fn tree(&self) -> TreeSnapshot {
        self.0.borrow().tree.clone()
    }

    /// Runs `read` against the current tree without cloning the snapshot and returns its result.
    pub fn with_tree<T>(&self, read: impl FnOnce(&TreeSnapshot) -> T) -> T {
        read(&self.0.borrow().tree)
    }

    /// Returns a clone of the node with `id`, if it is present in the tree.
    #[must_use]
    pub fn node(&self, id: InspectNodeId) -> Option<NodeSnapshot> {
        self.with_tree(|tree| tree.nodes.iter().find(|node| node.id == id).cloned())
    }

    /// Records a UI-side frame, dropping the oldest record when the history is full.
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

    /// Adds renderer measurements to the latest UI frame, or records `record` if none exists.
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

    /// Returns a copy of the retained frame history in chronological order.
    #[must_use]
    pub fn frames(&self) -> Vec<FrameRecord> {
        self.0.borrow().frames.iter().cloned().collect()
    }

    /// Removes all retained frame records and starts a new synchronization epoch.
    pub fn clear_frames(&self) {
        let mut state = self.0.borrow_mut();
        state.frames.clear();
        state.frame_epoch = state.frame_epoch.wrapping_add(1);
    }

    /// Pauses or resumes inspector recording.
    /// `paused` is `true` to suspend recording and `false` to resume it.
    pub fn set_paused(&self, paused: bool) {
        self.0.borrow_mut().paused = paused;
    }

    /// Reports whether recording is paused.
    #[must_use]
    pub fn paused(&self) -> bool {
        self.0.borrow().paused
    }

    /// Enables or disables frame recording.
    pub fn set_recording(&self, recording: bool) {
        self.0.borrow_mut().recording = recording;
    }

    /// Reports whether recording is enabled and not paused.
    #[must_use]
    pub fn recording(&self) -> bool {
        let state = self.0.borrow();
        state.recording && !state.paused
    }

    /// Selects `node` for inspection, or clears the selection when it is `None`.
    pub fn select(&self, node: Option<InspectNodeId>) {
        self.0.borrow_mut().selected = node;
    }

    /// Returns the currently selected node, if any.
    #[must_use]
    pub fn selected(&self) -> Option<InspectNodeId> {
        self.0.borrow().selected
    }

    /// Sets the node currently under the inspector pointer, or clears it with `None`.
    pub fn set_hovered(&self, node: Option<InspectNodeId>) {
        self.0.borrow_mut().hovered = node;
    }

    /// Returns the currently hovered node, if any.
    #[must_use]
    pub fn highlighted(&self) -> Option<InspectNodeId> {
        self.0.borrow().hovered
    }

    /// Returns the visually foremost inspected node containing `point` within `viewport`.
    /// Later nodes win equal stacking levels, matching retained paint order;
    /// deeper nodes win inside the same branch.
    #[must_use]
    pub fn hit_test(&self, point: Point, viewport: Rect) -> Option<InspectNodeId> {
        self.hit_stack(point, viewport).into_iter().next()
    }

    /// Returns every inspected node under `point` within `viewport`, ordered from the most useful
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

    /// Enables or disables the override for this node/property, returning its new enabled state.
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

    /// Returns whether an override is enabled, or `None` if no override exists.
    /// `node` is the inspected node and `property` is the style attribute to query.
    #[must_use]
    pub fn property_enabled(&self, node: InspectNodeId, property: StyleProperty) -> Option<bool> {
        self.0.borrow().overrides.iter().find_map(|entry| {
            (entry.node == node && entry.property == property).then_some(entry.enabled)
        })
    }

    /// Sets and enables `value` as the explicit override for `node`'s `property`.
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

    /// Returns the explicit override value, if one has been set.
    /// `node` is the inspected node and `property` is the style attribute to query.
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

    /// Removes every style override from the inspection session.
    pub fn clear_overrides(&self) {
        self.0.borrow_mut().overrides.clear();
    }

    /// Nodes with edited or disabled properties; allows hosts to retain untouched subtrees.
    /// Returns the IDs of all nodes with an override entry.
    #[must_use]
    pub fn overridden_nodes(&self) -> std::collections::HashSet<InspectNodeId> {
        self.0
            .borrow()
            .overrides
            .iter()
            .map(|entry| entry.node)
            .collect()
    }

    /// Restore the authored value and enabled state of a single property.
    /// Removes the override for one node/property pair, if present.
    pub fn clear_property_override(&self, node: InspectNodeId, property: StyleProperty) {
        self.0
            .borrow_mut()
            .overrides
            .retain(|entry| entry.node != node || entry.property != property);
    }

    /// Serializes the current tree metadata, selection, and frame history as pretty JSON.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::InvalidJson`] if the trace cannot be serialized.
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

    /// Imports a trace's tree metadata, selected node, and frame history from JSON.
    ///
    /// # Arguments
    ///
    /// * `json` — JSON text previously produced by this crate's trace format.
    ///
    /// # Errors
    ///
    /// Returns [`TraceError::InvalidJson`] for malformed data and
    /// [`TraceError::UnsupportedVersion`] when the format version is unknown.
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
