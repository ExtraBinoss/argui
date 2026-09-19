use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{
    GlobalShortcutEvent, PlatformEvent, TrayConfig, TrayEvent, WindowKey, WindowLevel, WindowSpec,
};
use argui_render::DamageTracking;
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
    /// A registered system-wide keyboard shortcut changed state.
    GlobalShortcut(GlobalShortcutEvent),
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
    /// Set an explicit safe-area override, or return to native detection with `None`.
    SetSafeAreaInsets {
        window: WindowKey,
        insets: Option<argui_core::Insets>,
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
    /// Changes adaptive damage rendering for one window at runtime.
    ///
    /// Use [`DamageTracking::enabled`] for the adaptive policy,
    /// [`DamageTracking::disabled`] to force full-frame rendering, or customize
    /// the enabled policy's region and area thresholds.
    SetDamageTracking {
        window: WindowKey,
        tracking: DamageTracking,
    },
    /// Enables or disables renderer profile events for one window.
    ///
    /// GPU timing is available when the renderer was initialized with profiling
    /// support, including windows hosted by DevTools.
    SetRendererProfiling {
        window: WindowKey,
        enabled: bool,
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
    /// Creates an update with no invalidations, commands, or tray changes.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            windows: Vec::new(),
            commands: Vec::new(),
            tray_changed: false,
        }
    }

    #[must_use]
    /// Adds or combines an invalidation for one window.
    /// `window` is the target window and `update` describes the required redraw.
    pub fn window(mut self, window: WindowKey, update: ViewUpdate) -> Self {
        self.invalidate(window, update);
        self
    }

    #[must_use]
    /// Adds a command for the host to process.
    /// `command` describes the requested operation.
    pub fn command(mut self, command: AppCommand) -> Self {
        self.commands.push(command);
        self
    }

    #[must_use]
    /// Marks the tray configuration as changed.
    pub fn tray_changed(mut self) -> Self {
        self.tray_changed = true;
        self
    }

    /// Adds or strengthens an invalidation for `window` using `update`.
    /// A `None` update is ignored; repeated invalidations keep the strongest update.
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
    /// Takes pending UI commands for a window. The default implementation has none.
    ///
    /// `_window` identifies the window whose pending commands are requested.
    fn take_ui_commands(&mut self, _window: &WindowKey) -> Vec<argui_ui::UiCommand> {
        Vec::new()
    }
    /// Handles completed work associated with the window and returns its effects.
    ///
    /// `_window` identifies the window whose completed work is being delivered.
    fn tasks_ready(&mut self, _window: &WindowKey) -> AppUpdate {
        AppUpdate::none()
    }
    /// Builds a window's current view using its platform environment.
    /// `None` indicates that this application has no view for the window.
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element>;

    /// Returns the retained entity that should receive routed UI events, if any.
    ///
    /// `_window` identifies the window whose event router is requested.
    fn event_router(&self, _window: &WindowKey) -> Option<crate::AnyEntity> {
        None
    }

    /// Allows a composed application to observe events before a retained child router.
    fn captures_ui_events(&self) -> bool {
        false
    }

    /// Updates application state for `event` and returns the resulting effects.
    fn update(&mut self, event: &AppEvent) -> AppUpdate;

    /// Advances time-dependent state for `window` by one animation `frame`.
    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        let _ = (window, frame);
        AppUpdate::none()
    }

    /// Returns whether the application needs animation frames for the window.
    ///
    /// `_window` identifies the window whose animation demand is queried.
    fn wants_animation_frame(&self, _window: &WindowKey) -> bool {
        false
    }

    /// Handles a new layout snapshot for the window.
    ///
    /// `_window` identifies the window whose layout changed; `_layout` is its new
    /// layout snapshot.
    fn layout_changed(&mut self, _window: &WindowKey, _layout: &LayoutSnapshot) -> AppUpdate {
        AppUpdate::none()
    }

    /// Returns tray configuration when this application provides a tray.
    fn tray(&self) -> Option<TrayConfig> {
        None
    }

    /// Returns image assets referenced by the application.
    fn image_assets(&self) -> Vec<ImageAsset> {
        Vec::new()
    }

    /// Returns vector assets referenced by the application.
    fn vector_assets(&self) -> Vec<VectorAsset> {
        Vec::new()
    }

    /// Returns the inspection handle for the window, if inspection is enabled.
    ///
    /// `_window` identifies the window whose inspector is requested.
    fn inspector(&self, _window: &WindowKey) -> Option<InspectorHandle> {
        None
    }

    /// Takes a pending clipboard request for the window, if present.
    ///
    /// `_window` identifies the window whose request is taken.
    fn take_clipboard_request(&mut self, _window: &WindowKey) -> Option<ClipboardRequest> {
        None
    }

    /// Takes a pending scroll request for the window, if present.
    ///
    /// `_window` identifies the window whose request is taken.
    fn take_scroll_request(&mut self, _window: &WindowKey) -> Option<ScrollRequest> {
        None
    }

    /// Takes a pending focus request for the window, if present.
    ///
    /// `_window` identifies the window whose request is taken.
    fn take_focus_request(&mut self, _window: &WindowKey) -> Option<FocusRequest> {
        None
    }

    /// Takes a pending text-selection request for the window, if present.
    ///
    /// `_window` identifies the window whose request is taken.
    fn take_text_selection_request(&mut self, _window: &WindowKey) -> Option<TextSelectionRequest> {
        None
    }

    /// Takes a pending theme request for the window, if present.
    ///
    /// `_window` identifies the window whose request is taken.
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
    /// Creates a single-window application from a new retained model.
    ///
    /// `app` is the render model presented in the window.
    #[must_use]
    pub fn new(app: A) -> Self {
        Self::from_entity(Entity::new(app)).expect("new application model is open")
    }

    /// Create an independent window presentation in an existing model domain.
    /// Other windows may retain the same entity without sharing this mount.
    ///
    /// # Arguments
    /// * `app` — retained model to present in this window.
    ///
    /// # Errors
    /// Returns [`crate::ScopeClosed`] if the entity's resource scope has closed.
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
        self.app.entity.erase().image_assets()
    }

    fn vector_assets(&self) -> Vec<VectorAsset> {
        self.app.entity.erase().vector_assets()
    }

    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        (window == &self.window)
            .then(|| self.app.entity.erase().inspector())
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
