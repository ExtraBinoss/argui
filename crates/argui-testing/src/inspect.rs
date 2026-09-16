use argui_accessibility::{Role, SemanticState, SemanticTree, SemanticValue};
use argui_core::Rect;
use argui_runtime::{AppModel, Render};
use argui_ui::{Display, NodeId};

use crate::{Selector, SelectorCount, SemanticMatcher, TestApp, TestError};

#[derive(Clone, Debug)]
pub(crate) struct ResolvedNode {
    pub node: NodeId,
    pub role: Role,
    pub label: Option<String>,
    pub state: SemanticState,
    pub bounds: Option<Rect>,
}

impl<A: Render> TestApp<A> {
    /// Returns the current renderer-independent accessibility snapshot.
    #[must_use]
    pub fn semantics(&self) -> SemanticTree {
        self.ui().semantic_tree(&self.output().semantic_bounds, 1.0)
    }

    /// Returns a compact current tree and semantic dump for diagnostics.
    #[must_use]
    pub fn dump(&self) -> String {
        self.compact_dump()
    }

    /// Panics unless visible text containing `text` exists.
    pub fn assert_text(&self, text: &str) {
        assert!(
            self.texts()
                .iter()
                .any(|candidate| candidate.contains(text)),
            "expected visible text containing {text:?}\n{}",
            self.compact_dump()
        );
    }

    /// Panics if visible text containing `text` exists.
    pub fn assert_no_text(&self, text: &str) {
        assert!(
            self.texts()
                .iter()
                .all(|candidate| !candidate.contains(text)),
            "expected no visible text containing {text:?}\n{}",
            self.compact_dump()
        );
    }

