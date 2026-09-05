use std::collections::{BTreeMap, BTreeSet};

use argui_paint::VectorId;
use argui_ui::{Element, Role, Semantics};

use super::{TreeNode, TreeView};
use crate::WidgetTheme;

/// Retained tree rows, bounded to the current viewport and its overscan.
#[derive(Default)]
pub struct TreeViewCache {
    nodes: Vec<TreeNode>,
    collapsed: BTreeSet<String>,
    visible: Vec<usize>,
    theme: Option<WidgetTheme>,
    disclosure: Option<VectorId>,
    rows: BTreeMap<usize, Row>,
}

struct Row {
    selected: bool,
    element: Element,
}

impl TreeView<'_> {
    /// Reuses unchanged rows across scroll events; invalidates on data or style changes.
    #[must_use]
    pub fn build_cached(&self, theme: &WidgetTheme, cache: &mut TreeViewCache) -> Element {
        if cache.nodes != self.nodes || &cache.collapsed != self.collapsed {
            cache.nodes.clear();
            cache.nodes.extend_from_slice(self.nodes);
            cache.collapsed.clone_from(self.collapsed);
            cache.visible = self.visible_indices();
            cache.rows.clear();
        }
        if cache.theme.as_ref() != Some(theme) || cache.disclosure != self.disclosure {
            cache.theme = Some(theme.clone());
            cache.disclosure = self.disclosure;
            cache.rows.clear();
        }
        let range = self
            .list
            .config(cache.visible.len())
            .window(self.list.offset)
            .range;
        cache.rows.retain(|position, _| range.contains(position));
        self.list
            .build(cache.visible.len(), theme, |position| {
                let index = cache.visible[position];
                let selected = self.selected == Some(self.nodes[index].key.as_str());
                let row = cache.rows.entry(position).or_insert_with(|| Row {
                    selected,
                    element: self.row(index, theme),
                });
                if row.selected != selected {
                    row.selected = selected;
                    row.element = self.row(index, theme);
                }
                row.element.clone()
            })
            .semantics(Semantics::new(Role::Tree).label("Elements"))
    }
}
