use argui_platform::{GlobalShortcutEvent, PlatformEvent, TrayEvent, WindowKey};
use argui_render::RenderProfile;
use argui_ui::{TreeUpdate, UiEvent};

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AnimationProfile {
    pub frame_interval: std::time::Duration,
    pub model_time: std::time::Duration,
    pub tree_time: std::time::Duration,
    pub paint_time: std::time::Duration,
    pub tree_update: TreeUpdate,
}

use crate::app::Application;

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEvent {
    Platform(PlatformEvent),
    Ui(UiEvent),
    ViewUpdated(TreeUpdate),
    RendererReady,
    RenderProfile(Box<RenderProfile>),
    AnimationProfile(AnimationProfile),
    /// Renderer initialization succeeded after selecting a compatibility fallback.
    RendererFallback(String),
    RendererFailed(String),
    /// One retained GPU canvas entered a recoverable failure state.
    GpuCanvasFailed(argui_render::GpuCanvasDiagnostic),
    /// One retained GPU canvas recovered after a later successful render.
    GpuCanvasRecovered(argui_render::GpuCanvasDiagnostic),
    LayoutFailed(String),
    DesktopBackdropUnavailable(String),
    /// A requested native overlay was retained in its parent surface.
    PopupFallback {
        node: argui_ui::NodeId,
        reason: String,
    },
    Window {
        window: WindowKey,
        event: WindowRuntimeEvent,
    },
    Tray(TrayEvent),
    TrayUnavailable(String),
    TrayFailed(String),
    /// A registered system-wide keyboard shortcut changed state.
    GlobalShortcut(GlobalShortcutEvent),
    /// Global shortcut registration is unsupported in the current build or platform.
    GlobalShortcutsUnavailable(String),
    /// Native global shortcut initialization or registration failed.
    GlobalShortcutsFailed(String),
    CommandFailed(String),
}

#[derive(Clone, Debug, PartialEq)]
pub enum WindowRuntimeEvent {
    Platform(PlatformEvent),
    Ui(UiEvent),
    ViewUpdated(TreeUpdate),
    RendererReady,
    RenderProfile(Box<RenderProfile>),
    AnimationProfile(AnimationProfile),
    /// Renderer initialization succeeded after selecting a compatibility fallback.
    RendererFallback(String),
    RendererFailed(String),
    /// One retained GPU canvas entered a recoverable failure state.
    GpuCanvasFailed(argui_render::GpuCanvasDiagnostic),
    /// One retained GPU canvas recovered after a later successful render.
    GpuCanvasRecovered(argui_render::GpuCanvasDiagnostic),
    LayoutFailed(String),
    DesktopBackdropUnavailable(String),
    /// A requested native overlay was retained in its parent surface.
    PopupFallback {
        node: argui_ui::NodeId,
        reason: String,
    },
}

impl RuntimeEvent {
    pub(crate) fn scoped(self, window: WindowKey) -> Self {
        let event = match self {
            Self::Platform(event) => WindowRuntimeEvent::Platform(event),
            Self::Ui(event) => WindowRuntimeEvent::Ui(event),
            Self::ViewUpdated(event) => WindowRuntimeEvent::ViewUpdated(event),
            Self::RendererReady => WindowRuntimeEvent::RendererReady,
            Self::RenderProfile(event) => WindowRuntimeEvent::RenderProfile(event),
            Self::AnimationProfile(event) => WindowRuntimeEvent::AnimationProfile(event),
            Self::RendererFallback(event) => WindowRuntimeEvent::RendererFallback(event),
            Self::RendererFailed(event) => WindowRuntimeEvent::RendererFailed(event),
            Self::GpuCanvasFailed(event) => WindowRuntimeEvent::GpuCanvasFailed(event),
            Self::GpuCanvasRecovered(event) => WindowRuntimeEvent::GpuCanvasRecovered(event),
            Self::LayoutFailed(event) => WindowRuntimeEvent::LayoutFailed(event),
            Self::DesktopBackdropUnavailable(reason) => {
                WindowRuntimeEvent::DesktopBackdropUnavailable(reason)
            }
            Self::PopupFallback { node, reason } => {
                WindowRuntimeEvent::PopupFallback { node, reason }
            }
            other => return other,
        };
        Self::Window { window, event }
    }
}

#[derive(Debug)]
pub(crate) enum UserEvent {
    ModelsReady,
    #[cfg(feature = "tasks")]
    TasksReady,
    #[cfg(all(feature = "webview", target_os = "linux"))]
    NativeInput {
        window: WindowKey,
    },
    Preferences {
        window: WindowKey,
        preferences: argui_platform::SystemPreferences,
    },
    #[cfg(target_arch = "wasm32")]
    ClipboardText {
        window: WindowKey,
        target: Option<argui_ui::NodeId>,
        text: String,
    },
    #[cfg(target_arch = "wasm32")]
    Accessibility {
        window: WindowKey,
        request: argui_accessibility::SemanticRequest,
    },
    #[cfg(not(target_arch = "wasm32"))]
    AccessKit(accesskit_winit::Event),
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    Tray(TrayEvent),
    #[cfg(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    ))]
    GlobalShortcut(GlobalShortcutEvent),
    #[cfg(all(
        feature = "global-shortcuts",
        any(target_os = "linux", target_os = "windows", target_os = "macos")
    ))]
    GlobalShortcutsFailed(String),
}

impl Application {
    pub(crate) fn set_event_proxy(&mut self, proxy: impl Into<crate::host::EventProxy>) {
        let proxy = proxy.into();
        if let Some(model) = &self.model {
            let wake = proxy.clone();
            model.set_model_wake(move || {
                let _ = wake.send_event(UserEvent::ModelsReady);
            });
        }
        #[cfg(feature = "tasks")]
        {
            let wake = proxy.clone();
            let tasks = self.tasks.get_or_insert_with(|| {
                crate::tasks::TaskRuntime::new(move || {
                    let _ = wake.send_event(UserEvent::TasksReady);
                })
            });
            if let Some(model) = &self.model {
                model.set_task_runtime(tasks.clone());
            }
        }
        self.event_proxy = Some(proxy);
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<accesskit_winit::Event> for UserEvent {
    fn from(event: accesskit_winit::Event) -> Self {
        Self::AccessKit(event)
    }
}
