use crate::{Button, Popover, PopoverAction, PopoverBehavior, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    ActionInvocation, Element, FocusTarget, JustifyContent, Role, Semantics, UiEvent, UiEventKind,
    percent,
};

mod intent;
mod item;
pub use intent::MenuIntent;
pub use item::{MenuItem, MenuItemKind};

#[derive(Clone, Debug, PartialEq)]
pub enum MenuResponse {
    Toggle,
    Open {
        focus: FocusTarget,
    },
    Close,
    Focus(FocusTarget),
    Invoke(ActionInvocation),
    Submenu {
        path: Vec<String>,
        focus: FocusTarget,
    },
    Checked {
        id: String,
        checked: argui_ui::CheckedState,
    },
    Radio {
        id: String,
        group: String,
    },
}

/// Controlled action menu with stable item identities and nested levels. Handle `response` in the owning component.
#[derive(Clone, Debug)]
pub struct Menu {
    pub key: String,
    pub label: String,
    pub open: bool,
    pub items: Vec<MenuItem>,
    presence: Option<crate::Presence>,
    pub path: Vec<String>,
    pub rtl: bool,
    icons: Option<crate::WidgetAssets>,
}

impl Menu {
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        open: bool,
        items: Vec<MenuItem>,
    ) -> Self {
        fn validate(items: &[MenuItem], ids: &mut std::collections::HashSet<String>) {
            for item in items {
                assert!(
                    ids.insert(item.id.clone()),
                    "menu item identities must be unique"
                );
                validate(item.children(), ids);
            }
        }
        validate(&items, &mut std::collections::HashSet::new());
        Self {
            key: key.into(),
            label: label.into(),
            open,
            items,
            presence: None,
            path: Vec::new(),
            rtl: false,
            icons: None,
        }
    }
    #[must_use]
    pub fn icons(mut self, icons: &crate::WidgetAssets) -> Self {
        self.icons = Some(icons.clone());
        self
    }
    /// Retain entry/exit motion; the owner advances this presence each frame.
    #[must_use]
    pub fn presence(mut self, presence: &crate::Presence) -> Self {
        self.open = presence.is_open();
        self.presence = Some(presence.clone());
        self
    }
    #[must_use]
    pub fn build(&self, trigger: Element, theme: &WidgetTheme) -> Element {
        self.with_content(trigger, self.content(theme), theme)
    }
    pub(crate) fn with_content(
        &self,
        trigger: Element,
        content: Element,
        theme: &WidgetTheme,
    ) -> Element {
        let mut popover = Popover::new(&self.key, &self.label, self.open, trigger, content)
            .trap_focus(false)
            .padding(6.0)
            .size(240.0, 360.0);
        if let Some(presence) = &self.presence {
            popover = popover.presence(presence);
        }
        let mut root = popover.build(theme);
        if let Some(content) = root.children.get_mut(1) {
            content.focus_scope = Some(Self::focus_scope());
        }
        root.children[0]
            .semantics
            .as_mut()
            .expect("popover trigger semantics")
            .popup = Some(argui_ui::PopupKind::Menu);
        root
    }
    pub(crate) fn focus_scope() -> argui_ui::FocusScope {
        argui_ui::FocusScope {
            containment: argui_ui::FocusContainment::None,
            initial: Some(argui_ui::InitialFocus::First),
            restore: true,
        }
    }
    pub(crate) fn content(&self, theme: &WidgetTheme) -> Element {
        self.content_level(&self.items, &self.label, theme)
    }
    fn content_level(&self, items: &[MenuItem], label: &str, theme: &WidgetTheme) -> Element {
        Element::column(items.iter().map(|item| {
            if let MenuItemKind::Group(children) = &item.kind {
                let mut group = self.content_level(children, &item.state.label, theme);
                group.semantics = Some(Semantics::new(Role::Group).label(&item.state.label));
                return Element::column([
                    Element::text(item.state.label.as_str())
                        .text_style(theme.ghost_button().label)
                        .padding(argui_ui::sides(8.0, 6.0)),
                    group,
                ]);
            }
            if matches!(item.kind, MenuItemKind::Separator) {
                return Element::container([])
                    .height(argui_ui::length(1.0))
                    .background(theme.border)
                    .semantics(Semantics::new(Role::Separator));
            }
            let mut style = theme.ghost_button();
            style.layout.padding = argui_ui::sides(8.0, 0.0);
            style.label.weight = 400;
            let mut button = Button::new(self.item_key(&item.id), &item.state.label, style)
                .enabled(item.state.enabled)
                .leading(item::indicator(item, theme, self.icons.as_ref()));
            if let Some(shortcut) = &item.state.shortcut {
                button = button.trailing(Element::text(shortcut.label()).text_style(
                    argui_text::TextStyle {
                        color: theme.muted_foreground,
                        font_size: 12.0,
                        wrap: argui_text::TextWrap::None,
                        ..theme.ghost_button().label
                    },
                ));
            }
            if matches!(item.kind, MenuItemKind::Submenu(_)) {
                button = button.trailing(self.icons.as_ref().map_or_else(
                    || {
                        Element::text(if self.rtl { "‹" } else { "›" })
                            .text_style(theme.ghost_button().label)
                            .semantic_hidden(true)
                    },
                    |icons| {
                        icons
                            .icon(
                                if self.rtl {
                                    crate::TablerIcon::ChevronLeft
                                } else {
                                    crate::TablerIcon::ChevronRight
                                },
                                16.0,
                            )
                            .vector_color(theme.muted_foreground)
                    },
                ));
            }
            let mut element = button
                .build()
                .width(percent(1.0))
                .justify_content(JustifyContent::START);
            element.children[1] = element.children[1].clone().grow(1.0);
            element
                .interaction
                .as_mut()
                .expect("button interaction")
                .focus_policy = argui_ui::FocusPolicy::Programmatic;
            let semantics = element.semantics.as_mut().expect("button semantics");
            semantics.role = Role::MenuItem;
            match &item.kind {
                MenuItemKind::Action(invocation) => {
                    element = element.action_from(*invocation);
                }
                MenuItemKind::Checkbox(checked) => {
                    semantics.role = Role::MenuItemCheckBox;
                    semantics.state.checked = Some(*checked);
                }
                MenuItemKind::Radio { selected, .. } => {
                    semantics.role = Role::MenuItemRadio;
                    semantics.state.checked = Some(if *selected {
                        argui_ui::CheckedState::Checked
                    } else {
                        argui_ui::CheckedState::Unchecked
                    });
                }
                MenuItemKind::Submenu(children) => {
                    let open = self.path.contains(&item.id);
                    semantics.popup = Some(argui_ui::PopupKind::Menu);
                    semantics.state.expanded = Some(open);
                    element = Popover::new(
                        self.item_key(&item.id),
                        &item.state.label,
                        open,
                        element,
                        self.content_level(children, &item.state.label, theme),
                    )
                    .placement(argui_ui::FloatingPlacement::new(if self.rtl {
                        argui_ui::Placement::LeftStart
                    } else {
                        argui_ui::Placement::RightStart
                    }))
                    .padding(6.0)
                    .build(theme);
                    // The root menu dismisses the entire chain, including nested portals.
                    if let Some(content) = element.children.get_mut(1) {
                        content.portal.as_mut().expect("submenu portal").dismiss =
                            argui_ui::DismissPolicy::Manual;
                    }
                    let trigger = &mut element.children[0];
                    let semantics = trigger
                        .semantics
                        .as_mut()
                        .expect("popover trigger semantics");
                    semantics.role = Role::MenuItem;
                    semantics.popup = Some(argui_ui::PopupKind::Menu);
                    trigger
                        .interaction
                        .as_mut()
                        .expect("popover trigger interaction")
                        .focus_policy = argui_ui::FocusPolicy::Programmatic;
                }
                MenuItemKind::Separator | MenuItemKind::Group(_) => {
                    unreachable!("structural entries rendered above")
                }
            }
            element
        }))
        .gap(2.0)
        .semantics(Semantics::new(Role::Menu).label(label))
    }
    pub(crate) fn edge_focus(&self, last: bool) -> Option<FocusTarget> {
        let items = item::level(&self.items);
        let item = if last {
            items.into_iter().rev().find(|item| item.enabled())
        } else {
            items.into_iter().find(|item| item.enabled())
        }?;
        Some(self.item_key(&item.id).into())
    }

    pub fn submenu_content_key(&self, id: &str) -> String {
        format!("{}::content", self.item_key(id))
    }

    pub(crate) fn item_key(&self, id: &str) -> String {
        format!("{}::item::{id}", self.key)
    }
    fn event_level_at(&self, key: &str) -> Option<(usize, Vec<&MenuItem>)> {
        let mut items = self.items.as_slice();
        for depth in 0..=self.path.len() {
            let level = item::level(items);
            if (depth == 0 && key == self.key)
                || level.iter().any(|item| key == self.item_key(&item.id))
            {
                return Some((depth, level));
            }
            items = level
                .into_iter()
                .find(|item| Some(&item.id) == self.path.get(depth))?
                .children();
        }
        None
    }
    fn event_level(&self, key: &str) -> Option<Vec<&MenuItem>> {
        self.event_level_at(key).map(|(_, items)| items)
    }
    fn open_submenu(&self, item: &MenuItem) -> Option<MenuResponse> {
        if matches!(item.kind, MenuItemKind::Submenu(_)) {
            self.hover_response(&item.id)
        } else {
            None
        }
    }
    pub fn search(
        &self,
        event: &UiEvent,
        search: &mut crate::Typeahead,
        now: std::time::Duration,
    ) -> Option<MenuResponse> {
        if !self.open {
            return None;
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        let Key::Character(value) = &input.key else {
            return None;
        };
        if input.state != KeyState::Pressed || input.modifiers.command() || input.modifiers.alt {
            return None;
        }
        let items = self.event_level(event.target_key()?)?;
        let active = items
            .iter()
            .position(|item| event.target_key() == Some(self.item_key(&item.id).as_str()))?;
        let found = search.search(
            value,
            now,
            Some(active),
            items.len(),
            |i| items[i].enabled().then_some(items[i].state.label.as_str()),
            crate::unicode_prefix,
        )?;
        Some(MenuResponse::Focus(self.item_key(&items[found].id).into()))
    }
    #[must_use]
    pub fn response(&self, event: &UiEvent) -> Option<MenuResponse> {
        if matches!(event.kind, UiEventKind::KeyInput(_))
            && !event
                .target_key()
                .is_some_and(|key| key == self.key || key.starts_with(&format!("{}::", self.key)))
        {
            return None;
        }
        if let UiEventKind::KeyInput(input) = &event.kind
            && input.state == KeyState::Pressed
        {
            if self.open && input.key == Key::Tab {
                return Some(MenuResponse::Close);
            }
            if event.target_key() == Some(self.key.as_str())
                && matches!(input.key, Key::ArrowDown | Key::ArrowUp)
            {
                return Some(MenuResponse::Open {
                    focus: self.edge_focus(input.key == Key::ArrowUp)?,
                });
            }
        }
        if self.open
            && !self.path.is_empty()
            && let UiEventKind::KeyInput(input) = &event.kind
            && input.state == KeyState::Pressed
            && (input.key == Key::Escape
                || input.key
                    == if self.rtl {
                        Key::ArrowRight
                    } else {
                        Key::ArrowLeft
                    })
        {
            let key = event.target_key()?;
            let (depth, _) = self.event_level_at(key)?;
            if depth == 0 && input.key != Key::Escape {
                return None;
            }
            if depth == 0 {
                return Some(MenuResponse::Close);
            }
            let mut path = self.path[..depth].to_vec();
            let parent = path.pop()?;
            return Some(MenuResponse::Submenu {
                path,
                focus: self.item_key(&parent).into(),
            });
        }
        if let Some(action) = PopoverBehavior::new(&self.key, &self.label, self.open).action(event)
        {
            return Some(match action {
                PopoverAction::Toggle => MenuResponse::Toggle,
                PopoverAction::Close => MenuResponse::Close,
            });
        }
        if !self.open {
            return None;
        }
        let items = self.event_level(event.target_key()?)?;
        let current_item = items.iter().copied().find(|item| {
            item.enabled() && event.target_key() == Some(self.item_key(&item.id).as_str())
        });
        if matches!(event.kind, UiEventKind::Click(_))
            && let Some(item) = current_item
        {
            return match &item.kind {
                MenuItemKind::Action(_) => Some(MenuResponse::Close),
                MenuItemKind::Checkbox(checked) => Some(MenuResponse::Checked {
                    id: item.id.clone(),
                    checked: checked.toggled(),
                }),
                MenuItemKind::Radio { group, .. } => Some(MenuResponse::Radio {
                    id: item.id.clone(),
                    group: group.clone(),
                }),
                MenuItemKind::Submenu(_) => self.open_submenu(item),
                _ => None,
            };
        }
        let UiEventKind::KeyInput(input) = &event.kind else {
            return None;
        };
        if input.state != KeyState::Pressed {
            return None;
        }
        if input.key
            == if self.rtl {
                Key::ArrowLeft
            } else {
                Key::ArrowRight
            }
        {
            return self.open_submenu(current_item?);
        }
        let enabled: Vec<_> = items.into_iter().filter(|item| item.enabled()).collect();
        if enabled.is_empty() {
            return None;
        }
        let current = enabled
            .iter()
            .position(|item| event.target_key() == Some(self.item_key(&item.id).as_str()));
        let next = match input.key {
            Key::ArrowDown => current.map_or(0, |index| (index + 1) % enabled.len()),
            Key::ArrowUp => current.map_or(enabled.len() - 1, |index| {
                (index + enabled.len() - 1) % enabled.len()
            }),
            Key::Home => 0,
            Key::End => enabled.len() - 1,
            _ => return None,
        };
        Some(MenuResponse::Focus(self.item_key(&enabled[next].id).into()))
    }
}
