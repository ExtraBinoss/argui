use std::collections::HashMap;
use taffy::{
    NodeId, Style, TaffyError,
    tree::{Cache, Layout},
};

mod algorithms;
mod boundary;

const NONE: u32 = u32::MAX;

fn compact_index(index: usize) -> u32 {
    assert!(
        index < NONE as usize,
        "layout exceeds compact index capacity"
    );
    index as u32
}

#[derive(Debug)]
struct Node {
    style: Style,
    context: u32,
    children: Box<[NodeId]>,
    parent: u32,
}

/// Argui owns retained graph storage; Taffy's low-level algorithms own CSS layout.
/// Geometry and measurement caches are dense columns, separate from cold node
/// metadata. Swap removal keeps storage proportional to the live graph; stable
/// Taffy IDs go through a small map and never alias a subsequently created node.
#[derive(Debug, Default)]
pub(crate) struct LayoutTree {
    positions: HashMap<NodeId, usize>,
    ids: Vec<NodeId>,
    nodes: Vec<Node>,
    caches: Vec<Cache>,
    unrounded: Vec<Layout>,
    layouts: Vec<Layout>,
    boundaries: HashMap<NodeId, boundary::Boundary>,
    custom: HashMap<
        NodeId,
        (
            argui_ui::CustomDescription,
            std::rc::Rc<argui_ui::CustomState>,
        ),
    >,
    next: usize,
}