    /// Panics unless the unique element with `key` currently owns focus.
    pub fn assert_focused(&self, key: &str) {
        let resolved = self
            .resolve_unique(&Selector::key(key))
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            self.ui().focused_node(),
            Some(resolved.node),
            "expected key {key:?} to be focused\n{}",
            self.compact_dump()
        );
    }

    /// Panics unless `selector` resolves to exactly one visible element.
    pub fn assert_exists(&self, selector: impl Into<Selector>) {
        self.resolve_unique(&selector.into())
            .unwrap_or_else(|error| panic!("{error}"));
    }

    /// Panics unless the unique `selector` is visible with non-empty bounds.
    pub fn assert_visible(&self, selector: impl Into<Selector>) {
        let selector = selector.into();
        let resolved = self
            .resolve_unique(&selector)
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(
            resolved
                .bounds
                .is_some_and(|bounds| bounds.size.width > 0.0 && bounds.size.height > 0.0),
            "expected {selector} to have visible bounds\n{}",
            self.compact_dump()
        );
    }

    /// Panics unless `selector` has the requested accessible `state`.
    pub fn assert_state(&self, selector: impl Into<Selector>, state: SemanticMatcher) {
        let selector = selector.into();
        let resolved = self
            .resolve_unique(&selector)
            .unwrap_or_else(|error| panic!("{error}"));
        assert!(
            state.matches(&resolved.state),
            "expected {selector} to match {state:?}, got {:?}\n{}",
            resolved.state,
            self.compact_dump()
        );
    }

    /// Panics unless the unique text input selected by `selector` has `expected` value.
    pub fn assert_input_value(&self, selector: impl Into<Selector>, expected: &str) {
        let selector = selector.into();
        let resolved = self
            .resolve_unique(&selector)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            self.ui().text_input_value(resolved.node),
            Some(expected),
            "unexpected value for {selector}\n{}",
            self.compact_dump()
        );
    }

    /// Panics unless `selector` has exactly `expected` logical layout bounds.
    pub fn assert_bounds(&self, selector: impl Into<Selector>, expected: Rect) {
        let selector = selector.into();
        let resolved = self
            .resolve_unique(&selector)
            .unwrap_or_else(|error| panic!("{error}"));
        assert_eq!(
            resolved.bounds,
            Some(expected),
            "unexpected bounds for {selector}"
        );
    }

    /// Returns the logical layout bounds of the unique `selector`.
    ///
    /// # Errors
    /// Returns a selector diagnostic or [`TestError::MissingBounds`].
    pub fn bounds(&self, selector: impl Into<Selector>) -> Result<Rect, TestError> {
        let selector = selector.into();
        self.resolve_unique(&selector)?
            .bounds
            .ok_or_else(|| TestError::MissingBounds {
                selector,
                tree: self.compact_dump(),
            })
    }

    /// Returns the retained scroll offset for the unique `selector`.
    ///
    /// # Errors
    /// Returns a selector diagnostic when `selector` is missing or ambiguous.
    pub fn scroll_offset(
        &self,
        selector: impl Into<Selector>,
    ) -> Result<argui_core::Point, TestError> {
        let selector = selector.into();
        let node = self.resolve_unique(&selector)?.node;
        Ok(self.ui().scroll_offset(node))
    }

    /// Panics unless the current in-memory clipboard equals `expected`.
    pub fn assert_clipboard(&self, expected: &str) {
        assert_eq!(self.clipboard.as_deref(), Some(expected));
    }

    /// Panics unless the recorded application commands equal `expected` in order.
    pub fn assert_commands(&self, expected: &[argui_runtime::AppCommand]) {
        assert_eq!(self.commands(), expected, "unexpected application commands");
    }

    /// Panics unless no render, model, animation, or scroll work is pending.
    pub fn assert_quiescent(&self) {
        assert!(
            !self.pending
                && !self.model.wants_animation_frame(&self.window)
                && !self.ui().wants_animation_frame()
                && !self.ui().wants_scroll_frame(),
            "expected application to be quiescent\n{}",
            self.compact_dump()
        );
    }

    /// Panics unless no asynchronous operation remains registered.
    pub fn assert_no_pending_tasks(&self) {
        assert_eq!(
            self.pending_tasks(),
            0,
            "expected no pending tasks\n{}",
            self.compact_dump()
        );
    }

    pub(crate) fn resolve_unique(&self, selector: &Selector) -> Result<ResolvedNode, TestError> {
        let matches = self.resolve_all(selector);
        if matches.len() == 1 {
            return Ok(matches.into_iter().next().expect("one match"));
        }
        Err(TestError::Selector {
            selector: selector.clone(),
            count: if matches.is_empty() {
                SelectorCount::None
            } else {
                SelectorCount::Multiple(matches.len())
            },
            focused: self
                .ui()
                .focused_node()
                .map(|node| self.describe_node(node)),
            candidates: self.candidate_dump(),
            tree: self.compact_dump(),
        })
    }

    fn resolve_all(&self, selector: &Selector) -> Vec<ResolvedNode> {
        if let Selector::Key(key) = selector {
            return self
                .ui()
                .node_ids()
                .iter()
                .copied()
                .filter(|node| self.ui().key(*node) == Some(key.as_str()))
                .filter(|node| self.node_visible(*node))
                .map(|node| self.resolved_node(node))
                .collect();
        }
        if matches!(selector, Selector::Focused) {
            return self
                .ui()
                .focused_node()
                .into_iter()
                .map(|node| self.resolved_node(node))
                .collect();
        }
        self.semantics()
            .nodes
            .iter()
            .filter(|node| match selector {
                Selector::RoleName { role, name } => {
                    node.semantics.role == *role
                        && node.semantics.label.as_deref() == Some(name.as_str())
                }
                Selector::Text(text) => {
                    node.semantics.label.as_deref() == Some(text.as_str())
                        || matches!(
                            &node.semantics.value,
                            Some(SemanticValue::Text(value)) if value == text
                        )
                }
                Selector::Label(label) => node.semantics.label.as_deref() == Some(label.as_str()),
                Selector::State(state) => state.matches(&node.semantics.state),
                Selector::Key(_) | Selector::Focused => false,
            })
            .filter_map(|semantic| {
                self.ui()
                    .node_ids()
                    .iter()
                    .copied()
                    .find(|node| node.get() == semantic.id.get())
            })
            .map(|node| self.resolved_node(node))
            .collect()
    }

    fn resolved_node(&self, node: NodeId) -> ResolvedNode {
        let semantic = self
            .semantics()
            .nodes
            .into_iter()
            .find(|semantic| semantic.id.get() == node.get());
        ResolvedNode {
            node,
            role: semantic
                .as_ref()
                .map_or(Role::Generic, |value| value.semantics.role),
            label: semantic
                .as_ref()
                .and_then(|value| value.semantics.label.clone()),
            state: semantic
                .as_ref()
                .map_or_else(SemanticState::default, |value| {
                    value.semantics.state.clone()
                }),
            bounds: self
                .output()
                .nodes
                .iter()
                .find(|layout| layout.node == node)
                .map(|layout| layout.bounds),
        }
    }

    fn node_visible(&self, node: NodeId) -> bool {
        let mut current = Some(node);
        while let Some(candidate) = current {
            let Some(element) = self.ui().element_for(candidate) else {
                return false;
            };
            if self.ui().resolved_layout_style(candidate, element).display == Display::None {
                return false;
            }
            current = self.ui().parent_of(candidate);
        }
        true
    }

    fn texts(&self) -> Vec<String> {
        self.semantics()
            .nodes
            .into_iter()
            .flat_map(|node| {
                let mut values = node.semantics.label.into_iter().collect::<Vec<_>>();
                if let Some(SemanticValue::Text(value)) = node.semantics.value {
                    values.push(value);
                }
                values
            })
            .collect()
    }

    fn describe_node(&self, node: NodeId) -> String {
        let resolved = self.resolved_node(node);
        format!(
            "role={:?} label={:?} key={:?}",
            resolved.role,
            resolved.label,
            self.ui().key(node)
        )
    }

    fn candidate_dump(&self) -> String {
        let candidates = self
            .semantics()
            .nodes
            .iter()
            .take(12)
            .map(|node| {
                format!(
                    "  {:?} label={:?} value={:?}",
                    node.semantics.role, node.semantics.label, node.semantics.value
                )
            })
            .collect::<Vec<_>>()
            .join("\n");
        format!("close candidates:\n{candidates}")
    }

    pub(crate) fn compact_dump(&self) -> String {
        let mut lines = vec![format!(
            "tree viewport={:?} focused={:?} pending={}",
            self.viewport,
            self.ui()
                .focused_node()
                .map(|node| self.describe_node(node)),
            self.pending
        )];
        for node in self.semantics().nodes.iter().take(40) {
            lines.push(format!(
                "- {:?} label={:?} value={:?} state={:?} bounds={:?}",
                node.semantics.role,
                node.semantics.label,
                node.semantics.value,
                node.semantics.state,
                node.bounds
            ));
        }
        lines.join("\n")
    }
}

impl From<&str> for Selector {
    fn from(value: &str) -> Self {
        Self::key(value)
    }
}

impl From<String> for Selector {
    fn from(value: String) -> Self {
        Self::key(value)
    }
}
