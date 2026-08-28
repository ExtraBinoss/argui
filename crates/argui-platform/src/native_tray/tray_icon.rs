use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

use tray_icon::{
    MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{CheckMenuItem, IsMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem, Submenu},
};

use crate::{
    ApplicationId, IconSet, TrayAction, TrayConfig, TrayEvent, TrayItemId, TrayMenuItem,
    TrayPointerButton, native_tray::TrayEventHandler,
};

type Actions = Arc<RwLock<HashMap<String, (TrayItemId, TrayAction)>>>;

pub(super) struct PlatformTray {
    tray: TrayIcon,
    actions: Actions,
}

impl PlatformTray {
    pub(super) fn new(
        _application_id: &ApplicationId,
        config: TrayConfig,
        fallback_icons: &IconSet,
        handler: TrayEventHandler,
    ) -> Result<Self, String> {
        let actions = Arc::new(RwLock::new(HashMap::new()));
        install_event_handlers(Arc::clone(&actions), handler);
        let menu = build_menu(&config.menu, &actions)?;
        let mut builder = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_menu_on_left_click(config.show_menu_on_left_click)
            .with_icon_as_template(config.icon_is_template);
        if let Some(tooltip) = &config.tooltip {
            builder = builder.with_tooltip(tooltip);
        }
        if let Some(title) = &config.title {
            builder = builder.with_title(title);
        }
        if let Some(icon) = native_icon(config.icon.as_ref().unwrap_or(fallback_icons))? {
            builder = builder.with_icon(icon);
        }
        let tray = builder.build().map_err(|error| error.to_string())?;
        tray.set_visible(config.visible)
            .map_err(|error| error.to_string())?;
        Ok(Self { tray, actions })
    }

    pub(super) fn sync(
        &mut self,
        config: TrayConfig,
        fallback_icons: &IconSet,
    ) -> Result<(), String> {
        let menu = build_menu(&config.menu, &self.actions)?;
        self.tray.set_menu(Some(Box::new(menu)));
        self.tray
            .set_tooltip(config.tooltip.as_deref())
            .map_err(|error| error.to_string())?;
        self.tray.set_title(config.title.as_deref());
        self.tray.set_icon_as_template(config.icon_is_template);
        self.tray
            .set_icon(native_icon(config.icon.as_ref().unwrap_or(fallback_icons))?)
            .map_err(|error| error.to_string())?;
        self.tray
            .set_visible(config.visible)
            .map_err(|error| error.to_string())?;
        self.tray
            .set_show_menu_on_left_click(config.show_menu_on_left_click);
        Ok(())
    }
}

fn install_event_handlers(actions: Actions, handler: TrayEventHandler) {
    let menu_handler = Arc::clone(&handler);
    MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
        let action = actions
            .read()
            .ok()
            .and_then(|actions| actions.get(&event.id.0).cloned());
        if let Some((id, action)) = action {
            menu_handler(TrayEvent::Action { id, action });
        }
    }));
    TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| match event {
        TrayIconEvent::Click {
            button,
            button_state: MouseButtonState::Up,
            ..
        } => handler(TrayEvent::Click {
            button: pointer_button(button),
            double: false,
        }),
        TrayIconEvent::DoubleClick { button, .. } => handler(TrayEvent::Click {
            button: pointer_button(button),
            double: true,
        }),
        _ => {}
    }));
}

fn pointer_button(button: MouseButton) -> TrayPointerButton {
    match button {
        MouseButton::Left => TrayPointerButton::Primary,
        MouseButton::Right => TrayPointerButton::Secondary,
        MouseButton::Middle => TrayPointerButton::Middle,
    }
}

fn native_icon(icons: &IconSet) -> Result<Option<tray_icon::Icon>, String> {
    icons
        .best_square(32)
        .map(|icon| {
            tray_icon::Icon::from_rgba(icon.rgba8.to_vec(), icon.width, icon.height)
                .map_err(|error| error.to_string())
        })
        .transpose()
}

fn build_menu(items: &[TrayMenuItem], actions: &Actions) -> Result<Menu, String> {
    let menu = Menu::new();
    let mut next_actions = HashMap::new();
    append_items(&menu, items, &mut next_actions)?;
    *actions
        .write()
        .map_err(|_| "tray action map is poisoned".to_owned())? = next_actions;
    Ok(menu)
}

trait AppendTarget {
    fn append_item(&self, item: &dyn IsMenuItem) -> tray_icon::menu::Result<()>;
}

impl AppendTarget for Menu {
    fn append_item(&self, item: &dyn IsMenuItem) -> tray_icon::menu::Result<()> {
        self.append(item)
    }
}

impl AppendTarget for Submenu {
    fn append_item(&self, item: &dyn IsMenuItem) -> tray_icon::menu::Result<()> {
        self.append(item)
    }
}

fn append_items(
    target: &impl AppendTarget,
    items: &[TrayMenuItem],
    actions: &mut HashMap<String, (TrayItemId, TrayAction)>,
) -> Result<(), String> {
    for item in items {
        match item {
            TrayMenuItem::Action {
                id,
                label,
                enabled,
                action,
            } => {
                register_action(actions, id, action)?;
                let item = MenuItem::with_id(id.as_str(), label, *enabled, None);
                target
                    .append_item(&item)
                    .map_err(|error| error.to_string())?;
            }
            TrayMenuItem::Check {
                id,
                label,
                enabled,
                checked,
                action,
            } => {
                register_action(actions, id, action)?;
                let item = CheckMenuItem::with_id(id.as_str(), label, *enabled, *checked, None);
                target
                    .append_item(&item)
                    .map_err(|error| error.to_string())?;
            }
            TrayMenuItem::Separator => {
                let item = PredefinedMenuItem::separator();
                target
                    .append_item(&item)
                    .map_err(|error| error.to_string())?;
            }
            TrayMenuItem::Submenu {
                id,
                label,
                enabled,
                items,
            } => {
                let submenu = Submenu::with_id(id.as_str(), label, *enabled);
                append_items(&submenu, items, actions)?;
                target
                    .append_item(&submenu)
                    .map_err(|error| error.to_string())?;
            }
        }
    }
    Ok(())
}

fn register_action(
    actions: &mut HashMap<String, (TrayItemId, TrayAction)>,
    id: &TrayItemId,
    action: &TrayAction,
) -> Result<(), String> {
    if actions
        .insert(id.as_str().to_owned(), (id.clone(), action.clone()))
        .is_some()
    {
        Err(format!("duplicate tray item id: {}", id.as_str()))
    } else {
        Ok(())
    }
}
