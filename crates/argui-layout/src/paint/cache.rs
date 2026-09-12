use super::{PaintContext, ScrollPaintUpdate};
use crate::LayoutNode;
use argui_core::Rect;
use argui_ui::{Element, HitRegion, NodeId, UiTree};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub(super) struct CachedFragment {
    pub(super) element: Element,
    // Descendant geometry and text block indices can change while the root stays put.
    pub(super) nodes: Vec<LayoutNode>,
    pub(super) parent: PaintContext,
    pub(super) commands: Vec<argui_paint::DisplayCommand>,
    pub(super) hit_regions: Vec<HitRegion>,
    pub(super) semantic_bounds: Vec<(NodeId, Rect)>,
    pub(super) desktop_backdrops: Vec<crate::DesktopBackdropRegion>,
    pub(super) backdrop_state: argui_ui::DesktopBackdropState,
    pub(super) text_orders: Vec<(NodeId, usize)>,
    pub(super) scroll_updates: Vec<ScrollPaintUpdate>,
    pub(super) visual_revision: u64,
    pub(super) selection_active: bool,
    pub(super) selection_revision: u64,
}

#[derive(Default, Debug)]
pub(crate) struct PaintCache {
    pub(super) fragments: HashMap<NodeId, CachedFragment>,
    pub(crate) visited: usize,
    pub(crate) reused: usize,
    pub(crate) reused_commands: usize,
}

impl PaintCache {
    pub(crate) fn retain(&mut self, ui: &UiTree) {
        self.fragments.retain(|id, fragment| {
            let index = fragment.nodes[0].index;
            ui.node_id_at(index) == Some(*id)
                && ui
                    .element_at(index)
                    .is_some_and(|element| element.ptr_eq(&fragment.element))
        });
    }
}
