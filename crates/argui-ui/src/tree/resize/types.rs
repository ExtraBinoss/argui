use std::collections::{HashMap, HashSet};

use argui_core::{PointerId, Size};

use crate::{Dimension, NodeId, ResizeAxis, ResizeHandle};

#[derive(Clone, Debug)]
pub(in crate::tree) struct Target {
    pub node: NodeId,
    pub config: ResizeHandle,
}

#[derive(Clone, Copy, Debug)]
pub(in crate::tree) struct Override {
    pub axis: ResizeAxis,
    pub authored: Dimension,
    pub value: f32,
}

#[derive(Clone, Debug)]
pub(in crate::tree) struct Drag {
    pub handle: NodeId,
    pub pointer: PointerId,
    pub target: Target,
    pub origin: f32,
    pub initial: f32,
    pub authored: Dimension,
    pub previous: Option<Override>,
}

#[derive(Clone, Debug, Default)]
pub(in crate::tree) struct ResizeState {
    pub targets: HashMap<NodeId, Target>,
    pub bounds: HashMap<NodeId, Size>,
    pub overrides: HashMap<NodeId, Override>,
    pub restored: HashSet<NodeId>,
    pub drag: Option<Drag>,
}
