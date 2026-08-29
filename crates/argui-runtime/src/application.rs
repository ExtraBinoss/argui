use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{PlatformEvent, TrayConfig, TrayEvent, WindowKey, WindowSpec};
use argui_ui::{ClipboardRequest, Element, FocusRequest, UiEvent};

use crate::{Entity, LayoutSnapshot, Render, ScrollRequest, ViewUpdate};

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

    fn take_focus_request(&mut self, _window: &WindowKey) -> Option<FocusRequest> {
        None
    }
}

pub(crate) struct SingleWindowModel<A: Render> {
    app: Entity<A>,
    clipboard: std::cell::RefCell<Option<ClipboardRequest>>,
    scroll: std::cell::RefCell<Option<ScrollRequest>>,
    focus: std::cell::RefCell<Option<FocusRequest>>,
    animation_requested: std::cell::Cell<bool>,
}

impl<A: Render> SingleWindowModel<A> {
    pub(crate) fn new(app: A) -> Self {
        Self {
            app: Entity::new(app),
            clipboard: std::cell::RefCell::new(None),
            scroll: std::cell::RefCell::new(None),
            focus: std::cell::RefCell::new(None),
            animation_requested: std::cell::Cell::new(false),
        }
    }

    fn drain_effects(&self, window: &WindowKey) -> AppUpdate {
        let effects = self.app.take_effects();
        if effects.clipboard.is_some() {
            *self.clipboard.borrow_mut() = effects.clipboard;
        }
        if effects.scroll.is_some() {
            *self.scroll.borrow_mut() = effects.scroll;
        }
        if effects.focus.is_some() {
            *self.focus.borrow_mut() = effects.focus;
        }
        self.animation_requested
            .set(self.animation_requested.get() || effects.animation_frame);
        AppUpdate {
            windows: (effects.update != ViewUpdate::None)
                .then(|| WindowInvalidation {
                    window: window.clone(),
                    update: effects.update,
                })
                .into_iter()
                .collect(),
            commands: effects.commands,
            tray_changed: false,
        }
    }
}

impl<A: Render> AppModel for SingleWindowModel<A> {
    fn view(&self, window: &WindowKey) -> Option<Element> {
        (window.as_str() == WindowKey::MAIN_VALUE).then(|| self.app.render())
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Ui { window, event } if window.as_str() == WindowKey::MAIN_VALUE => {
                self.app.event(event);
                self.drain_effects(window)
            }
            _ => AppUpdate::none(),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        self.animation_requested.set(false);
        self.app.animation_frame(frame);
        self.drain_effects(window)
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        window.as_str() == WindowKey::MAIN_VALUE
            && (self.animation_requested.get() || self.app.wants_frame())
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        self.app.layout_changed(layout);
        self.drain_effects(window)
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.app.read(Render::vector_assets)
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.app.read(Render::inspector))
            .flatten()
    }

    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.clipboard.borrow_mut().take())
            .flatten()
    }

    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.scroll.borrow_mut().take())
            .flatten()
    }

    fn take_focus_request(&mut self, window: &WindowKey) -> Option<FocusRequest> {
        (window.as_str() == WindowKey::MAIN_VALUE)
            .then(|| self.focus.borrow_mut().take())
            .flatten()
    }
}
