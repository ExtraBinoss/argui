#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use std::sync::Arc;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use argui_platform::{ApplicationConfig, CloseBehavior, WindowKey, WindowSpec};
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use argui_platform::{TrayAction, TrayEvent};
use argui_render::RendererConfig;
use argui_text::TextEngine;
use argui_ui::{Element, UiTree};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, EventLoopProxy},
    window::WindowId,
};

use crate::{
    AppCommand, AppEvent, AppModel, AppUpdate, Context, Entity, LayoutSnapshot, Render,
    RuntimeError, RuntimeEvent, ViewUpdate, app::Application, event::UserEvent,
};

type SharedModel = Rc<RefCell<Box<dyn AppModel>>>;
type SharedUpdates = Rc<RefCell<Vec<AppUpdate>>>;
type SharedCallback = Rc<RefCell<Box<dyn FnMut(RuntimeEvent)>>>;

struct WindowEntry {
    spec: WindowSpec,
    runtime: Application,
}

pub(crate) struct MultiApplication {
    config: ApplicationConfig,
    renderer_config: RendererConfig,
    initial_text_engine: Option<TextEngine>,
    renderer_device: Rc<RefCell<Option<argui_render::RendererDevice>>>,
    model: SharedModel,
    pending: SharedUpdates,
    callback: SharedCallback,
    windows: HashMap<WindowKey, WindowEntry>,
    by_winit: HashMap<WindowId, WindowKey>,
    event_proxy: Option<EventLoopProxy<UserEvent>>,
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    native_tray: Option<argui_platform::NativeTray>,
    #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
    tray_unavailable_announced: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fatal_error: Option<RuntimeError>,
}

