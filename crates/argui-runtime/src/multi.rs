use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

#[cfg(feature = "tasks")]
use crate::event::UserEvent;
use crate::{AppEvent, AppModel, AppUpdate, Entity, RuntimeError, RuntimeEvent, app::Application};
use argui_platform::{ApplicationConfig, WindowKey, WindowLevel, WindowSpec};
use argui_render::RendererConfig;
use argui_text::TextEngine;
#[cfg(not(target_arch = "wasm32"))]
use argui_ui::UiTree;

mod command;
mod event_loop;
mod global_shortcuts;
#[cfg(all(feature = "webview", target_os = "linux"))]
pub(crate) mod gtk;
mod lifecycle;
mod model_updates;
use model_updates::WindowModel;
#[cfg(not(target_arch = "wasm32"))]
mod native_host_application;
mod tray;
mod window_input;

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
    input_region: argui_platform::WindowInputRegion,
}

#[cfg(not(target_arch = "wasm32"))]
struct NativeHostPresentation {
    host: Option<crate::NativeHost>,
    assets: crate::NativeHostAssets,
    events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
    windows: HashSet<WindowKey>,
}

pub(crate) struct MultiApplication {
    #[cfg(feature = "tasks")]
    tasks: Option<crate::tasks::TaskRuntime>,
    config: ApplicationConfig,
    renderer_config: RendererConfig,
    initial_text_engine: Option<TextEngine>,
    extra_text_engines: HashMap<WindowKey, TextEngine>,
    renderer_device: Rc<RefCell<Option<argui_render::RendererDevice>>>,
    model: SharedModel,
    pending: SharedUpdates,
    callback: SharedCallback,
    windows: HashMap<WindowKey, WindowEntry>,
    retired: Vec<crate::ModelRuntime>,
    by_native: HashMap<crate::host::HostId, WindowKey>,
    #[cfg(not(target_arch = "wasm32"))]
    native_host: Option<NativeHostPresentation>,
    event_proxy: Option<crate::host::EventProxy>,
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    native_tray: Option<argui_platform::NativeTray>,
    #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
    tray_unavailable_announced: bool,
    #[cfg(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    ))]
    native_global_shortcuts: Option<argui_platform::NativeGlobalShortcuts>,
    #[cfg(all(feature = "global-shortcuts", target_os = "linux"))]
    global_shortcut_setup_error: Option<String>,
    pending_activation_token: Option<String>,
    ui_zoom_factor: f32,
    #[cfg(not(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    )))]
    global_shortcuts_unavailable_announced: bool,
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
        let model: Box<dyn AppModel> = Box::new(model);
        #[cfg(all(feature = "global-shortcuts", target_os = "linux"))]
        let global_shortcut_setup_error = (!config.global_shortcuts.is_empty()
            && std::env::var_os("WAYLAND_DISPLAY").is_some()
            && std::env::var("DISPLAY")
                .ok()
                .is_none_or(|display| display.is_empty()))
        .then(|| {
            argui_platform::prepare_wayland_global_shortcuts(config.identity.linux_application_id())
                .err()
        })
        .flatten();
        Ok(Self {
            config,
            renderer_config,
            initial_text_engine: text_engine,
            extra_text_engines: HashMap::new(),
            renderer_device: Rc::new(RefCell::new(None)),
            model: Rc::new(RefCell::new(model)),
            pending: Rc::new(RefCell::new(Vec::new())),
            callback: Rc::new(RefCell::new(Box::new(on_event))),
            windows: HashMap::new(),
            retired: Vec::new(),
            by_native: HashMap::new(),
            #[cfg(not(target_arch = "wasm32"))]
            native_host: None,
            event_proxy: None,
            #[cfg(feature = "tasks")]
            tasks: None,
            #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
            native_tray: None,
            #[cfg(any(not(feature = "tray"), target_arch = "wasm32"))]
            tray_unavailable_announced: false,
            #[cfg(all(
                feature = "global-shortcuts",
                any(target_os = "linux", target_os = "windows", target_os = "macos")
            ))]
            native_global_shortcuts: None,
            #[cfg(all(feature = "global-shortcuts", target_os = "linux"))]
            global_shortcut_setup_error,
            pending_activation_token: None,
            ui_zoom_factor: 1.0,
            #[cfg(not(all(
                feature = "global-shortcuts",
                any(target_os = "linux", target_os = "windows", target_os = "macos")
            )))]
            global_shortcuts_unavailable_announced: false,
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

    /// Retains the font engines in `engines` until each named window is created.
    /// A later entry for the same key replaces its pending engine.
    pub(crate) fn install_window_text_engines(
        &mut self,
        engines: impl IntoIterator<Item = (WindowKey, TextEngine)>,
    ) {
        self.extra_text_engines.extend(engines);
    }

    /// Installs retained presentations for the keys in `windows`.
    /// `host` owns the main graph, `assets` resolve shared media, and `events`
    /// receives UI callbacks. Other selected windows receive independent graphs.
    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn install_native_host(
        &mut self,
        host: crate::NativeHost,
        assets: crate::NativeHostAssets,
        events: std::sync::mpsc::Sender<crate::NativeHostDelivery>,
        windows: impl IntoIterator<Item = WindowKey>,
    ) {
        self.native_host = Some(NativeHostPresentation {
            host: Some(host),
            assets,
            events,
            windows: windows.into_iter().collect(),
        });
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
        #[cfg(not(target_arch = "wasm32"))]
        let presentation = self
            .native_host
            .as_mut()
            .filter(|presentation| presentation.windows.contains(&key))
            .map(|presentation| {
                let host = if key == WindowKey::main() {
                    presentation
                        .host
                        .take()
                        .expect("main native host is installed")
                } else {
                    crate::NativeHost::with_builtins().expect("built-in native host schema")
                };
                (
                    host,
                    presentation.assets.clone(),
                    presentation.events.clone(),
                )
            });
        #[cfg(not(target_arch = "wasm32"))]
        let initial = presentation
            .as_ref()
            .and_then(|presentation| presentation.0.root_element())
            .map(UiTree::new);
        let mut runtime = Application::new(
            spec.window.clone(),
            self.renderer_config.clone(),
            if key == WindowKey::main() {
                self.initial_text_engine.take().unwrap_or_default()
            } else {
                self.extra_text_engines.remove(&key).unwrap_or_default()
            },
            None,
            #[cfg(not(target_arch = "wasm32"))]
            initial,
            #[cfg(target_arch = "wasm32")]
            None,
            #[cfg(not(target_arch = "wasm32"))]
            presentation.is_none().then(|| {
                Entity::new(adapter)
                    .mount()
                    .expect("new window model is open")
                    .entity
                    .erase()
            }),
            #[cfg(target_arch = "wasm32")]
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
        .with_theme(self.model.borrow().theme(&key))
        .initially_visible(spec.visible)
        .ui_zoom(self.config.ui_zoom.enabled, self.ui_zoom_factor)
        .preference_overrides(self.config.preferences)
        .shared_renderer_device(Rc::clone(&self.renderer_device));
        #[cfg(not(target_arch = "wasm32"))]
        if let Some((host, assets, events)) = presentation {
            runtime.install_native_host_assets(assets);
            runtime.native_host = Some(host);
            runtime.native_host_events = Some(events);
        }
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
        self.windows.insert(
            key,
            WindowEntry {
                spec,
                runtime,
                input_region: argui_platform::WindowInputRegion::Full,
            },
        );
    }

    /// Propagates a locally requested accessibility zoom to every open window.
    ///
    /// `source` identifies the window that received the keyboard or touch
    /// gesture. Windows without a pending request leave the global factor unchanged.
    fn synchronize_ui_zoom(&mut self, source: &WindowKey) {
        let Some(factor) = self
            .windows
            .get_mut(source)
            .and_then(|entry| entry.runtime.take_ui_zoom_request())
        else {
            return;
        };
        self.ui_zoom_factor = factor;
        for (key, entry) in &mut self.windows {
            if key != source {
                entry.runtime.install_ui_zoom(factor, false);
            }
        }
        let keys: Vec<_> = self.windows.keys().cloned().collect();
        for key in keys {
            self.refresh_window_input_region(&key);
        }
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

    fn emit(&self, event: RuntimeEvent) {
        (self.callback.borrow_mut())(event);
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
