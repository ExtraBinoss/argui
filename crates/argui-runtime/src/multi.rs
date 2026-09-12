#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use std::sync::Arc;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

use argui_platform::{ApplicationConfig, WindowKey, WindowLevel, WindowSpec};
#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use argui_platform::{TrayAction, TrayEvent};
use argui_render::RendererConfig;
use argui_text::TextEngine;
use argui_ui::Element;
use winit::{
    application::ApplicationHandler, event::WindowEvent, event_loop::ActiveEventLoop,
    window::WindowId,
};

#[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
use crate::AppCommand;
use crate::{
    AppEvent, AppModel, AppUpdate, Context, Entity, LayoutSnapshot, Render, RuntimeError,
    RuntimeEvent, ViewUpdate, app::Application, event::UserEvent,
};

mod command;
#[cfg(all(feature = "webview", target_os = "linux"))]
pub(crate) mod gtk;
mod lifecycle;
mod model_updates;

type SharedModel = Rc<RefCell<Box<dyn AppModel>>>;
type SharedUpdates = Rc<RefCell<Vec<AppUpdate>>>;
type SharedCallback = Rc<RefCell<Box<dyn FnMut(RuntimeEvent)>>>;

impl Drop for MultiApplication {
    fn drop(&mut self) {
        self.shutdown();
    }
}

struct WindowEntry {
    spec: WindowSpec,
    runtime: Application,
}

pub(crate) struct MultiApplication {
    #[cfg(feature = "tasks")]
    tasks: Option<crate::tasks::TaskRuntime>,
    config: ApplicationConfig,
    renderer_config: RendererConfig,
    initial_text_engine: Option<TextEngine>,
    renderer_device: Rc<RefCell<Option<argui_render::RendererDevice>>>,
    model: SharedModel,
    pending: SharedUpdates,
    callback: SharedCallback,
    windows: HashMap<WindowKey, WindowEntry>,
    retired: Vec<crate::ModelRuntime>,
    by_native: HashMap<crate::host::HostId, WindowKey>,
    event_proxy: Option<crate::host::EventProxy>,
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    native_tray: Option<argui_platform::NativeTray>,
    #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
    tray_unavailable_announced: bool,
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fatal_error: Option<RuntimeError>,
}

