use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use argui_core::{Point, PointerId, Size};
use argui_ui::{HitRegion, NodeId, RetainedIdentity, ScrollRegion, UiTree, VisualStates};

/// Bounded scroll geometry in logical pixels for one retained scroll container.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObservedScroll {
    /// Visual offset clamped to the layout region's valid scroll range.
    pub offset: Point,
    /// Visible width and height of the scroll container.
    pub viewport: Size,
    /// Content extent, including at least the viewport's width and height.
    pub content: Size,
}

/// Read-only interaction state for one retained source identity.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct ObservedInteraction {
    /// Current hover, press, and focus states of the retained node.
    pub states: VisualStates,
    /// Current pointer position in the node's local coordinates, when interacting.
    pub pointer_position: Option<Point>,
    /// Current pointer position in window logical coordinates, when interacting.
    pub pointer_global_position: Option<Point>,
    /// Most recent press origin in local coordinates, retained after release
    /// until the next press or removal of the node.
    pub pressed_position: Option<Point>,
    /// Bounded scroll geometry when this node owns a layout scroll region.
    pub scroll: Option<ObservedScroll>,
    /// Size from the last completed layout, in logical pixels.
    pub measured: Option<Size>,
}

/// Cloneable view of retained interaction state during render and event dispatch.
#[derive(Clone, Default)]
pub struct ObservationReader {
    snapshot: Rc<RefCell<InteractionSnapshot>>,
    watched: Rc<RefCell<HashSet<RetainedIdentity>>>,
}

impl ObservationReader {
    /// Registers `identity` for sampling on future host interaction updates.
    ///
    /// Call this while building an event callback whose first `.get()` may run
    /// later. The reader's clones share this watch set.
    pub fn watch(&self, identity: &RetainedIdentity) {
        self.watched.borrow_mut().insert(identity.clone());
    }

    /// Reads `identity` and records it in the context's dependency set.
    ///
    /// Returns the idle state when no matching node exists in the current
    /// snapshot. Clones of this reader share the same dependency set.
    #[must_use]
    pub fn get(&self, identity: &RetainedIdentity) -> ObservedInteraction {
        self.watch(identity);
        self.snapshot.borrow().get(identity)
    }

    /// Creates a reader from the live `snapshot` cell and shared `watched` set.
    /// Returns a handle that keeps both available to render and event closures.
    pub(super) fn new(
        snapshot: Rc<RefCell<InteractionSnapshot>>,
        watched: Rc<RefCell<HashSet<RetainedIdentity>>>,
    ) -> Self {
        Self { snapshot, watched }
    }
}

/// Cached lookup from retained source identities to current UI nodes.
///
/// Custom hosts can reuse one index across pointer events. It scans a tree
/// only on first use or after the tree's structural revision changes.
#[derive(Default)]
pub struct SourceIdentityIndex {
    revision: Option<u64>,
    nodes: HashMap<RetainedIdentity, NodeId>,
}

impl SourceIdentityIndex {
    /// Samples `watched` identities from `tree` using the current `regions`.
    ///
    /// `scroll_regions` supply bounded scroll geometry. `primary_touch`
    /// selects the active or most recent touch ahead of the
    /// mouse. Returns an empty map without indexing the tree when `watched` is
    /// empty or `tree` is absent. Subsequent calls reuse the index until the
    /// tree's structural revision changes.
    pub fn observations(
        &mut self,
        tree: Option<&UiTree>,
        regions: &[HitRegion],
        scroll_regions: &[ScrollRegion],
        primary_touch: Option<PointerId>,
        watched: &HashSet<RetainedIdentity>,
    ) -> HashMap<RetainedIdentity, ObservedInteraction> {
        let Some(tree) = tree else {
            self.clear();
            return HashMap::new();
        };
        if watched.is_empty() {
            return HashMap::new();
        }
        if self.revision != Some(tree.revision()) {
            self.nodes.clear();
            for (index, node) in tree.node_ids().iter().copied().enumerate() {
                if let Some(identity) = tree
                    .element_at(index)
                    .and_then(|element| element.source_identity())
                {
                    self.nodes.insert(identity.clone(), node);
                }
            }
            self.revision = Some(tree.revision());
        }
        watched
            .iter()
            .filter_map(|identity| {
                self.nodes.get(identity).map(|node| {
                    (
                        identity.clone(),
                        ObservedInteraction::from_node_with_scroll(
                            tree,
                            *node,
                            regions,
                            scroll_regions,
                            primary_touch,
                        ),
                    )
                })
            })
            .collect()
    }

    /// Clears the cached identity lookup after replacing the UI tree.
    pub fn clear(&mut self) {
        self.revision = None;
        self.nodes.clear();
    }

    /// Returns the number of source identities indexed by the last nonempty sample.
    #[must_use]
    pub fn indexed_len(&self) -> usize {
        self.nodes.len()
    }
}

