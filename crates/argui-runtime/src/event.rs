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
