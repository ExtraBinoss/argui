use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{PlatformEvent, TrayConfig, TrayEvent, WindowKey, WindowLevel, WindowSpec};
use argui_ui::{ClipboardRequest, Element, FocusRequest, TextSelectionRequest, UiEvent};

use crate::{
    Entity, LayoutSnapshot, Render, ScrollRequest, ThemeRequest, ViewUpdate, WindowEnvironment,
};

#[derive(Clone, Debug, PartialEq)]
pub enum AppEvent {
    WindowReady {
        window: WindowKey,
    },
    WindowFailed {
        window: WindowKey,
        error: String,
    },
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
    SetWindowTitle {
        window: WindowKey,
        title: String,
    },
    MinimizeWindow(WindowKey),
    SetWindowMaximized {
        window: WindowKey,
        maximized: bool,
    },
    ToggleWindowMaximized(WindowKey),
    SetWindowLevel {
        window: WindowKey,
        level: WindowLevel,
    },
    SetWindowMousePassthrough {
        window: WindowKey,
        passthrough: bool,
    },
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
    fn take_ui_commands(&mut self, _window: &WindowKey) -> Vec<argui_ui::UiCommand> {
        Vec::new()
    }
    fn tasks_ready(&mut self, _window: &WindowKey) -> AppUpdate {
        AppUpdate::none()
    }
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element>;

    fn event_router(&self, _window: &WindowKey) -> Option<crate::AnyEntity> {
        None
    }

    /// Allows a composed application to observe events before a retained child router.
    fn captures_ui_events(&self) -> bool {
        false
    }

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

    fn take_text_selection_request(&mut self, _window: &WindowKey) -> Option<TextSelectionRequest> {
        None
    }

    fn take_theme_request(&mut self, _window: &WindowKey) -> Option<ThemeRequest> {
        None
    }
}

/// Headless-friendly [`AppModel`] adapter for one retained component window.
pub struct SingleWindowModel<A: Render> {
    window: WindowKey,
    ui_commands: std::cell::RefCell<Vec<argui_ui::UiCommand>>,
    app: crate::Mount<A>,
    clipboard: std::cell::RefCell<Option<ClipboardRequest>>,
    scroll: std::cell::RefCell<Option<ScrollRequest>>,
    focus: std::cell::RefCell<Option<FocusRequest>>,
    text_selection: std::cell::RefCell<Option<TextSelectionRequest>>,
    theme: std::cell::RefCell<Option<ThemeRequest>>,
    animation_requested: std::cell::Cell<bool>,
}

impl<A: Render> SingleWindowModel<A> {
    #[must_use]
    pub fn new(app: A) -> Self {
        Self::from_entity(Entity::new(app)).expect("new application model is open")
    }

    /// Create an independent window presentation in an existing model domain.
    /// Other windows may retain the same entity without sharing this mount.
    pub fn from_entity(app: Entity<A>) -> Result<Self, crate::ScopeClosed> {
        Ok(Self {
            window: WindowKey::main(),
            ui_commands: std::cell::RefCell::new(Vec::new()),
            app: app.mount()?,
            clipboard: std::cell::RefCell::new(None),
            scroll: std::cell::RefCell::new(None),
            focus: std::cell::RefCell::new(None),
            text_selection: std::cell::RefCell::new(None),
            theme: std::cell::RefCell::new(None),
            animation_requested: std::cell::Cell::new(false),
        })
    }

    /// Bind this single presentation to a named application window.
    #[must_use]
    pub fn window_key(mut self, window: WindowKey) -> Self {
        self.window = window;
        self
    }

    fn drain_effects(&self, window: &WindowKey) -> AppUpdate {
        let effects = self.app.entity.take_effects();
        self.ui_commands.borrow_mut().extend(effects.ui_commands);
        if effects.clipboard.is_some() {
            *self.clipboard.borrow_mut() = effects.clipboard;
        }
        if effects.scroll.is_some() {
            *self.scroll.borrow_mut() = effects.scroll;
        }
        if effects.focus.is_some() {
            *self.focus.borrow_mut() = effects.focus;
        }
        if effects.text_selection.is_some() {
            *self.text_selection.borrow_mut() = effects.text_selection;
        }
        if effects.theme.is_some() {
            *self.theme.borrow_mut() = effects.theme;
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

impl<A: Render> Drop for SingleWindowModel<A> {
    fn drop(&mut self) {
        self.app.close();
    }
}

impl<A: Render> AppModel for SingleWindowModel<A> {
    #[cfg(feature = "tasks")]
    fn tasks_ready(&mut self, window: &WindowKey) -> AppUpdate {
        if window != &self.window {
            return AppUpdate::none();
        }
        let effects = self.app.entity.take_task_effects();
        self.app.entity.store_effects(effects);
        self.drain_effects(window)
    }
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        (window == &self.window).then(|| self.app.entity.render_in(environment))
    }

    fn event_router(&self, window: &WindowKey) -> Option<crate::AnyEntity> {
        (window == &self.window).then(|| self.app.entity.erase())
    }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::Window {
                window,
                event: PlatformEvent::VisibilityChanged(visible),
            } if window == &self.window => {
                self.app.entity.erase().set_host_visible(*visible);
                AppUpdate::none()
            }
            AppEvent::Window {
                window,
                event: PlatformEvent::Closed,
            } if window == &self.window => {
                self.app.entity.erase().close_presentation();
                AppUpdate::none()
            }
            AppEvent::Ui { window, event } if window == &self.window => {
                self.app.entity.dispatch_event(event);
                self.drain_effects(window)
            }
            _ => AppUpdate::none(),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        if window != &self.window {
            return AppUpdate::none();
        }
        self.animation_requested.set(false);
        self.app.entity.animation_frame(frame);
        self.drain_effects(window)
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        window == &self.window && (self.animation_requested.get() || self.app.entity.wants_frame())
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        if window != &self.window {
            return AppUpdate::none();
        }
        self.app.entity.layout_changed(layout);
        self.drain_effects(window)
    }

    fn image_assets(&self) -> Vec<ImageAsset> {
        self.app.entity.read(Render::image_assets)
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.app.entity.read(Render::vector_assets)
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        (window == &self.window)
            .then(|| self.app.entity.read(Render::inspector))
            .flatten()
    }

    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        (window == &self.window)
            .then(|| self.clipboard.borrow_mut().take())
            .flatten()
    }

    fn take_ui_commands(&mut self, window: &WindowKey) -> Vec<argui_ui::UiCommand> {
        if window == &self.window {
            self.ui_commands.take()
        } else {
            Vec::new()
        }
    }

    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        (window == &self.window)
            .then(|| self.scroll.borrow_mut().take())
            .flatten()
    }

    fn take_focus_request(&mut self, window: &WindowKey) -> Option<FocusRequest> {
        (window == &self.window)
            .then(|| self.focus.borrow_mut().take())
            .flatten()
    }

    fn take_text_selection_request(&mut self, window: &WindowKey) -> Option<TextSelectionRequest> {
        (window == &self.window)
            .then(|| self.text_selection.borrow_mut().take())
            .flatten()
    }

    fn take_theme_request(&mut self, window: &WindowKey) -> Option<ThemeRequest> {
        (window == &self.window)
            .then(|| self.theme.borrow_mut().take())
            .flatten()
    }
}