impl ObservedInteraction {
    /// Reads one node's visual state and local and window pointer coordinates from a UI tree.
    ///
    /// `tree` contains current interaction state, `node` identifies the target,
    /// `regions` provide its hit geometry, and `primary_touch` selects the
    /// active touch contact ahead of the mouse when one exists. Returns the
    /// node's current state; absent pointer coordinates are `None`.
    #[must_use]
    pub fn from_node(
        tree: &UiTree,
        node: argui_ui::NodeId,
        regions: &[HitRegion],
        primary_touch: Option<PointerId>,
    ) -> Self {
        Self::from_node_with_scroll(tree, node, regions, &[], primary_touch)
    }

    /// Reads one node's interaction state and bounded scroll geometry.
    ///
    /// `tree` contains current state, `node` identifies the target,
    /// `regions` provide local hit geometry, `scroll_regions` provide layout
    /// scroll geometry, and `primary_touch` selects the active touch ahead of
    /// the mouse. Coordinates use the same hovered or captured pointer; scroll
    /// state is idle when the node is not scrollable.
    #[must_use]
    pub fn from_node_with_scroll(
        tree: &UiTree,
        node: argui_ui::NodeId,
        regions: &[HitRegion],
        scroll_regions: &[ScrollRegion],
        primary_touch: Option<PointerId>,
    ) -> Self {
        let pointers = [primary_touch, Some(PointerId::MOUSE)];
        let pointer_position = pointers
            .into_iter()
            .flatten()
            .find_map(|pointer| tree.pointer_position(pointer, node, regions));
        let pointer_global_position = pointers
            .into_iter()
            .flatten()
            .find_map(|pointer| tree.pointer_global_position(pointer, node));
        let pressed_position = pointers
            .into_iter()
            .flatten()
            .find_map(|pointer| tree.pressed_position(pointer, node));
        Self {
            states: tree.visual_states(node),
            pointer_position,
            pointer_global_position,
            pressed_position,
            measured: None,
            scroll: scroll_regions
                .iter()
                .find(|region| region.node == node)
                .map(|region| ObservedScroll::from_region(tree, region)),
        }
    }
}

impl ObservedScroll {
    /// Samples one scroll region, clamping offset to its layout range.
    ///
    /// `tree` supplies the node's current visual offset and `region` supplies
    /// viewport size and maximum offset. Returns finite logical pixel values.
    fn from_region(tree: &UiTree, region: &ScrollRegion) -> Self {
        let extent = |value: f32| {
            if value.is_finite() {
                value.max(0.0)
            } else {
                0.0
            }
        };
        let viewport = Size::new(
            extent(region.bounds.size.width),
            extent(region.bounds.size.height),
        );
        let max = Point::new(extent(region.max_offset.x), extent(region.max_offset.y));
        let raw = tree.scroll_offset(region.node);
        let offset = Point::new(
            if raw.x.is_finite() {
                raw.x.clamp(0.0, max.x)
            } else {
                0.0
            },
            if raw.y.is_finite() {
                raw.y.clamp(0.0, max.y)
            } else {
                0.0
            },
        );
        Self {
            offset,
            viewport,
            content: Size::new(
                (viewport.width + max.x).min(f32::MAX),
                (viewport.height + max.y).min(f32::MAX),
            ),
        }
    }
}

/// One immutable view of the current retained tree's interaction state.
#[derive(Clone, Default)]
pub(crate) struct InteractionSnapshot(Rc<HashMap<RetainedIdentity, ObservedInteraction>>);

impl InteractionSnapshot {
    /// Samples `watched` source identities through `index` using current UI data.
    /// `regions` supplies local geometry, `scroll_regions` supplies layout
    /// scroll geometry, and `primary_touch` selects the touch
    /// contact ahead of the mouse. Returns an empty snapshot when no watched
    /// identity exists in `tree`.
    pub(crate) fn capture(
        tree: Option<&UiTree>,
        regions: &[HitRegion],
        scroll_regions: &[ScrollRegion],
        primary_touch: Option<PointerId>,
        watched: &HashSet<RetainedIdentity>,
        index: &mut SourceIdentityIndex,
    ) -> Self {
        Self(Rc::new(index.observations(
            tree,
            regions,
            scroll_regions,
            primary_touch,
            watched,
        )))
    }

    /// Adds sizes from the last completed layout for watched retained nodes.
    ///
    /// `measurements` pairs source identities with logical sizes. The snapshot
    /// stays immutable to readers and ignores identities not already watched.
    pub(crate) fn with_measured(
        mut self,
        measurements: impl IntoIterator<Item = (RetainedIdentity, Size)>,
    ) -> Self {
        let values = Rc::make_mut(&mut self.0);
        for (identity, size) in measurements {
            if let Some(state) = values.get_mut(&identity) {
                state.measured = Some(size);
            }
        }
        self
    }

    /// Returns the state for `identity`, or the idle state if it is absent.
    pub(super) fn get(&self, identity: &RetainedIdentity) -> ObservedInteraction {
        self.0.get(identity).copied().unwrap_or_default()
    }

    /// Compares `watched` identities in this snapshot and `next`.
    /// Returns the identities whose observed values differ.
    pub(crate) fn changed(
        &self,
        next: &Self,
        watched: &HashSet<RetainedIdentity>,
    ) -> HashSet<RetainedIdentity> {
        watched
            .iter()
            .filter(|identity| self.get(identity) != next.get(identity))
            .cloned()
            .collect()
    }
}
