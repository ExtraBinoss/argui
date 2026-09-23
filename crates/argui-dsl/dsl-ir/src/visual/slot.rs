//! Resolved caller content for named component slots.
use super::*;

impl VisualLowerer<'_> {
    /// Lowers named slot bodies on `node` for `target`, retaining caller bindings.
    /// Returns resolved wrappers; semantic checking has rejected unknown slots.
    pub(super) fn slot_contents(
        &mut self,
        node: &SyntaxNode,
        target: &IrElementTarget,
    ) -> Vec<IrNode> {
        let IrElementTarget::Component(component) = target else {
            return Vec::new();
        };
        let Some(members) = self
            .tables
            .components
            .iter()
            .find(|(_, id)| *id == component)
            .and_then(|(symbol, _)| self.tables.component_members.get(symbol))
        else {
            return Vec::new();
        };
        node.children()
            .filter(|node| node.kind() == SyntaxKind::SlotContent)
            .filter_map(|node| {
                let name = direct_identifier(&node)?;
                let slot = members.slots.get(&name).copied()?;
                let body = self.visual_children(&node);
                Some(IrNode::SlotContent {
                    slot,
                    body,
                    source: self.source(&node, None),
                })
            })
            .collect()
    }
}
