use std::collections::BTreeSet;

use argui_core::{Key, KeyState};
use argui_ui::{
    Element, EventFilter, EventType, Role, SemanticAction, SemanticState, Semantics, UiEvent,
    UiEventKind, ValueHandler, length,
};

use crate::{Button, VList, WidgetTheme};

mod cache;
pub use cache::TreeViewCache;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TreeNode {
    pub key: String,
    pub label: String,
    pub depth: usize,
    pub icon: Option<argui_paint::VectorId>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TreeAction {
    Select(String),
    Expand(String),
    Collapse(String),
}

/// A controlled tree accepting preorder rows from any data source.
pub struct TreeView<'a> {
    pub nodes: &'a [TreeNode],
    pub selected: Option<&'a str>,
    pub collapsed: &'a BTreeSet<String>,
    pub list: VList,
    pub disclosure: Option<argui_paint::VectorId>,
    select_handlers: Vec<ValueHandler<String>>,
    activate_handlers: Vec<ValueHandler<String>>,
}

impl TreeView<'_> {
    /// Creates a controlled tree from preorder `nodes` and a virtual `list` viewport.
    ///
    /// `selected` identifies the active row and `collapsed` contains stable ids of
    /// closed branches. The application owns both values.
    #[must_use]
    pub fn new<'a>(
        nodes: &'a [TreeNode],
        selected: Option<&'a str>,
        collapsed: &'a BTreeSet<String>,
        list: VList,
    ) -> TreeView<'a> {
        TreeView {
            nodes,
            selected,
            collapsed,
            list,
            disclosure: None,
            select_handlers: Vec::new(),
            activate_handlers: Vec::new(),
        }
    }

    /// Sets the optional disclosure `icon` shown for expandable branches.
    #[must_use]
    pub fn disclosure(mut self, icon: Option<argui_paint::VectorId>) -> Self {
        self.disclosure = icon;
        self
    }

    /// Adds a handler that receives the stable id of a selected node.
    #[must_use]
    pub fn on_select(mut self, handler: ValueHandler<String>) -> Self {
        self.select_handlers.push(handler);
        self
    }

    /// Adds a handler that receives the stable id of a node activated by double click.
    #[must_use]
    pub fn on_activate(mut self, handler: ValueHandler<String>) -> Self {
        self.activate_handlers.push(handler);
        self
    }

    /// Returns source indices visible after applying collapsed ancestors.
    #[must_use]
    pub fn visible_indices(&self) -> Vec<usize> {
        let mut hidden_below = None;
        self.nodes
            .iter()
            .enumerate()
            .filter_map(|(index, node)| {
                if hidden_below.is_some_and(|depth| node.depth > depth) {
                    return None;
                }
                hidden_below = self.collapsed.contains(&node.key).then_some(node.depth);
                Some(index)
            })
            .collect()
    }

    fn has_children(&self, index: usize) -> bool {
        self.nodes
            .get(index + 1)
            .is_some_and(|next| next.depth > self.nodes[index].depth)
    }

    #[must_use]
    /// Interprets `event` as selecting, expanding or collapsing a tree node.
    pub fn action(&self, event: &UiEvent) -> Option<TreeAction> {
        let index = self
            .nodes
            .iter()
            .position(|node| Some(node.key.as_str()) == event.target_key())?;
        let node = &self.nodes[index];
        match &event.kind {
            UiEventKind::Click(click) if click.count >= 2 && self.has_children(index) => {
                Some(if self.collapsed.contains(&node.key) {
                    TreeAction::Expand(node.key.clone())
                } else {
                    TreeAction::Collapse(node.key.clone())
                })
            }
            UiEventKind::Click(_) => Some(TreeAction::Select(node.key.clone())),
            UiEventKind::KeyInput(input) if input.state == KeyState::Pressed => {
                let visible = self.visible_indices();
                let position = visible.iter().position(|candidate| *candidate == index)?;
                match input.key {
                    Key::ArrowRight
                        if self.has_children(index) && self.collapsed.contains(&node.key) =>
                    {
                        Some(TreeAction::Expand(node.key.clone()))
                    }
                    Key::ArrowLeft
                        if self.has_children(index) && !self.collapsed.contains(&node.key) =>
                    {
                        Some(TreeAction::Collapse(node.key.clone()))
                    }
                    Key::ArrowLeft => self.nodes[..index]
                        .iter()
                        .rev()
                        .find(|parent| parent.depth < node.depth)
                        .map(|parent| TreeAction::Select(parent.key.clone())),
                    Key::ArrowRight if self.has_children(index) => {
                        Some(TreeAction::Select(self.nodes[index + 1].key.clone()))
                    }
                    Key::ArrowUp
                    | Key::ArrowDown
                    | Key::Home
                    | Key::End
                    | Key::PageUp
                    | Key::PageDown => {
                        let next = match input.key {
                            Key::ArrowUp => position.saturating_sub(1),
                            Key::ArrowDown => (position + 1).min(visible.len() - 1),
                            Key::PageUp => position.saturating_sub(self.page_size(visible.len())),
                            Key::PageDown => {
                                (position + self.page_size(visible.len())).min(visible.len() - 1)
                            }
                            Key::Home => 0,
                            _ => visible.len() - 1,
                        };
                        Some(TreeAction::Select(self.nodes[visible[next]].key.clone()))
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }

    /// Returns a selection action for a visible node matching printable input.
    /// `event` is the printable input; `search` retains typeahead state and `now` supplies the monotonic timestamp.
    pub fn search(
        &self,
        event: &UiEvent,
        search: &mut crate::Typeahead,
        now: std::time::Duration,
    ) -> Option<TreeAction> {
        let visible = self.visible_indices();
        let active = visible
            .iter()
            .position(|&index| Some(self.nodes[index].key.as_str()) == event.target_key())?;
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        let Key::Character(query) = &input.key else {
            return None;
        };
        if input.state != KeyState::Pressed || input.modifiers.command() || input.modifiers.alt {
            return None;
        }
        let found = search.search(
            query,
            now,
            Some(active),
            visible.len(),
            |index| Some(self.nodes[visible[index]].label.as_str()),
            crate::unicode_prefix,
        )?;
        Some(TreeAction::Select(self.nodes[visible[found]].key.clone()))
    }

    #[must_use]
    /// Builds the visible tree rows using `theme` for their controls.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let visible = self.visible_indices();
        let active = self.active_position(&visible);
        self.list
            .build_pinned(visible.len(), theme, active, |position| {
                self.row(visible[position], active == Some(position), theme)
            })
            .semantics(Semantics::new(Role::Tree).label("Elements"))
    }

    fn page_size(&self, count: usize) -> usize {
        self.list
            .config(count)
            .visible_range(self.list.offset)
            .len()
            .max(1)
    }

    fn active_position(&self, visible: &[usize]) -> Option<usize> {
        visible
            .iter()
            .position(|&index| self.selected == Some(self.nodes[index].key.as_str()))
            .or_else(|| (!visible.is_empty()).then_some(0))
    }

    fn row(&self, index: usize, active: bool, theme: &WidgetTheme) -> Element {
        let node = &self.nodes[index];
        let selected = self.selected == Some(node.key.as_str());
        let mut style = if selected {
            theme.button()
        } else {
            theme.ghost_button()
        };
        style.layout.padding = argui_ui::sides(8.0 + node.depth as f32 * 12.0, 2.0);
        style.layout.justify_content = Some(argui_ui::JustifyContent::START);
        style.label.font_size = 12.0;
        let color = if selected {
            theme.primary_foreground
        } else {
            theme.foreground
        };
        let mut leading = Element::container([])
            .width(length(14.0))
            .height(length(14.0))
            .shrink(0.0);
        if self.has_children(index) {
            let angle = if self.collapsed.contains(&node.key) {
                0.0
            } else {
                std::f32::consts::FRAC_PI_2
            };
            leading = match self.disclosure {
                Some(icon) => Element::vector(icon)
                    .width(length(14.0))
                    .height(length(14.0))
                    .vector_color(color)
                    .transform(argui_core::Transform2D::IDENTITY.rotate(angle)),
                None => Element::container([(4.0, 1.0), (8.0, -1.0)].map(|(top, direction)| {
                    Element::container([])
                        .absolute(argui_ui::Sides {
                            top: length(top),
                            left: length(4.0),
                            right: argui_ui::auto(),
                            bottom: argui_ui::auto(),
                        })
                        .width(length(7.0))
                        .height(length(2.0))
                        .background(color)
                        .transform(
                            argui_core::Transform2D::IDENTITY
                                .rotate(direction * std::f32::consts::FRAC_PI_4),
                        )
                }))
                .width(length(14.0))
                .height(length(14.0))
                .transform(argui_core::Transform2D::IDENTITY.rotate(angle)),
            };
        }
        let mut icons = vec![leading.semantic_hidden(true)];
        if let Some(icon) = node.icon {
            icons.push(
                Element::vector(icon)
                    .width(length(16.0))
                    .height(length(16.0))
                    .vector_color(color)
                    .semantic_hidden(true),
            );
        }
        let mut row = Button::new(&node.key, &node.label, style)
            .leading(Element::row(icons).gap(4.0).shrink(0.0))
            .build()
            .width(argui_ui::percent(1.0))
            .min_width(length(0.0))
            .semantics(
                Semantics::new(Role::TreeItem)
                    .label(&node.label)
                    .level((node.depth + 1) as u32)
                    .state(SemanticState {
                        selected,
                        expanded: self
                            .has_children(index)
                            .then_some(!self.collapsed.contains(&node.key)),
                        ..SemanticState::default()
                    })
                    .action(SemanticAction::Click)
                    .action(SemanticAction::Focus),
            );
        for handler in &self.select_handlers {
            row = row.on(handler.direct_listener_value(EventType::Click, node.key.clone()));
        }
        for handler in &self.activate_handlers {
            row = row.on(handler
                .direct_listener_value(EventType::Click, node.key.clone())
                .filter(EventFilter::DoubleClick));
        }
        let visible = self.visible_indices();
        let position = visible.iter().position(|candidate| *candidate == index);
        for (filter, target) in [
            (
                EventFilter::ArrowLeftPressed,
                self.nodes[..index]
                    .iter()
                    .rev()
                    .find(|parent| parent.depth < node.depth)
                    .map(|parent| parent.key.clone()),
            ),
            (
                EventFilter::ArrowRightPressed,
                self.has_children(index)
                    .then(|| self.nodes[index + 1].key.clone()),
            ),
            (
                EventFilter::ArrowUpPressed,
                position.and_then(|position| {
                    visible
                        .get(position.saturating_sub(1))
                        .map(|index| self.nodes[*index].key.clone())
                }),
            ),
            (
                EventFilter::ArrowDownPressed,
                position.and_then(|position| {
                    visible
                        .get((position + 1).min(visible.len().saturating_sub(1)))
                        .map(|index| self.nodes[*index].key.clone())
                }),
            ),
            (
                EventFilter::HomePressed,
                visible.first().map(|index| self.nodes[*index].key.clone()),
            ),
            (
                EventFilter::EndPressed,
                visible.last().map(|index| self.nodes[*index].key.clone()),
            ),
        ] {
            if let Some(target) = target {
                for handler in &self.select_handlers {
                    row = row.on(handler
                        .direct_listener_value(EventType::Key, target.clone())
                        .filter(filter));
                }
            }
        }
        row.interaction
            .as_mut()
            .expect("tree row button interaction")
            .focus_policy = if active {
            argui_ui::FocusPolicy::TabStop
        } else {
            argui_ui::FocusPolicy::Programmatic
        };
        row
    }
}
