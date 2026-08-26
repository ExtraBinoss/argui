use argui_platform::PlatformEvent;
use argui_ui::{TreeUpdate, UiEvent};
use winit::event_loop::EventLoopProxy;

use crate::app::Application;

#[derive(Clone, Debug, PartialEq)]
pub enum RuntimeEvent {
    Platform(PlatformEvent),
    Ui(UiEvent),
    ViewUpdated(TreeUpdate),
    RendererReady,
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
