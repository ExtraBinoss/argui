use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{PlatformEvent, TrayConfig, TrayEvent, WindowKey, WindowSpec};
use argui_render::EffectShader;
use argui_ui::{ClipboardRequest, Element, UiEvent};

use crate::{LayoutSnapshot, ScrollRequest, UiApp, ViewUpdate};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    Ui {
        window: WindowKey,
        event: UiEvent,
    },
    Window {
        window: WindowKey,
        event: PlatformEvent,
    },
    Tray(TrayEvent),
}

#[derive(Clone, Debug, PartialEq)]
pub enum AppCommand {
    OpenWindow(WindowSpec),
    CloseWindow(WindowKey),
    ShowWindow(WindowKey),
    HideWindow(WindowKey),
    ToggleWindow(WindowKey),
    FocusWindow(WindowKey),
    SetWindowTitle { window: WindowKey, title: String },
    Quit,
}

#[derive(Clone, Debug, PartialEq)]
pub struct WindowInvalidation {
    pub window: WindowKey,
    pub update: ViewUpdate,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct AppUpdate {
    pub windows: Vec<WindowInvalidation>,
    pub commands: Vec<AppCommand>,
    pub tray_changed: bool,
}

impl AppUpdate {
    #[must_use]
    pub const fn none() -> Self {
        Self {
            windows: Vec::new(),
            commands: Vec::new(),
            tray_changed: false,
        }
    }

    #[must_use]
    pub fn window(mut self, window: WindowKey, update: ViewUpdate) -> Self {
        self.invalidate(window, update);
        self
    }

    #[must_use]
    pub fn command(mut self, command: AppCommand) -> Self {
        self.commands.push(command);
        self
    }

    #[must_use]
    pub fn tray_changed(mut self) -> Self {
        self.tray_changed = true;
        self
    }

    pub fn invalidate(&mut self, window: WindowKey, update: ViewUpdate) {
        if update == ViewUpdate::None {
            return;
        }
        if let Some(existing) = self
            .windows
            .iter_mut()
            .find(|candidate| candidate.window == window)
        {
            existing.update = strongest(existing.update, update);
        } else {
            self.windows.push(WindowInvalidation { window, update });
        }
    }
}

fn strongest(left: ViewUpdate, right: ViewUpdate) -> ViewUpdate {
    match (left, right) {
        (ViewUpdate::Rebuild, _) | (_, ViewUpdate::Rebuild) => ViewUpdate::Rebuild,
        (ViewUpdate::Paint, _) | (_, ViewUpdate::Paint) => ViewUpdate::Paint,
        _ => ViewUpdate::None,
    }
}

pub trait AppModel: 'static {
    fn view(&self, window: &WindowKey) -> Option<Element>;

    fn update(&mut self, event: &AppEvent) -> AppUpdate;

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        let _ = (window, frame);
        AppUpdate::none()
    }

    fn wants_animation_frame(&self, _window: &WindowKey) -> bool {
        false
    }

    fn layout_changed(&mut self, _window: &WindowKey, _layout: &LayoutSnapshot) -> AppUpdate {
        AppUpdate::none()
    }

    fn tray(&self) -> Option<TrayConfig> {
        None
    }

    fn effect_shaders(&self) -> &'static [EffectShader] {
        &[]
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    fn inspector(&self, _window: &WindowKey) -> Option<InspectorHandle> {
        None
    }

    fn take_clipboard_request(&mut self, _window: &WindowKey) -> Option<ClipboardRequest> {
        None
    }

    fn take_scroll_request(&mut self, _window: &WindowKey) -> Option<ScrollRequest> {
        None
    }
}

pub(crate) struct SingleWindowModel<A> {
    app: A,
}

impl<A> SingleWindowModel<A> {
    pub(crate) const fn new(app: A) -> Self {
        Self { app }
    }
}

impl<A: UiApp> AppModel for SingleWindowModel<A> {
    fn view(&self, window: &WindowKey) -> Option<Element> {
        (window.as_str() == WindowKey::MAIN_VALUE).then(|| self.app.view())
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Ui { window, event } if window.as_str() == WindowKey::MAIN_VALUE => {
                AppUpdate::none().window(window.clone(), self.app.update(event))
            }
            _ => AppUpdate::none(),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        AppUpdate::none().window(window.clone(), self.app.animation_frame(frame))
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        window.as_str() == WindowKey::MAIN_VALUE && self.app.wants_animation_frame()
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        AppUpdate::none().window(window.clone(), self.app.layout_changed(layout))
    }

    fn effect_shaders(&self) -> &'static [EffectShader] {
        self.app.effect_shaders()
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.image_assets()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.app.vector_assets()
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.app.inspector())
            .flatten()
    }

    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.app.take_clipboard_request())
            .flatten()
    }

    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.app.take_scroll_request())
            .flatten()
    }
}
