use std::collections::HashMap;
use taffy::{
    NodeId, Style, TaffyError,
    tree::{Cache, Layout},
};

mod algorithms;

#[derive(Debug)]
struct Node {
    style: Style,
    context: Option<usize>,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
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
        let id = NodeId::from(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("layout node identities exhausted");
        self.positions.insert(id, self.nodes.len());
        self.ids.push(id);
        self.nodes.push(Node {
            style,
            context: None,
            children: Vec::new(),
            parent: None,
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
        self.node_mut(id)?.context = Some(context);
        Ok(id)
    }
    pub(crate) fn new_with_children(
        &mut self,
        style: Style,
        children: &[NodeId],
    ) -> Result<NodeId, TaffyError> {
        for &child in children {
            self.node(child)?;
        }
        let id = self.new_leaf(style)?;
        self.set_children(id, children)?;
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
        self.node_mut(id)?.context = context;
        self.mark_dirty(id)
    }
    pub(crate) fn set_children(
        &mut self,
        id: NodeId,
        children: &[NodeId],
    ) -> Result<(), TaffyError> {
        self.node(id)?;
        for &child in children {
            self.node(child)?;
            let mut ancestor = Some(id);
            while let Some(parent) = ancestor {
                if parent == child {
                    return Err(TaffyError::InvalidChildNode(child));
                }
                ancestor = self.node(parent)?.parent;
            }
        }
        let previous = std::mem::take(&mut self.node_mut(id)?.children);
        for child in previous {
            self.node_mut(child)?.parent = None;
        }
        for &child in children {
            if let Some(old_parent) = self.node(child)?.parent {
                self.node_mut(old_parent)?
                    .children
                    .retain(|candidate| *candidate != child);
                self.mark_dirty(old_parent)?;
            }
            self.node_mut(child)?.parent = Some(id);
        }
        self.node_mut(id)?.children.extend_from_slice(children);
        self.mark_dirty(id)
    }
    pub(crate) fn remove(&mut self, id: NodeId) -> Result<(), TaffyError> {
        let index = self.index(id)?;
        self.positions.remove(&id);
        self.custom.remove(&id);
        self.ids.swap_remove(index);
        let node = self.nodes.swap_remove(index);
        self.caches.swap_remove(index);
        self.unrounded.swap_remove(index);
        self.layouts.swap_remove(index);
        if let Some(moved) = self.ids.get(index) {
            self.positions.insert(*moved, index);
        }
        if let Some(parent) = node.parent {
            self.node_mut(parent)?.children.retain(|child| *child != id);
            self.mark_dirty(parent)?;
        }
        for child in node.children {
            self.node_mut(child)?.parent = None;
        }
        Ok(())
    }
    pub(crate) fn mark_dirty(&mut self, id: NodeId) -> Result<(), TaffyError> {
        let mut current = Some(id);
        while let Some(id) = current {
            let index = self.index(id)?;
            self.caches[index].clear();
            current = self.nodes[index].parent;
        }
        Ok(())
    }
}