impl LayoutTree {
    pub(crate) fn set_custom(
        &mut self,
        id: NodeId,
        custom: Option<(
            argui_ui::CustomDescription,
            std::rc::Rc<argui_ui::CustomState>,
        )>,
    ) -> Result<(), TaffyError> {
        self.index(id)?;
        if let Some(custom) = custom {
            self.custom.insert(id, custom);
        } else {
            self.custom.remove(&id);
        }
        Ok(())
    }
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
    pub(crate) fn new() -> Self {
        Self::default()
    }
    // Allocate once for the known tree size; large payloads do not pay for
    // empty hash buckets. The small ID-to-position map preserves stable IDs.
    pub(crate) fn reserve_nodes(&mut self, total: usize) {
        let total = if !self.nodes.is_empty() && total > self.nodes.capacity() {
            total.saturating_add(total / 4)
        } else {
            total
        };
        let additional = total.saturating_sub(self.len());
        self.positions.reserve(additional);
        self.ids.reserve_exact(additional);
        self.nodes.reserve_exact(additional);
        self.caches.reserve_exact(additional);
        self.unrounded.reserve_exact(additional);
        self.layouts.reserve_exact(additional);
    }
    pub(crate) fn compact(&mut self) {
        // Reconciliation may temporarily hold removed and newly mounted nodes.
        // Bound retained slack after it finishes, keeping modest growth room.
        if self.nodes.capacity() > self.len() + self.len() / 4 {
            self.ids.shrink_to_fit();
            self.nodes.shrink_to_fit();
            self.caches.shrink_to_fit();
            self.unrounded.shrink_to_fit();
            self.layouts.shrink_to_fit();
        }
        if self.positions.capacity() > self.len().saturating_mul(2) {
            self.positions.shrink_to_fit();
        }
        if self.custom.capacity() > self.custom.len().saturating_mul(2) {
            self.custom.shrink_to_fit();
        }
        if self.boundaries.capacity() > self.boundaries.len().saturating_mul(2) {
            self.boundaries.shrink_to_fit();
        }
    }
    fn index(&self, id: NodeId) -> Result<usize, TaffyError> {
        self.positions
            .get(&id)
            .copied()
            .ok_or(TaffyError::InvalidInputNode(id))
    }
    fn node(&self, id: NodeId) -> Result<&Node, TaffyError> {
        Ok(&self.nodes[self.index(id)?])
    }
    fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, TaffyError> {
        let index = self.index(id)?;
        Ok(&mut self.nodes[index])
    }
    pub(crate) fn new_leaf(&mut self, style: Style) -> Result<NodeId, TaffyError> {
        compact_index(self.nodes.len());
        let id = NodeId::from(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("layout node identities exhausted");
        self.positions.insert(id, self.nodes.len());
        self.ids.push(id);
        self.nodes.push(Node {
            style,
            context: NONE,
            children: Box::default(),
            parent: NONE,
        });
        self.caches.push(Cache::new());
        self.unrounded.push(Layout::default());
        self.layouts.push(Layout::default());
        Ok(id)
    }
    pub(crate) fn new_leaf_with_context(
        &mut self,
        style: Style,
        context: usize,
    ) -> Result<NodeId, TaffyError> {
        let id = self.new_leaf(style)?;
        self.node_mut(id)?.context = compact_index(context);
        Ok(id)
    }
    pub(crate) fn new_with_children(
        &mut self,
        style: Style,
        children: &[NodeId],
        boundary: bool,
    ) -> Result<NodeId, TaffyError> {
        for &child in children {
            self.node(child)?;
        }
        let id = self.new_leaf(style)?;
        self.set_layout_children(id, children, boundary)?;
        Ok(id)
    }
    pub(crate) fn style(&self, id: NodeId) -> Result<&Style, TaffyError> {
        Ok(&self.node(id)?.style)
    }
    pub(crate) fn layout(&self, id: NodeId) -> Result<&Layout, TaffyError> {
        Ok(&self.layouts[self.index(id)?])
    }
    pub(crate) fn set_style(&mut self, id: NodeId, style: Style) -> Result<(), TaffyError> {
        if self.node(id)?.style != style {
            self.node_mut(id)?.style = style;
            self.mark_dirty(id)?;
        }
        Ok(())
    }
    pub(crate) fn set_node_context(
        &mut self,
        id: NodeId,
        context: Option<usize>,
    ) -> Result<(), TaffyError> {
        self.node_mut(id)?.context = context.map_or(NONE, compact_index);
        self.mark_dirty(id)
    }
    pub(crate) fn set_children(
        &mut self,
        id: NodeId,
        children: &[NodeId],
    ) -> Result<(), TaffyError> {
        let parent = self.index(id)?;
        for &child in children {
            let child_index = self.index(child)?;
            let mut ancestor = compact_index(parent);
            while ancestor != NONE {
                if ancestor as usize == child_index {
                    return Err(TaffyError::InvalidChildNode(child));
                }
                ancestor = self.nodes[ancestor as usize].parent;
            }
        }
        let mut previous = std::mem::take(&mut self.nodes[parent].children);
        for &child in &previous {
            self.node_mut(child)?.parent = NONE;
        }
        for &child in children {
            let old_parent = self.node(child)?.parent;
            if old_parent != NONE {
                let old_parent = old_parent as usize;
                self.remove_child(old_parent, child);
                self.mark_dirty(self.ids[old_parent])?;
            }
            self.node_mut(child)?.parent = compact_index(parent);
        }
        self.nodes[parent].children = if previous.len() == children.len() {
            previous.copy_from_slice(children);
            previous
        } else {
            children.into()
        };
        self.mark_dirty(id)
    }
    fn remove_child(&mut self, parent: usize, child: NodeId) {
        self.nodes[parent].children = self.nodes[parent]
            .children
            .iter()
            .copied()
            .filter(|id| *id != child)
            .collect();
    }
    pub(crate) fn remove(&mut self, id: NodeId) -> Result<(), TaffyError> {
        if let Some(boundary) = self.boundaries.remove(&id) {
            self.remove(boundary.root)?;
        }
        let index = self.index(id)?;
        // Detach before swapping: the removed node's parent may itself move.
        let parent = self.nodes[index].parent;
        if parent != NONE {
            let parent = parent as usize;
            self.remove_child(parent, id);
            self.mark_dirty(self.ids[parent])?;
        }
        let children = std::mem::take(&mut self.nodes[index].children);
        for child in children {
            self.node_mut(child)?.parent = NONE;
        }
        self.positions.remove(&id);
        self.custom.remove(&id);
        self.ids.swap_remove(index);
        self.nodes.swap_remove(index);
        self.caches.swap_remove(index);
        self.unrounded.swap_remove(index);
        self.layouts.swap_remove(index);
        if let Some(&moved) = self.ids.get(index) {
            self.positions.insert(moved, index);
            for offset in 0..self.nodes[index].children.len() {
                let child = self.nodes[index].children[offset];
                self.node_mut(child)?.parent = compact_index(index);
            }
        }
        Ok(())
    }
    pub(crate) fn mark_dirty(&mut self, id: NodeId) -> Result<(), TaffyError> {
        let mut current = compact_index(self.index(id)?);
        while current != NONE {
            let index = current as usize;
            if matches!(
                self.caches[index].clear(),
                taffy::tree::ClearState::AlreadyEmpty
            ) {
                break;
            }
            current = self.nodes[index].parent;
        }
        Ok(())
    }
}
