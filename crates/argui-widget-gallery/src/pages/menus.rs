use crate::navigation::Page;
use argui::{
    core::{Point, Rect},
    runtime::{
        Context, Entity, LayoutSnapshot, Render,
        tasks::{self, TaskSlot},
    },
    ui::{ActionState, CheckedState, Element, EventType, UiEvent},
    widgets::{
        Button, ContextMenu, Menu, MenuIntent, MenuItem, MenuItemKind, MenuResponse, Menubar,
        MenubarResponse, Typeahead, shadcn,
    },
};
use std::time::Duration;
use web_time::Instant;

pub(crate) struct MenusDemo {
    page: Page,
    open: bool,
    path: Vec<String>,
    checked: CheckedState,
    radio: String,
    position: Option<Point>,
    submenu_bounds: Option<Rect>,
    active_menu: String,
    search: Typeahead,
    intent: MenuIntent,
    timer: TaskSlot,
    origin: Instant,
    error: String,
}
impl Default for MenusDemo {
    fn default() -> Self {
        Self {
            page: Page::Menu,
            open: false,
            path: Vec::new(),
            checked: CheckedState::Mixed,
            radio: "first".into(),
            position: None,
            submenu_bounds: None,
            active_menu: "first-menu".into(),
            search: Typeahead::default(),
            intent: MenuIntent::default(),
            timer: TaskSlot::default(),
            origin: Instant::now(),
            error: String::new(),
        }
    }
}
pub(crate) fn render(
    entity: &Entity<MenusDemo>,
    page: Page,
    cx: &mut Context<crate::WidgetGallery>,
) -> Element {
    if entity.read(|demo| demo.page != page) {
        entity.update(|demo, cx| {
            demo.page = page;
            demo.open = false;
            demo.path.clear();
            demo.intent.cancel();
            demo.timer.cancel();
            cx.notify();
        });
    }
    cx.entity(entity)
}
impl MenusDemo {
    fn menu(&self, key: &str) -> Menu {
        let entries = vec![
            MenuItem::entry(
                "check",
                ActionState::new("Checkbox"),
                MenuItemKind::Checkbox(self.checked),
            ),
            MenuItem::entry("separator", ActionState::new(""), MenuItemKind::Separator),
            MenuItem::entry(
                "options",
                ActionState::new("Options"),
                MenuItemKind::Submenu(
                    ["first", "second"]
                        .into_iter()
                        .map(|id| {
                            MenuItem::entry(
                                id,
                                ActionState::new(id),
                                MenuItemKind::Radio {
                                    group: "choice".into(),
                                    selected: self.radio == id,
                                },
                            )
                        })
                        .collect(),
                ),
            ),
        ];
        let mut menu = Menu::new(
            key,
            "Options",
            self.open && (self.page != Page::Menubar || self.active_menu == key),
            entries,
        );
        menu.path = self.path.clone();
        menu
    }
    fn apply(&mut self, response: MenuResponse, cx: &mut Context<Self>) {
        self.timer.cancel();
        self.intent.cancel();
        match response {
            MenuResponse::Open { focus } => {
                self.open = true;
                self.path.clear();
                cx.request_focus(focus);
            }
            MenuResponse::Toggle => {
                self.open = !self.open;
                self.path.clear();
            }
            MenuResponse::Close => {
                self.open = false;
                self.path.clear();
                self.timer.cancel();
                self.intent.cancel();
            }
            MenuResponse::Focus(focus) => cx.request_focus(focus),
            MenuResponse::Submenu { path, focus } => {
                self.path = path;
                cx.request_focus(focus);
            }
            MenuResponse::Checked { checked, .. } => self.checked = checked,
            MenuResponse::Radio { id, .. } => self.radio = id,
            MenuResponse::Invoke(invocation) => {
                cx.invoke_action(invocation);
                self.open = false;
            }
        }
    }
    fn schedule(&mut self, cx: &mut Context<Self>) {
        self.timer.cancel();
        if let Some(deadline) = self.intent.next_deadline() {
            let delay = deadline.saturating_sub(self.origin.elapsed());
            if let Err(error) =
                cx.spawn_latest(&mut self.timer, tasks::sleep(delay), |demo, _, cx| {
                    if let Some(id) = demo.intent.take_due(demo.origin.elapsed()) {
                        let key = if demo.page == Page::Menubar {
                            demo.active_menu.as_str()
                        } else {
                            "menu"
                        };
                        if let Some(response) = demo.menu(key).hover_response(&id) {
                            demo.apply(response, cx);
                            cx.notify();
                        }
                    }
                })
            {
                self.error = error.to_string();
            }
        }
    }
    fn event(&mut self, event: &UiEvent, cx: &mut Context<Self>) {
        let key = if self.page == Page::Menubar {
            self.active_menu.clone()
        } else {
            "menu".into()
        };
        let menu = self.menu(&key);
        if let argui::ui::UiEventKind::Pointer(pointer) = &event.kind
            && pointer.phase == argui::core::PointerPhase::Moved
            && let Some(bounds) = self.submenu_bounds
        {
            let previous = self.intent.next_deadline();
            self.intent.pointer_moved(
                pointer.position,
                bounds,
                self.origin.elapsed(),
                Duration::from_millis(200),
            );
            if previous != self.intent.next_deadline() {
                self.schedule(cx);
            }
            return;
        }
        if menu.schedule_hover(
            event,
            &mut self.intent,
            self.origin.elapsed(),
            Duration::from_millis(200),
        ) {
            self.schedule(cx);
            return;
        }
        let response = if self.page == Page::ContextMenu {
            let context = ContextMenu {
                menu,
                position: self.position,
            };
            if let Some(position) = context.open_action(event) {
                self.open = true;
                self.position = position;
                self.path.clear();
                cx.notify();
                let _ = event.prevent_default();
                return;
            }
            context
                .menu
                .search(event, &mut self.search, self.origin.elapsed())
                .or_else(|| context.response(event))
        } else if self.page == Page::Menubar {
            let menus = [self.menu("first-menu"), self.menu("second-menu")];
            match (Menubar {
                key: "bar",
                label: "Menu bar",
                menus: &menus,
                active: Some(&self.active_menu),
                rtl: false,
            })
            .response(event)
            {
                Some(MenubarResponse::Focus { key }) => {
                    self.active_menu = key.clone();
                    Some(MenuResponse::Focus(key.into()))
                }
                Some(MenubarResponse::Open { key, focus }) => {
                    self.active_menu = key;
                    self.open = true;
                    self.path.clear();
                    Some(MenuResponse::Focus(focus))
                }
                Some(MenubarResponse::Menu { key, response }) => {
                    self.active_menu = key;
                    Some(response)
                }
                None => menu.search(event, &mut self.search, self.origin.elapsed()),
            }
        } else {
            menu.search(event, &mut self.search, self.origin.elapsed())
                .or_else(|| menu.response(event))
        };
        if let Some(response) = response {
            self.apply(response, cx);
            if !matches!(&event.kind, argui::ui::UiEventKind::KeyInput(input) if input.key == argui::core::Key::Tab)
            {
                let _ = event.prevent_default();
            }
            event.stop_propagation();
            cx.notify();
        }
    }
}
impl Render for MenusDemo {
    fn layout_changed(&mut self, layout: &LayoutSnapshot, _cx: &mut Context<Self>) {
        let key = if self.page == Page::Menubar {
            self.active_menu.as_str()
        } else {
            "menu"
        };
        self.submenu_bounds = self
            .path
            .last()
            .and_then(|id| layout.bounds(&self.menu(key).submenu_content_key(id)));
    }

    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let themes = shadcn(cx.environment().primary);
        let theme = themes.resolve(cx.environment().color_scheme);
        let content = match self.page {
            Page::ContextMenu => ContextMenu {
                menu: self.menu("menu"),
                position: self.position,
            }
            .build(
                Button::new("menu", "Right-click or Shift+F10", theme.outline_button()).build(),
                theme,
            ),
            Page::Menubar => Menubar {
                key: "bar",
                label: "Menu bar",
                menus: &[self.menu("first-menu"), self.menu("second-menu")],
                active: Some(&self.active_menu),
                rtl: false,
            }
            .build(theme),
            _ => self.menu("menu").build(
                Button::new("trigger", "Open menu", theme.outline_button()).build(),
                theme,
            ),
        };
        Element::column([content, Element::text(self.error.as_str())])
            .on(cx.listener(EventType::Click, Self::event))
            .on(cx.listener(EventType::Key, Self::event))
            .on(cx
                .listener(EventType::PointerEnter, Self::event)
                .capture(true))
            .on(cx
                .listener(EventType::PointerMove, Self::event)
                .capture(true))
            .on(cx.listener(EventType::PointerOutside, Self::event))
            .on(cx.listener(EventType::ContextMenu, Self::event))
    }
}
