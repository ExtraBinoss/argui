use super::LayoutTree;
mod custom;
use taffy::tree::traits::CacheTree;
use taffy::{
    AvailableSpace, BlockContext, Display, NodeId, Style,
    geometry::Size,
    tree::{
        Layout, LayoutBlockContainer, LayoutFlexboxContainer, LayoutGridContainer, LayoutInput,
        LayoutOutput, LayoutPartialTree, RoundTree, RunMode, TraversePartialTree, TraverseTree,
    },
};

type Measure<'a> = dyn FnMut(LayoutInput, NodeId, Option<usize>, &Style) -> LayoutOutput + 'a;
struct Computation<'a> {
    error: Option<String>,
    tree: &'a mut LayoutTree,
    measure: &'a mut Measure<'a>,
}

impl LayoutTree {
    pub(crate) fn compute_layout_with_measure(
        &mut self,
        root: NodeId,
        available: Size<AvailableSpace>,
        boundary_input: Option<LayoutInput>,
        mut measure: impl FnMut(LayoutInput, NodeId, Option<usize>, &Style) -> LayoutOutput,
    ) -> Result<(), crate::LayoutError> {
        self.node(root)?;
        let mut computation = Computation {
            error: None,
            tree: self,
            measure: &mut measure,
        };
        if let Some(input) = boundary_input {
            let output = computation.compute(root, input, None);
            let index = computation.tree.index(root)?;
            computation.tree.unrounded[index].scrollable_overflow_rect =
                output.scrollable_overflow_rect;
        } else {
            taffy::compute_root_layout(&mut computation, root, available);
        }
        if let Some(error) = computation.error.take() {
            for cache in &mut computation.tree.caches {
                cache.clear();
            }
            return Err(crate::LayoutError::Custom(error));
        }
        taffy::round_layout(computation.tree, root);
        Ok(())
    }
}

impl LayoutTree {
    pub(crate) fn round_root(&mut self, root: NodeId) {
        taffy::round_layout(self, root);
    }
}

impl Computation<'_> {
    fn compute(
        &mut self,
        id: NodeId,
        inputs: LayoutInput,
        block: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        if inputs.run_mode == RunMode::PerformLayout
            && let Some(boundary) = self.tree.boundaries.get_mut(&id)
        {
            boundary.input = Some(inputs);
        }
        if inputs.run_mode == RunMode::PerformHiddenLayout {
            return taffy::compute_hidden_layout(self, id);
        }
        taffy::compute_cached_layout(self, id, inputs, |tree, id, inputs| {
            if tree.tree.node(id).expect("live layout node").style.display != Display::None
                && let Some((description, state)) = tree.tree.custom.get(&id).cloned()
            {
                return match tree.custom_layout(id, inputs, &description, &state) {
                    Ok(output) => output,
                    Err(error) => {
                        tree.error = Some(error);
                        LayoutOutput::DEFAULT
                    }
                };
            }
            let node = tree.tree.node(id).expect("live layout node");
            match (node.style.display, node.children.is_empty()) {
                (Display::None, _) => taffy::compute_hidden_layout(tree, id),
                (Display::Block, false) => taffy::compute_block_layout(tree, id, inputs, block),
                (Display::FlowRoot, false) => taffy::compute_block_layout(tree, id, inputs, None),
                (Display::Flex, false) => taffy::compute_flexbox_layout(tree, id, inputs),
                (Display::Grid, false) => taffy::compute_grid_layout(tree, id, inputs),
                (_, true) => {
                    let node = tree.tree.node(id).expect("live layout node");
                    let context = (node.context != super::NONE).then_some(node.context as usize);
                    (tree.measure)(inputs, id, context, &node.style)
                }
            }
        })
    }
}

impl TraversePartialTree for LayoutTree {
    type ChildIter<'a>
        = std::iter::Copied<std::slice::Iter<'a, NodeId>>
    where
        Self: 'a;
    fn child_ids(&self, id: NodeId) -> Self::ChildIter<'_> {
        self.node(id)
            .expect("live layout node")
            .children
            .iter()
            .copied()
    }
    fn child_count(&self, id: NodeId) -> usize {
        self.node(id).expect("live layout node").children.len()
    }
    fn get_child_id(&self, id: NodeId, index: usize) -> NodeId {
        self.node(id).expect("live layout node").children[index]
    }
}
impl TraverseTree for LayoutTree {}
impl TraversePartialTree for Computation<'_> {
    type ChildIter<'a>
        = <LayoutTree as TraversePartialTree>::ChildIter<'a>
    where
        Self: 'a;
    fn child_ids(&self, id: NodeId) -> Self::ChildIter<'_> {
        self.tree.child_ids(id)
    }
    fn child_count(&self, id: NodeId) -> usize {
        self.tree.child_count(id)
    }
    fn get_child_id(&self, id: NodeId, index: usize) -> NodeId {
        self.tree.get_child_id(id, index)
    }
}
impl TraverseTree for Computation<'_> {}
impl LayoutPartialTree for Computation<'_> {
    type CoreContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type CustomIdent = String;
    fn get_core_container_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
    fn set_unrounded_layout(&mut self, id: NodeId, layout: &Layout) {
        let index = self.tree.index(id).expect("live layout node");
        self.tree.unrounded[index] = *layout;
    }
    fn compute_child_layout(&mut self, id: NodeId, inputs: LayoutInput) -> LayoutOutput {
        self.compute(id, inputs, None)
    }
}
impl CacheTree for Computation<'_> {
    fn cache_get(&mut self, id: NodeId, input: &LayoutInput) -> Option<LayoutOutput> {
        let index = self.tree.index(id).expect("live layout node");
        self.tree.caches[index].get(input)
    }
    fn cache_store(&mut self, id: NodeId, input: &LayoutInput, output: LayoutOutput) {
        let index = self.tree.index(id).expect("live layout node");
        self.tree.caches[index].store(input, output);
    }
    fn cache_clear(&mut self, id: NodeId) {
        let index = self.tree.index(id).expect("live layout node");
        self.tree.caches[index].clear();
    }
}
impl RoundTree for LayoutTree {
    fn get_unrounded_layout(&self, id: NodeId) -> Layout {
        self.unrounded[self.index(id).expect("live layout node")]
    }
    fn set_final_layout(&mut self, id: NodeId, layout: &Layout) {
        let index = self.index(id).expect("live layout node");
        self.layouts[index] = *layout;
    }
}
impl LayoutBlockContainer for Computation<'_> {
    type BlockContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type BlockItemStyle<'a>
        = &'a Style
    where
        Self: 'a;
    fn get_block_container_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
    fn get_block_child_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
    fn compute_block_child_layout(
        &mut self,
        id: NodeId,
        inputs: LayoutInput,
        block: Option<&mut BlockContext<'_>>,
    ) -> LayoutOutput {
        self.compute(id, inputs, block)
    }
}
impl LayoutFlexboxContainer for Computation<'_> {
    type FlexboxContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type FlexboxItemStyle<'a>
        = &'a Style
    where
        Self: 'a;
    fn get_flexbox_container_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
    fn get_flexbox_child_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
}
impl LayoutGridContainer for Computation<'_> {
    type GridContainerStyle<'a>
        = &'a Style
    where
        Self: 'a;
    type GridItemStyle<'a>
        = &'a Style
    where
        Self: 'a;
    fn get_grid_container_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
    fn get_grid_child_style(&self, id: NodeId) -> &Style {
        &self.tree.node(id).expect("live layout node").style
    }
}
