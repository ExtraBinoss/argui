use argui_platform::PlatformEvent;
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
    RenderProfile(RenderProfile),
    AnimationProfile(AnimationProfile),
    RendererFailed(String),
    LayoutFailed(String),
}

#[derive(Clone, Debug)]
pub(crate) enum UserEvent {
    #[cfg(target_arch = "wasm32")]
    ClipboardText(String),
}

impl Application {
    #[cfg(target_arch = "wasm32")]
    pub(crate) fn set_event_proxy(&mut self, proxy: EventLoopProxy<UserEvent>) {
        self.event_proxy = Some(proxy);
    }

    #[cfg(not(target_arch = "wasm32"))]
    pub(crate) fn set_event_proxy(&mut self, _proxy: EventLoopProxy<UserEvent>) {}
}
