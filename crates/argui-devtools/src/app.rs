use std::cell::RefCell;

use argui_animation::Frame;
use argui_inspect::InspectorHandle;
use argui_paint::{ImageAsset, VectorAsset};
use argui_platform::{CloseBehavior, PlatformEvent, WindowConfig, WindowKey, WindowSpec};
use argui_runtime::{
    AnyEntity, AppCommand, AppEvent, AppModel, AppUpdate, Context, LayoutSnapshot, Render,
    ViewUpdate, WindowEnvironment,
};
use argui_ui::{ClipboardRequest, Element, FocusRequest, ScrollRequest, TextSelectionRequest};

use crate::{DockMode, host::DevtoolsHost, view};

pub(crate) struct EmptyApp;

impl Render for EmptyApp {
    fn render(&mut self, _: &mut Context<Self>) -> Element { Element::container([]) }
}

/// Adds a shared inspection session to an application, including a native tools window.
pub struct DevtoolsApp<M> {
    app: M,
    tools: RefCell<DevtoolsHost<EmptyApp>>,
    target: WindowKey,
    detached: WindowKey,
    pending_detach: bool,
    last_dock: DockMode,
    error: Option<String>,
}

impl<M: AppModel> DevtoolsApp<M> {
    #[must_use]
    pub fn new(app: M) -> Self {
        let mut tools = DevtoolsHost::new(EmptyApp);
        tools.detach_available = !cfg!(target_arch = "wasm32");
        Self {
            app, tools: RefCell::new(tools), target: WindowKey::main(),
            detached: WindowKey::new("__argui-devtools"), pending_detach: false,
            last_dock: DockMode::Bottom, error: None,
        }
    }

    #[must_use]
    pub fn target(mut self, target: WindowKey) -> Self {
        assert!(target != self.detached);
        self.target = target;
        self
    }

    #[must_use]
    pub fn open(mut self, open: bool) -> Self {
        let tools = self.tools.get_mut();
        tools.set_open_immediate(open);
        self
    }

    #[must_use]
    pub fn inspector(&self) -> InspectorHandle { self.tools.borrow().inspector() }

    #[must_use]
    pub fn dock_mode(&self) -> DockMode { self.tools.borrow().dock_mode }

    #[must_use]
    pub fn tools_window(&self) -> &WindowKey {
        if self.dock_mode() == DockMode::Detached { &self.detached } else { &self.target }
    }

    /// Requests presentation changes; the dock stays visible until the new surface is ready.
    pub fn set_dock_mode(&mut self, mode: DockMode) -> AppUpdate {
        if mode == DockMode::Detached {
            if cfg!(target_arch = "wasm32") { return AppUpdate::none(); }
            if self.pending_detach || self.dock_mode() == mode {
                return AppUpdate::none().command(AppCommand::FocusWindow(self.detached.clone()));
            }
            self.pending_detach = true;
            self.error = None;
            self.last_dock = self.dock_mode();
            return AppUpdate::none().command(AppCommand::OpenWindow(WindowSpec::new(
                self.detached.clone(),
                WindowConfig { title: "Argui DevTools".into(), width: 900.0, height: 650.0, close_behavior: CloseBehavior::NotifyApp, ..WindowConfig::default() },
            )));
        }
        let close = self.pending_detach || self.dock_mode() == DockMode::Detached;
        self.pending_detach = false;
        self.last_dock = mode;
        self.tools.get_mut().set_dock_mode(mode);
        let update = AppUpdate::none().window(self.target.clone(), ViewUpdate::Rebuild);
        if close { update.command(AppCommand::CloseWindow(self.detached.clone())) } else { update }
    }

    fn close_tools(&mut self) -> AppUpdate {
        let mut update = self.set_dock_mode(self.last_dock);
        let tools = self.tools.get_mut();
        tools.set_open_immediate(false);
        tools.picking = false;
        tools.inspector.set_hovered(None);
        tools.inspector.set_recording(false);
        update.invalidate(self.target.clone(), ViewUpdate::Rebuild);
        update
    }

    fn failed(&mut self, error: &str) -> AppUpdate {
        self.error = Some(error.to_owned());
        self.set_dock_mode(self.last_dock)
    }

    fn merge(mut update: AppUpdate, other: AppUpdate) -> AppUpdate {
        for invalidation in other.windows { update.invalidate(invalidation.window, invalidation.update); }
        update.commands.extend(other.commands);
        update.tray_changed |= other.tray_changed;
        update
    }
}

impl<M: AppModel> AppModel for DevtoolsApp<M> {
    fn view(&self, window: &WindowKey, environment: WindowEnvironment) -> Option<Element> {
        let themes = argui_widgets::shadcn(environment.primary);
        let theme = themes.resolve(environment.color_scheme);
        if window == &self.detached {
            let tools = self.tools.borrow();
            return Some(view::dock(&tools, tools.viewport.size.height.max(1.0), theme));
        }
        let app = self.app.view(window, environment)?;
        if window != &self.target { return Some(app); }
        let tools = self.tools.borrow();
        let mut root = view::host(&tools, app, theme, None);
        if let Some(error) = &self.error {
            root.children.push(Element::text(format!("Could not open developer tools window: {error}")));
        }
        Some(root)
    }

    fn event_router(&self, window: &WindowKey) -> Option<AnyEntity> {
        if window == &self.detached { None } else { self.app.event_router(window) }
    }