// Window creation, visibility, tray registration, and command dispatch all require a live
// `ActiveEventLoop`. Keep policy/data transformations below this boundary independently tested.
#[cfg_attr(coverage_nightly, coverage(off))]
impl MultiApplication {
    pub(crate) fn new(
        config: ApplicationConfig,
        renderer_config: RendererConfig,
        model: impl AppModel,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Result<Self, RuntimeError> {
        Self::new_with_text_engine(config, renderer_config, None, model, on_event)
    }

    pub(crate) fn new_with_text_engine(
        config: ApplicationConfig,
        renderer_config: RendererConfig,
        text_engine: Option<TextEngine>,
        model: impl AppModel,
        on_event: impl FnMut(RuntimeEvent) + 'static,
    ) -> Result<Self, RuntimeError> {
        config
            .validate()
            .map_err(|error| RuntimeError::Configuration(error.to_string()))?;
        Ok(Self {
            config,
            renderer_config,
            initial_text_engine: text_engine,
            renderer_device: Rc::new(RefCell::new(None)),
            model: Rc::new(RefCell::new(Box::new(model))),
            pending: Rc::new(RefCell::new(Vec::new())),
            callback: Rc::new(RefCell::new(Box::new(on_event))),
            windows: HashMap::new(),
            by_winit: HashMap::new(),
            event_proxy: None,
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            native_tray: None,
            #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
            tray_unavailable_announced: false,
            #[cfg(not(target_arch = "wasm32"))]
            fatal_error: None,
        })
    }

    pub(crate) fn set_event_proxy(&mut self, proxy: EventLoopProxy<UserEvent>) {
        self.event_proxy = Some(proxy);
    }

    fn open_window(&mut self, event_loop: &ActiveEventLoop, spec: WindowSpec) {
        if self.windows.contains_key(&spec.key) {
            self.emit(RuntimeEvent::CommandFailed(format!(
                "window already exists: {}",
                spec.key.as_str()
            )));
            return;
        }
        let key = spec.key.clone();
        let adapter = WindowModel {
            key: key.clone(),
            model: Rc::clone(&self.model),
            pending: Rc::clone(&self.pending),
        };
        let root = adapter
            .view(crate::WindowEnvironment::default())
            .unwrap_or_else(|| Element::container(Vec::<Element>::new()));
        let callback = scoped_callback(
            key.clone(),
            Rc::clone(&self.model),
            Rc::clone(&self.pending),
            Rc::clone(&self.callback),
        );
        let mut runtime = Application::new(
            spec.window.clone(),
            self.renderer_config.clone(),
            self.initial_text_engine.take().unwrap_or_default(),
            None,
            Some(UiTree::new(root)),
            Some(Entity::new(adapter).erase()),
            callback,
        )
        .identified(self.config.identity.clone(), key.clone())
        .initially_visible(spec.visible)
        .preference_overrides(self.config.preferences)
        .shared_renderer_device(Rc::clone(&self.renderer_device));
        if let Some(proxy) = &self.event_proxy {
            runtime.set_event_proxy(proxy.clone());
        }
        runtime.resumed(event_loop);
        let Some(window_id) = runtime.window_id() else {
            return;
        };
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = runtime.window()
            && let Err(error) = argui_platform::attach_web_canvas(
                window,
                &key,
                spec.window.web_parent_id.as_deref(),
            )
        {
            self.emit(RuntimeEvent::CommandFailed(error));
        }
        #[cfg(target_arch = "wasm32")]
        if let Err(error) = runtime.initialize_web_accessibility() {
            self.emit(RuntimeEvent::CommandFailed(error));
        }
        self.by_winit.insert(window_id, key.clone());
        self.windows.insert(key, WindowEntry { spec, runtime });
    }

    fn close_window(&mut self, key: &WindowKey) {
        if let Some(entry) = self.windows.remove(key)
            && let Some(window_id) = entry.runtime.window_id()
        {
            self.by_winit.remove(&window_id);
        }
    }

    fn process_pending(&mut self, event_loop: &ActiveEventLoop) {
        for _ in 0..64 {
            let updates = std::mem::take(&mut *self.pending.borrow_mut());
            if updates.is_empty() {
                return;
            }
            for update in updates {
                for invalidation in update.windows {
                    if let Some(entry) = self.windows.get_mut(&invalidation.window) {
                        entry.runtime.invalidate(invalidation.update);
                    }
                }
                if update.tray_changed {
                    self.sync_tray();
                }
                for command in update.commands {
                    self.apply_command(event_loop, command);
                }
            }
        }
        self.emit(RuntimeEvent::CommandFailed(
            "application command loop exceeded 64 passes".into(),
        ));
    }

    fn apply_command(&mut self, event_loop: &ActiveEventLoop, command: AppCommand) {
        match command {
            AppCommand::OpenWindow(spec) => self.open_window(event_loop, spec),
            AppCommand::CloseWindow(key) => self.close_window(&key),
            AppCommand::ShowWindow(key) => self.set_visible(&key, true),
            AppCommand::HideWindow(key) => self.set_visible(&key, false),
            AppCommand::ToggleWindow(key) => {
                if let Some(window) = self
                    .windows
                    .get(&key)
                    .and_then(|entry| entry.runtime.window())
                {
                    self.set_visible(&key, !window.is_visible().unwrap_or(true));
                }
            }
            AppCommand::FocusWindow(key) => {
                if let Some(window) = self
                    .windows
                    .get(&key)
                    .and_then(|entry| entry.runtime.window())
                {
                    window.set_visible(true);
                    window.focus_window();
                }
            }
            AppCommand::SetWindowTitle { window, title } => {
                if let Some(window) = self
                    .windows
                    .get(&window)
                    .and_then(|entry| entry.runtime.window())
                {
                    window.set_title(&title);
                }
            }
            AppCommand::Quit => event_loop.exit(),
        }
    }

    fn set_visible(&self, key: &WindowKey, visible: bool) {
        if let Some(window) = self
            .windows
            .get(key)
            .and_then(|entry| entry.runtime.window())
        {
            window.set_visible(visible);
        }
    }

    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    fn tray_event(&mut self, event_loop: &ActiveEventLoop, event: TrayEvent) {
        self.emit(RuntimeEvent::Tray(event.clone()));
        if let TrayEvent::Action { action, .. } = &event
            && let Some(command) = tray_command(action.clone())
        {
            self.apply_command(event_loop, command);
            return;
        }
        let update = self.model.borrow_mut().update(&AppEvent::Tray(event));
        self.pending.borrow_mut().push(update);
        self.process_pending(event_loop);
    }

    fn sync_tray(&mut self) {
        let config = self
            .model
            .borrow()
            .tray()
            .or_else(|| self.config.tray.clone());
        let Some(config) = config else {
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            {
                self.native_tray = None;
            }
            return;
        };

        #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
        {
            let result = if let Some(tray) = &mut self.native_tray {
                tray.sync(config, &self.config.identity.icons)
            } else {
                let Some(proxy) = self.event_proxy.clone() else {
                    return;
                };
                let handler = Arc::new(move |event| {
                    let _ = proxy.send_event(UserEvent::Tray(event));
                });
                argui_platform::NativeTray::new(
                    &self.config.identity.id,
                    config,
                    &self.config.identity.icons,
                    handler,
                )
                .map(|tray| self.native_tray = Some(tray))
            };
            if let Err(error) = result {
                self.emit(RuntimeEvent::TrayFailed(error));
            }
        }

        #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
        if !self.tray_unavailable_announced {
            let _ = config;
            self.tray_unavailable_announced = true;
            self.emit(RuntimeEvent::TrayUnavailable(
                if cfg!(target_arch = "wasm32") {
                    "system tray is unavailable on the Web"
                } else {
                    "rebuild Argui with the `tray` feature"
                }
                .into(),
            ));
        }
    }

    fn emit(&self, event: RuntimeEvent) {
        (self.callback.borrow_mut())(event);
    }
}

#[cfg_attr(coverage_nightly, coverage(off))]
impl ApplicationHandler<UserEvent> for MultiApplication {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        #[cfg(target_arch = "wasm32")]
        if let Err(error) = argui_platform::apply_web_identity(&self.config.identity) {
            self.emit(RuntimeEvent::CommandFailed(error));
        }
        let initial = self.config.windows.clone();
        for spec in initial {
            self.open_window(event_loop, spec);
        }
        self.sync_tray();
        self.process_pending(event_loop);
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        for entry in self.windows.values_mut() {
            entry.runtime.suspended(event_loop);
        }
    }

    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let _ = event_loop;
        match event {
            UserEvent::Preferences { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(target_arch = "wasm32")]
            UserEvent::ClipboardText { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(target_arch = "wasm32")]
            UserEvent::Accessibility { ref window, .. } => {
                if let Some(entry) = self.windows.get_mut(window) {
                    entry.runtime.user_event(event_loop, event);
                }
            }
            #[cfg(not(target_arch = "wasm32"))]
            UserEvent::AccessKit(event) => {
                if let Some(key) = self.by_winit.get(&event.window_id).cloned()
                    && let Some(entry) = self.windows.get_mut(&key)
                {
                    entry
                        .runtime
                        .user_event(event_loop, UserEvent::AccessKit(event));
                }
            }
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            UserEvent::Tray(event) => self.tray_event(event_loop, event),
        }
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        let Some(key) = self.by_winit.get(&window_id).cloned() else {
            return;
        };
        let close = matches!(event, WindowEvent::CloseRequested);
        if let Some(entry) = self.windows.get_mut(&key) {
            entry.runtime.window_event(event_loop, window_id, event);
        }
        self.process_pending(event_loop);
        if !close {
            return;
        }
        let behavior = self.windows.get(&key).map_or(CloseBehavior::Quit, |entry| {
            entry.spec.window.close_behavior
        });
        match behavior {
            CloseBehavior::Quit => event_loop.exit(),
            CloseBehavior::CloseWindow => self.close_window(&key),
            CloseBehavior::Hide => self.set_visible(&key, false),
            CloseBehavior::NotifyApp => {}
        }
    }
}

