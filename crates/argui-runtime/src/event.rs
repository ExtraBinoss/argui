use argui_platform::{PlatformEvent, TrayEvent, WindowKey};
use argui_render::RenderProfile;
use argui_ui::{TreeUpdate, UiEvent};
use winit::event_loop::EventLoopProxy;

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
            other => return other,
        };
        Self::Window { window, event }
    }
}

#[derive(Clone, Debug)]
pub(crate) enum UserEvent {
    #[cfg(target_arch = "wasm32")]
    ClipboardText { window: WindowKey, text: String },
    #[cfg(all(feature = "tray", not(target_arch = "wasm32")))]
    Tray(TrayEvent),
}

impl Application {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn set_event_proxy(&mut self, proxy: EventLoopProxy<UserEvent>) {
        self.event_proxy = Some(proxy);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn set_event_proxy(&mut self, _proxy: EventLoopProxy<UserEvent>) {}
}
