use argui_platform::{PlatformEvent, TrayEvent, WindowKey};
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
    RendererFailed(String),
    LayoutFailed(String),
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
    RendererFailed(String),
    LayoutFailed(String),
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
            Self::RendererFailed(event) => WindowRuntimeEvent::RendererFailed(event),
            Self::LayoutFailed(event) => WindowRuntimeEvent::LayoutFailed(event),
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
