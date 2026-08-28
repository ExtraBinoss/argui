use ksni::{
    blocking::{Handle, TrayMethods as _},
    menu,
};

use crate::{
    ApplicationId, IconSet, TrayAction, TrayConfig, TrayEvent, TrayItemId, TrayMenuItem,
    TrayPointerButton, native_tray::TrayEventHandler,
};

pub(super) struct PlatformTray {
    handle: Handle<LinuxTray>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl PlatformTray {
    pub(super) fn new(
        application_id: &ApplicationId,
        config: TrayConfig,
        fallback_icons: &IconSet,
        handler: TrayEventHandler,
    ) -> Result<Self, String> {
        let tray = LinuxTray {
            application_id: application_id.as_str().to_owned(),
            config,
            fallback_icons: fallback_icons.clone(),
            handler,
        };
        let handle = tray
            .assume_sni_available(true)
            .spawn()
            .map_err(|error| error.to_string())?;
        Ok(Self { handle })
    }

    pub(super) fn sync(
        &mut self,
        config: TrayConfig,
        fallback_icons: &IconSet,
    ) -> Result<(), String> {
        self.handle
            .update(|tray| {
                tray.config = config;
                tray.fallback_icons = fallback_icons.clone();
            })
            .ok_or_else(|| "Linux tray service has stopped".to_owned())
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl Drop for PlatformTray {
    fn drop(&mut self) {
        self.handle.shutdown().wait();
    }
}

struct LinuxTray {
    application_id: String,
    config: TrayConfig,
    fallback_icons: IconSet,
    handler: TrayEventHandler,
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ksni::Tray for LinuxTray {
    fn id(&self) -> String {
        self.application_id.clone()
    }

    fn title(&self) -> String {
        self.config
            .title
            .clone()
            .or_else(|| self.config.tooltip.clone())
            .unwrap_or_default()
    }

    fn status(&self) -> ksni::Status {
        if self.config.visible {
            ksni::Status::Active
        } else {
            ksni::Status::Passive
        }
    }

    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let icons = self.config.icon.as_ref().unwrap_or(&self.fallback_icons);
        icons
            .icons()
            .iter()
            .filter_map(|icon| {
                let width = i32::try_from(icon.width).ok()?;
                let height = i32::try_from(icon.height).ok()?;
                let mut argb = Vec::with_capacity(icon.rgba8.len());
                for rgba in icon.rgba8.as_chunks::<4>().0 {
                    argb.extend_from_slice(&[rgba[3], rgba[0], rgba[1], rgba[2]]);
                }
                Some(ksni::Icon {
                    width,
                    height,
                    data: argb,
                })
            })
            .collect()
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        (self.handler)(TrayEvent::Click {
            button: TrayPointerButton::Primary,
            double: false,
        });
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        (self.handler)(TrayEvent::Click {
            button: TrayPointerButton::Middle,
            double: false,
        });
    }

    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        linux_menu(&self.config.menu)
    }
}

fn linux_menu(items: &[TrayMenuItem]) -> Vec<ksni::MenuItem<LinuxTray>> {
    items
        .iter()
        .map(|item| match item {
            TrayMenuItem::Action {
                id,
                label,
                enabled,
                action,
            } => menu::StandardItem {
                label: label.clone(),
                enabled: *enabled,
                activate: activation(id.clone(), action.clone()),
                ..menu::StandardItem::default()
            }
            .into(),
            TrayMenuItem::Check {
                id,
                label,
                enabled,
                checked,
                action,
            } => menu::CheckmarkItem {
                label: label.clone(),
                enabled: *enabled,
                checked: *checked,
                activate: activation(id.clone(), action.clone()),
                ..menu::CheckmarkItem::default()
            }
            .into(),
            TrayMenuItem::Separator => ksni::MenuItem::Separator,
            TrayMenuItem::Submenu {
                label,
                enabled,
                items,
                ..
            } => menu::SubMenu {
                label: label.clone(),
                enabled: *enabled,
                submenu: linux_menu(items),
                ..menu::SubMenu::default()
            }
            .into(),
        })
        .collect()
}

fn activation(id: TrayItemId, action: TrayAction) -> Box<dyn Fn(&mut LinuxTray) + Send + Sync> {
    Box::new(move |tray| {
        (tray.handler)(TrayEvent::Action {
            id: id.clone(),
            action: action.clone(),
        });
    })
}
