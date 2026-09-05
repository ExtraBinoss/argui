use argui_core::{CaretAffinity, TextPosition};

use crate::{Element, ElementKind, NodeId, TextSelectionStyle, UiTree, UserSelect};

use super::{DocumentTextPoint, SelectionGranularity};

impl UiTree {
    #[must_use]
    pub fn resolved_user_select(&self, node: NodeId) -> UserSelect {
        self.selection_context(node)
            .map_or(UserSelect::None, |value| value.0)
    }

    #[must_use]
    pub fn resolved_selection_style(&self, node: NodeId) -> TextSelectionStyle {
        self.selection_context(node)
            .map_or_else(TextSelectionStyle::default, |value| value.1)
    }

    pub(super) fn expanded_point(
        &self,
        point: DocumentTextPoint,
        granularity: SelectionGranularity,
    ) -> Option<(DocumentTextPoint, DocumentTextPoint, Option<NodeId>)> {
        let entries = self.selectable_text_entries();
        let entry_index = entries.iter().position(|entry| entry.node == point.node)?;
        let entry = &entries[entry_index];
        if let Some(atomic_root) = entry.atomic_root {
            let first = entries
                .iter()
                .find(|candidate| candidate.atomic_root == Some(atomic_root))?;
            let last = entries
                .iter()
                .rev()
                .find(|candidate| candidate.atomic_root == Some(atomic_root))?;
            return Some((
                DocumentTextPoint::new(first.node, TextPosition::new(0, CaretAffinity::Before)),
                DocumentTextPoint::new(
                    last.node,
                    TextPosition::new(last.text.len(), CaretAffinity::After),
                ),
                entry.contain_root,
            ));
        }
        let range = match entry.policy {
            UserSelect::Text | UserSelect::Auto | UserSelect::Contain => match granularity {
                SelectionGranularity::Character => point.position.index..point.position.index,
                SelectionGranularity::Word => {
                    argui_text::word_range(entry.text, point.position.index)
                }
                SelectionGranularity::Line => {
                    argui_text::line_range(entry.text, point.position.index)
                }
            },
            UserSelect::None | UserSelect::All => return None,
        };
        Some((
            DocumentTextPoint::new(
                point.node,
                TextPosition::new(range.start, CaretAffinity::Before),
            ),
            DocumentTextPoint::new(
                point.node,
                TextPosition::new(range.end, CaretAffinity::After),
            ),
            entry.contain_root,
        ))
    }

    pub(super) fn clamp_to_selection_scope(&self, point: DocumentTextPoint) -> DocumentTextPoint {
        let Some(scope) = self.document_selection.scope else {
            return point;
        };
        let entries = self.selectable_text_entries();
        if entries
            .iter()
            .any(|entry| entry.node == point.node && entry.contain_root == Some(scope))
        {
            return point;
        }
        let scoped = entries
            .iter()
            .filter(|entry| entry.contain_root == Some(scope))
            .collect::<Vec<_>>();
        let (Some(first), Some(last)) = (scoped.first(), scoped.last()) else {
            return point;
        };
        let point_index = self.node_index(point.node).unwrap_or(usize::MAX);
        let first_index = self.node_index(first.node).unwrap_or(0);
        if point_index <= first_index {
            DocumentTextPoint::new(first.node, TextPosition::new(0, CaretAffinity::Before))
        } else {
            DocumentTextPoint::new(
                last.node,
                TextPosition::new(last.text.len(), CaretAffinity::After),
            )
        }
    }

    fn selection_context(&self, target: NodeId) -> Option<(UserSelect, TextSelectionStyle)> {
        self.index.selection(target)
    }

    pub(super) fn selectable_text_entries(&self) -> Vec<TextEntry<'_>> {
        let mut output = Vec::new();
        collect_entries(
            self.root(),
            self.node_ids(),
            &mut 0,
            UserSelect::Text,
            None,
            None,
            &mut output,
        );
        output
    }
}

pub(super) struct TextEntry<'a> {
    pub(super) node: NodeId,
    pub(super) text: &'a str,
    policy: UserSelect,
    atomic_root: Option<NodeId>,
    pub(super) contain_root: Option<NodeId>,
}

fn collect_entries<'a>(
    element: &'a Element,
    ids: &[NodeId],
    index: &mut usize,
    parent_policy: UserSelect,
    parent_atomic_root: Option<NodeId>,
    parent_contain_root: Option<NodeId>,
    output: &mut Vec<TextEntry<'a>>,
) {
    let node = ids[*index];
    *index += 1;
    let (policy, atomic_root, contain_root) = match element.user_select {
        UserSelect::Auto => (parent_policy, parent_atomic_root, parent_contain_root),
        UserSelect::All => (UserSelect::All, Some(node), parent_contain_root),
        UserSelect::Text => (UserSelect::Text, None, parent_contain_root),
        UserSelect::None => (UserSelect::None, None, None),
        UserSelect::Contain => (UserSelect::Contain, None, Some(node)),
    };
    if policy != UserSelect::None
        && let ElementKind::Text { content, .. } = &element.kind
    {
        output.push(TextEntry {
            node,
            text: content.as_str(),
            policy,
            atomic_root,
            contain_root,
        });
    }
    for child in &element.children {
        collect_entries(child, ids, index, policy, atomic_root, contain_root, output);
    }
}
