use std::collections::HashMap;
use taffy::{
    NodeId, Style, TaffyError,
    tree::{Cache, Layout},
};

mod algorithms;

#[derive(Debug)]
struct Node {
    custom: Option<(
        argui_ui::CustomDescription,
        std::rc::Rc<argui_ui::CustomState>,
    )>,
    style: Style,
    context: Option<usize>,
    children: Vec<NodeId>,
    parent: Option<NodeId>,
    cache: Cache,
    unrounded: Layout,
    layout: Layout,
}

/// Argui owns retained graph storage; Taffy's low-level algorithms own CSS layout.
/// This boundary permits custom containers to recurse through the same algorithms.
#[derive(Debug, Default)]
pub(crate) struct LayoutTree {
    nodes: HashMap<NodeId, Node>,
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
        self.node_mut(id)?.custom = custom;
        Ok(())
    }
    pub(crate) fn len(&self) -> usize {
        self.nodes.len()
    }
    pub(crate) fn new() -> Self {
        Self::default()
    }
    fn node(&self, id: NodeId) -> Result<&Node, TaffyError> {
        self.nodes.get(&id).ok_or(TaffyError::InvalidInputNode(id))
    }
    fn node_mut(&mut self, id: NodeId) -> Result<&mut Node, TaffyError> {
        self.nodes
            .get_mut(&id)
            .ok_or(TaffyError::InvalidInputNode(id))
    }
    pub(crate) fn new_leaf(&mut self, style: Style) -> Result<NodeId, TaffyError> {
        let id = NodeId::from(self.next);
        self.next = self
            .next
            .checked_add(1)
            .expect("layout node identities exhausted");
        self.nodes.insert(
            id,
            Node {
                custom: None,
                style,
                context: None,
                children: Vec::new(),
                parent: None,
                cache: Cache::new(),
                unrounded: Layout::default(),
                layout: Layout::default(),
            },
        );
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
        Ok(&self.node(id)?.layout)
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
        let node = self
            .nodes
            .remove(&id)
            .ok_or(TaffyError::InvalidInputNode(id))?;
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
            let node = self.node_mut(id)?;
            node.cache.clear();
            current = node.parent;
        }
        Ok(())
    }
}
