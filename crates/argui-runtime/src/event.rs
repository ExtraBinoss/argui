use argui_platform::PlatformEvent;
use argui_ui::{TreeUpdate, UiEvent};

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
