use crate::{Button, ButtonBehavior, Popover, WidgetTheme, choice_navigation::navigate};
use argui_core::{Key, KeyState};
use argui_ui::{Element, FocusPolicy, Orientation, Role, Semantics, UiEvent, UiEventKind};

#[derive(Clone, Debug)]
pub struct NavigationItem {
    pub id: String,
    pub label: String,
    pub panel: Option<Element>,
    pub current: bool,
    pub enabled: bool,
}

impl NavigationItem {
    /// Creates a navigation entry with a stable `id` and display `label`.
    #[must_use]
    pub fn new(id: impl Into<String>, label: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            panel: None,
            current: false,
            enabled: true,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NavigationMenuAction {
    Activate(String),
    Open { id: String, focus_panel: bool },
    Close,
    Focus(String),
}

#[derive(Clone, Debug)]
pub struct NavigationMenu {
    pub key: String,
    pub label: String,
    pub items: Vec<NavigationItem>,
    pub open: Option<String>,
    pub rtl: bool,
}

impl NavigationMenu {
    /// Creates a controlled navigation menu from its ordered items.
    /// `key` identifies the menu and `label` names it accessibly.
    #[must_use]
    pub fn new(
        key: impl Into<String>,
        label: impl Into<String>,
        items: impl IntoIterator<Item = NavigationItem>,
    ) -> Self {
        Self {
            key: key.into(),
            label: label.into(),
            items: items.into_iter().collect(),
            open: None,
            rtl: false,
        }
    }

    #[must_use]
    /// Returns the element key for the item identified by `id`.
    pub fn item_key(&self, id: &str) -> String {
        format!("{}::item::{id}", self.key)
    }

    #[must_use]
    /// Returns the navigation action requested by `event`, if any.
    pub fn action(&self, event: &UiEvent) -> Option<NavigationMenuAction> {
        let key = event.target_key()?;
        if self.open.is_some()
            && matches!(&event.kind, UiEventKind::KeyInput(input) if input.key == Key::Escape && input.state == KeyState::Pressed)
        {
            return Some(NavigationMenuAction::Close);
        }
        if let Some(id) = &self.open
            && key == format!("{}::content", self.item_key(id))
            && matches!(
                event.kind,
                UiEventKind::PointerOutside(_) | UiEventKind::DismissRequested
            )
        {
            return Some(NavigationMenuAction::Close);
        }
        let index = self
            .items
            .iter()
            .position(|item| key == self.item_key(&item.id) && item.enabled)?;
        let item = &self.items[index];
        if let Some(next) = navigate(
            event,
            index,
            self.items.len(),
            Orientation::Horizontal,
            self.rtl,
            |i| self.items[i].enabled,
        ) {
            return Some(NavigationMenuAction::Focus(self.items[next].id.clone()));
        }
        let down = matches!(&event.kind, UiEventKind::KeyInput(input) if input.key == Key::ArrowDown && input.state == KeyState::Pressed);
        let clicked = ButtonBehavior::new(key, &item.label)
            .action(event)
            .is_some();
        if item.panel.is_some() && (down || clicked) {
            return Some(if self.open.as_deref() == Some(&item.id) && clicked {
                NavigationMenuAction::Close
            } else {
                NavigationMenuAction::Open {
                    id: item.id.clone(),
                    focus_panel: down,
                }
            });
        }
        clicked.then(|| NavigationMenuAction::Activate(item.id.clone()))
    }

    #[must_use]
    /// Builds the navigation menu using `theme` for its appearance.
    ///
    /// # Panics
    ///
    /// Panics if generated navigation controls are missing their expected semantics or interaction data.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::row(self.items.iter().map(|item| {
            let key = self.item_key(&item.id);
            let trigger = Button::new(
                &key,
                &item.label,
                if item.current {
                    theme.secondary_button()
                } else {
                    theme.ghost_button()
                },
            )
            .enabled(item.enabled)
            .build();
            if let Some(panel) = &item.panel {
                let open = item.enabled && self.open.as_deref() == Some(&item.id);
                let mut root =
                    Popover::new(&key, &item.label, open, trigger, panel.clone()).build(theme);
                // Popover owns trigger semantics; preserve the navigation item's disabled state.
                let trigger = &mut root.children[0];
                trigger.interaction.as_mut().expect("trigger").enabled = item.enabled;
                if !item.enabled {
                    trigger.interaction.as_mut().expect("trigger").focus_policy = FocusPolicy::None;
                }
                trigger.semantics.as_mut().expect("trigger").state.disabled = !item.enabled;
                if open {
                    root.children[0] = root.children[0]
                        .clone()
                        .controls([format!("{key}::content")]);
                    root.children[1]
                        .interaction
                        .as_mut()
                        .expect("panel")
                        .focus_policy = FocusPolicy::Programmatic;
                }
                root.display(argui_ui::Display::Flex)
                    .flex_direction(argui_ui::FlexDirection::Column)
                    .shrink(0.0)
            } else {
                let mut link = trigger;
                let semantics = link.semantics.as_mut().expect("navigation link");
                semantics.role = Role::Link;
                if item.current {
                    semantics.description = Some("Current page".into());
                }
                link
            }
        }))
        .keyed(&self.key)
        .gap(4.0)
        .flex_wrap(argui_ui::FlexWrap::Wrap)
        .direction_scope(if self.rtl {
            argui_ui::WritingDirection::Rtl
        } else {
            argui_ui::WritingDirection::Ltr
        })
        .semantics(Semantics::new(Role::Navigation).label(&self.label))
    }
}
