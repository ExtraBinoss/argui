use argui_widgets::{TreeNode, TreeView, VList};
use std::cell::{Ref, RefCell};
use std::collections::BTreeSet;

use super::DevtoolsHost;

#[derive(Default)]
pub(crate) struct TreeCache {
    revision: Option<u64>,
    query: String,
    nodes: Vec<TreeNode>,
}

impl<A> DevtoolsHost<A> {
    pub(crate) fn tree_nodes(&self) -> Ref<'_, Vec<TreeNode>> {
        self.inspector.with_tree(|snapshot| {
            let mut cache = self.tree_cache.borrow_mut();
            if cache.revision != Some(snapshot.revision) || cache.query != self.search {
                let query = self.search.to_lowercase();
                cache.nodes = snapshot
                    .nodes
                    .iter()
                    .filter(|node| query.is_empty() || crate::view::matches_query(node, &query))
                    .map(|node| {
                        let identity = node.key.as_ref().map_or_else(
                            || node.kind.clone(),
                            |key| format!("{}  #{key}", node.kind),
                        );
                        TreeNode {
                            key: format!("__devtools-node-{}", node.id.0),
                            label: node.summary.as_ref().map_or(identity.clone(), |summary| {
                                format!("{identity} — {summary}")
                            }),
                            depth: if query.is_empty() { node.depth } else { 0 },
                            icon: Some(self.icons.node_icon(&node.kind)),
                        }
                    })
                    .collect();
                cache.revision = Some(snapshot.revision);
                cache.query.clone_from(&self.search);
            }
        });
        Ref::map(self.tree_cache.borrow(), |cache| &cache.nodes)
    }

    pub(crate) fn tree_view<'a>(
        &'a self,
        nodes: &'a [TreeNode],
        selected: Option<&'a str>,
    ) -> TreeView<'a> {
        TreeView {
            nodes,
            selected,
            collapsed: &self.collapsed,
            list: VList::new("__devtools-tree", 28.0, self.tree_height, self.tree_offset)
                .effects(self.scroll_effect.clone()),
            disclosure: Some(self.icons.chevron),
        }
    }
}

pub(crate) fn cache() -> RefCell<TreeCache> {
    RefCell::default()
}
pub(crate) fn collapsed() -> BTreeSet<String> {
    BTreeSet::new()
}
