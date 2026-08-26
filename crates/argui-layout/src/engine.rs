use taffy::TaffyTree;

#[derive(Debug, Default)]
pub struct LayoutEngine {
    tree: TaffyTree<()>,
}

impl LayoutEngine {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn tree_mut(&mut self) -> &mut TaffyTree<()> {
        &mut self.tree
    }
}