#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
fn tray_command(action: TrayAction) -> Option<AppCommand> {
    match action {
        TrayAction::Custom(_) => None,
        TrayAction::ShowWindow(key) => Some(AppCommand::ShowWindow(key)),
        TrayAction::HideWindow(key) => Some(AppCommand::HideWindow(key)),
        TrayAction::ToggleWindow(key) => Some(AppCommand::ToggleWindow(key)),
        TrayAction::FocusWindow(key) => Some(AppCommand::FocusWindow(key)),
        TrayAction::CloseWindow(key) => Some(AppCommand::CloseWindow(key)),
        TrayAction::Quit => Some(AppCommand::Quit),
    }
}

fn scoped_callback(
    key: WindowKey,
    model: SharedModel,
    pending: SharedUpdates,
    callback: SharedCallback,
) -> impl FnMut(RuntimeEvent) {
    move |event| {
        if let RuntimeEvent::Platform(platform) = &event {
            let update = model.borrow_mut().update(&AppEvent::Window {
                window: key.clone(),
                event: platform.clone(),
            });
            pending.borrow_mut().push(update);
        }
        (callback.borrow_mut())(event.scoped(key.clone()));
    }
}

struct WindowModel {
    key: WindowKey,
    model: SharedModel,
    pending: SharedUpdates,
}

