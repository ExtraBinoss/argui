use super::LayoutTree;
use argui_core::Size;
use taffy::{
    NodeId, Style, TaffyError,
    tree::{LayoutInput, RunMode},
};

#[derive(Debug)]
pub(super) struct Boundary {
    pub(super) root: NodeId,
    hidden: bool,
    pub(super) input: Option<LayoutInput>,
}

#[derive(Debug)]
pub(crate) struct BoundaryWork {
    pub(crate) root: NodeId,
    pub(crate) size: Size,
    pub(crate) input: LayoutInput,
    pub(crate) dirty: bool,
    pub(crate) moved: bool,
}

impl LayoutTree {
    pub(crate) fn set_layout_children(
        &mut self,
        id: NodeId,
        children: &[NodeId],
        isolated: bool,
    ) -> Result<(), TaffyError> {
        if isolated {
            if !self.node(id)?.children.is_empty() {
                self.set_children(id, &[])?;
            }
            let root = if let Some(boundary) = self.boundaries.get(&id) {
                boundary.root
            } else {
                let root = self.new_leaf(Style::default())?;
                self.boundaries.insert(
                    id,
                    Boundary {
                        root,
                        hidden: false,
                        input: None,
                    },
                );
                root
            };
            self.set_children(root, children)
        } else {
            if let Some(boundary) = self.boundaries.remove(&id) {
                self.remove(boundary.root)?;
            }
            self.set_children(id, children)
        }
    }

    pub(crate) fn set_boundary_hidden(&mut self, id: NodeId, hidden: bool) {
        if let Some(boundary) = self.boundaries.get_mut(&id) {
            boundary.hidden = hidden;
        }
    }

    /// The outer node is a sizing proxy. Its private root applies the same
    /// container algorithm inside the allocated box, preserving child placement.
    pub(crate) fn prepare_boundary(
        &mut self,
        id: NodeId,
    ) -> Result<BoundaryWork, crate::LayoutError> {
        let boundary = &self.boundaries[&id];
        let root = boundary.root;
        let mut input = if boundary.hidden {
            LayoutInput::HIDDEN
        } else {
            boundary.input.expect("visible sizing proxy was laid out")
        };
        let outer = self.index(id)?;
        let mut layout = self.unrounded[outer];
        let mut style = self.style(id)?.clone();
        if style.overflow.x == taffy::Overflow::Visible
            || style.overflow.y == taffy::Overflow::Visible
        {
            return Err(crate::LayoutError::InvalidBoundary(
                "both axes must clip or scroll",
            ));
        }
        let size = Size::new(layout.size.width, layout.size.height);
        if input.run_mode != RunMode::PerformHiddenLayout {
            input.known_dimensions = layout.size.map(Some);
            input.known_dimensions_are_definite = taffy::geometry::Size {
                width: true,
                height: true,
            };
        } else {
            style.display = taffy::Display::None;
        }
        self.set_style(root, style)?;
        // Round in the same coordinate space as the proxy, including fractional
        // ancestor positions. Private roots themselves are not emitted as UI nodes.
        let mut position = super::compact_index(outer);
        let mut origin = taffy::geometry::Point::ZERO;
        while position != super::NONE {
            let index = position as usize;
            origin.x += self.unrounded[index].location.x;
            origin.y += self.unrounded[index].location.y;
            position = self.nodes[index].parent;
        }
        let inner = self.index(root)?;
        let dirty = self.caches[inner].get(&input).is_none();
        let moved = self.unrounded[inner].location != origin;
        layout.location = origin;
        if dirty {
            self.unrounded[inner] = layout;
        } else {
            self.unrounded[inner].location = origin;
        }
        Ok(BoundaryWork {
            root,
            size,
            input,
            dirty,
            moved,
        })
    }

    pub(crate) fn finish_boundary(&mut self, id: NodeId) -> Result<(), TaffyError> {
        let root = self.index(self.boundaries[&id].root)?;
        let outer = self.index(id)?;
        // Scrolling still needs the content extents. These are not exported to
        // ancestors through Taffy's sizing cache: the proxy clips its content.
        self.unrounded[outer].scrollable_overflow_rect =
            self.unrounded[root].scrollable_overflow_rect;
        self.layouts[outer].scrollable_overflow_rect = self.layouts[root].scrollable_overflow_rect;
        Ok(())
    }
}
