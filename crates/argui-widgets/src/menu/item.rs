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
    pub icon: Option<argui_ui::VectorId>,
}

impl MenuItem {
    /// Creates an actionable entry with stable `id`, action `invocation`, and accessible `state`.
    pub fn new(id: impl Into<String>, invocation: ActionInvocation, state: ActionState) -> Self {
        Self {
            id: id.into(),
            state,
            kind: MenuItemKind::Action(invocation),
            icon: None,
        }
    }
    /// Creates a menu entry with stable `id`, accessible `state`, and supplied `kind`.
    pub fn entry(id: impl Into<String>, state: ActionState, kind: MenuItemKind) -> Self {
        Self {
            id: id.into(),
            state,
            kind,
            icon: None,
        }
    }
    /// Sets the vector icon displayed beside this entry.
    pub fn icon(mut self, icon: argui_ui::VectorId) -> Self {
        self.icon = Some(icon);
        self
    }
    /// Returns the action invocation for actionable entries.
    pub fn invocation(&self) -> Option<ActionInvocation> {
        if let MenuItemKind::Action(invocation) = self.kind {
            Some(invocation)
        } else {
            None
        }
    }
    /// Returns whether this entry can be activated.
    pub fn enabled(&self) -> bool {
        self.state.enabled && !matches!(self.kind, MenuItemKind::Group(_) | MenuItemKind::Separator)
    }
    /// Returns submenu or group children, or an empty slice for leaf entries.
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
pub(super) fn indicator(
    item: &MenuItem,
    theme: &crate::WidgetTheme,
    icons: Option<&crate::WidgetAssets>,
) -> argui_ui::Element {
    use crate::TablerIcon;
    let kind = &item.kind;
    let icon = match kind {
        MenuItemKind::Checkbox(CheckedState::Checked) => Some(TablerIcon::Check),
        MenuItemKind::Checkbox(CheckedState::Mixed) => Some(TablerIcon::Minus),
        MenuItemKind::Radio { selected: true, .. } => Some(TablerIcon::Circle),
        _ => None,
    };
    if let Some(id) = icon
        .and_then(|icon| icons.map(|icons| icons.vector_id(icon)))
        .or(item.icon)
    {
        return argui_ui::Element::vector(id)
            .width(argui_ui::length(16.0))
            .height(argui_ui::length(16.0))
            .shrink(0.0)
            .vector_color(theme.foreground)
            .semantic_hidden(true);
    }
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