impl WindowModel {
    fn view(&self, environment: crate::WindowEnvironment) -> Option<Element> {
        self.model.borrow().view(&self.key, environment)
    }

    fn record(&self, mut update: AppUpdate) -> ViewUpdate {
        let own = update
            .windows
            .iter()
            .filter(|candidate| candidate.window == self.key)
            .fold(ViewUpdate::None, |current, candidate| {
                if candidate.update == ViewUpdate::Rebuild || current == ViewUpdate::Rebuild {
                    ViewUpdate::Rebuild
                } else if candidate.update == ViewUpdate::Paint || current == ViewUpdate::Paint {
                    ViewUpdate::Paint
                } else {
                    ViewUpdate::None
                }
            });
        update
            .windows
            .retain(|candidate| candidate.window != self.key);
        if !update.windows.is_empty() || !update.commands.is_empty() || update.tray_changed {
            self.pending.borrow_mut().push(update);
        }
        own
    }
}

impl Render for WindowModel {
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        self.view(cx.environment())
            .unwrap_or_else(|| Element::container(Vec::<Element>::new()))
    }

    fn event(&mut self, event: &argui_ui::UiEvent, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().update(&AppEvent::Ui {
            window: self.key.clone(),
            event: event.clone(),
        });
        request_update(cx, self.record(update));
        if let Some(request) = self.model.borrow_mut().take_clipboard_request(&self.key) {
            cx.write_clipboard(request);
        }
        if let Some(request) = self.model.borrow_mut().take_scroll_request(&self.key) {
            cx.scroll_to(request.key, request.offset);
        }
        if let Some(request) = self.model.borrow_mut().take_focus_request(&self.key) {
            match request {
                argui_ui::FocusRequest::Focus(target) => cx.request_focus(target),
                argui_ui::FocusRequest::Clear => cx.clear_focus(),
            }
        }
        if let Some(request) = self.model.borrow_mut().take_theme_request(&self.key) {
            cx.set_theme(request);
        }
    }

    fn animation_frame(&mut self, frame: argui_animation::Frame, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().animation_frame(&self.key, frame);
        request_update(cx, self.record(update));
    }

    fn wants_animation_frame(&self) -> bool {
        self.model.borrow().wants_animation_frame(&self.key)
    }

    fn layout_changed(&mut self, layout: &LayoutSnapshot, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().layout_changed(&self.key, layout);
        request_update(cx, self.record(update));
    }

    fn image_assets(&self) -> Vec<argui_paint::ImageAsset> {
        self.model.borrow().image_assets()
    }

    fn vector_assets(&self) -> Vec<argui_paint::VectorAsset> {
        self.model.borrow().vector_assets()
    }

    fn inspector(&self) -> Option<argui_inspect::InspectorHandle> {
        self.model.borrow().inspector(&self.key)
    }
}

fn request_update<T: Render>(cx: &mut Context<T>, update: ViewUpdate) {
    match update {
        ViewUpdate::None => {}
        ViewUpdate::Paint => cx.request_paint(),
        ViewUpdate::Rebuild => cx.notify(),
    }
}

#[cfg(all(test, feature = "tray", not(target_arch = "wasm32")))]
mod tests;
