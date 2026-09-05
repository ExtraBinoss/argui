use std::collections::BTreeSet;

use argui_core::{Key, KeyState};
use argui_ui::{
    Element, Role, SemanticAction, SemanticState, Semantics, UiEvent, UiEventKind, length,
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
}

impl TreeView<'_> {
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
                    Key::ArrowUp | Key::ArrowDown | Key::Home | Key::End => {
                        let next = match input.key {
                            Key::ArrowUp => position.saturating_sub(1),
                            Key::ArrowDown => (position + 1).min(visible.len() - 1),
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

    #[must_use]
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        let visible = self.visible_indices();
        self.list
            .build(visible.len(), theme, |position| {
                self.row(visible[position], theme)
            })
            .semantics(Semantics::new(Role::Tree).label("Elements"))
    }

    fn row(&self, index: usize, theme: &WidgetTheme) -> Element {
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
        Button::new(&node.key, &node.label, style)
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
            )
    }
}
