use std::collections::HashMap;

use argui_inspect::TreeSnapshot;
use argui_layout::{LayoutNode, LayoutOutput, PortalLayout};
use argui_ui::{Element, NodeId, UiTree};

use super::Inspection;

/// Suppresses snapshot allocation when only uninspectable UI has changed.
/// Retains shallow node descriptions and reuses its traversal storage.
#[derive(Default)]
pub struct InspectionCache {
    initialized: bool,
    entries: Vec<Entry>,
    layout: HashMap<NodeId, LayoutNode>,
}

struct Entry {
    node: NodeId,
    depth: usize,
    element: Element,
    child_count: usize,
    geometry: Option<(argui_core::Rect, Option<argui_core::Rect>)>,
    portal: Option<PortalLayout>,
}

impl InspectionCache {
    /// Returns a new snapshot only when inspectable content or geometry changed.
    ///
    /// `tree` is the retained UI tree to inspect; `layout` provides the current node
    /// geometry. Returns `None` when the cached snapshot remains current.
    pub fn snapshot(&mut self, tree: &UiTree, layout: &LayoutOutput) -> Option<TreeSnapshot> {
        self.layout.clear();
        self.layout
            .extend(layout.nodes.iter().map(|node| (node.node, *node)));
        let mut cursor = 0;
        let mut entry = 0;
        let mut changed = !self.initialized;
        self.visit(
            tree.root(),
            tree.node_ids(),
            layout,
            0,
            &mut cursor,
            &mut entry,
            &mut changed,
        );
        changed |= entry != self.entries.len();
        self.entries.truncate(entry);
        self.initialized = true;
        changed.then(|| Inspection::snapshot(tree, layout))
    }

    #[allow(clippy::too_many_arguments)]
    fn visit(
        &mut self,
        element: &Element,
        ids: &[NodeId],
        layout: &LayoutOutput,
        depth: usize,
        cursor: &mut usize,
        entry: &mut usize,
        changed: &mut bool,
    ) {
        let node = ids[*cursor];
        *cursor += 1;
        if !element.inspectable {
            *cursor += super::descendant_count(element);
            return;
        }
        let geometry = self.layout.get(&node).map(|node| (node.bounds, node.clip));
        let portal = layout.portals.iter().find(|portal| portal.node == node);
        let same = self.entries.get(*entry).is_some_and(|previous| {
            previous.node == node
                && previous.depth == depth
                && previous.geometry == geometry
                && previous.portal.as_ref() == portal
                && previous.child_count == element.children.len()
                && same_content(&previous.element, element)
        });
        if !same {
            let mut shallow = element.clone();
            if !shallow.children.is_empty() {
                shallow.children.clear();
            }
            let next = Entry {
                node,
                depth,
                element: shallow,
                child_count: element.children.len(),
                geometry,
                portal: portal.cloned(),
            };
            if *entry < self.entries.len() {
                self.entries[*entry] = next;
            } else {
                self.entries.push(next);
            }
            *changed = true;
        }
        *entry += 1;
        for child in &element.children {
            self.visit(child, ids, layout, depth + 1, cursor, entry, changed);
        }
    }
}

fn same_content(left: &Element, right: &Element) -> bool {
    left.ptr_eq(right)
        || (left.kind == right.kind
            && left.text_privacy == right.text_privacy
            && left.key == right.key
            && left.style == right.style
            && left.paint == right.paint
            && left.transform == right.transform
            && left.layer == right.layer
            && left.effects == right.effects
            && left.z_index == right.z_index
            && left.interaction.as_ref().is_some_and(|value| value.enabled)
                == right
                    .interaction
                    .as_ref()
                    .is_some_and(|value| value.enabled))
}
