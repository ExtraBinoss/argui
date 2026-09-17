use crate::{Button, Menu, MenuResponse, WidgetTheme};
use argui_core::{Key, KeyState};
use argui_ui::{
    Element, FocusPolicy, FocusTarget, Role, Semantics, UiEvent, UiEventKind, ValueHandler,
};

#[derive(Clone, Debug, PartialEq)]
pub enum MenubarResponse {
    Open { key: String, focus: FocusTarget },
    Focus { key: String },
    Menu { key: String, response: MenuResponse },
}

pub struct Menubar<'a> {
    pub key: &'a str,
    pub label: &'a str,
    pub menus: &'a [Menu],
    pub active: Option<&'a str>,
    pub rtl: bool,
    action_handlers: Vec<ValueHandler<String>>,
    open_handlers: Vec<ValueHandler<bool>>,
}

impl<'a> Menubar<'a> {
    /// Creates a controlled menubar from `menus`, identified and named by `key` and `label`.
    #[must_use]
    pub fn new(key: &'a str, label: &'a str, menus: &'a [Menu]) -> Menubar<'a> {
        Menubar {
            key,
            label,
            menus,
            active: None,
            rtl: false,
            action_handlers: Vec::new(),
            open_handlers: Vec::new(),
        }
    }

    /// Sets the stable key of the currently active top-level menu.
    #[must_use]
    pub fn active(mut self, active: Option<&'a str>) -> Self {
        self.active = active;
        self
    }

    /// Sets right-to-left keyboard and visual navigation when `rtl` is true.
    #[must_use]
    pub fn rtl(mut self, rtl: bool) -> Self {
        self.rtl = rtl;
        self
    }

    /// Adds a handler that receives the stable id of an activated menu item.
    #[must_use]
    pub fn on_action(mut self, handler: ValueHandler<String>) -> Self {
        self.action_handlers.push(handler);
        self
    }

    /// Adds a handler receiving requested nested-menu open states.
    #[must_use]
    pub fn on_open_change(mut self, handler: ValueHandler<bool>) -> Self {
        self.open_handlers.push(handler);
        self
    }

    /// Builds the configured menus in a horizontal menubar using `theme` for their controls.
    ///
    /// # Panics
    ///
    /// Panics if a built menu trigger is missing its expected interaction or semantic data.
    pub fn build(&self, theme: &WidgetTheme) -> Element {
        Element::row(self.menus.iter().enumerate().map(|(index, menu)| {
            let mut menu = menu.clone();
            menu.rtl = self.rtl;
            for handler in &self.action_handlers {
                menu = menu.on_action(*handler);
            }
            for handler in &self.open_handlers {
                menu = menu.on_open_change(*handler);
            }
            let trigger = Button::new(&menu.key, &menu.label, theme.ghost_button()).build();
            let mut built = menu.build(trigger, theme);
            let trigger = &mut built.children[0];
            let semantics = trigger.semantics.as_mut().expect("menu trigger semantics");
            semantics.role = Role::MenuItem;
            semantics.popup = Some(argui_ui::PopupKind::Menu);
            trigger
                .interaction
                .as_mut()
                .expect("menu trigger interaction")
                .focus_policy = if self.active.map_or(index == 0, |active| active == menu.key) {
                FocusPolicy::TabStop
            } else {
                FocusPolicy::Programmatic
            };

            built
        }))
        .keyed(self.key)
        .semantics(Semantics::new(Role::MenuBar).label(self.label))
    }
    /// Interprets `event` as a menubar focus change or nested menu response.
    pub fn response(&self, event: &UiEvent) -> Option<MenubarResponse> {
        let key = event.target_key()?;
        let current = self
            .menus
            .iter()
            .position(|menu| key == menu.key || key.starts_with(&format!("{}::", menu.key)))?;
        let mut menu = self.menus[current].clone();
        menu.rtl = self.rtl;
        if let UiEventKind::KeyInput(input) = &event.kind
            && input.state == KeyState::Pressed
            && menu.path.is_empty()
        {
            if key == menu.key && matches!(input.key, Key::ArrowDown | Key::ArrowUp | Key::Enter) {
                return Some(MenubarResponse::Open {
                    key: menu.key.clone(),
                    focus: menu.edge_focus(input.key == Key::ArrowUp)?,
                });
            }
            if let Some(response @ MenuResponse::Submenu { .. }) = menu.response(event) {
                return Some(MenubarResponse::Menu {
                    key: menu.key.clone(),
                    response,
                });
            }
            let backwards = if self.rtl {
                Key::ArrowRight
            } else {
                Key::ArrowLeft
            };
            let forwards = if self.rtl {
                Key::ArrowLeft
            } else {
                Key::ArrowRight
            };
            let next = if input.key == backwards {
                Some((current + self.menus.len() - 1) % self.menus.len())
            } else if input.key == forwards {
                Some((current + 1) % self.menus.len())
            } else if input.key == Key::Home && key == menu.key {
                Some(0)
            } else if input.key == Key::End && key == menu.key {
                Some(self.menus.len() - 1)
            } else {
                None
            };
            if let Some(next) = next {
                let target = &self.menus[next];
                return if menu.open {
                    Some(MenubarResponse::Open {
                        key: target.key.clone(),
                        focus: target
                            .edge_focus(false)
                            .unwrap_or_else(|| target.key.clone().into()),
                    })
                } else {
                    Some(MenubarResponse::Focus {
                        key: target.key.clone(),
                    })
                };
            }
        }
        menu.response(event).map(|response| MenubarResponse::Menu {
            key: menu.key.clone(),
            response,
        })
    }
}