// Window creation, visibility, tray registration, and command dispatch all require a live
// `ActiveEventLoop`. Keep policy/data transformations below this boundary independently tested.
impl MultiApplication {
    #[cfg(feature = "tasks")]
    pub(crate) fn shutdown_tasks(&self) {
        if let Some(tasks) = &self.tasks {
            tasks.shutdown();
        }
    }
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
            retired: Vec::new(),
            by_native: HashMap::new(),
            event_proxy: None,
            #[cfg(feature = "tasks")]
            tasks: None,
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            native_tray: None,
            #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
            tray_unavailable_announced: false,
            #[cfg(not(target_arch = "wasm32"))]
            fatal_error: None,
        })
    }

    pub(crate) fn set_event_proxy(&mut self, proxy: impl Into<crate::host::EventProxy>) {
        let proxy = proxy.into();
        #[cfg(feature = "tasks")]
        {
            let wake = proxy.clone();
            self.tasks = Some(crate::tasks::TaskRuntime::new(move || {
                let _ = wake.send_event(UserEvent::TasksReady);
            }));
        }
        self.event_proxy = Some(proxy);
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn open_window(&mut self, event_loop: &dyn crate::host::WindowFactory, spec: WindowSpec) {
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
            None,
            Some(
                Entity::new(adapter)
                    .mount()
                    .expect("new window model is open")
                    .entity
                    .erase(),
            ),
            callback,
        )
        .identified(self.config.identity.clone(), key.clone())
        .initially_visible(spec.visible)
        .preference_overrides(self.config.preferences)
        .shared_renderer_device(Rc::clone(&self.renderer_device));
        if let Some(proxy) = &self.event_proxy {
            #[cfg(feature = "tasks")]
            {
                runtime.tasks = self.tasks.clone();
            }
            runtime.set_event_proxy(proxy.clone());
        }
        event_loop.open(&mut runtime);
        if let Some(window) = runtime.window() {
            let capabilities = window.capabilities();
            if spec.window.level != WindowLevel::Normal && !capabilities.window_level {
                self.emit(RuntimeEvent::CommandFailed(format!(
                    "window level {:?} is unavailable on this window backend",
                    spec.window.level
                )));
            }
            if spec.window.native_shadow && !capabilities.native_shadow {
                self.emit(RuntimeEvent::CommandFailed(
                    "native window shadows are unavailable on this window backend".into(),
                ));
            }
        }
        let Some(window_id) = runtime.window_id() else {
            return;
        };
        #[cfg(target_arch = "wasm32")]
        if let Some(window) = runtime.window().and_then(crate::host::WindowHost::winit)
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
        self.by_native.insert(window_id, key.clone());
        self.windows.insert(key, WindowEntry { spec, runtime });
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn process_pending(&mut self, event_loop: &dyn crate::host::WindowFactory) {
        for _ in 0..64 {
            let mut updates = std::mem::take(&mut *self.pending.borrow_mut());
            self.collect_app_commands(&mut updates);
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

    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    #[cfg_attr(coverage_nightly, coverage(off))]
    fn tray_event(&mut self, event_loop: &dyn crate::host::WindowFactory, event: TrayEvent) {
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

    #[cfg_attr(coverage_nightly, coverage(off))]
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

impl ApplicationHandler<UserEvent> for MultiApplication {
    #[cfg_attr(coverage_nightly, coverage(off))]
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

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn exiting(&mut self, _event_loop: &ActiveEventLoop) {
        self.shutdown();
    }

    fn suspended(&mut self, event_loop: &ActiveEventLoop) {
        for entry in self.windows.values_mut() {
            entry.runtime.suspended(event_loop);
        }
    }

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn user_event(&mut self, event_loop: &ActiveEventLoop, event: UserEvent) {
        let _ = event_loop;
        match event {
            UserEvent::ModelsReady => self.models_ready(event_loop),
            #[cfg(feature = "tasks")]
            UserEvent::TasksReady => self.tasks_ready(event_loop),
            #[cfg(all(feature = "webview", target_os = "linux"))]
            UserEvent::NativeInput { .. } => {}
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
                if let Some(key) = self
                    .by_native
                    .get(&crate::host::HostId::Winit(event.window_id))
                    .cloned()
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

    #[cfg_attr(coverage_nightly, coverage(off))]
    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        #[cfg(all(feature = "native-popups", not(target_arch = "wasm32")))]
        if let Some(entry) = self
            .windows
            .values_mut()
            .find(|entry| entry.runtime.is_popup_window(window_id))
        {
            entry.runtime.popup_event(event_loop, window_id, event);
            self.process_pending(event_loop);
            return;
        }
        let Some(key) = self
            .by_native
            .get(&crate::host::HostId::Winit(window_id))
            .cloned()
        else {
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
        self.handle_close(&key, event_loop);
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
        let lifecycle = match &event {
            RuntimeEvent::RendererReady => Some(AppEvent::WindowReady {
                window: key.clone(),
            }),
            RuntimeEvent::RendererFailed(error) => Some(AppEvent::WindowFailed {
                window: key.clone(),
                error: error.clone(),
            }),
            _ => None,
        };
        if let Some(event) = lifecycle {
            pending.borrow_mut().push(model.borrow_mut().update(&event));
        }
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

    fn handle_ui_event(&mut self, event: &argui_ui::UiEvent, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().update(&AppEvent::Ui {
            window: self.key.clone(),
            event: event.clone(),
        });
        request_update(cx, self.record(update));
        self.forward_requests(cx);
    }

    fn forward_requests(&mut self, cx: &mut Context<Self>) {
        for command in self.model.borrow_mut().take_ui_commands(&self.key) {
            cx.ui_command(command);
        }
        if let Some(request) = self.model.borrow_mut().take_clipboard_request(&self.key) {
            cx.write_clipboard(request);
        }
        if let Some(request) = self.model.borrow_mut().take_scroll_request(&self.key) {
            cx.scroll(request);
        }
        if let Some(request) = self.model.borrow_mut().take_focus_request(&self.key) {
            match request {
                argui_ui::FocusRequest::Focus(target) => cx.request_focus(target),
                argui_ui::FocusRequest::Clear => cx.clear_focus(),
            }
        }
        if let Some(request) = self
            .model
            .borrow_mut()
            .take_text_selection_request(&self.key)
        {
            cx.select_text(request.target, request.selection);
        }
        if let Some(request) = self.model.borrow_mut().take_theme_request(&self.key) {
            cx.set_theme(request);
        }
    }
}

impl Render for WindowModel {
    #[cfg(feature = "tasks")]
    fn tasks_ready(&mut self, cx: &mut Context<Self>) {
        let update = self.model.borrow_mut().tasks_ready(&self.key);
        request_update(cx, self.record(update));
        self.forward_requests(cx);
    }
    fn render(&mut self, cx: &mut Context<Self>) -> Element {
        let mut root = self
            .view(cx.environment())
            .unwrap_or_else(|| Element::container(Vec::<Element>::new()));
        if let Some(router) = self.model.borrow().event_router(&self.key) {
            cx.route_events_to(router);
            if !self.model.borrow().captures_ui_events() {
                return root;
            }
        }
        for event in argui_ui::EventType::ALL {
            root = root.on(cx
                .listener(event, |model, event, cx| model.handle_ui_event(event, cx))
                .capture(true));
        }
        root
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