    fn captures_ui_events(&self) -> bool { true }

    fn update(&mut self, event: &AppEvent) -> AppUpdate {
        match event {
            AppEvent::WindowReady { window } if window == &self.detached && self.pending_detach => {
                self.pending_detach = false;
                self.tools.get_mut().set_dock_mode(DockMode::Detached);
                AppUpdate::none().window(self.target.clone(), ViewUpdate::Rebuild)
                    .window(self.detached.clone(), ViewUpdate::Rebuild)
                    .command(AppCommand::FocusWindow(self.detached.clone()))
            }
            AppEvent::WindowFailed { window, error } if window == &self.detached => self.failed(error),
            AppEvent::Window { window, event: PlatformEvent::WindowCreationFailed(error) } if window == &self.detached => self.failed(error),
            AppEvent::Window { window, event: PlatformEvent::CloseRequested } if window == &self.detached => self.close_tools(),
            AppEvent::Window { window, event: PlatformEvent::CloseRequested } if window == &self.target => {
                let update = self.app.update(event);
                let close = self.close_tools();
                Self::merge(update, close)
            }
            AppEvent::Ui { window, event } if window == &self.target || window == &self.detached => {
                if event.target_key().is_some_and(|key| key.starts_with("__devtools")) {
                    let change = self.tools.get_mut().update(event);
                    let mode = self.tools.get_mut().requested_mode.take();
                    let mut update = AppUpdate::none().window(self.tools_window().clone(), change);
                    // Picking and property overrides must invalidate the inspected surface too.
                    update.invalidate(self.target.clone(), change);
                    if let Some(mode) = mode {
                        update = Self::merge(update, self.set_dock_mode(mode));
                    }
                    if !self.tools.get_mut().open && (window == &self.detached || self.pending_detach) {
                        update = Self::merge(update, self.close_tools());
                    }
                    update
                } else if self.app.event_router(window).is_none() {
                    self.app.update(&AppEvent::Ui { window: window.clone(), event: event.clone() })
                } else { AppUpdate::none() }
            }
            _ => self.app.update(event),
        }
    }

    fn animation_frame(&mut self, window: &WindowKey, frame: Frame) -> AppUpdate {
        let mut update = if window == &self.detached { AppUpdate::none() } else { self.app.animation_frame(window, frame) };
        if window == self.tools_window() {
            let change = self.tools.get_mut().animation_frame(frame);
            update.invalidate(window.clone(), change);
        }
        update
    }

    fn wants_animation_frame(&self, window: &WindowKey) -> bool {
        (window != &self.detached && self.app.wants_animation_frame(window))
            || (window == self.tools_window() && self.tools.borrow().wants_animation_frame())
    }

    fn layout_changed(&mut self, window: &WindowKey, layout: &LayoutSnapshot) -> AppUpdate {
        let panel_window = window == self.tools_window();
        let tools = self.tools.get_mut();
        let mut changed = false;
        if panel_window {
            changed = tools.viewport != layout.viewport;
            tools.viewport = layout.viewport;
            if let Some(bounds) = layout.bounds("__devtools-tree") {
                changed |= tools.tree_height != bounds.size.height;
                tools.tree_height = bounds.size.height;
            }
        }
        if window == &self.detached {
            return AppUpdate::none().window(window.clone(), if changed { ViewUpdate::Rebuild } else { ViewUpdate::None });
        }
        let mut application = layout.clone();
        if window == &self.target {
            if let Some(bounds) = layout.bounds("__devtools-app-root") { application.viewport = bounds; }
            tools.app_viewport = application.viewport;
        }
        let mut update = self.app.layout_changed(window, &application);
        if changed { update.invalidate(window.clone(), ViewUpdate::Rebuild); }
        update
    }

    fn image_assets(&self) -> Vec<ImageAsset> { self.app.image_assets() }
    fn vector_assets(&self) -> Vec<VectorAsset> {
        let mut assets = self.app.vector_assets();
        assets.extend_from_slice(self.tools.borrow().icons.assets());
        assets
    }
    fn inspector(&self, window: &WindowKey) -> Option<InspectorHandle> {
        if window == &self.target { Some(self.inspector()) }
        else if window == &self.detached { None } else { self.app.inspector(window) }
    }
    fn tray(&self) -> Option<argui_platform::TrayConfig> { self.app.tray() }
    fn take_clipboard_request(&mut self, window: &WindowKey) -> Option<ClipboardRequest> {
        self.tools.get_mut().take_clipboard_request().or_else(|| self.app.take_clipboard_request(window))
    }
    fn take_scroll_request(&mut self, window: &WindowKey) -> Option<ScrollRequest> {
        self.tools.get_mut().take_scroll_request().or_else(|| self.app.take_scroll_request(window))
    }
    fn take_focus_request(&mut self, window: &WindowKey) -> Option<FocusRequest> {
        if window == self.tools_window() {
            self.tools.get_mut().take_focus_request().or_else(|| self.app.take_focus_request(window))
        } else { self.app.take_focus_request(window) }
    }
    fn take_text_selection_request(&mut self, window: &WindowKey) -> Option<TextSelectionRequest> { self.app.take_text_selection_request(window) }
    fn take_theme_request(&mut self, window: &WindowKey) -> Option<argui_runtime::ThemeRequest> { self.app.take_theme_request(window) }
}
