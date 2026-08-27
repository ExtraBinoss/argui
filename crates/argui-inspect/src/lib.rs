use std::{cell::RefCell, collections::VecDeque, rc::Rc, time::Duration};

use argui_core::{Point, Rect};

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct InspectNodeId(pub u64);

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum StyleProperty {
    Background,
    Border,
    Opacity,
    Clip,
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
        Self::Clip,
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
            Self::Clip => "clip",
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
    Color([f32; 4]),
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
            Self::Color(values) => ["r", "g", "b", "a"]
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
            Self::Color(values) => values.get_mut(index).is_some_and(|current| {
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
            Self::Color([red, green, blue, alpha]) => {
                format!("rgba({red:.3}, {green:.3}, {blue:.3}, {alpha:.3})")
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
    pub visible: bool,
    pub properties: Vec<PropertySnapshot>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeSnapshot {
    pub revision: u64,
    pub nodes: Vec<NodeSnapshot>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct FrameRecord {
    pub interval: Duration,
    pub model: Duration,
    pub tree: Duration,
    pub paint: Duration,
    pub render_cpu: Duration,
    pub update: Invalidation,
    pub layers: usize,
    pub passes: usize,
    pub offscreen_pixels: u64,
    pub textures: usize,
    pub reused_textures: usize,
    pub texture_bytes: u64,
}

impl FrameRecord {
    #[must_use]
    pub fn total_cpu(self) -> Duration {
        self.model + self.tree + self.paint + self.render_cpu
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
    tree: TreeSnapshot,
    frames: VecDeque<FrameRecord>,
    capacity: usize,
    selected: Option<InspectNodeId>,
    overrides: Vec<StyleOverride>,
    paused: bool,
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
            tree: TreeSnapshot::default(),
            frames: VecDeque::with_capacity(capacity),
            capacity,
            selected: None,
            overrides: Vec::new(),
            paused: false,
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
        if state.paused || state.capacity == 0 {
            return;
        }
        if state.frames.len() == state.capacity {
            state.frames.pop_front();
        }
        state.frames.push_back(record);
    }

    pub fn record_render(&self, record: FrameRecord) {
        let mut state = self.0.borrow_mut();
        if state.paused || state.capacity == 0 {
            return;
        }
        if let Some(frame) = state.frames.back_mut() {
            frame.render_cpu = record.render_cpu;
            frame.layers = record.layers;
            frame.passes = record.passes;
            frame.offscreen_pixels = record.offscreen_pixels;
            frame.textures = record.textures;
            frame.reused_textures = record.reused_textures;
            frame.texture_bytes = record.texture_bytes;
        } else {
            state.frames.push_back(record);
        }
    }

    #[must_use]
    pub fn frames(&self) -> Vec<FrameRecord> {
        self.0.borrow().frames.iter().copied().collect()
    }

    pub fn clear_frames(&self) {
        self.0.borrow_mut().frames.clear();
    }

    pub fn set_paused(&self, paused: bool) {
        self.0.borrow_mut().paused = paused;
    }

    #[must_use]
    pub fn paused(&self) -> bool {
        self.0.borrow().paused
    }

    pub fn select(&self, node: Option<InspectNodeId>) {
        self.0.borrow_mut().selected = node;
    }

    #[must_use]
    pub fn selected(&self) -> Option<InspectNodeId> {
        self.0.borrow().selected
    }

    /// Returns the visually foremost inspected node containing `point`.
    /// Later nodes win equal stacking levels, matching retained paint order;
    /// deeper nodes win inside the same branch.
    #[must_use]
    pub fn hit_test(&self, point: Point, viewport: Rect) -> Option<InspectNodeId> {
        if !viewport.contains(point) {
            return None;
        }
        self.0
            .borrow()
            .tree
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| {
                node.visible
                    && node.bounds.contains(point)
                    && node.clip.is_none_or(|clip| clip.contains(point))
            })
            .max_by_key(|(order, node)| (node.z_index, node.depth, *order))
            .map(|(_, node)| node.id)
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

    #[must_use]
    pub fn trace_text(&self) -> String {
        let state = self.0.borrow();
        let mut output = String::from("argui-profile-v1\n");
        output.push_str(&format!(
            "tree revision={} nodes={} selected={:?}\n",
            state.tree.revision,
            state.tree.nodes.len(),
            state.selected.map(|id| id.0)
        ));
        for (index, frame) in state.frames.iter().enumerate() {
            output.push_str(&format!(
                "frame {index}: interval={:.3}ms model={:.3}ms tree={:.3}ms paint={:.3}ms render={:.3}ms invalidation={:?} layers={} passes={} offscreen_px={} textures={} reused={} bytes={}\n",
                frame.interval.as_secs_f64() * 1_000.0,
                frame.model.as_secs_f64() * 1_000.0,
                frame.tree.as_secs_f64() * 1_000.0,
                frame.paint.as_secs_f64() * 1_000.0,
                frame.render_cpu.as_secs_f64() * 1_000.0,
                frame.update,
                frame.layers,
                frame.passes,
                frame.offscreen_pixels,
                frame.textures,
                frame.reused_textures,
                frame.texture_bytes,
            ));
        }
        output
    }
}

impl Default for InspectorHandle {
    fn default() -> Self {
        Self::new(300)
    }
}
