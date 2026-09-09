use argui_ui::{ActionInvocation, ActionState, CheckedState};

#[derive(Clone, Debug, PartialEq)]
pub enum MenuItemKind {
    Action(ActionInvocation),
    Checkbox(CheckedState),
    Radio { group: String, selected: bool },
    Submenu(Vec<MenuItem>),
    Group(Vec<MenuItem>),
    Separator,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MenuItem {
    pub id: String,
    pub state: ActionState,
    pub kind: MenuItemKind,
}

impl MenuItem {
    pub fn new(id: impl Into<String>, invocation: ActionInvocation, state: ActionState) -> Self {
        Self {
            id: id.into(),
            state,
            kind: MenuItemKind::Action(invocation),
        }
    }
    pub fn entry(id: impl Into<String>, state: ActionState, kind: MenuItemKind) -> Self {
        Self {
            id: id.into(),
            state,
            kind,
        }
    }
    pub fn invocation(&self) -> Option<ActionInvocation> {
        if let MenuItemKind::Action(invocation) = self.kind {
            Some(invocation)
        } else {
            None
        }
    }
    pub fn enabled(&self) -> bool {
        self.state.enabled && !matches!(self.kind, MenuItemKind::Group(_) | MenuItemKind::Separator)
    }
    pub fn children(&self) -> &[MenuItem] {
        match &self.kind {
            MenuItemKind::Submenu(items) | MenuItemKind::Group(items) => items,
            _ => &[],
        }
    }
}

pub(super) fn level(items: &[MenuItem]) -> Vec<&MenuItem> {
    items
        .iter()
        .flat_map(|item| match &item.kind {
            MenuItemKind::Group(children) => level(children),
            _ => vec![item],
        })
        .collect()
}

/// Reserve the same leading space for every actionable entry.
pub(super) fn indicator(kind: &MenuItemKind, theme: &crate::WidgetTheme) -> argui_ui::Element {
    let mark = match kind {
        MenuItemKind::Checkbox(CheckedState::Checked) => "✓",
        MenuItemKind::Checkbox(CheckedState::Mixed) => "−",
        MenuItemKind::Radio { selected: true, .. } => "●",
        _ => "",
    };
    argui_ui::Element::text(mark)
        .text_style(theme.ghost_button().label)
        .width(argui_ui::length(16.0))
        .shrink(0.0)
        .semantic_hidden(true)
}
